# Protocol Firewall

The server includes lightweight protocol-abuse controls before input validation.

## Controls

Current controls:

- protocol version validation
- maximum JSON line size
- per-connection message-rate limiting
- malformed JSON rejection

## Config

Config fields:

    max_line_bytes = 4096
    max_messages_per_second = 30

## Evidence

Protocol firewall events can produce evidence records:

    ProtocolViolation
    RateLimitViolation

## Bot scenarios

Flood test:

    cargo run -p bot -- flood

Bad protocol version test:

    cargo run -p bot -- bad-protocol

## Investigation

    cargo run -p cli -- summary samples/session.jsonl
    cargo run -p cli -- risk samples/session.jsonl
    cargo run -p cli -- timeline samples/session.jsonl
    cargo run -p cli -- evidence samples/session.jsonl