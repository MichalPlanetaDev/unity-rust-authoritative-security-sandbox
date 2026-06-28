# Unity Client

The Unity client connects to the Rust authoritative server over TCP.

Current scripts:

    unity-client/Assets/Scripts/Networking/RustServerSettings.cs
    unity-client/Assets/Scripts/Networking/RustProtocolModels.cs
    unity-client/Assets/Scripts/Networking/RustJsonMessages.cs
    unity-client/Assets/Scripts/Networking/RustTcpClient.cs
    unity-client/Assets/Scripts/Networking/AuthoritativePlayerClient.cs
    unity-client/Assets/Scripts/Networking/SuspiciousInputTester.cs

## Setup in Unity

1. Create or open a Unity project in `unity-client`.
2. Add an empty GameObject named `RustNetwork`.
3. Add the `RustTcpClient` component.
4. Create a `RustServerSettings` asset from:
   `Create -> Security Sandbox -> Rust Server Settings`.
5. Assign the settings asset to `RustTcpClient`.
6. Add a player object, for example a Capsule.
7. Add `AuthoritativePlayerClient` to the player object.
8. Assign the `RustTcpClient` object and `RustServerSettings` asset.
9. Optional: add `SuspiciousInputTester` to the same player object.

## Local Rust server

Run the Rust server before starting Play Mode:

    cargo run -p server

## Controls

Movement:

    WASD / Arrow keys

Fire:

    Space

Suspicious test inputs:

    F1 = impossible movement claim
    F2 = repeated fire input
    F3 = repeated sequence input

## Expected behavior

The Unity client sends input intentions to the Rust server.

The Rust server owns authoritative state and returns snapshots.

Suspicious Unity inputs are written to:

    samples/session.jsonl

Use the CLI to inspect telemetry:

    cargo run -p cli -- summary samples/session.jsonl
    cargo run -p cli -- risk samples/session.jsonl