# Investigation Dashboard

The dashboard is a static HUD served by the investigation API.

## Purpose

The dashboard gives a visual investigation surface over the SQLite evidence database.

It consumes the read-only API instead of reading JSONL or SQLite directly.

## Run

Generate telemetry, evidence, and database first:

    just docker-demo

Or locally:

    cargo run -p cli -- ingest-db samples/session.jsonl reports/investigation.db

Start the API:

    cargo run -p investigation-api -- serve reports/investigation.db 127.0.0.1:8080

Open:

    http://127.0.0.1:8080

## Views

The dashboard shows:

- API health
- total events
- total violations
- suspicious players
- violation breakdown
- selected player timeline

## Design boundary

The dashboard is static HTML/CSS/JS.

The API remains responsible for data access.

The dashboard remains responsible for presentation only.