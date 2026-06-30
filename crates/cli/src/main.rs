use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use investigation::InvestigationDatabase;
use protocol::{ConnectionId, InputCommand, Milliseconds, PlayerId, SuspicionKind, TelemetryEvent};
use telemetry::read_events;
use validation::{EvidenceRecord, evidence_records_from_events};

fn main() {
    if let Err(error) = run() {
        eprintln!("cli failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        Some("summary") => require_one_path(&args, print_summary),
        Some("risk") => require_one_path(&args, print_risk),
        Some("timeline") => require_one_path(&args, print_timeline),
        Some("evidence") => require_one_path(&args, print_evidence),
        Some("export-evidence") => {
            let Some(input_path) = args.get(1) else {
                eprintln!("missing telemetry path");
                std::process::exit(2);
            };

            let Some(json_path) = args.get(2) else {
                eprintln!("missing JSON output path");
                std::process::exit(2);
            };

            let Some(csv_path) = args.get(3) else {
                eprintln!("missing CSV output path");
                std::process::exit(2);
            };

            export_evidence(input_path, json_path, csv_path)
        }
        Some("ingest-db") => {
            let Some(input_path) = args.get(1) else {
                eprintln!("missing telemetry path");
                std::process::exit(2);
            };

            let Some(database_path) = args.get(2) else {
                eprintln!("missing database path");
                std::process::exit(2);
            };

            ingest_database(input_path, database_path)
        }
        Some("query-db") => {
            let Some(query_name) = args.get(1) else {
                eprintln!("missing database query name");
                std::process::exit(2);
            };

            query_database(query_name, &args[2..])
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

fn require_one_path(args: &[String], handler: fn(&str) -> io::Result<()>) -> io::Result<()> {
    let Some(path) = args.get(1) else {
        eprintln!("missing telemetry path");
        std::process::exit(2);
    };

    handler(path)
}

fn print_help() {
    println!("Usage:");
    println!("  cargo run -p cli -- summary samples/session.jsonl");
    println!("  cargo run -p cli -- risk samples/session.jsonl");
    println!("  cargo run -p cli -- timeline samples/session.jsonl");
    println!("  cargo run -p cli -- evidence samples/session.jsonl");
    println!(
        "  cargo run -p cli -- export-evidence samples/session.jsonl reports/evidence.json reports/evidence.csv"
    );
    println!("  cargo run -p cli -- ingest-db samples/session.jsonl reports/investigation.db");
    println!("  cargo run -p cli -- query-db suspicious-players reports/investigation.db");
    println!("  cargo run -p cli -- query-db violation-breakdown reports/investigation.db");
    println!("  cargo run -p cli -- query-db player-timeline reports/investigation.db 2");
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

fn print_evidence(path: &str) -> io::Result<()> {
    let events = read_events(path)?;
    let records = evidence_records_from_events(&events);

    println!("Evidence records");
    println!();
    println!("File: {path}");
    println!("Records: {}", records.len());
    println!();

    if records.is_empty() {
        println!("No evidence records found.");
        return Ok(());
    }

    for record in records {
        println!(
            "player={:?} seq={} code={:?} severity={:?} observed={:.3} limit={:.3} time={}ms",
            record.player_id,
            record.sequence,
            record.violation_code,
            record.severity,
            record.observed_value,
            record.expected_limit,
            record.server_time_ms
        );
        println!("  reason={}", record.reason);
    }

    Ok(())
}

fn export_evidence(input_path: &str, json_path: &str, csv_path: &str) -> io::Result<()> {
    let events = read_events(input_path)?;
    let records = evidence_records_from_events(&events);

    write_evidence_json(json_path, &records)?;
    write_evidence_csv(csv_path, &records)?;

    println!("Exported evidence");
    println!();
    println!("Input: {input_path}");
    println!("JSON: {json_path}");
    println!("CSV: {csv_path}");
    println!("Records: {}", records.len());

    Ok(())
}

fn ingest_database(input_path: &str, database_path: &str) -> io::Result<()> {
    let events = read_events(input_path)?;
    let mut database = InvestigationDatabase::open(database_path).map_err(to_io_error)?;
    let evidence_count = database.ingest_events(&events).map_err(to_io_error)?;
    let health = database.database_health().map_err(to_io_error)?;

    println!("Investigation database ingested");
    println!();
    println!("Input: {input_path}");
    println!("Database: {database_path}");
    println!("Events: {}", health.event_count);
    println!("Violations: {}", health.violation_count);
    println!("Evidence records: {evidence_count}");

    Ok(())
}

fn query_database(query_name: &str, args: &[String]) -> io::Result<()> {
    match query_name {
        "suspicious-players" => {
            let Some(database_path) = args.first() else {
                eprintln!("missing database path");
                std::process::exit(2);
            };

            query_suspicious_players(database_path)
        }
        "violation-breakdown" => {
            let Some(database_path) = args.first() else {
                eprintln!("missing database path");
                std::process::exit(2);
            };

            query_violation_breakdown(database_path)
        }
        "player-timeline" => {
            let Some(database_path) = args.first() else {
                eprintln!("missing database path");
                std::process::exit(2);
            };

            let Some(player_id) = args.get(1) else {
                eprintln!("missing player id");
                std::process::exit(2);
            };

            let player_id = player_id.parse::<u64>().map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("invalid player id '{player_id}': {error}"),
                )
            })?;

            query_player_timeline(database_path, PlayerId(player_id))
        }
        unknown => {
            eprintln!("unknown database query: {unknown}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn query_suspicious_players(database_path: &str) -> io::Result<()> {
    let database = InvestigationDatabase::open(database_path).map_err(to_io_error)?;
    let rows = database.suspicious_players().map_err(to_io_error)?;

    println!("Suspicious players");
    println!();
    println!("Database: {database_path}");
    println!("Rows: {}", rows.len());
    println!();

    if rows.is_empty() {
        println!("No suspicious players found.");
        return Ok(());
    }

    for row in rows {
        println!(
            "player={:?} reports={} severity_score={} last_seen={}ms",
            row.player_id, row.report_count, row.severity_score, row.last_seen_ms
        );
    }

    Ok(())
}

fn query_violation_breakdown(database_path: &str) -> io::Result<()> {
    let database = InvestigationDatabase::open(database_path).map_err(to_io_error)?;
    let rows = database.violation_breakdown().map_err(to_io_error)?;

    println!("Violation breakdown");
    println!();
    println!("Database: {database_path}");
    println!("Rows: {}", rows.len());
    println!();

    if rows.is_empty() {
        println!("No violations found.");
        return Ok(());
    }

    for row in rows {
        println!(
            "code={} severity={} count={} first_seen={}ms last_seen={}ms",
            row.violation_code, row.severity, row.count, row.first_seen_ms, row.last_seen_ms
        );
    }

    Ok(())
}

fn query_player_timeline(database_path: &str, player_id: PlayerId) -> io::Result<()> {
    let database = InvestigationDatabase::open(database_path).map_err(to_io_error)?;
    let rows = database.player_timeline(player_id).map_err(to_io_error)?;

    println!("Database player timeline");
    println!();
    println!("Database: {database_path}");
    println!("Player: {:?}", player_id);
    println!("Rows: {}", rows.len());
    println!();

    if rows.is_empty() {
        println!("No timeline rows found.");
        return Ok(());
    }

    for row in rows {
        println!(
            "#{:04} [{:08}ms] type={} conn={} seq={} {}",
            row.event_index,
            row.server_time_ms,
            row.event_type,
            format_connection_id(row.connection_id),
            format_sequence(row.sequence),
            row.summary
        );
    }

    Ok(())
}

fn write_evidence_json(path: impl AsRef<Path>, records: &[EvidenceRecord]) -> io::Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, records).map_err(to_invalid_data)
}

fn write_evidence_csv(path: impl AsRef<Path>, records: &[EvidenceRecord]) -> io::Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "player_id,sequence,violation_code,severity,reason,observed_value,expected_limit,server_time_ms"
    )?;

    for record in records {
        writeln!(
            writer,
            "{},{},{:?},{:?},{},{:.3},{:.3},{}",
            record.player_id.0,
            record.sequence,
            record.violation_code,
            record.severity,
            csv_escape(&record.reason),
            record.observed_value,
            record.expected_limit,
            record.server_time_ms
        )?;
    }

    writer.flush()
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

fn format_sequence(sequence: Option<u64>) -> String {
    match sequence {
        Some(sequence) => sequence.to_string(),
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
        SuspicionKind::ProtocolViolation => 35,
        SuspicionKind::RateLimitViolation => 15,
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn to_io_error(error: impl std::error::Error + Send + Sync + 'static) -> io::Error {
    io::Error::other(error)
}

fn to_invalid_data(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
