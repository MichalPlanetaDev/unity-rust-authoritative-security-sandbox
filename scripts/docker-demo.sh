#!/usr/bin/env bash
set -euo pipefail

rm -f samples/session.jsonl

docker compose up -d server

cleanup() {
  docker compose down --remove-orphans
}

trap cleanup EXIT

docker compose run --rm bot-normal
docker compose run --rm bot-suspicious
docker compose run --rm bot-sequence
docker compose run --rm summary
docker compose run --rm risk

test -f samples/session.jsonl
grep -q "Suspicion" samples/session.jsonl