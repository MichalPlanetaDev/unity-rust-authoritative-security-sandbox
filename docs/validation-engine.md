# Validation Engine

The validation engine lives in:

    crates/validation

It separates server-authoritative validation from TCP networking code.

## Validators

Current validation areas:

- sequence ordering
- alive/dead state transitions
- movement claim validation
- fire cooldown validation
- client timestamp validation

## Design

The server owns player state.

The client sends input intentions. The validation engine compares each input against the previous server-authoritative state and produces a `ValidationDecision`.

A decision contains:

- whether the input is accepted
- optional rejection reason
- suspicion reports

Some violations reject input immediately, such as repeated sequence numbers.

Other violations are recorded as evidence while still allowing the simulation to continue, such as suspicious timing or movement claims.