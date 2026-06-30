# Operations

The server uses an async Tokio runtime with structured logging.

## Runtime

The TCP server accepts clients asynchronously and creates one Tokio task per connection.

Each connection has a server-side connection ID. The ID is included in logs and telemetry.

## Logging

The server uses `tracing`.

Default log level:

    info

Enable debug logs:

    RUST_LOG=debug cargo run -p server

In Docker:

    RUST_LOG=debug HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose up server

## Graceful shutdown

The server listens for Ctrl+C.

On shutdown:

- the accept loop stops
- connection tasks receive a shutdown signal
- connection tasks write `ClientDisconnected`
- telemetry remains flushed through normal file writes

## Local run

    cargo run -p server

## Docker run

    just docker-demo