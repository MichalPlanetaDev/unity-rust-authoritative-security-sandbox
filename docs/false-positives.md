# False-Positive Analysis

A false positive occurs when legitimate behavior is classified as suspicious.

This project treats false positives as engineering failures. A validation finding should be explainable, reproducible, and supported by server-side context.

The sandbox does not ban, kick, suspend, or punish players. It produces reviewable evidence.

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

## General controls

The project uses these controls to reduce false-positive risk:

```text
server-authoritative state
explicit validation policies
structured evidence records
observed values
expected limits
rule-specific reasons
replayable synthetic scenarios
CLI investigation views
SQLite investigation queries
read-only API
dashboard timeline review
no automatic enforcement
```

The most important design principle is that a single finding is not treated as proof of intent.

## Movement validation

Movement validation currently checks whether a client-claimed position exceeds the server movement budget.

### Possible false-positive causes in real games

```text
packet loss
jitter
client prediction mismatch
server reconciliation correction
wrong movement mode
wrong grounded state
slope or terrain mismatch
jump-state mismatch
falling-state mismatch
vehicle-state mismatch
swimming-state mismatch
ladder-state mismatch
collision resolution edge case
server tick mismatch
unit conversion bug
outdated movement configuration
```

### Sandbox controls

```text
simplified 2D movement
fixed tick budget
explicit tolerance
server-owned player position
evidence with observed distance
evidence with allowed distance
```

### Production improvement

A production movement validator should use a movement envelope.

The envelope should consider:

```text
movement mode
acceleration
maximum speed
friction
gravity
jump state
fall state
grounded state
slope state
collision state
vehicle state
water state
ladder state
server tick delta
latency tolerance
recent authoritative history
```

A production rule should usually classify movement findings into levels such as corrected, suspicious, invalid, and insufficient context instead of treating every abnormal movement sample the same way.

## Fire-rate validation

Fire-rate validation checks whether a client fires before the authoritative cooldown expires.

### Possible false-positive causes in real games

```text
weapon configuration mismatch
reload-state mismatch
ammo-state mismatch
fire-mode mismatch
server/client cooldown disagreement
duplicated input command
queued input command
clock drift
server tick mismatch
old weapon data
```

### Sandbox controls

```text
server-owned cooldown
fixed cooldown policy
evidence with observed server time
evidence with expected next allowed fire time
no automatic enforcement
```

### Production improvement

A production fire-rate rule should attach more context:

```text
weapon ID
weapon configuration version
ammo state
reload state
fire mode
input sequence
server tick
cooldown source
rule version
```

A production detector should distinguish one duplicated fire input from sustained cooldown abuse.

## Packet sequence validation

Packet sequence validation checks whether command sequence numbers increase.

### Possible false-positive causes in real games

```text
out-of-order UDP delivery
packet retransmission
input replay after reconnect
duplicate input buffering
server migration
session resume
client rollback bug
old client version
network-layer edge case
```

### Sandbox controls

```text
TCP transport
strict monotonic sequence validation
clear rejection reason
evidence with observed sequence
evidence with expected next sequence
```

### Production improvement

A production protocol should define:

```text
acknowledgment behavior
reorder window
replay protection
reconnect behavior
input buffering rules
session identity
command expiration
```

Strict sequence validation is acceptable in this sandbox because the transport and scenarios are controlled. It would need more nuance in a real UDP-based game protocol.

## Client timestamp validation

Client timestamp validation checks whether the client timestamp moves backward or jumps too far forward.

### Possible false-positive causes in real games

```text
local clock drift
client frame stall
packet batching
jitter
temporary client performance problem
sleep/resume behavior
low-end hardware frame spikes
incorrect client time source
server tick alignment issue
```

### Sandbox controls

```text
maximum client time step
backwards timestamp detection
evidence with observed timestamp
evidence with expected timestamp or limit
```

### Production improvement

A production system should compare client timing against:

