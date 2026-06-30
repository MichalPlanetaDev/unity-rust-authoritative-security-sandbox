FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --workspace --release

FROM debian:bookworm-slim

WORKDIR /workspace

COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY --from=builder /app/target/release/bot /usr/local/bin/bot
COPY --from=builder /app/target/release/cli /usr/local/bin/cli
COPY --from=builder /app/target/release/investigation-api /usr/local/bin/investigation-api