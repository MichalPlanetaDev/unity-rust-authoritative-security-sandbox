# Session Lifecycle Telemetry

The server records TCP client lifecycle events and command-level replay data.

## Events

    ClientConnected
    ClientDisconnected
    CommandAccepted
    PlayerSnapshot
    Suspicion

Each connection receives a server-side connection ID.

Accepted commands now include server time, so telemetry can be replayed as an ordered session timeline.

## Purpose

Connection lifecycle telemetry helps investigation tooling answer:

- how many clients connected
- whether a client disconnected cleanly
- which player ID was associated with a connection
- what input commands were accepted
- what snapshots were produced
- what suspicious behavior appeared during a session

## Example flow

    client connects
    client sends Join
    server writes ClientConnected
    client sends input commands
    server writes CommandAccepted / PlayerSnapshot / Suspicion
    client disconnects
    server writes ClientDisconnected

## CLI

Summarize telemetry:

    cargo run -p cli -- summary samples/session.jsonl

Show risk scoring:

    cargo run -p cli -- risk samples/session.jsonl

Replay timeline:

    cargo run -p cli -- timeline samples/session.jsonl