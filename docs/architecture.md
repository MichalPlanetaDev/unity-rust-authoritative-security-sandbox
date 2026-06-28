# Architecture

Current pipeline:

    bot -> TCP JSON messages -> server -> JSONL telemetry -> cli

Planned full pipeline:

    Unity client -> Rust authoritative server -> telemetry -> investigation tools

The Unity client will send input intentions. The Rust server owns the authoritative game state and validates movement, fire rate, packet ordering, and suspicious client claims.