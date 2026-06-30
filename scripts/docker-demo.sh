#!/usr/bin/env bash
set -euo pipefail

export HOST_UID="${HOST_UID:-$(id -u)}"
export HOST_GID="${HOST_GID:-$(id -g)}"

mkdir -p samples reports
rm -f samples/session.jsonl
rm -f reports/evidence.json reports/evidence.csv

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

test -f samples/session.jsonl
test -f reports/evidence.json
test -f reports/evidence.csv

grep -q "Suspicion" samples/session.jsonl
grep -q "ClientTimeViolation" samples/session.jsonl
grep -q "RateLimitViolation" samples/session.jsonl
grep -q "ProtocolViolation" samples/session.jsonl
grep -q "RateLimitViolation" reports/evidence.json
grep -q "ProtocolViolation" reports/evidence.csv

echo
echo "Docker demo finished successfully."
echo "Telemetry owner:"
stat -c '%U %G %a %n' samples/session.jsonl || true