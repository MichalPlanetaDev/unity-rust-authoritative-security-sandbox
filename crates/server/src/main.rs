use std::{
    collections::HashMap,
    io,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use protocol::{
    ClientMessage, InputCommand, Milliseconds, PlayerId, PlayerSnapshot, ServerMessage,
    SuspicionKind, TelemetryEvent, Vec2,
};
use telemetry::TelemetryWriter;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream, tcp::OwnedWriteHalf},
    sync::broadcast,
};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;
use validation::{PlayerValidationState, ValidationPolicy, validate_input};

const DEFAULT_CONFIG_PATH: &str = "config/default.toml";

#[derive(Debug, Clone, serde::Deserialize)]
struct ServerConfig {
    bind_addr: String,
    telemetry_path: String,
    max_speed_units_per_second: f32,
    movement_tolerance_units: f32,
    fixed_tick_ms: Milliseconds,
    fire_cooldown_ms: Milliseconds,
    max_client_time_step_ms: Milliseconds,
}

impl ServerConfig {
    fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        toml::from_str(&content).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse config '{}': {}", path.display(), error),
            )
        })
    }

    fn validation_policy(&self) -> ValidationPolicy {
        ValidationPolicy {
            max_speed_units_per_second: self.max_speed_units_per_second,
            movement_tolerance_units: self.movement_tolerance_units,
            fixed_tick_ms: self.fixed_tick_ms,
            fire_cooldown_ms: self.fire_cooldown_ms,
            max_client_time_step_ms: self.max_client_time_step_ms,
        }
    }
}

#[derive(Debug, Clone)]
struct PlayerState {
    position: Vec2,
    health: i32,
    alive: bool,
    last_sequence: u64,
    last_client_time_ms: Option<Milliseconds>,
    next_allowed_fire_time_ms: Milliseconds,
}

impl PlayerState {
    fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            health: 100,
            alive: true,
            last_sequence: 0,
            last_client_time_ms: None,
            next_allowed_fire_time_ms: 0,
        }
    }

    fn snapshot(&self, player_id: PlayerId, server_time_ms: Milliseconds) -> PlayerSnapshot {
        PlayerSnapshot {
            player_id,
            position: self.position,
            health: self.health,
            alive: self.alive,
            server_time_ms,
        }
    }

    fn validation_state(&self) -> PlayerValidationState {
        PlayerValidationState {
            position: self.position,
            alive: self.alive,
            last_sequence: self.last_sequence,
            last_client_time_ms: self.last_client_time_ms,
            next_allowed_fire_time_ms: self.next_allowed_fire_time_ms,
        }
    }
}

struct GameWorld {
    config: ServerConfig,
    players: HashMap<PlayerId, PlayerState>,
}

impl GameWorld {
    fn new(config: ServerConfig) -> Self {
        Self {
            config,
            players: HashMap::new(),
        }
    }
}

struct SharedServer {
    started_at: Instant,
    next_connection_id: AtomicU64,
    world: Mutex<GameWorld>,
    telemetry: Mutex<TelemetryWriter>,
}

impl SharedServer {
    fn new(config: ServerConfig, telemetry: TelemetryWriter) -> Self {
        Self {
            started_at: Instant::now(),
            next_connection_id: AtomicU64::new(1),
            world: Mutex::new(GameWorld::new(config)),
            telemetry: Mutex::new(telemetry),
        }
    }

    fn allocate_connection_id(&self) -> u64 {
        self.next_connection_id.fetch_add(1, Ordering::Relaxed)
    }

    fn server_time_ms(&self) -> Milliseconds {
        self.started_at.elapsed().as_millis() as Milliseconds
    }

    fn write_event(&self, event: &TelemetryEvent) {
        let result = self
            .telemetry
            .lock()
            .expect("telemetry lock poisoned")
            .write_event(event);

        if let Err(error) = result {
            error!(%error, "failed to write telemetry");
        }
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    if let Err(error) = run().await {
        error!(%error, "server failed");
        std::process::exit(1);
    }
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

async fn run() -> io::Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());

    let config = ServerConfig::load(&config_path)?;
    let telemetry = TelemetryWriter::create(&config.telemetry_path)?;
    let listener = TcpListener::bind(&config.bind_addr).await?;

    info!("unity-rust-authoritative-security-sandbox server");
    info!(config_path = %config_path, "loaded config");
    info!(bind_addr = %config.bind_addr, "listening");
    info!(telemetry_path = %config.telemetry_path, "telemetry configured");

    let shared = Arc::new(SharedServer::new(config, telemetry));
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let accept_loop = accept_connections(listener, Arc::clone(&shared), shutdown_tx.subscribe());

    tokio::select! {
        result = accept_loop => {
            result?;
        }
        signal_result = tokio::signal::ctrl_c() => {
            signal_result?;
            info!("shutdown signal received");
        }
    }

    let _ = shutdown_tx.send(());

    info!("server shutdown complete");
    Ok(())
}

