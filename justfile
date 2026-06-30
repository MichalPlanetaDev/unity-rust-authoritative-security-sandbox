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

bot-flood:
    cargo run -p bot -- flood

bot-bad-protocol:
    cargo run -p bot -- bad-protocol

docker-bot-flood:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm bot-flood

docker-bot-bad-protocol:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm bot-bad-protocol

ingest-db:
    cargo run -p cli -- ingest-db samples/session.jsonl reports/investigation.db

query-db-suspicious:
    cargo run -p cli -- query-db suspicious-players reports/investigation.db

query-db-breakdown:
    cargo run -p cli -- query-db violation-breakdown reports/investigation.db

query-db-player-timeline:
    cargo run -p cli -- query-db player-timeline reports/investigation.db 2

docker-ingest-db:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm ingest-db

docker-query-db-suspicious:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm query-db-suspicious

docker-query-db-breakdown:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm query-db-breakdown

docker-query-db-player-timeline:
    HOST_UID=$(id -u) HOST_GID=$(id -g) docker compose run --rm query-db-player-timeline