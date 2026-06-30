# Investigation Database

The investigation database stores telemetry and evidence in SQLite.

## Purpose

JSONL telemetry is good for append-only capture and replay.

SQLite is better for investigation workflows:

- finding suspicious players
- grouping violations
- querying player timelines
- powering dashboards
- preserving normalized evidence records

## Database path

Default local path:

    reports/investigation.db

## Ingest

    cargo run -p cli -- ingest-db samples/session.jsonl reports/investigation.db

Ingestion is idempotent for the selected database. Existing rows are cleared before new telemetry is inserted.

## Queries

Suspicious players:

    cargo run -p cli -- query-db suspicious-players reports/investigation.db

Violation breakdown:

    cargo run -p cli -- query-db violation-breakdown reports/investigation.db

Player timeline:

    cargo run -p cli -- query-db player-timeline reports/investigation.db 2

## Schema

Main tables:

    events
    violations

The `events` table stores normalized event metadata and raw JSON.

The `violations` table stores investigation-ready evidence records derived from suspicion reports.