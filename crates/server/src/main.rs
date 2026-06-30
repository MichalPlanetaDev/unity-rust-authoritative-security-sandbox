use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
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
    world: Arc<Mutex<GameWorld>>,
    telemetry: Arc<Mutex<TelemetryWriter>>,
}

impl SharedServer {
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
            eprintln!("failed to write telemetry: {error}");
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("server failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());

    let config = ServerConfig::load(&config_path)?;
    let telemetry = TelemetryWriter::create(&config.telemetry_path)?;
    let listener = TcpListener::bind(&config.bind_addr)?;

    println!("unity-rust-authoritative-security-sandbox server");
    println!("Loaded config: {config_path}");
    println!("Listening on: {}", config.bind_addr);
    println!("Telemetry: {}", config.telemetry_path);
    println!();

    let shared = Arc::new(SharedServer {
        started_at: Instant::now(),
        next_connection_id: AtomicU64::new(1),
        world: Arc::new(Mutex::new(GameWorld::new(config))),
        telemetry: Arc::new(Mutex::new(telemetry)),
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let shared = Arc::clone(&shared);

                std::thread::spawn(move || {
                    if let Err(error) = handle_client(stream, shared) {
                        eprintln!("client handler failed: {error}");
                    }
                });
            }
            Err(error) => {
                eprintln!("failed to accept client: {error}");
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, shared: Arc<SharedServer>) -> io::Result<()> {
    let peer_addr = stream.peer_addr()?;
    let connection_id = shared.allocate_connection_id();
    let mut joined_player_id = None;

    println!("client connected: {peer_addr} connection_id={connection_id}");

    let reader_stream = stream.try_clone()?;
    let reader = BufReader::new(reader_stream);

    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let parsed_message = serde_json::from_str::<ClientMessage>(&line);

        if let Ok(ClientMessage::Join { player_id }) = &parsed_message {
            joined_player_id = Some(*player_id);
        }

        let response = match parsed_message {
            Ok(message) => process_message(message, connection_id, &shared),
            Err(error) => ServerMessage::Rejected {
                reason: format!("invalid client message: {error}"),
            },
        };

        serde_json::to_writer(&mut stream, &response).map_err(to_invalid_data)?;
        stream.write_all(b"\n")?;
        stream.flush()?;
    }

    let disconnected_at = shared.server_time_ms();

    shared.write_event(&TelemetryEvent::ClientDisconnected {
        connection_id,
        player_id: joined_player_id,
        server_time_ms: disconnected_at,
    });

    println!("client disconnected: {peer_addr} connection_id={connection_id}");

    Ok(())
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
            events.push(TelemetryEvent::Suspicion(report));
        }

        if !accepted {
            ServerMessage::Rejected {
                reason: rejection_reason.unwrap_or_else(|| "input rejected".to_string()),
            }
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
