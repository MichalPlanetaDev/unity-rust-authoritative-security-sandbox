use std::{
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    thread,
    time::{Duration, Instant},
};

use protocol::{ClientMessage, InputCommand, PlayerId, Vec2};

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:4000";

fn main() {
    if let Err(error) = run() {
        eprintln!("bot failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let scenario = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "normal".to_string());

    let server_addr =
        std::env::var("SERVER_ADDR").unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string());

    let (player_id, commands) = match scenario.as_str() {
        "normal" => (PlayerId(1), normal_commands(PlayerId(1))),
        "suspicious" => (PlayerId(2), suspicious_commands(PlayerId(2))),
        "sequence" => (PlayerId(3), sequence_violation_commands(PlayerId(3))),
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
    println!("Scenario: {scenario}");
    println!("Player: {:?}", player_id);
    println!();

    let mut stream = connect_with_retry(&server_addr, Duration::from_secs(10))?;
    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);

    send_message(&mut stream, &mut reader, &ClientMessage::Join { player_id })?;

    for command in commands {
        send_message(&mut stream, &mut reader, &ClientMessage::Input(command))?;
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
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
    println!();
    println!("Environment:");
    println!("  SERVER_ADDR=127.0.0.1:4000");
}

fn send_message(
    stream: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
    message: &ClientMessage,
) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, message).map_err(to_invalid_data)?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response = String::new();
    reader.read_line(&mut response)?;

    println!("server -> {}", response.trim());

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
