fmt:
    cargo fmt --all

check:
    cargo check --workspace

test:
    cargo test --workspace

ci: fmt check test

server:
    cargo run -p server

bot-normal:
    cargo run -p bot -- normal

bot-suspicious:
    cargo run -p bot -- suspicious

bot-sequence:
    cargo run -p bot -- sequence

summary:
    cargo run -p cli -- summary samples/session.jsonl

risk:
    cargo run -p cli -- risk samples/session.jsonl