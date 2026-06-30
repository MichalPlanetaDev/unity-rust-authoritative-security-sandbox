use std::{
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    thread,
    time::{Duration, Instant},
};

use protocol::{ClientMessage, InputCommand, PROTOCOL_VERSION, PlayerId, Vec2};

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:4000";

struct Scenario {
    player_id: PlayerId,
    commands: Vec<InputCommand>,
    delay_between_commands: Duration,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("bot failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let scenario_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "normal".to_string());

    let server_addr =
        std::env::var("SERVER_ADDR").unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string());

    let scenario = match scenario_name.as_str() {
        "normal" => Scenario {
            player_id: PlayerId(1),
            commands: normal_commands(PlayerId(1)),
            delay_between_commands: Duration::from_millis(100),
        },
        "suspicious" => Scenario {
            player_id: PlayerId(2),
            commands: suspicious_commands(PlayerId(2)),
            delay_between_commands: Duration::from_millis(100),
        },
        "sequence" => Scenario {
            player_id: PlayerId(3),
            commands: sequence_violation_commands(PlayerId(3)),
            delay_between_commands: Duration::from_millis(100),
        },
        "timing" => Scenario {
            player_id: PlayerId(4),
            commands: timing_violation_commands(PlayerId(4)),
            delay_between_commands: Duration::from_millis(100),
        },
        "flood" => return run_flood_scenario(&server_addr),
        "bad-protocol" => return run_bad_protocol_scenario(&server_addr),
        "help" | "--help" | "-h" => {
            print_help();
            return Ok(());
        }
        unknown => {
            eprintln!("unknown scenario: {unknown}");
            print_help();
            std::process::exit(2);
        }
    };

    println!("Connecting to {server_addr}");
    println!("Scenario: {scenario_name}");
    println!("Player: {:?}", scenario.player_id);
    println!();

    let mut stream = connect_with_retry(&server_addr, Duration::from_secs(10))?;
    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);

    send_message(
        &mut stream,
        &mut reader,
        &ClientMessage::Join {
            player_id: scenario.player_id,
            protocol_version: Some(PROTOCOL_VERSION),
        },
    )?;

    for command in scenario.commands {
        send_message(&mut stream, &mut reader, &ClientMessage::Input(command))?;

        if !scenario.delay_between_commands.is_zero() {
            thread::sleep(scenario.delay_between_commands);
        }
    }

    Ok(())
}

fn run_flood_scenario(server_addr: &str) -> io::Result<()> {
    let player_id = PlayerId(5);

    println!("Connecting to {server_addr}");
    println!("Scenario: flood");
    println!("Player: {:?}", player_id);
    println!();

    let mut stream = connect_with_retry(server_addr, Duration::from_secs(10))?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_millis(750)))?;

    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);

    send_message(
        &mut stream,
        &mut reader,
        &ClientMessage::Join {
            player_id,
            protocol_version: Some(PROTOCOL_VERSION),
        },
    )?;

    for command in flood_commands(player_id) {
        write_message(&mut stream, &ClientMessage::Input(command))?;
    }

    stream.flush()?;
    read_available_responses(&mut reader)?;

    Ok(())
}

fn run_bad_protocol_scenario(server_addr: &str) -> io::Result<()> {
    let player_id = PlayerId(6);

    println!("Connecting to {server_addr}");
    println!("Scenario: bad-protocol");
    println!("Player: {:?}", player_id);
    println!();

    let mut stream = connect_with_retry(server_addr, Duration::from_secs(10))?;
    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);

    send_message(
        &mut stream,
        &mut reader,
        &ClientMessage::Join {
            player_id,
            protocol_version: Some(PROTOCOL_VERSION + 999),
        },
    )
}

fn connect_with_retry(addr: &str, timeout: Duration) -> io::Result<TcpStream> {
    let started_at = Instant::now();
    let mut last_error = None;

    while started_at.elapsed() < timeout {
        match TcpStream::connect(addr) {
            Ok(stream) => return Ok(stream),
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(200));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            format!("timed out connecting to {addr}"),
        )
    }))
}

