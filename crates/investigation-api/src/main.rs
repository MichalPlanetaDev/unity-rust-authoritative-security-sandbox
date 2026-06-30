use std::{
    error::Error,
    io::{self, Read, Write},
    net::TcpStream,
    path::PathBuf,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use investigation::InvestigationDatabase;
use protocol::PlayerId;
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Clone)]
struct AppState {
    database_path: PathBuf,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    database_path: String,
    event_count: usize,
    violation_count: usize,
}

#[derive(Debug, Serialize)]
struct SuspiciousPlayersResponse {
    players: Vec<SuspiciousPlayerDto>,
}

#[derive(Debug, Serialize)]
struct SuspiciousPlayerDto {
    player_id: u64,
    report_count: usize,
    severity_score: u32,
    last_seen_ms: u64,
}

#[derive(Debug, Serialize)]
struct ViolationBreakdownResponse {
    violations: Vec<ViolationBreakdownDto>,
}

#[derive(Debug, Serialize)]
struct ViolationBreakdownDto {
    violation_code: String,
    severity: String,
    count: usize,
    first_seen_ms: u64,
    last_seen_ms: u64,
}

#[derive(Debug, Serialize)]
struct PlayerTimelineResponse {
    player_id: u64,
    events: Vec<PlayerTimelineDto>,
}

#[derive(Debug, Serialize)]
struct PlayerTimelineDto {
    event_index: usize,
    event_type: String,
    server_time_ms: u64,
    connection_id: Option<u64>,
    sequence: Option<u64>,
    summary: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn internal(error: impl Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: error.to_string(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    if let Err(error) = run().await {
        error!(%error, "investigation api failed");
        std::process::exit(1);
    }
}

async fn run() -> AppResult<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        Some("serve") => {
            let database_path = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| "reports/investigation.db".to_string());

            let bind_addr = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:8080".to_string());

            serve(database_path.into(), &bind_addr).await
        }
        Some("smoke") => {
            let base_addr = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:8080".to_string());

            run_smoke_test(&base_addr)?;
            Ok(())
        }
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(unknown) => Err(format!("unknown command: {unknown}").into()),
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

fn print_help() {
    println!("Usage:");
    println!("  investigation-api serve reports/investigation.db 127.0.0.1:8080");
    println!("  investigation-api smoke 127.0.0.1:8080");
}

async fn serve(database_path: PathBuf, bind_addr: &str) -> AppResult<()> {
    let state = AppState {
        database_path: database_path.clone(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/players/suspicious", get(suspicious_players))
        .route("/violations/breakdown", get(violation_breakdown))
        .route("/players/:player_id/timeline", get(player_timeline))
        .with_state(state);

    let listener = TcpListener::bind(bind_addr).await?;

    info!(
        bind_addr,
        database_path = %database_path.display(),
        "investigation api listening"
    );

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    let database = open_database(&state)?;
    let health = database.database_health().map_err(ApiError::internal)?;

    Ok(Json(HealthResponse {
        status: "ok",
        database_path: state.database_path.display().to_string(),
        event_count: health.event_count,
        violation_count: health.violation_count,
    }))
}

async fn suspicious_players(
    State(state): State<AppState>,
) -> Result<Json<SuspiciousPlayersResponse>, ApiError> {
    let database = open_database(&state)?;
    let rows = database.suspicious_players().map_err(ApiError::internal)?;

    let players = rows
        .into_iter()
        .map(|row| SuspiciousPlayerDto {
            player_id: row.player_id.0,
            report_count: row.report_count,
            severity_score: row.severity_score,
            last_seen_ms: row.last_seen_ms,
        })
        .collect();

    Ok(Json(SuspiciousPlayersResponse { players }))
}

async fn violation_breakdown(
    State(state): State<AppState>,
) -> Result<Json<ViolationBreakdownResponse>, ApiError> {
    let database = open_database(&state)?;
    let rows = database.violation_breakdown().map_err(ApiError::internal)?;

    let violations = rows
        .into_iter()
        .map(|row| ViolationBreakdownDto {
            violation_code: row.violation_code,
            severity: row.severity,
            count: row.count,
            first_seen_ms: row.first_seen_ms,
            last_seen_ms: row.last_seen_ms,
        })
        .collect();

    Ok(Json(ViolationBreakdownResponse { violations }))
}

async fn player_timeline(
    State(state): State<AppState>,
    Path(player_id): Path<u64>,
) -> Result<Json<PlayerTimelineResponse>, ApiError> {
    if player_id == 0 {
        return Err(ApiError::bad_request("player id must be greater than zero"));
    }

    let database = open_database(&state)?;
    let rows = database
        .player_timeline(PlayerId(player_id))
        .map_err(ApiError::internal)?;

    let events = rows
        .into_iter()
        .map(|row| PlayerTimelineDto {
            event_index: row.event_index,
            event_type: row.event_type,
            server_time_ms: row.server_time_ms,
            connection_id: row.connection_id,
            sequence: row.sequence,
            summary: row.summary,
        })
        .collect();

    Ok(Json(PlayerTimelineResponse { player_id, events }))
}

fn open_database(state: &AppState) -> Result<InvestigationDatabase, ApiError> {
    InvestigationDatabase::open(&state.database_path).map_err(ApiError::internal)
}

fn run_smoke_test(base_addr: &str) -> io::Result<()> {
    let host = normalize_base_addr(base_addr);
    let paths = [
        "/health",
        "/players/suspicious",
        "/violations/breakdown",
        "/players/2/timeline",
    ];

    for path in paths {
        let response = get_with_retry(&host, path, Duration::from_secs(10))?;

        if !response.starts_with("HTTP/1.1 200") && !response.starts_with("HTTP/1.0 200") {
            return Err(io::Error::other(format!(
                "API smoke request failed for {path}: {}",
                response.lines().next().unwrap_or("<empty response>")
            )));
        }

        println!("API smoke ok: {path}");
    }

    Ok(())
}

fn get_with_retry(host: &str, path: &str, timeout: Duration) -> io::Result<String> {
    let started_at = Instant::now();
    let mut last_error = None;

    while started_at.elapsed() < timeout {
        match http_get(host, path) {
            Ok(response) => return Ok(response),
            Err(error) => {
                last_error = Some(error);
                std::thread::sleep(Duration::from_millis(250));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            format!("timed out waiting for API at {host}"),
        )
    }))
}

fn http_get(host: &str, path: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect(host)?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;

    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )?;
    stream.flush()?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    Ok(response)
}

fn normalize_base_addr(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_http_base_address() {
        assert_eq!(normalize_base_addr("http://api:8080/"), "api:8080");
    }

    #[test]
    fn keeps_plain_host_port() {
        assert_eq!(normalize_base_addr("127.0.0.1:8080"), "127.0.0.1:8080");
    }
}
