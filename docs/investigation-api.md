# Investigation API

The investigation API exposes read-only endpoints over the SQLite investigation database.

## Purpose

The API is the boundary between backend investigation data and future UI/dashboard work.

The dashboard should not read JSONL telemetry or SQLite directly. It should call this API.

## Run

First create the database:

    cargo run -p cli -- ingest-db samples/session.jsonl reports/investigation.db

Start the API:

    cargo run -p investigation-api -- serve reports/investigation.db 127.0.0.1:8080

Smoke test:

    cargo run -p investigation-api -- smoke 127.0.0.1:8080

## Endpoints

Health:

    GET /health

Suspicious players:

    GET /players/suspicious

Violation breakdown:

    GET /violations/breakdown

Player timeline:

    GET /players/:player_id/timeline

Example:

    GET /players/2/timeline

## Docker

The Docker demo starts the API and runs the smoke test after telemetry is ingested into SQLite.