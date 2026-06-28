# Bot

The `bot` crate sends controlled client traffic to the Rust server.

Scenarios:

    cargo run -p bot -- normal
    cargo run -p bot -- suspicious
    cargo run -p bot -- sequence

The normal scenario sends valid movement and one fire input.

The suspicious scenario sends an impossible claimed position and fires too quickly.

The sequence scenario sends a repeated sequence number.