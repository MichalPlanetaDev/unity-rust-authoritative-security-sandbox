# Server

The `server` crate implements the Rust authoritative TCP server.

It listens on:

    127.0.0.1:4000

The bind address is configured in:

    config/default.toml

The server accepts line-delimited JSON client messages and returns line-delimited JSON server messages.

Current validation:

- movement budget validation
- fire cooldown validation
- packet sequence validation
- invalid state transition detection

Telemetry is written to:

    samples/session.jsonl