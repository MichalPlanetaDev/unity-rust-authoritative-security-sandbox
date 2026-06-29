# Network Timing Validation

The server validates client-side timestamps for accepted input commands.

## Detection

The server tracks the previous client timestamp per player.

It reports `ClientTimeViolation` when:

- a client timestamp does not increase
- a client timestamp jumps forward more than the configured maximum step

## Config

Default config:

    config/default.toml

Field:

    max_client_time_step_ms = 250

## Bot scenario

Run:

    cargo run -p bot -- timing

This sends:

- one normal timestamp
- one backwards timestamp
- one large forward timestamp jump

## Investigation

Summarize telemetry:

    cargo run -p cli -- summary samples/session.jsonl

Show player risk:

    cargo run -p cli -- risk samples/session.jsonl

Replay timeline:

    cargo run -p cli -- timeline samples/session.jsonl