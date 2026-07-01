# Rule Catalog

This document describes the validation findings currently produced by the sandbox.

A rule finding is evidence. It is not a ban decision, kick decision, suspension decision, or final accusation.

The project intentionally separates validation from enforcement.

```text
client behavior
        ↓
server validation
        ↓
finding
        ↓
evidence record
        ↓
investigation
        ↓
review
```

## Severity model

Evidence records currently use this severity model:

```text
Low       informational or weak signal
Medium    suspicious behavior requiring context
High      strong server-side validation failure
Critical  reserved for severe production-only conditions
```

The current sandbox mainly emits `Medium` and `High` severity findings.

Severity is not a punishment level. It is a review-prioritization signal.

## SpeedHack

### Purpose

`SpeedHack` detects claimed movement that exceeds the server movement envelope.

The client may send a claimed position, but the server decides whether that claim is plausible.

### Trigger

The rule triggers when the distance between the current authoritative server position and the client-claimed position exceeds the allowed movement budget plus tolerance.

Current simplified formula:

```text
allowed_distance =
    max_speed_units_per_second * fixed_tick_ms / 1000
    + movement_tolerance_units
```

If the observed distance is greater than the allowed distance, the server emits `SpeedHack`.

### Evidence

The evidence records:

```text
player ID
sequence number
observed distance
allowed distance
server time
reason
```

### Example reason

```text
claimed position exceeded server movement budget
```

### Current limitations

The sandbox uses simplified 2D movement.

It does not model:

```text
acceleration curves
jumping
falling
slopes
terrain
collision resolution
vehicles
swimming
ladders
crouching
sprinting
server reconciliation
packet loss windows
```

### False-positive risk

Low inside the sandbox because movement is deterministic and simple.

Higher in a production game if movement modes, latency, collision, or reconciliation are modeled incorrectly.

## FireRateViolation

### Purpose

`FireRateViolation` detects firing before the authoritative server cooldown expires.

The client may request to fire. The server owns the cooldown state.

### Trigger

The rule triggers when:

```text
command.fire == true
server_time_ms < next_allowed_fire_time_ms
```

### Evidence

The evidence records:

```text
player ID
sequence number
observed server time
expected next allowed fire time
server time
reason
```

### Example reason

```text
fire input arrived before cooldown expired
```

### Current limitations

The sandbox does not model:

```text
weapon classes
weapon inventory
ammo
reloads
attachments
spread
recoil
fire modes
server-authoritative weapon configuration
```

### False-positive risk

Low inside the sandbox.

Higher in production if the server and client disagree about weapon state, reload state, cooldown configuration, or command timing.

## InvalidStateTransition

### Purpose

`InvalidStateTransition` detects actions that are impossible under the current authoritative player state.

### Trigger

Current trigger examples:

```text
dead player sends input
dead player submits hit claim
```

### Evidence

The evidence records:

```text
player ID
sequence number
observed invalid state
expected valid state
server time
reason
```

### Example reasons

```text
dead player attempted to send input
dead player attempted to submit hit claim
```

### Current limitations

The sandbox currently models only a small player state:

```text
position
health
alive flag
last sequence
last client timestamp
next allowed fire time
```

A production game would need richer state machines for movement, combat, inventory, vehicles, building, spectating, sleeping, respawning, and disconnect/reconnect behavior.

### False-positive risk

Low inside the sandbox.

Higher in production if state transitions are not explicit or if reconnect, respawn, spectator, or death timing edge cases are handled inconsistently.

## PacketSequenceViolation

### Purpose

`PacketSequenceViolation` detects stale, repeated, or replayed client commands.

The server expects command sequence numbers to increase.

### Trigger

For input commands and hit claims, the rule triggers when:

```text
incoming_sequence <= last_sequence
```

### Evidence

The evidence records:

```text
player ID
observed sequence
expected next sequence
server time
reason
```

### Example reasons

```text
command sequence number did not increase
hit claim sequence number did not increase
```

### Current limitations

The sandbox uses strict monotonic sequence validation.

A production game may need a more nuanced model depending on transport protocol and command buffering.

Production concerns include:

```text
UDP packet reordering
packet loss
input retransmission
reconnect behavior
server migration
acknowledgment windows
replay protection
```

### False-positive risk

Low inside the sandbox because the TCP transport preserves order.

Higher in production if the network layer permits reordering or retransmission and the protocol does not define how to handle those cases.

## ClientTimeViolation

### Purpose

