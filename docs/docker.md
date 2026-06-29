# Docker

The project includes a Dockerized networking demo.

## Build

    docker build -t unity-rust-authoritative-security-sandbox .

## Run the server

    docker compose up -d server

## Run bot scenarios

    docker compose run --rm bot-normal
    docker compose run --rm bot-suspicious
    docker compose run --rm bot-sequence

## Inspect telemetry

    docker compose run --rm summary
    docker compose run --rm risk

## Full demo

    ./scripts/docker-demo.sh

or:

    just docker-demo

The full demo starts the server, runs normal/suspicious/sequence bot scenarios, summarizes telemetry, prints player risk, verifies that telemetry exists, and shuts the server down.

## File ownership

Docker Compose runs services with the host UID/GID through:

    HOST_UID
    HOST_GID

The demo script sets these automatically:

    ./scripts/docker-demo.sh

This prevents generated files such as `samples/session.jsonl` from being owned by root after Docker runs.