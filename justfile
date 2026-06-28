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

docker-build:
    docker build -t unity-rust-authoritative-security-sandbox .

docker-up:
    docker compose up -d server

docker-down:
    docker compose down --remove-orphans

docker-bot-normal:
    docker compose run --rm bot-normal

docker-bot-suspicious:
    docker compose run --rm bot-suspicious

docker-bot-sequence:
    docker compose run --rm bot-sequence

docker-summary:
    docker compose run --rm summary

docker-risk:
    docker compose run --rm risk

docker-demo:
    ./scripts/docker-demo.sh