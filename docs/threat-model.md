# Threat Model

This document defines the security scope of the Unity Rust Authoritative Security Sandbox.

The project is a lawful, defensive, self-contained multiplayer-security laboratory. It models how an untrusted client can send suspicious gameplay requests to an authoritative server, and how the server can validate those requests, record evidence, and expose the result to investigation tools.

The project does not contain real-game cheat code, bypass tooling, injection tooling, kernel code, DMA tooling, credential theft, malware, or instructions for attacking live services.

## Scope

The sandbox focuses on these defensive areas:

```text
server-authoritative gameplay validation
protocol validation
rate limiting
structured telemetry
evidence generation
SQLite investigation storage
CLI/API/dashboard review
false-positive documentation
```

The project intentionally models only a small multiplayer-security slice. It does not attempt to become a commercial anti-cheat product.

## Assets

The main assets protected by this sandbox are:

```text
authoritative player state
server validation policy
session telemetry
evidence records
investigation database
rule meaning
reviewer trust
demo reproducibility
repository safety boundary
```

In a production game, additional assets would include account reputation, inventories, game economy, server configuration, moderation actions, private player data, signing keys, rule bundles, build artifacts, admin tools, and enforcement history.

## Attacker model

The attacker controls the client.

That means the server must treat the following as untrusted:

```text
position claims
movement intent
client timestamps
command sequence numbers
fire requests
hit claims
target identifiers
claimed hit distances
protocol version
message size
message rate
JSON message structure
```

The attacker may attempt to send impossible or malformed requests.

The attacker does not control the authoritative server state, validation policy, telemetry writer, investigation database, or Docker demo script.

## Trust boundaries

The most important trust boundary is between the client and the server.

The client can submit requests. The server decides whether those requests are valid.

The client is not trusted to decide final gameplay state.

Trusted side:

```text
authoritative server state
validation policy
server clock
server-side target table
telemetry writer
evidence conversion
investigation database
operator CLI
read-only API
dashboard presentation
```

Untrusted side:

```text
client-provided position
client-provided time
client-provided sequence
client-provided hit target
client-provided hit distance
client-provided direction vector
client-provided protocol version
client-provided message body
```

## Abuse cases

### Impossible movement

A client claims a position that exceeds the server movement envelope.

Current mitigation:

```text
server-side movement budget
movement tolerance
SpeedHack evidence
```

The sandbox logs the observed distance and the expected allowed distance.

### Fire cooldown violation

A client sends a fire command before the authoritative cooldown expires.

Current mitigation:

```text
server-owned fire cooldown
FireRateViolation evidence
```

The sandbox logs the observed server time and the next allowed fire time.

### Packet replay or stale command

A client repeats a sequence number or sends a sequence number lower than the last accepted command.

Current mitigation:

```text
monotonic sequence validation
PacketSequenceViolation evidence
```

The sandbox rejects stale input and records the expected sequence boundary.

### Client timestamp manipulation

A client sends timestamps that move backward or jump too far forward.

Current mitigation:

```text
client timestamp monotonicity check
maximum client time step
ClientTimeViolation evidence
```

The sandbox records the observed timestamp or delta and the expected limit.

### Protocol mismatch

A client joins with an unsupported protocol version.

Current mitigation:

```text
protocol version validation
ProtocolViolation evidence
```

The sandbox rejects unsupported versions and records the observed and expected protocol values.

### Malformed protocol message

A client sends invalid JSON or an oversized message line.

Current mitigation:

```text
JSON parse validation
maximum line size
ProtocolViolation evidence when player context exists
```

The sandbox rejects malformed messages without trusting their content.

### Connection flood

A client sends too many messages within a one-second window.

Current mitigation:

```text
per-connection message counter
configured messages-per-second limit
RateLimitViolation evidence
```

The sandbox records the observed message count and the configured limit.

### Invalid hit claim

A client claims a hit that does not match server geometry.

Current mitigation:

```text
target existence validation
non-empty direction validation
maximum hit distance validation
claimed distance validation
ray-to-target validation
HitValidationViolation evidence
```

The sandbox uses deterministic 2D test targets. This proves the server-authoritative validation path without implementing a full combat system.

## Synthetic attacker scenarios

The `bot` crate provides controlled local scenarios:

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

These scenarios are not offensive tooling. They are synthetic test clients for this repository’s own local server.

## Out of scope

The following are intentionally out of scope:

```text
real game memory scanning
real cheat detection signatures
DLL injection
anti-cheat bypasses
kernel-mode anti-cheat
DMA analysis
hypervisor detection
malware analysis
credential theft
attacks against live games
production ban logic
account enforcement
moderator authentication
privacy/legal production review
```

These topics should not be forced into this repository. Future projects can cover safe native-code analysis, Unity build security, or defensive reverse-engineering literacy separately.

## Security assumptions

The sandbox assumes:

```text
the operator controls the local machine
the server binary is trusted
configuration files are operator-controlled
telemetry files are generated by the server
the investigation database is generated from telemetry
all player IDs are synthetic
all scenarios are local and controlled
```

The sandbox does not defend against host compromise, malicious Docker daemon behavior, malicious local filesystem access, malicious CI infrastructure, or production-scale adversaries.

## Privacy boundary

The project uses synthetic telemetry only.

It does not collect real personal data.

Generated artifacts include:

```text
samples/session.jsonl
reports/evidence.json
reports/evidence.csv
reports/investigation.db
```

In a production anti-cheat system, telemetry collection would require data minimization, retention policy, access control, audit logging, privacy review, and clear internal governance.

## Enforcement boundary

This project does not ban, kick, suspend, or punish players.

The correct lifecycle is:

```text
validation finding
        ↓
evidence record
        ↓
investigation view
        ↓
review
        ↓
policy decision
```

A validation finding is not the same as an enforcement action.

This separation is intentional. It prevents the project from pretending that one suspicious event is enough for punishment.

## False-positive principle

False positives are treated as engineering failures.

A finding should be:

```text
explainable
reproducible
attached to observed values
attached to expected limits
connected to server-side state
reviewable through investigation tools
```

A production system would also need latency context, rule versioning, rollout modes, reviewer actions, appeal handling, and rollback procedures.

## Defensive-use statement

This repository is intended for defensive engineering education and portfolio demonstration.

It should remain:

```text
lawful
self-contained
defensive
explainable
reproducible
safe to review
```

It should not be modified into offensive tooling against real games, players, anti-cheat systems, or services.