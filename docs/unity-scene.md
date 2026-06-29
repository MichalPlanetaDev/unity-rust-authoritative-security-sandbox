# Unity Scene

The Unity scene connects to the Rust authoritative TCP server.

## Scene

    unity-client/Assets/Scenes/Main.unity

## Required objects

    RustNetwork
    Player
    ArenaFloor
    Main Camera
    Directional Light

## Components

`RustNetwork`:

    RustTcpClient

`Player`:

    AuthoritativePlayerClient
    SuspiciousInputTester

## Settings asset

    unity-client/Assets/ScriptableObjects/RustServerSettings.asset

Default values:

    Host = 127.0.0.1
    Port = 4000
    Player Id = 10

## Run

Start the Rust server first:

    cargo run -p server

Then open Unity and press Play.

## Controls

    WASD / Arrow keys = movement
    Space = fire
    F1 = impossible movement claim
    F2 = repeated fire input
    F3 = repeated sequence input

## Telemetry

After testing, inspect telemetry:

    cargo run -p cli -- summary samples/session.jsonl
    cargo run -p cli -- risk samples/session.jsonl