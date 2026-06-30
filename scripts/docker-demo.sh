#!/usr/bin/env bash
set -euo pipefail

export HOST_UID="${HOST_UID:-$(id -u)}"
export HOST_GID="${HOST_GID:-$(id -g)}"

mkdir -p samples reports
rm -f samples/session.jsonl
rm -f reports/evidence.json reports/evidence.csv
rm -f reports/investigation.db reports/investigation.db-shm reports/investigation.db-wal

docker compose down --remove-orphans
docker compose build

docker compose up -d server

cleanup() {
  docker compose down --remove-orphans
}

trap cleanup EXIT

docker compose run --rm bot-normal
docker compose run --rm bot-suspicious
docker compose run --rm bot-sequence
docker compose run --rm bot-timing
docker compose run --rm bot-flood
docker compose run --rm bot-bad-protocol

docker compose run --rm summary
docker compose run --rm risk
docker compose run --rm timeline
docker compose run --rm evidence
docker compose run --rm export-evidence

docker compose run --rm ingest-db
docker compose run --rm query-db-suspicious
docker compose run --rm query-db-breakdown
docker compose run --rm query-db-player-timeline

test -f samples/session.jsonl
test -f reports/evidence.json
test -f reports/evidence.csv
test -f reports/investigation.db

grep -q "Suspicion" samples/session.jsonl
grep -q "ClientTimeViolation" samples/session.jsonl
grep -q "RateLimitViolation" samples/session.jsonl
grep -q "ProtocolViolation" samples/session.jsonl

grep -q "RateLimitViolation" reports/evidence.json
grep -q "ProtocolViolation" reports/evidence.csv

echo
echo "Docker demo finished successfully."
echo "Generated artifacts:"
stat -c '%U %G %a %n' samples/session.jsonl reports/evidence.json reports/evidence.csv reports/investigation.db || true