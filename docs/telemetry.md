# Telemetry

Telemetry is stored as JSONL: one JSON event per line.

Current telemetry events:

- client connection
- accepted input command
- server-authoritative player snapshot
- suspicion report

JSONL is useful because it can be streamed, searched, replayed, summarized, and exported later.

Planned investigation commands:

    cargo run -p cli -- summary samples/session.jsonl
    cargo run -p cli -- risk samples/session.jsonl