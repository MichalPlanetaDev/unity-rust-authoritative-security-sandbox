# Rust-auth sandbox

A portfolio project combining a Unity client, Rust authoritative server, custom client/server protocol, telemetry, bot-driven suspicious traffic, Dockerized networking demos, and investigation tooling.

Current milestone:

    Rust bot -> TCP JSON protocol -> Rust authoritative server -> JSONL telemetry -> CLI summary/risk

Planned full pipeline:

    Unity client -> Rust authoritative server -> telemetry -> investigation tools

## Current status

- Rust TCP server
- Line-delimited JSON protocol
- Server-authoritative movement validation
- Fire cooldown validation
- Packet sequence validation
- Suspicious bot scenarios
- JSONL telemetry output
- CLI telemetry summary
- CLI player risk scoring
- Docker Compose networking demo
- Rust CI and Docker CI
- Connection lifecycle telemetry
- Server-side session replay timeline
- Client timestamp anomaly detection
- Timing violation bot scenario
- Dedicated validation engine crate
- Evidence model with JSON/CSV export
- Async Tokio TCP server
- Structured tracing logs
- Graceful shutdown handling

## Architecture

    bot-normal / bot-suspicious / bot-sequence
        -> TCP JSON messages
            -> server
                -> JSONL telemetry
                    -> cli summary / cli risk

Telemetry includes client connection, disconnection, accepted commands, snapshots, and suspicion reports.

## Local run

Terminal 1:

    cargo run -p server

Terminal 2:

    cargo run -p bot -- normal
    cargo run -p bot -- suspicious
    cargo run -p bot -- sequence
    cargo run -p cli -- summary samples/session.jsonl
    cargo run -p cli -- risk samples/session.jsonl

Replay ordered telemetry timeline:

    cargo run -p cli -- timeline samples/session.jsonl

Timing anomaly bot:

    cargo run -p bot -- timing

Evidence inspection:

    cargo run -p cli -- evidence samples/session.jsonl

Evidence export:

    cargo run -p cli -- export-evidence samples/session.jsonl reports/evidence.json reports/evidence.csv

Structured logging:

    RUST_LOG=debug cargo run -p server

Graceful shutdown:

    Press Ctrl+C while the server is running.

## Docker run

    just docker-demo

or:

    ./scripts/docker-demo.sh

## Unity client

The repository includes first-pass Unity TCP client scripts under:

    unity-client/Assets/Scripts/Networking

The Unity client can connect to the Rust server, send movement/fire input, receive authoritative snapshots, and send controlled suspicious inputs for local testing.

See:

    docs/unity-client.md

## Unity scene

The repository includes a playable Unity client scene under:

    unity-client/Assets/Scenes/Main.unity

The scene connects to the Rust TCP server, sends movement/fire input, receives authoritative snapshots, and can generate controlled suspicious inputs for telemetry testing.

See:

    docs/unity-scene.md

## Config

Local server config:

    config/default.toml

Docker server config:

    config/docker.toml

## Scope

This is a portfolio project.

It does not include cheats, bypasses, injectors, malware, commercial game reverse engineering, or instructions for attacking third-party software.
