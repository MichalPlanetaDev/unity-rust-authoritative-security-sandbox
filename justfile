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

timeline:
    cargo run -p cli -- timeline samples/session.jsonl

docker-build:
    docker build -t unity-rust-authoritative-security-sandbox .

docker-up:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose up -d server

docker-down:
    docker compose down --remove-orphans

docker-bot-normal:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm bot-normal

docker-bot-suspicious:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm bot-suspicious

docker-bot-sequence:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm bot-sequence

docker-summary:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm summary

docker-risk:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm risk

docker-demo:
    ./scripts/docker-demo.sh

bot-timing:
    cargo run -p bot -- timing

docker-bot-timing:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm bot-timing

evidence:
    cargo run -p cli -- evidence samples/session.jsonl

export-evidence:
    cargo run -p cli -- export-evidence samples/session.jsonl reports/evidence.json reports/evidence.csv

docker-timeline:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm timeline

docker-evidence:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm evidence

docker-export-evidence:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm export-evidence