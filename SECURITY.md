# Security Policy

This repository is a lawful defensive multiplayer-security sandbox.

It is intended to demonstrate server-authoritative validation, controlled suspicious-client scenarios, structured telemetry, evidence generation, and investigation workflows.

It is not intended to help attack games, players, anti-cheat systems, services, accounts, or production infrastructure.

## Defensive scope

The project demonstrates:

```text
server-authoritative validation
movement validation
fire-rate validation
packet sequence validation
client timestamp validation
protocol validation
rate limiting
hit-claim validation
structured telemetry
evidence export
SQLite investigation queries
read-only API access
dashboard review
Docker reproducibility
```

All suspicious behavior is synthetic and targets only this repository’s local toy server.

## Explicitly prohibited use

Do not use this project to build, publish, or explain:

```text
cheat loaders
game bypasses
DLL injectors
kernel evasion tools
DMA tooling
credential theft
malware
real-game memory scanning
real-game offsets
attacks against live games
attacks against live services
instructions for evading real anti-cheat systems
```

The repository should remain safe to review by engineers, recruiters, and security-minded readers.

## Synthetic bot scenarios

The `bot` crate contains controlled scenarios such as:

```text
normal
suspicious
sequence
timing
flood
bad-protocol
hit
bad-hit
```

These scenarios are allowed because they target only the local sandbox server.

They exist to test defensive validation and evidence generation.

They are not intended to model or distribute real cheat tooling.

## Reporting safety concerns

Treat a change as a safety concern if it introduces:

```text
real-game targeting
bypass or evasion logic
injection tooling
credential or token leakage
unsafe default configuration
excessive data collection
unclear privacy boundary
dashboard or API mutation without access control
instructions that could help attack live services
```

The correct fix is to remove or redesign the unsafe behavior.

## Data handling

The project uses synthetic player IDs and synthetic telemetry.

Generated local artifacts include:

```text
samples/session.jsonl
reports/evidence.json
reports/evidence.csv
reports/investigation.db
```

These files should not contain real personal data.

A production anti-cheat system would require data minimization, retention limits, access control, privacy review, audit logging, and internal governance.

## Secrets

Do not commit secrets.

Do not commit:

```text
API keys
tokens
passwords
private keys
database credentials
signing keys
real player data
private telemetry
```

This repository should run locally without production secrets.

## Enforcement boundary

This project does not implement player punishment.

It does not ban, kick, suspend, or penalize players.

Validation findings are reviewable evidence, not automatic enforcement actions.

The intended lifecycle is:

```text
finding
        ↓
evidence
        ↓
investigation
        ↓
review
        ↓
decision
```

## API and dashboard boundary

The investigation API is read-only.

The dashboard is a presentation layer over the API.

If future work adds mutation, reviewer accounts, case management, or enforcement recommendations, that work must include authentication, authorization, audit logging, and privacy review.

## Maintainer rule

The repository should remain:

```text
lawful
defensive
self-contained
explainable
reproducible
safe to review
```

Do not expand it into offensive tooling or real-game targeting.