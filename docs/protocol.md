# Protocol

The first networking version uses line-delimited JSON over TCP.

Each client message is serialized as one JSON object followed by a newline. The server responds with one JSON object followed by a newline.

## Client messages

- `Join`
- `Input`
- `Ping`

## Server messages

- `Welcome`
- `Snapshot`
- `Rejected`
- `Pong`

## Input model

The client sends input intentions:

- movement direction
- fire request
- sequence number
- client timestamp
- optional claimed position

The server owns the final authoritative state. A claimed client position can be inspected, but it is not treated as truth.

## Security relevance

The protocol is designed around server authority. The client may request actions, but the server validates movement, fire rate, sequence ordering, and state transitions.