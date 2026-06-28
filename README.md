# unity-rust-authoritative-security-sandbox

A multiplayer-security portfolio project combining a Unity client, Rust authoritative server, custom client/server protocol, telemetry, bot-driven suspicious traffic, and investigation tooling.

The first milestone implements the Rust networking backbone:

    Rust bot -> TCP JSON protocol -> Rust authoritative server -> JSONL telemetry -> CLI summary/risk

## Current milestone

- TCP server in Rust
- Line-delimited JSON protocol
- Server-authoritative movement validation
- Fire cooldown validation
- Packet sequence validation
- Suspicious bot scenarios
- JSONL telemetry output
- CLI telemetry summary
- CLI player risk scoring

## Run

Terminal 1:

    cargo run -p server

Terminal 2:

    cargo run -p bot -- suspicious
    cargo run -p cli -- summary samples/session.jsonl
    cargo run -p cli -- risk samples/session.jsonl

## Scope

This is a defensive portfolio project. It does not include cheats, bypasses, injectors, malware, commercial game reverse engineering, or instructions for attacking third-party software.