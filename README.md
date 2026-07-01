# Unity Rust Authoritative Security Sandbox

A defensive multiplayer-security sandbox for server-authoritative validation, telemetry, evidence generation, and investigation tooling.

This project models a small online-game security pipeline:

```text
untrusted client input
        ↓
authoritative Rust server
        ↓
server-side validation
        ↓
structured telemetry
        ↓
evidence records
        ↓
SQLite investigation database
        ↓
CLI / API / dashboard review
```

The goal is not to build a commercial anti-cheat product and not to publish offensive tooling. The goal is to demonstrate the engineering discipline behind multiplayer security systems: distrust client authority, validate gameplay claims on the server, preserve evidence, support investigator workflows, and document false-positive risks.

## What this demonstrates

- Server-authoritative multiplayer validation
- Movement validation
- Client timestamp validation
- Packet sequence validation
- Fire-rate validation
- Protocol version validation
- Per-connection rate limiting
- Authoritative hit-claim validation
- Structured JSONL telemetry
- Evidence export to JSON and CSV
- SQLite investigation database
- CLI investigation queries
- Read-only investigation API
- Static investigation dashboard
- Docker Compose demo
- CI-ready Rust workspace
- Defensive, lawful project scope

## What this intentionally does not include

This repository does not contain cheat loaders, game bypasses, DLL injection tooling, kernel code, DMA tooling, real-game offsets, credential theft, or instructions for attacking live services.

The abuse scenarios are synthetic and controlled. They exist only to test defensive validation and investigation workflows inside this toy sandbox.

## Repository structure

```text
crates/
  protocol/             Shared protocol types and telemetry event schema
  validation/           Pure validation logic and evidence conversion
  telemetry/            JSONL telemetry reader/writer
  investigation/        SQLite storage and investigation queries
  investigation-api/    Read-only HTTP API and dashboard hosting
  server/               Authoritative TCP game-security server
  bot/                  Controlled client scenarios
  cli/                  Operator and investigation commands

config/
  default.toml          Local server configuration
  docker.toml           Docker server configuration

dashboard/
  index.html            Static investigation dashboard
  styles.css            Dashboard styling
  dashboard.js          API-backed dashboard behavior

docs/
  architecture.md
  threat-model.md
  rule-catalog.md
  false-positives.md
  investigation-workflow.md
  release-process.md

scripts/
  docker-demo.sh        End-to-end Docker demonstration
```

## Quick start

Run all checks:

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run the full Docker demo:

```bash
just docker-demo
```

The demo starts the server, runs normal and suspicious clients, generates telemetry, exports evidence, ingests the investigation database, runs database queries, starts the API, and smoke-tests the dashboard endpoints.

Generated artifacts:

```text
samples/session.jsonl
reports/evidence.json
reports/evidence.csv
reports/investigation.db
```

## Local manual run

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

Then inspect telemetry:

```bash
cargo run -p cli -- summary samples/session.jsonl
cargo run -p cli -- risk samples/session.jsonl
cargo run -p cli -- timeline samples/session.jsonl
cargo run -p cli -- evidence samples/session.jsonl
```

Export evidence:

```bash
cargo run -p cli -- export-evidence samples/session.jsonl reports/evidence.json reports/evidence.csv
```

Create investigation database:

```bash
cargo run -p cli -- ingest-db samples/session.jsonl reports/investigation.db
```

Query the database:

```bash
cargo run -p cli -- query-db suspicious-players reports/investigation.db
cargo run -p cli -- query-db violation-breakdown reports/investigation.db
cargo run -p cli -- query-db player-timeline reports/investigation.db 2
```

Start the API and dashboard:

```bash
cargo run -p investigation-api -- serve reports/investigation.db 127.0.0.1:8080
```

Open:

```text
http://127.0.0.1:8080
```

API smoke test:

```bash
cargo run -p investigation-api -- smoke 127.0.0.1:8080
```

## Bot scenarios

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

Scenario purpose:

| Scenario | Purpose |
|---|---|
| `normal` | Baseline valid client behavior |
| `suspicious` | Speed and fire-rate validation |
| `sequence` | Repeated packet sequence rejection |
| `timing` | Invalid client timestamp behavior |
| `flood` | Message-rate limiter validation |
| `bad-protocol` | Protocol version rejection |
| `hit` | Valid hit-claim validation |
| `bad-hit` | Invalid hit-claim evidence |

## Detection rules

Current validation findings:

| Rule | Suspicion kind |
|---|---|
| Movement envelope exceeded | `SpeedHack` |
| Fire cooldown violated | `FireRateViolation` |
| Invalid state transition | `InvalidStateTransition` |
| Packet sequence did not increase | `PacketSequenceViolation` |
| Client timestamp invalid | `ClientTimeViolation` |
| Protocol message invalid or unsupported | `ProtocolViolation` |
| Connection message rate exceeded | `RateLimitViolation` |
| Hit claim failed server geometry | `HitValidationViolation` |

See:

```text
docs/rule-catalog.md
docs/false-positives.md
```

## Design principles

1. The client is untrusted.
2. The server owns authoritative state.
3. Validation emits evidence, not direct punishment.
4. Findings must be explainable.
5. False positives are treated as engineering failures.
6. Investigation tooling is part of the system, not an afterthought.
7. The repository remains lawful, defensive, and self-contained.

## Documentation

```text
docs/architecture.md              System architecture and data flow
docs/threat-model.md              Assets, trust boundaries, abuse cases, scope
docs/rule-catalog.md              Validation rules and evidence fields
docs/false-positives.md           Known false-positive risks and controls
docs/investigation-workflow.md    How evidence becomes reviewable information
docs/release-process.md           Quality gates and release workflow
SECURITY.md                       Safety boundary and responsible-use statement
CHANGELOG.md                      Versioned project history
```

## Status

Current feature-freeze target:

```text
v1.0.0  — final portfolio release
```