use std::{collections::BTreeMap, io};

use protocol::{ConnectionId, InputCommand, Milliseconds, PlayerId, SuspicionKind, TelemetryEvent};
use telemetry::read_events;

fn main() {
    if let Err(error) = run() {
        eprintln!("cli failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        Some("summary") => {
            let Some(path) = args.get(1) else {
                eprintln!("missing telemetry path");
                std::process::exit(2);
            };

            print_summary(path)
        }
        Some("risk") => {
            let Some(path) = args.get(1) else {
                eprintln!("missing telemetry path");
                std::process::exit(2);
            };

            print_risk(path)
        }
        Some("timeline") => {
            let Some(path) = args.get(1) else {
                eprintln!("missing telemetry path");
                std::process::exit(2);
            };

            print_timeline(path)
        }
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(unknown) => {
            eprintln!("unknown command: {unknown}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!("Usage:");
    println!("  cargo run -p cli -- summary samples/session.jsonl");
    println!("  cargo run -p cli -- risk samples/session.jsonl");
    println!("  cargo run -p cli -- timeline samples/session.jsonl");
}

fn print_summary(path: &str) -> io::Result<()> {
    let events = read_events(path)?;

    let mut connected = 0;
    let mut disconnected = 0;
    let mut accepted = 0;
    let mut snapshots = 0;
    let mut suspicions = 0;
    let mut by_kind = BTreeMap::new();

    for event in &events {
        match event {
            TelemetryEvent::ClientConnected { .. } => {
                connected += 1;
            }
            TelemetryEvent::ClientDisconnected { .. } => {
                disconnected += 1;
            }
            TelemetryEvent::CommandAccepted { .. } => {
                accepted += 1;
            }
            TelemetryEvent::PlayerSnapshot(_) => {
                snapshots += 1;
            }
            TelemetryEvent::Suspicion(report) => {
                suspicions += 1;
                *by_kind.entry(format!("{:?}", report.kind)).or_insert(0) += 1;
            }
        }
    }

    println!("Telemetry summary");
    println!();
    println!("File: {path}");
    println!("Total events: {}", events.len());
    println!("Client connections: {connected}");
    println!("Client disconnections: {disconnected}");
    println!("Accepted commands: {accepted}");
    println!("Snapshots: {snapshots}");
    println!("Suspicion reports: {suspicions}");
    println!();

    if by_kind.is_empty() {
        println!("No suspicious behavior detected.");
    } else {
        println!("Suspicion breakdown:");

        for (kind, count) in by_kind {
            println!("  {kind}: {count}");
        }
    }

    Ok(())
}

fn print_risk(path: &str) -> io::Result<()> {
    let events = read_events(path)?;
    let mut player_scores: BTreeMap<u64, u32> = BTreeMap::new();
    let mut player_counts: BTreeMap<u64, usize> = BTreeMap::new();
    let mut breakdown: BTreeMap<u64, BTreeMap<String, usize>> = BTreeMap::new();

    for event in events {
        let TelemetryEvent::Suspicion(report) = event else {
            continue;
        };

        let player_id = report.player_id.0;
        let kind_name = format!("{:?}", report.kind);

        *player_scores.entry(player_id).or_insert(0) += risk_weight(&report.kind);
        *player_counts.entry(player_id).or_insert(0) += 1;
        *breakdown
            .entry(player_id)
            .or_default()
            .entry(kind_name)
            .or_insert(0) += 1;
    }

    println!("Player risk summary");
    println!();

    if player_scores.is_empty() {
        println!("No suspicious behavior detected.");
        return Ok(());
    }

    for (player_id, raw_score) in player_scores {
        println!("PlayerId({player_id})");
        println!(
            "  Reports: {}",
            player_counts.get(&player_id).copied().unwrap_or(0)
        );
        println!("  Risk score: {}", raw_score.min(100));
        println!("  Breakdown:");

        if let Some(by_kind) = breakdown.get(&player_id) {
            for (kind, count) in by_kind {
                println!("    {kind}: {count}");
            }
        }

        println!();
    }

    Ok(())
}

fn print_timeline(path: &str) -> io::Result<()> {
    let events = read_events(path)?;

    println!("Session timeline");
    println!();
    println!("File: {path}");
    println!("Events: {}", events.len());
    println!();

    if events.is_empty() {
        println!("No telemetry events found.");
        return Ok(());
    }

    for event in events {
        let row = TimelineRow::from_event(event);
        row.print();
    }

    Ok(())
}

#[derive(Debug)]
struct TimelineRow {
    server_time_ms: Milliseconds,
    connection_id: Option<ConnectionId>,
    player_id: Option<PlayerId>,
    event_name: &'static str,
    detail: String,
}

impl TimelineRow {
    fn from_event(event: TelemetryEvent) -> Self {
        match event {
            TelemetryEvent::ClientConnected {
                connection_id,
                player_id,
                server_time_ms,
            } => Self {
                server_time_ms,
                connection_id: Some(connection_id),
                player_id: Some(player_id),
                event_name: "ClientConnected",
                detail: "joined".to_string(),
            },
            TelemetryEvent::ClientDisconnected {
                connection_id,
                player_id,
                server_time_ms,
            } => Self {
                server_time_ms,
                connection_id: Some(connection_id),
                player_id,
                event_name: "ClientDisconnected",
                detail: "disconnected".to_string(),
            },
            TelemetryEvent::CommandAccepted {
                command,
                server_time_ms,
            } => Self {
                server_time_ms,
                connection_id: None,
                player_id: Some(command.player_id),
                event_name: "CommandAccepted",
                detail: command_detail(&command),
            },
            TelemetryEvent::PlayerSnapshot(snapshot) => Self {
                server_time_ms: snapshot.server_time_ms,
                connection_id: None,
                player_id: Some(snapshot.player_id),
                event_name: "PlayerSnapshot",
                detail: format!(
                    "pos=({:.2}, {:.2}) health={} alive={}",
                    snapshot.position.x, snapshot.position.y, snapshot.health, snapshot.alive
                ),
            },
            TelemetryEvent::Suspicion(report) => Self {
                server_time_ms: report.server_time_ms,
                connection_id: None,
                player_id: Some(report.player_id),
                event_name: "Suspicion",
                detail: format!(
                    "kind={:?} seq={} observed={:.3} limit={:.3} reason={}",
                    report.kind,
                    report.sequence,
                    report.observed_value,
                    report.expected_limit,
                    report.reason
                ),
            },
        }
    }

    fn print(&self) {
        println!(
            "[{:08}ms] conn={} player={} {} {}",
            self.server_time_ms,
            format_connection_id(self.connection_id),
            format_player_id(self.player_id),
            self.event_name,
            self.detail
        );
    }
}

fn command_detail(command: &InputCommand) -> String {
    let claimed_position = match command.claimed_position {
        Some(position) => format!("claimed=({:.2}, {:.2})", position.x, position.y),
        None => "claimed=none".to_string(),
    };

    format!(
        "seq={} fire={} movement=({:.2}, {:.2}) {}",
        command.sequence, command.fire, command.movement.x, command.movement.y, claimed_position
    )
}

fn format_connection_id(connection_id: Option<ConnectionId>) -> String {
    match connection_id {
        Some(connection_id) => connection_id.to_string(),
        None => "-".to_string(),
    }
}

fn format_player_id(player_id: Option<PlayerId>) -> String {
    match player_id {
        Some(player_id) => format!("{player_id:?}"),
        None => "-".to_string(),
    }
}

fn risk_weight(kind: &SuspicionKind) -> u32 {
    match kind {
        SuspicionKind::SpeedHack => 40,
        SuspicionKind::FireRateViolation => 25,
        SuspicionKind::InvalidStateTransition => 35,
        SuspicionKind::PacketSequenceViolation => 20,
        SuspicionKind::ClientTimeViolation => 20,
    }
}
