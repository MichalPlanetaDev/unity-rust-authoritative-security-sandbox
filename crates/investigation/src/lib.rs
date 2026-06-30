use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
};

use protocol::{ConnectionId, Milliseconds, PlayerId, TelemetryEvent};
use rusqlite::{Connection, OptionalExtension, params};
use validation::{EvidenceRecord, evidence_records_from_events};

pub type InvestigationResult<T> = Result<T, InvestigationError>;

#[derive(Debug)]
pub enum InvestigationError {
    Io(std::io::Error),
    Sql(rusqlite::Error),
    Json(serde_json::Error),
}

impl Display for InvestigationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Sql(error) => write!(formatter, "SQLite error: {error}"),
            Self::Json(error) => write!(formatter, "JSON error: {error}"),
        }
    }
}

impl Error for InvestigationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Sql(error) => Some(error),
            Self::Json(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for InvestigationError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for InvestigationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql(error)
    }
}

impl From<serde_json::Error> for InvestigationError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub struct InvestigationDatabase {
    connection: Connection,
}

impl InvestigationDatabase {
    pub fn open(path: impl AsRef<Path>) -> InvestigationResult<Self> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(path)?;

        connection.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            "#,
        )?;

        let database = Self { connection };
        database.migrate()?;

        Ok(database)
    }

    fn migrate(&self) -> InvestigationResult<()> {
        self.connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                event_id        INTEGER PRIMARY KEY AUTOINCREMENT,
                event_index     INTEGER NOT NULL UNIQUE,
                event_type      TEXT NOT NULL,
                server_time_ms  INTEGER NOT NULL,
                connection_id   INTEGER,
                player_id       INTEGER,
                sequence        INTEGER,
                summary         TEXT NOT NULL,
                raw_json        TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS violations (
                violation_id     INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id         INTEGER NOT NULL,
                player_id        INTEGER NOT NULL,
                sequence         INTEGER NOT NULL,
                violation_code   TEXT NOT NULL,
                severity         TEXT NOT NULL,
                reason           TEXT NOT NULL,
                observed_value   REAL NOT NULL,
                expected_limit   REAL NOT NULL,
                server_time_ms   INTEGER NOT NULL,
                FOREIGN KEY(event_id) REFERENCES events(event_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_events_player_time
                ON events(player_id, server_time_ms);

            CREATE INDEX IF NOT EXISTS idx_events_type_time
                ON events(event_type, server_time_ms);

            CREATE INDEX IF NOT EXISTS idx_violations_player
                ON violations(player_id);

            CREATE INDEX IF NOT EXISTS idx_violations_code
                ON violations(violation_code);

            CREATE INDEX IF NOT EXISTS idx_violations_time
                ON violations(server_time_ms);
            "#,
        )?;

        Ok(())
    }

    pub fn ingest_events(&mut self, events: &[TelemetryEvent]) -> InvestigationResult<usize> {
        let evidence_records = evidence_records_from_events(events);
        let transaction = self.connection.transaction()?;

        transaction.execute("DELETE FROM violations", [])?;
        transaction.execute("DELETE FROM events", [])?;

        for (event_index, event) in events.iter().enumerate() {
            let descriptor = EventDescriptor::from_event(event);
            let raw_json = serde_json::to_string(event)?;

            transaction.execute(
                r#"
                INSERT INTO events (
                    event_index,
                    event_type,
                    server_time_ms,
                    connection_id,
                    player_id,
                    sequence,
                    summary,
                    raw_json
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    event_index as i64,
                    descriptor.event_type,
                    descriptor.server_time_ms as i64,
                    descriptor.connection_id.map(|value| value as i64),
                    descriptor.player_id.map(|value| value.0 as i64),
                    descriptor.sequence.map(|value| value as i64),
                    descriptor.summary,
                    raw_json,
                ],
            )?;

            if let TelemetryEvent::Suspicion(report) = event {
                let record = EvidenceRecord::from_suspicion(report);
                let event_id = transaction.last_insert_rowid();

                insert_violation(&transaction, event_id, &record)?;
            }
        }

        transaction.commit()?;

        Ok(evidence_records.len())
    }

    pub fn event_count(&self) -> InvestigationResult<usize> {
        let count = self
            .connection
            .query_row("SELECT COUNT(*) FROM events", [], |row| {
                row.get::<_, i64>(0)
            })?;

        Ok(count as usize)
    }

    pub fn violation_count(&self) -> InvestigationResult<usize> {
        let count = self
            .connection
            .query_row("SELECT COUNT(*) FROM violations", [], |row| {
                row.get::<_, i64>(0)
            })?;

        Ok(count as usize)
    }

    pub fn suspicious_players(&self) -> InvestigationResult<Vec<SuspiciousPlayerRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                player_id,
                COUNT(*) AS report_count,
                SUM(
                    CASE severity
                        WHEN 'Critical' THEN 4
                        WHEN 'High' THEN 3
                        WHEN 'Medium' THEN 2
                        ELSE 1
                    END
                ) AS severity_score,
                MAX(server_time_ms) AS last_seen_ms
            FROM violations
            GROUP BY player_id
            ORDER BY severity_score DESC, report_count DESC, player_id ASC
            "#,
        )?;

        let rows = statement
            .query_map([], |row| {
                Ok(SuspiciousPlayerRow {
                    player_id: PlayerId(row.get::<_, i64>(0)? as u64),
                    report_count: row.get::<_, i64>(1)? as usize,
                    severity_score: row.get::<_, i64>(2)? as u32,
                    last_seen_ms: row.get::<_, i64>(3)? as Milliseconds,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn violation_breakdown(&self) -> InvestigationResult<Vec<ViolationBreakdownRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                violation_code,
                severity,
                COUNT(*) AS count,
                MIN(server_time_ms) AS first_seen_ms,
                MAX(server_time_ms) AS last_seen_ms
            FROM violations
            GROUP BY violation_code, severity
            ORDER BY count DESC, violation_code ASC
            "#,
        )?;

        let rows = statement
            .query_map([], |row| {
                Ok(ViolationBreakdownRow {
                    violation_code: row.get(0)?,
                    severity: row.get(1)?,
                    count: row.get::<_, i64>(2)? as usize,
                    first_seen_ms: row.get::<_, i64>(3)? as Milliseconds,
                    last_seen_ms: row.get::<_, i64>(4)? as Milliseconds,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn player_timeline(&self, player_id: PlayerId) -> InvestigationResult<Vec<TimelineDbRow>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                event_index,
                event_type,
                server_time_ms,
                connection_id,
                player_id,
                sequence,
                summary
            FROM events
            WHERE player_id = ?1
            ORDER BY server_time_ms ASC, event_index ASC
            "#,
        )?;

        let rows = statement
            .query_map(params![player_id.0 as i64], |row| {
                Ok(TimelineDbRow {
                    event_index: row.get::<_, i64>(0)? as usize,
                    event_type: row.get(1)?,
                    server_time_ms: row.get::<_, i64>(2)? as Milliseconds,
                    connection_id: row
                        .get::<_, Option<i64>>(3)?
                        .map(|value| value as ConnectionId),
                    player_id: row
                        .get::<_, Option<i64>>(4)?
                        .map(|value| PlayerId(value as u64)),
                    sequence: row.get::<_, Option<i64>>(5)?.map(|value| value as u64),
                    summary: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    pub fn database_health(&self) -> InvestigationResult<DatabaseHealth> {
        let schema_version = self
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .optional()?
            .unwrap_or(0);

        Ok(DatabaseHealth {
            schema_version,
            event_count: self.event_count()?,
            violation_count: self.violation_count()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuspiciousPlayerRow {
    pub player_id: PlayerId,
    pub report_count: usize,
    pub severity_score: u32,
    pub last_seen_ms: Milliseconds,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViolationBreakdownRow {
    pub violation_code: String,
    pub severity: String,
    pub count: usize,
    pub first_seen_ms: Milliseconds,
    pub last_seen_ms: Milliseconds,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineDbRow {
    pub event_index: usize,
    pub event_type: String,
    pub server_time_ms: Milliseconds,
    pub connection_id: Option<ConnectionId>,
    pub player_id: Option<PlayerId>,
    pub sequence: Option<u64>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseHealth {
    pub schema_version: i64,
    pub event_count: usize,
    pub violation_count: usize,
}

struct EventDescriptor {
    event_type: &'static str,
    server_time_ms: Milliseconds,
    connection_id: Option<ConnectionId>,
    player_id: Option<PlayerId>,
    sequence: Option<u64>,
    summary: String,
}

impl EventDescriptor {
    fn from_event(event: &TelemetryEvent) -> Self {
        match event {
            TelemetryEvent::ClientConnected {
                connection_id,
                player_id,
                server_time_ms,
            } => Self {
                event_type: "ClientConnected",
                server_time_ms: *server_time_ms,
                connection_id: Some(*connection_id),
                player_id: Some(*player_id),
                sequence: None,
                summary: "client joined".to_string(),
            },
            TelemetryEvent::ClientDisconnected {
                connection_id,
                player_id,
                server_time_ms,
            } => Self {
                event_type: "ClientDisconnected",
                server_time_ms: *server_time_ms,
                connection_id: Some(*connection_id),
                player_id: *player_id,
                sequence: None,
                summary: "client disconnected".to_string(),
            },
            TelemetryEvent::CommandAccepted {
                command,
                server_time_ms,
            } => Self {
                event_type: "CommandAccepted",
                server_time_ms: *server_time_ms,
                connection_id: None,
                player_id: Some(command.player_id),
                sequence: Some(command.sequence),
                summary: format!(
                    "accepted input seq={} fire={} movement=({:.2}, {:.2})",
                    command.sequence, command.fire, command.movement.x, command.movement.y
                ),
            },
            TelemetryEvent::PlayerSnapshot(snapshot) => Self {
                event_type: "PlayerSnapshot",
                server_time_ms: snapshot.server_time_ms,
                connection_id: None,
                player_id: Some(snapshot.player_id),
                sequence: None,
                summary: format!(
                    "snapshot position=({:.2}, {:.2}) health={} alive={}",
                    snapshot.position.x, snapshot.position.y, snapshot.health, snapshot.alive
                ),
            },
            TelemetryEvent::Suspicion(report) => Self {
                event_type: "Suspicion",
                server_time_ms: report.server_time_ms,
                connection_id: None,
                player_id: Some(report.player_id),
                sequence: Some(report.sequence),
                summary: format!(
                    "suspicion kind={:?} observed={:.3} limit={:.3}",
                    report.kind, report.observed_value, report.expected_limit
                ),
            },
        }
    }
}

fn insert_violation(
    transaction: &rusqlite::Transaction<'_>,
    event_id: i64,
    record: &EvidenceRecord,
) -> InvestigationResult<()> {
    transaction.execute(
        r#"
        INSERT INTO violations (
            event_id,
            player_id,
            sequence,
            violation_code,
            severity,
            reason,
            observed_value,
            expected_limit,
            server_time_ms
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            event_id,
            record.player_id.0 as i64,
            record.sequence as i64,
            format!("{:?}", record.violation_code),
            format!("{:?}", record.severity),
            record.reason,
            record.observed_value,
            record.expected_limit,
            record.server_time_ms as i64,
        ],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::{SuspicionKind, SuspicionReport};

    #[test]
    fn ingests_suspicion_into_database() {
        let mut database = InvestigationDatabase::open(":memory:").expect("database opens");
        let events = vec![TelemetryEvent::Suspicion(SuspicionReport::new(
            PlayerId(7),
            3,
            SuspicionKind::SpeedHack,
            "test speed violation",
            20.0,
            1.15,
            900,
        ))];

        let evidence_count = database
            .ingest_events(&events)
            .expect("events should ingest");

        assert_eq!(evidence_count, 1);
        assert_eq!(database.event_count().expect("event count"), 1);
        assert_eq!(database.violation_count().expect("violation count"), 1);
    }

    #[test]
    fn queries_suspicious_players() {
        let mut database = InvestigationDatabase::open(":memory:").expect("database opens");
        let events = vec![TelemetryEvent::Suspicion(SuspicionReport::new(
            PlayerId(2),
            1,
            SuspicionKind::ProtocolViolation,
            "bad protocol",
            1000.0,
            1.0,
            100,
        ))];

        database
            .ingest_events(&events)
            .expect("events should ingest");

        let players = database.suspicious_players().expect("players should query");

        assert_eq!(players.len(), 1);
        assert_eq!(players[0].player_id, PlayerId(2));
        assert_eq!(players[0].report_count, 1);
    }

    #[test]
    fn queries_violation_breakdown() {
        let mut database = InvestigationDatabase::open(":memory:").expect("database opens");
        let events = vec![TelemetryEvent::Suspicion(SuspicionReport::new(
            PlayerId(5),
            8,
            SuspicionKind::RateLimitViolation,
            "rate exceeded",
            31.0,
            30.0,
            200,
        ))];

        database
            .ingest_events(&events)
            .expect("events should ingest");

        let breakdown = database
            .violation_breakdown()
            .expect("breakdown should query");

        assert_eq!(breakdown.len(), 1);
        assert_eq!(breakdown[0].violation_code, "RateLimitViolation");
    }
}
