# Investigation Workflow

This document explains how a suspicious client action becomes reviewable evidence.

The project separates validation from investigation. The server does not decide punishment. The server validates gameplay requests, records findings, and exposes evidence through local investigation tools.

```text
client scenario
        ↓
server validation
        ↓
telemetry event
        ↓
evidence record
        ↓
database ingestion
        ↓
CLI / API / dashboard review
```

## Purpose

The investigation workflow exists because a validation finding is not enough by itself.

A useful anti-cheat signal must answer:

```text
what happened
who was involved
when it happened
which rule triggered
what value was observed
what limit was expected
what server-side context existed
whether the finding is isolated or repeated
how a reviewer can inspect it
```

The sandbox implements a small version of this workflow with JSONL telemetry, evidence export, SQLite ingestion, CLI queries, API endpoints, and a static dashboard.

## Step 1: Generate telemetry

Run the complete Docker demo:

```bash
just docker-demo
```

Or run manually.

Terminal 1:

```bash
rm -f samples/session.jsonl
cargo run -p server
```

Terminal 2:

```bash
cargo run -p bot -- normal
cargo run -p bot -- suspicious
cargo run -p bot -- sequence
cargo run -p bot -- timing
cargo run -p bot -- flood
cargo run -p bot -- bad-protocol
cargo run -p bot -- hit
cargo run -p bot -- bad-hit
```

Telemetry is written to:

```text
samples/session.jsonl
```

The telemetry file is line-delimited JSON. Each line is one event.

## Step 2: Summarize the session

Run:

```bash
cargo run -p cli -- summary samples/session.jsonl
```

This gives a high-level view of the session:

```text
total events
client connections
client disconnections
accepted commands
player snapshots
suspicion reports
suspicion kind breakdown
```

The summary is useful as a first check that the demo produced the expected event types.

## Step 3: Review risk triage

Run:

```bash
cargo run -p cli -- risk samples/session.jsonl
```

The risk view groups suspicion reports by player and applies simple weights.

The risk score is only triage information. It is not an enforcement decision.

A high score means the player has more or stronger findings in the synthetic session. It does not prove intent.

## Step 4: Inspect the timeline

Run:

```bash
cargo run -p cli -- timeline samples/session.jsonl
```

The timeline prints events in server-time order.

Timeline review is useful because isolated findings can be misleading. A reviewer should inspect what happened before and after a validation failure.

Useful timeline questions:

```text
Did the player connect normally?
What was the last accepted command?
Which sequence number triggered?
Did multiple rules trigger?
Was the finding isolated?
Was there a repeated pattern?
Did the server reject the command?
```

## Step 5: Inspect evidence records

Run:

```bash
cargo run -p cli -- evidence samples/session.jsonl
```

Evidence records are derived from `Suspicion` telemetry events.

An evidence record contains:

```text
player ID
sequence number
violation code
severity
reason
observed value
expected limit
server time
```

This is the most important investigator-facing data model in the project.

The evidence record should describe the validation failure without making an unsupported accusation.

Good evidence language:

```text
PlayerId(8) submitted a hit claim for a missing target.
```

Bad evidence language:

```text
PlayerId(8) is definitely cheating.
```

## Step 6: Export evidence

Run:

```bash
cargo run -p cli -- export-evidence samples/session.jsonl reports/evidence.json reports/evidence.csv
```

This creates:

```text
reports/evidence.json
reports/evidence.csv
```

The JSON export is useful for structured review.

The CSV export is useful for spreadsheet inspection, reporting, or quick manual filtering.

## Step 7: Build the investigation database

Run:

```bash
cargo run -p cli -- ingest-db samples/session.jsonl reports/investigation.db
```

This creates:

```text
reports/investigation.db
```

The database contains normalized investigation data derived from the telemetry log.

Main tables:

```text
events
violations
```

The database makes the evidence queryable without repeatedly scanning the JSONL file.

## Step 8: Query suspicious players

Run:

```bash
cargo run -p cli -- query-db suspicious-players reports/investigation.db
```

This query answers:

```text
which synthetic players have findings
how many reports they have
what severity score they accumulated
when they were last seen
```

This is a triage view.

It is useful for deciding which player timeline to inspect first.

## Step 9: Query violation breakdown

Run:

```bash
cargo run -p cli -- query-db violation-breakdown reports/investigation.db
```

This query answers:

```text
which violation types appeared
how many times each appeared
what severity each violation has
when each violation first appeared
when each violation last appeared
```

This is useful for validating that the demo produced expected coverage.

For example, after the full Docker demo, `HitValidationViolation` should appear because the `bad-hit` bot sends invalid hit claims.

## Step 10: Query player timeline

Run:

```bash
cargo run -p cli -- query-db player-timeline reports/investigation.db 2
```

Replace `2` with another player ID when needed.

This query answers:

```text
what happened to this player
which events belong to the player
which sequences were involved
which validation findings appeared
what server time each event occurred at
```

This is the closest CLI view to a case timeline.

## Step 11: Start the API

Run:

```bash
cargo run -p investigation-api -- serve reports/investigation.db 127.0.0.1:8080
```

The API exposes read-only investigation endpoints:

```text
GET /health
GET /players/suspicious
GET /violations/breakdown
GET /players/:player_id/timeline
```

The API does not mutate evidence.

The API does not perform enforcement.

The API exists so the dashboard has a clean boundary and does not read SQLite directly.

## Step 12: Open the dashboard

Open:

```text
http://127.0.0.1:8080
```

The dashboard shows:

```text
API health
event count
violation count
suspicious player list
violation breakdown
selected player timeline
```

The dashboard is an investigator-facing review surface.

It is not the source of truth. The source of truth is the server-generated telemetry and derived investigation database.

## Docker workflow

The Docker demo runs the whole path:

```text
build images
start server
run synthetic clients
generate telemetry
export evidence
ingest SQLite database
run database queries
start investigation API
run API smoke test
```

Run:

```bash
just docker-demo
```

Expected generated files:

```text
samples/session.jsonl
reports/evidence.json
reports/evidence.csv
reports/investigation.db
```

## Evidence lifecycle

The intended evidence lifecycle is:

```text
server observes behavior
        ↓
validator checks authoritative state
        ↓
validator emits finding
        ↓
telemetry records finding
        ↓
evidence model normalizes finding
        ↓
database stores queryable violation
        ↓
CLI/API/dashboard expose review views
```

This lifecycle keeps the project conservative.

The server does not jump from finding to punishment.

## Review principles

A reviewer should ask:

```text
What rule triggered?
What exact value was observed?
What exact limit was expected?
What player state existed before the finding?
What sequence number was involved?
Was the finding repeated?
Was this a synthetic scenario?
Could a known false-positive path explain it?
Does the timeline support the finding?
```

The evidence should be clear enough that another engineer can reproduce and understand the finding.

## Current workflow limits

The current workflow is intentionally small.

It does not include:

```text
reviewer accounts
case assignment
moderator notes
appeal handling
rule rollout modes
shadow mode
alert-only mode
production enforcement
privacy approval workflow
audit log for human actions
```

Those systems belong in a production platform. This sandbox focuses on the technical path from server validation to reviewable evidence.

## Professional standard

The project should be judged by whether another engineer can:

```text
clone the repository
run the demo
inspect telemetry
export evidence
query the database
open the dashboard
understand why each finding occurred
read the false-positive notes
verify the project stays defensive and lawful
```

That is the purpose of the investigation workflow.