fn print_help() {
    println!("Usage:");
    println!("  cargo run -p bot -- normal");
    println!("  cargo run -p bot -- suspicious");
    println!("  cargo run -p bot -- sequence");
    println!("  cargo run -p bot -- timing");
    println!("  cargo run -p bot -- flood");
    println!("  cargo run -p bot -- bad-protocol");
    println!();
    println!("Environment:");
    println!("  SERVER_ADDR=127.0.0.1:4000");
}

fn send_message(
    stream: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
    message: &ClientMessage,
) -> io::Result<()> {
    write_message(stream, message)?;

    let mut response = String::new();
    reader.read_line(&mut response)?;

    println!("server -> {}", response.trim());

    Ok(())
}

fn write_message(stream: &mut TcpStream, message: &ClientMessage) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, message).map_err(to_invalid_data)?;
    stream.write_all(b"\n")
}

fn read_available_responses(reader: &mut BufReader<TcpStream>) -> io::Result<()> {
    let mut responses = 0usize;

    loop {
        let mut response = String::new();

        match reader.read_line(&mut response) {
            Ok(0) => break,
            Ok(_) => {
                responses += 1;
                println!("server -> {}", response.trim());

                if responses >= 120 {
                    break;
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(error) => return Err(error),
        }
    }

    println!("Read {responses} flood responses.");

    Ok(())
}

fn normal_commands(player_id: PlayerId) -> Vec<InputCommand> {
    vec![
        input(
            player_id,
            1,
            100,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(1.0, 0.0)),
        ),
        input(
            player_id,
            2,
            200,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(2.0, 0.0)),
        ),
        input(
            player_id,
            3,
            300,
            Vec2::new(1.0, 0.0),
            true,
            Some(Vec2::new(3.0, 0.0)),
        ),
        input(
            player_id,
            4,
            400,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(4.0, 0.0)),
        ),
        input(
            player_id,
            5,
            500,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(5.0, 0.0)),
        ),
    ]
}

fn suspicious_commands(player_id: PlayerId) -> Vec<InputCommand> {
    vec![
        input(
            player_id,
            1,
            100,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(1.0, 0.0)),
        ),
        input(
            player_id,
            2,
            200,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(25.0, 0.0)),
        ),
        input(
            player_id,
            3,
            300,
            Vec2::new(1.0, 0.0),
            true,
            Some(Vec2::new(3.0, 0.0)),
        ),
        input(
            player_id,
            4,
            400,
            Vec2::new(1.0, 0.0),
            true,
            Some(Vec2::new(4.0, 0.0)),
        ),
    ]
}

fn sequence_violation_commands(player_id: PlayerId) -> Vec<InputCommand> {
    vec![
        input(
            player_id,
            1,
            100,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(1.0, 0.0)),
        ),
        input(
            player_id,
            1,
            200,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(2.0, 0.0)),
        ),
    ]
}

fn timing_violation_commands(player_id: PlayerId) -> Vec<InputCommand> {
    vec![
        input(
            player_id,
            1,
            100,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(1.0, 0.0)),
        ),
        input(
            player_id,
            2,
            50,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(2.0, 0.0)),
        ),
        input(
            player_id,
            3,
            5_000,
            Vec2::new(1.0, 0.0),
            false,
            Some(Vec2::new(3.0, 0.0)),
        ),
    ]
}

fn flood_commands(player_id: PlayerId) -> Vec<InputCommand> {
    (1..=80)
        .map(|sequence| {
            input(
                player_id,
                sequence,
                sequence * 10,
                Vec2::new(1.0, 0.0),
                false,
                Some(Vec2::new(sequence as f32, 0.0)),
            )
        })
        .collect()
}

fn input(
    player_id: PlayerId,
    sequence: u64,
    client_time_ms: u64,
    movement: Vec2,
    fire: bool,
    claimed_position: Option<Vec2>,
) -> InputCommand {
    InputCommand {
        player_id,
        sequence,
        client_time_ms,
        movement,
        fire,
        claimed_position,
    }
}

fn to_invalid_data(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
