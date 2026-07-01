# Changelog

## v0.15.0 - Dashboard polish and screenshots

Presentation-focused release preparing the repository homepage and investigation HUD for final portfolio review.

Added:

```text
polished Investigation HUD layout
improved dashboard visual hierarchy
improved loading states
improved empty states
improved error states
selected-player highlighting
sorted suspicious-player triage
sorted violation breakdown
screenshot documentation
screenshots directory
dashboard overview screenshot
player timeline screenshot
recruiter-facing README homepage rewrite
```

Updated:

```text
dashboard/index.html
dashboard/styles.css
dashboard/dashboard.js
docs/screenshots.md
README.md
```

No backend detection systems were added in this milestone.

The purpose of this release is to make the project visually credible, easier to review from the GitHub homepage, and ready for final portfolio presentation.

## v0.14.0 - Architecture and portfolio documentation freeze

Documentation-focused release preparing the repository for recruiter review.

Added:

```text
README rewrite
architecture documentation
threat model
rule catalog
false-positive analysis
investigation workflow documentation
release process documentation
security policy
changelog cleanup based on Git history
```

No backend features were added in this milestone.

The purpose of this release is to make the project easier to review, run, audit, and understand without private explanation.

## v0.13.0 - Authoritative hit validation

Commit:

```text
afb65be added authoritative hit validation
```

Added server-side validation for client hit claims.

Implemented:

```text
HitClaim client message
HitAccepted server response
deterministic server-side test targets
target existence validation
non-empty hit direction validation
maximum hit distance validation
claimed distance validation
ray-to-target validation
HitValidationViolation evidence
hit bot scenario
bad-hit bot scenario
Docker demo coverage
hit-validation documentation
```

This milestone extended the server-authoritative validation model from movement, timing, sequencing, and protocol checks into combat-style claim validation.

## v0.12.0 - Investigation dashboard HUD

Commit:

```text
14f9dc8 added investigation dashboard HUD
```

Added a static dashboard served by the investigation API.

Implemented:

```text
API health view
event count view
violation count view
suspicious player list
violation breakdown
selected player timeline
dashboard static assets
dashboard smoke-test coverage
dashboard documentation
```

The dashboard consumes the read-only investigation API instead of reading telemetry or SQLite directly.

## v0.11.0 - Investigation API service

Commit:

```text
467f8dd added investigation API service
```

Added a read-only HTTP API over the SQLite investigation database.

Implemented:

```text
GET /health
GET /players/suspicious
GET /violations/breakdown
GET /players/:player_id/timeline
API smoke test
Docker demo integration
API documentation
```

This milestone created a clean boundary between stored investigation data and reviewer-facing tools.

## v0.10.0 - Investigation database and query layer

Commit:

```text
3a74533 added investigation database and query layer
```

Added SQLite-based investigation storage.

Implemented:

```text
investigation crate
SQLite schema creation
telemetry ingestion
events table
violations table
suspicious-player query
violation-breakdown query
player-timeline query
database health query
CLI database commands
Docker demo integration
investigation database documentation
```

This milestone made telemetry queryable instead of requiring direct JSONL inspection.

## v0.9.0 - Protocol firewall and rate limiting

Commit:

```text
20b631b added protocol firewall and rate limiting
```

Added defensive protocol-boundary controls.

Implemented:

```text
maximum line-size validation
invalid JSON protocol rejection
protocol violation reporting
per-connection message-rate limiting
RateLimitViolation evidence
bad-protocol scenario coverage
flood scenario coverage
Docker demo coverage
```

This milestone made the server more resilient against malformed clients and message flooding.

## v0.8.0 - Async server runtime and structured logging

Commit:

```text
c160a31 added async server runtime and structured logging
```

Moved the server runtime toward a more production-like asynchronous architecture.

Implemented:

```text
Tokio-based server runtime
asynchronous TCP client handling
structured tracing logs
connection task lifecycle
cleaner server observability
Docker-compatible runtime behavior
```

This milestone improved runtime structure, logging clarity, and concurrent client handling.

## v0.7.0 - Validation engine and evidence model

Commit:

```text
549716a added validation engine and evidence model
```

Added the core validation and evidence layer.

Implemented:

```text
validation crate
ValidationPolicy
PlayerValidationState
ValidationDecision
movement validation
fire-rate validation
packet sequence validation
client timestamp validation
SuspicionKind mapping
EvidenceRecord model
violation code mapping
severity mapping
evidence extraction from telemetry events
```

This milestone established the central project principle: validation findings should become structured evidence rather than direct punishment.

## v0.4.0 - Connection lifecycle telemetry

Commit:

```text
4462b89 added connection lifecycle telemetry
```

Added lifecycle visibility for client sessions.

Implemented:

```text
client connected telemetry
client disconnected telemetry
connection ID tracking
player ID association when available
server-time lifecycle events
connection lifecycle records in telemetry output
```

Related pre-release commits before this tag:

```text
535a8dd added playable Unity client scene
eb2af00 added Unity debug HUD
eb55f35 fix: run Docker services as host user
```

This milestone improved the project’s ability to reconstruct session activity from telemetry.

## v0.2.0 - Unity client networking scripts

Commit:

```text
1c8db20 added Unity client networking scripts
```

Added the first Unity-facing client integration layer.

Implemented:

```text
Unity networking scripts
Rust server settings
JSON protocol models
TCP client behavior
authoritative player client behavior
suspicious input tester
security sandbox HUD integration path
```

Related foundation commits before this tag:

```text
7cb6b88 added protocol and telemetry
6371d6f added TCP server bot and CLI investigation
cd2b438 added docker and CI networking demo
```

This milestone connected the Rust backend direction with the Unity client-side demonstration path.

## Pre-v0.2.0 - Project foundation

Commits:

```text
7cb6b88 added protocol and telemetry
6371d6f added TCP server bot and CLI investigation
cd2b438 added docker and CI networking demo
```

Established the first working foundation of the repository.

Implemented:

```text
Rust workspace structure
shared protocol crate
telemetry event model
JSONL telemetry writer
TCP server foundation
controlled bot client
CLI investigation foundation
Dockerfile
docker-compose.yml
CI networking demo
```

This early foundation created the basic path from client messages to server processing, telemetry generation, and local investigation tooling.

## Untagged development between v0.4.0 and v0.7.0

Commits:

```text
4e2472d added session replay timeline
4192f17 client timing anomaly detection + rebuilt docker images
```

Added timeline and timing-analysis work before the `v0.7.0` validation/evidence milestone.

Implemented:

```text
session replay timeline
timeline-oriented CLI inspection
client timing anomaly detection
Docker image rebuilds for updated demo behavior
```

These commits contributed to the later validation and evidence model.

## Untagged README update

Commit:

```text
5504c50 Update README.md
```

Updated repository documentation during early development.

## Current project status

The project is now in backend feature freeze.

Remaining planned milestones:

```text
v0.15.0  Dashboard polish and screenshots
v1.0.0   Final portfolio release
```

Future work should focus on presentation, screenshots, README clarity, dashboard polish, release quality, and final recruiter-facing polish.

New backend systems should not be added to this repository unless they fix a real bug or architectural problem.

## History source

This changelog is based on:

```bash
git tag --sort=v:refname
git log --oneline --decorate --reverse
```