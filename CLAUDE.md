# quire-contract-ir

Cycle-free semantic contract model and compatibility bridge for assurance tooling.

## Hash / digest / pin antipattern: present, slated for removal

This repository still contains hashes, digests, SHAs, pins, checksum catalogs and
records that track files, versions or tools. That is an antipattern and it is slated
for removal. Do not introduce any new use of it. When you touch code or specs that
contain one, remove it as part of the change. The only hash that stays is a canonical
identity digest that binds a proof to the exact content it proved. Package versions
live in Cargo.toml / package.json and their lockfiles only; reports name the app
version they ran.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make spec           # validate and cover all Quire artifacts
make release-check  # run all local release gates
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # all cargo-deny policy checks
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make ci             # all local release gates, including spec and exact Rust
```

## Safety scaffolding

- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses stable-channel 100-character formatting. CI fails on drift.

## Layout

```
crates/quire-contract-model/ # cycle-free semantic substrate and sole model source
src/lib.rs             # compatibility bridge and model API re-export
src/bin/               # compatibility-package conformance runner
tests/fixtures/        # compile fixtures, including the dependency-key alias proof
spec/                  # requirements artifacts
scripts/               # local tooling
```

`quire-contract-model` must not acquire QSL, observation, protocol, TL, or
`quire-contract-ir` dependencies. Owner integrations belong in the root bridge
package so the production graph remains acyclic. All Cargo gates must use
`--workspace`; a root-package-only result is incomplete.