async fn accept_connections(
    listener: TcpListener,
    shared: Arc<SharedServer>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> io::Result<()> {
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                let (stream, peer_addr) = accept_result?;
                let shared = Arc::clone(&shared);
                let connection_id = shared.allocate_connection_id();
                let client_shutdown_rx = shutdown_rx.resubscribe();

                info!(%peer_addr, connection_id, "client accepted");

                tokio::spawn(async move {
                    if let Err(error) = handle_client(stream, connection_id, shared, client_shutdown_rx).await {
                        warn!(%peer_addr, connection_id, %error, "client handler exited with error");
                    }
                });
            }
            _ = shutdown_rx.recv() => {
                info!("accept loop received shutdown");
                return Ok(());
            }
        }
    }
}

async fn handle_client(
    stream: TcpStream,
    connection_id: u64,
    shared: Arc<SharedServer>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> io::Result<()> {
    let peer_addr = stream.peer_addr()?;
    let mut joined_player_id = None;

    info!(%peer_addr, connection_id, "client connected");

    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    loop {
        tokio::select! {
            line_result = lines.next_line() => {
                let Some(line) = line_result? else {
                    break;
                };

                if line.trim().is_empty() {
                    continue;
                }

                debug!(connection_id, line = %line, "client message received");

                let parsed_message = serde_json::from_str::<ClientMessage>(&line);

                if let Ok(ClientMessage::Join { player_id }) = &parsed_message {
                    joined_player_id = Some(*player_id);
                }

                let response = match parsed_message {
                    Ok(message) => process_message(message, connection_id, &shared),
                    Err(error) => {
                        warn!(connection_id, %error, "invalid client message");
                        ServerMessage::Rejected {
                            reason: format!("invalid client message: {error}"),
                        }
                    }
                };

                write_response(&mut write_half, &response).await?;
            }
            _ = shutdown_rx.recv() => {
                info!(connection_id, "client task received shutdown");
                break;
            }
        }
    }

    let disconnected_at = shared.server_time_ms();

    shared.write_event(&TelemetryEvent::ClientDisconnected {
        connection_id,
        player_id: joined_player_id,
        server_time_ms: disconnected_at,
    });

    info!(
        %peer_addr,
        connection_id,
        player_id = ?joined_player_id,
        "client disconnected"
    );

    Ok(())
}

async fn write_response(writer: &mut OwnedWriteHalf, response: &ServerMessage) -> io::Result<()> {
    let json = serde_json::to_string(response).map_err(to_invalid_data)?;

    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await
}

fn process_message(
    message: ClientMessage,
    connection_id: u64,
    shared: &SharedServer,
) -> ServerMessage {
    let server_time_ms = shared.server_time_ms();

    match message {
        ClientMessage::Join { player_id } => {
            {
                let mut world = shared.world.lock().expect("world lock poisoned");
                world
                    .players
                    .entry(player_id)
                    .or_insert_with(PlayerState::new);
            }

            shared.write_event(&TelemetryEvent::ClientConnected {
                connection_id,
                player_id,
                server_time_ms,
            });

            info!(connection_id, player_id = ?player_id, "player joined");

            ServerMessage::Welcome { player_id }
        }
        ClientMessage::Input(command) => process_input(command, server_time_ms, shared),
        ClientMessage::Ping { client_time_ms } => ServerMessage::Pong {
            client_time_ms,
            server_time_ms,
        },
    }
}

fn process_input(
    command: InputCommand,
    server_time_ms: Milliseconds,
    shared: &SharedServer,
) -> ServerMessage {
    let mut events = Vec::new();

    let response = {
        let mut world = shared.world.lock().expect("world lock poisoned");
        let policy = world.config.validation_policy();

        let state = world
            .players
            .entry(command.player_id)
            .or_insert_with(PlayerState::new);

        let decision = validate_input(&command, state.validation_state(), policy, server_time_ms);

        let accepted = decision.accepted;
        let rejection_reason = decision.rejection_reason.clone();
        let has_fire_rate_violation = decision.has_kind(SuspicionKind::FireRateViolation);

        for report in decision.reports {
            warn!(
                player_id = ?report.player_id,
                sequence = report.sequence,
                kind = ?report.kind,
                observed_value = report.observed_value,
                expected_limit = report.expected_limit,
                "suspicion detected"
            );

            events.push(TelemetryEvent::Suspicion(report));
        }

        if !accepted {
            let reason = rejection_reason.unwrap_or_else(|| "input rejected".to_string());

            warn!(
                player_id = ?command.player_id,
                sequence = command.sequence,
                reason = %reason,
                "input rejected"
            );

            ServerMessage::Rejected { reason }
        } else {
            if command.fire && !has_fire_rate_violation {
                state.next_allowed_fire_time_ms = server_time_ms + policy.fire_cooldown_ms;
            }

            let movement_budget =
                policy.max_speed_units_per_second * (policy.fixed_tick_ms as f32 / 1000.0);
            let movement_delta = command.movement.normalized().scaled(movement_budget);

            state.position = state.position.add_vector(movement_delta);
            state.last_sequence = command.sequence;
            state.last_client_time_ms = Some(command.client_time_ms);

            let snapshot = state.snapshot(command.player_id, server_time_ms);

            events.push(TelemetryEvent::CommandAccepted {
                command,
                server_time_ms,
            });
            events.push(TelemetryEvent::PlayerSnapshot(snapshot.clone()));

            ServerMessage::Snapshot(snapshot)
        }
    };

    for event in events {
        shared.write_event(&event);
    }

    response
}

fn to_invalid_data(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
