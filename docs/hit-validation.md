# Authoritative Hit Validation

The server validates client hit claims instead of trusting the client.

## Purpose

A client may claim that a shot hit a target.

The server validates:

- target existence
- maximum hit distance
- ray direction
- ray-to-target distance
- claimed distance consistency

Invalid claims produce:

    HitValidationViolation

## Protocol

Client message:

    HitClaim

Server response:

    HitAccepted
    Rejected

## Test targets

The server creates deterministic test targets:

    EntityId(1001) at (8.0, 0.0)
    EntityId(1002) at (0.0, 8.0)

These targets are intentionally simple. They exist to validate server-side geometry, not to implement a full gameplay system.

## Bot scenarios

Valid hit:

    cargo run -p bot -- hit

Invalid hit:

    cargo run -p bot -- bad-hit

## Investigation

Hit validation failures appear in:

    cargo run -p cli -- evidence samples/session.jsonl
    cargo run -p cli -- query-db violation-breakdown reports/investigation.db
    http://127.0.0.1:8080