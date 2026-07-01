# Release Process

This repository uses milestone releases to keep the project reviewable and reproducible.

A release represents one coherent project state, not a random collection of changes.

## Current release track

```text
v0.13.0  Authoritative hit validation
v0.14.0  Architecture and portfolio documentation freeze
v0.15.0  Dashboard polish and screenshots
v1.0.0   Final portfolio release
```

After `v0.13.0`, backend feature work is frozen unless a bug fix or architecture correction requires it.

## Release checks

Before creating a release tag, the project must pass:

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
just docker-demo
```

These checks verify formatting, compilation, lint quality, test coverage, and the full Docker demo path.

## Docker demo coverage

The Docker demo verifies the complete portfolio workflow:

```text
server starts
synthetic clients run
telemetry is generated
evidence is exported
SQLite investigation database is created
database queries run
investigation API starts
API and dashboard routes pass smoke checks
```

Generated demo artifacts:

```text
samples/session.jsonl
reports/evidence.json
reports/evidence.csv
reports/investigation.db
```

## Release rule

Do not mix unrelated work in one release.

Documentation releases should not include hidden backend features.

Backend fixes should be committed separately from documentation polish.

## Tagging

Releases use annotated semantic version tags:

```bash
git tag -a v0.14.0 -m "Architecture and portfolio documentation freeze"
git push origin v0.14.0
```

## v0.14.0 scope

The `v0.14.0` milestone is documentation-only.

It includes:

```text
README rewrite
architecture documentation
threat model
rule catalog
false-positive analysis
investigation workflow
security policy
changelog
```

No new detection systems or backend features belong in this milestone.

## Final release standard

The final `v1.0.0` release should let another engineer clone the repository, run the demo, inspect evidence, open the dashboard, read the threat model, and understand the project without private explanation.