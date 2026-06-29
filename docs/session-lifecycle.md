# Session Lifecycle Telemetry

The server records TCP client lifecycle events.

## Events

    ClientConnected
    ClientDisconnected

Each connection receives a server-side connection ID.

## Purpose

Connection lifecycle telemetry helps investigation tooling answer:

- how many clients connected
- whether a client disconnected cleanly
- which player ID was associated with a connection
- what happened during a bot or Unity test session

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