```text
server tick time
arrival time
RTT estimate
jitter estimate
packet-loss window
input sequence
client performance metrics when available
```

Client timestamp anomalies should usually be treated as supporting evidence, not as a standalone punishment reason.

## Protocol validation

Protocol validation checks whether the client sends messages the server can safely understand.

### Possible false-positive causes in real games

```text
old client version during rollout
partial deployment
corrupted local files
proxy or middleware issue
serialization bug
client update race
server version mismatch
```

### Sandbox controls

```text
protocol version check
maximum line size
JSON parse rejection
clear rejection reason
ProtocolViolation evidence when player context exists
```

### Production improvement

A production system should include:

```text
protocol compatibility windows
clear upgrade path
deployment monitoring
version negotiation where appropriate
safe error reporting
schema migration policy
```

Protocol validation protects the server, but old-client behavior should not automatically be interpreted as malicious.

## Rate limiting

Rate limiting checks whether a connection sends more messages than the configured limit.

### Possible false-positive causes in real games

```text
reconnect storm
packet batching
telemetry flush
client bug
poor network condition
retry loop
legitimate high-frequency burst
load balancer behavior
```

### Sandbox controls

```text
per-connection one-second window
configured message limit
evidence with observed message count
evidence with expected limit
rejection instead of punishment
```

### Production improvement

A production system should consider:

```text
token bucket
leaky bucket
per-message weights
per-endpoint limits
per-account limits
temporary backoff
disconnect policy
server health context
privacy-aware correlation
```

Rate-limit findings are often operational signals. They require context before being treated as abuse.

## Hit validation

Hit validation checks whether a client hit claim matches server-known geometry.

### Possible false-positive causes in real games

```text
off-by-one tick in lag compensation
stale target snapshot
wrong hitbox transform
animation pose mismatch
projectile interpolation error
line-of-sight mismatch
weapon spread mismatch
recoil-state mismatch
client/server weapon configuration mismatch
latency window too narrow
target interpolation mismatch
floating-point tolerance issue
```

### Sandbox controls

```text
deterministic 2D targets
server-side target table
non-empty direction check
maximum hit distance check
claimed distance check
ray-to-target distance check
explicit hit tolerance
no automatic enforcement
```

### Production improvement

A production combat validator should use:

```text
historical authoritative snapshots
lag-compensated rewind
hitbox state
line-of-sight checks
weapon-specific spread
weapon-specific recoil
projectile travel time
projectile gravity
animation state
latency-aware tolerance
server-known target state
```

Hit validation is one of the highest-risk areas for false positives if latency compensation and historical state are modeled incorrectly.

## Investigation controls

The project provides several ways to review findings:

```text
cargo run -p cli -- summary samples/session.jsonl
cargo run -p cli -- risk samples/session.jsonl
cargo run -p cli -- timeline samples/session.jsonl
cargo run -p cli -- evidence samples/session.jsonl
cargo run -p cli -- query-db suspicious-players reports/investigation.db
cargo run -p cli -- query-db violation-breakdown reports/investigation.db
cargo run -p cli -- query-db player-timeline reports/investigation.db 2
cargo run -p investigation-api -- serve reports/investigation.db 127.0.0.1:8080
```

The goal is to make findings reviewable rather than opaque.

## Reviewer checklist

Before trusting a finding, ask:

```text
What rule triggered?
What exact value was observed?
What exact limit was expected?
What server state existed before the event?
Was this finding isolated or repeated?
Was latency or jitter relevant?
Was the player in a special movement or combat state?
Was the protocol version expected?
Was the rule version correct?
Can the session be replayed?
Is there a known false-positive path?
Would the finding still be valid under production-level context?
```

## Enforcement principle

This sandbox does not implement enforcement.

A production system should support:

```text
shadow mode
alert-only mode
manual-review mode
rule rollback
rule versioning
reviewer notes
appeal handling
privacy review
audit logging
```

The correct standard is evidence first, review second, decision last.