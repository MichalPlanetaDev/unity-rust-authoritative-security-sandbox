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
    SuspicionKind, SuspicionReport, TelemetryEvent, Vec2,
};
use telemetry::TelemetryWriter;

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

        let max_speed = world.config.max_speed_units_per_second;
        let tolerance = world.config.movement_tolerance_units;
        let fixed_tick_ms = world.config.fixed_tick_ms;
        let fire_cooldown_ms = world.config.fire_cooldown_ms;
        let max_client_time_step_ms = world.config.max_client_time_step_ms;

        let state = world
            .players
            .entry(command.player_id)
            .or_insert_with(PlayerState::new);

        if command.sequence <= state.last_sequence {
            let report = SuspicionReport::new(
                command.player_id,
                command.sequence,
                SuspicionKind::PacketSequenceViolation,
                "command sequence number did not increase",
                command.sequence as f32,
                (state.last_sequence + 1) as f32,
                server_time_ms,
            );

            events.push(TelemetryEvent::Suspicion(report));

            ServerMessage::Rejected {
                reason: "command sequence number did not increase".to_string(),
            }
        } else if !state.alive {
            let report = SuspicionReport::new(
                command.player_id,
                command.sequence,
                SuspicionKind::InvalidStateTransition,
                "dead player attempted to send input",
                1.0,
                0.0,
                server_time_ms,
            );

            events.push(TelemetryEvent::Suspicion(report));

            ServerMessage::Rejected {
                reason: "dead player cannot send input".to_string(),
            }
        } else {
            inspect_client_time(
                &command,
                state.last_client_time_ms,
                max_client_time_step_ms,
                server_time_ms,
                &mut events,
            );

            if let Some(claimed_position) = command.claimed_position {
                let observed_distance = state.position.distance(claimed_position);
                let allowed_distance = max_speed * (fixed_tick_ms as f32 / 1000.0) + tolerance;

                if observed_distance > allowed_distance {
                    let report = SuspicionReport::new(
                        command.player_id,
                        command.sequence,
                        SuspicionKind::SpeedHack,
                        "claimed position exceeded server movement budget",
                        observed_distance,
                        allowed_distance,
                        server_time_ms,
                    );

                    events.push(TelemetryEvent::Suspicion(report));
                }
            }

            if command.fire {
                if server_time_ms < state.next_allowed_fire_time_ms {
                    let report = SuspicionReport::new(
                        command.player_id,
                        command.sequence,
                        SuspicionKind::FireRateViolation,
                        "fire input arrived before cooldown expired",
                        server_time_ms as f32,
                        state.next_allowed_fire_time_ms as f32,
                        server_time_ms,
                    );

                    events.push(TelemetryEvent::Suspicion(report));
                } else {
                    state.next_allowed_fire_time_ms = server_time_ms + fire_cooldown_ms;
                }
            }

            let movement_budget = max_speed * (fixed_tick_ms as f32 / 1000.0);
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

fn inspect_client_time(
    command: &InputCommand,
    previous_client_time_ms: Option<Milliseconds>,
    max_client_time_step_ms: Milliseconds,
    server_time_ms: Milliseconds,
    events: &mut Vec<TelemetryEvent>,
) {
    let Some(previous_client_time_ms) = previous_client_time_ms else {
        return;
    };

    if command.client_time_ms <= previous_client_time_ms {
        let report = SuspicionReport::new(
            command.player_id,
            command.sequence,
            SuspicionKind::ClientTimeViolation,
            "client timestamp did not increase",
            command.client_time_ms as f32,
            (previous_client_time_ms + 1) as f32,
            server_time_ms,
        );

        events.push(TelemetryEvent::Suspicion(report));
        return;
    }

    let observed_step = command.client_time_ms - previous_client_time_ms;

    if observed_step > max_client_time_step_ms {
        let report = SuspicionReport::new(
            command.player_id,
            command.sequence,
            SuspicionKind::ClientTimeViolation,
            "client timestamp jumped too far forward",
            observed_step as f32,
            max_client_time_step_ms as f32,
            server_time_ms,
        );

        events.push(TelemetryEvent::Suspicion(report));
    }
}

fn to_invalid_data(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