`ClientTimeViolation` detects suspicious client timestamp behavior.

The client timestamp is not authoritative, but it is useful for detecting inconsistent timing patterns.

### Trigger

The rule triggers when one of these conditions occurs:

```text
client timestamp does not increase
client timestamp jumps too far forward
```

The maximum allowed jump is configured by:

```text
max_client_time_step_ms
```

### Evidence

The evidence records:

```text
player ID
sequence number
observed timestamp or timestamp delta
expected timestamp or maximum allowed delta
server time
reason
```

### Example reasons

```text
client timestamp did not increase
client timestamp jumped too far forward
```

### Current limitations

The sandbox uses simple client timestamp validation.

It does not model:

```text
RTT estimation
jitter
clock drift
client frame stalls
server tick alignment
interpolation delay
reconciliation windows
```

### False-positive risk

Medium in production.

Timing data is noisy. A production rule should avoid treating timestamp anomalies as automatic enforcement evidence without additional context.

## ProtocolViolation

### Purpose

`ProtocolViolation` detects unsupported or malformed protocol behavior.

### Trigger

Current trigger examples:

```text
unsupported protocol version
invalid JSON protocol message
message line exceeds maximum configured size
```

### Evidence

The evidence records:

```text
player ID when available
observed protocol value
expected value or limit
server time
reason
```

### Example reasons

```text
unsupported protocol version
invalid JSON protocol message
client message exceeded maximum line size
```

### Current limitations

The sandbox protocol is readable line-delimited JSON.

A production protocol may be:

```text
binary
compressed
encrypted
UDP-based
version-negotiated
delta encoded
snapshot based
```

### False-positive risk

Low for unsupported versions in the sandbox.

In production, protocol findings may also be caused by rollout mismatches, old clients, corrupted installs, proxies, partial deployments, or bugs.

## RateLimitViolation

### Purpose

`RateLimitViolation` detects clients that send messages faster than the configured connection limit.

Rate limiting protects the server and reduces protocol abuse.

### Trigger

The rule triggers when the per-connection message count exceeds:

```text
max_messages_per_second
```

inside the current one-second window.

### Evidence

The evidence records:

```text
player ID when available
observed message count
configured message limit
server time
reason
```

### Example reason

```text
connection exceeded message rate limit
```

### Current limitations

The sandbox uses a simple one-second message counter.

A production system may use:

```text
token bucket
leaky bucket
per-message weights
per-account limits
per-IP limits
backoff
temporary disconnects
circuit breakers
privacy-aware correlation
```

### False-positive risk

Medium in production.

Legitimate bursts can happen during reconnects, poor network conditions, telemetry flushes, or client bugs. Rate-limit findings should be treated as operational and security signals, not automatic proof of cheating.

## HitValidationViolation

### Purpose

`HitValidationViolation` detects invalid client hit claims.

The client may claim that a shot hit a target. The server validates that claim against server-known target geometry.

### Trigger

Current trigger examples:

```text
target does not exist
hit direction vector is empty
target is behind hit ray
target exceeds maximum hit distance
claimed distance exceeds maximum hit distance
claimed distance does not match server geometry
ray misses target radius plus tolerance
```

### Evidence

The evidence records:

```text
player ID
sequence number
observed value
expected limit
server time
reason
```

### Example reasons

```text
target does not exist
hit direction vector was empty
target was behind hit ray
target exceeded maximum hit distance
claimed hit distance exceeded maximum hit distance
claimed hit distance did not match server geometry
hit ray missed target
```

### Current limitations

The sandbox uses deterministic 2D test targets.

It does not model:

```text
historical player snapshots
lag compensation
server rewind
hitboxes
capsules
line of sight
projectile travel time
projectile gravity
weapon spread
recoil
animation pose
target interpolation
client prediction
```

### False-positive risk

Low inside the sandbox.

High in production if lag compensation, target history, hitboxes, projectile travel, line of sight, recoil, spread, or animation state are modeled incorrectly.

## Rule lifecycle

Current sandbox rules are always active.

A production system should support:

```text
shadow mode
alert-only mode
manual-review mode
enforcement mode
rule versioning
rule rollback
false-positive review
reviewer notes
appeal support
```

## Rule design principles

Rules should be:

```text
server-side
deterministic where possible
attached to observed values
attached to expected limits
versionable
testable
explainable
reviewable
conservative about enforcement
```

Rules should not directly punish players.

The correct output of a rule is evidence. Enforcement requires a separate operational policy.