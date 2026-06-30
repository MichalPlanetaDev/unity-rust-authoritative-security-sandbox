# Evidence Model

The evidence model converts suspicion reports into investigation records.

## Commands

Print evidence:

    cargo run -p cli -- evidence samples/session.jsonl

Export evidence:

    cargo run -p cli -- export-evidence samples/session.jsonl reports/evidence.json reports/evidence.csv

## Fields

Each evidence record contains:

- player ID
- sequence number
- violation code
- severity
- reason
- observed value
- expected limit
- server time

## Purpose

Evidence records are easier to use in investigations than raw telemetry alone.

Raw telemetry is useful for replay.

Evidence records are useful for reporting, triage, review, dashboards, and automated pipelines.