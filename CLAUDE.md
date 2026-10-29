# quire-contract-ir

Cycle-free semantic contract model and bounded-Kani interface for assurance tooling.

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
make ci             # all local release gates
make use-local      # patch first-party git deps (ix-trace-rs, quire-canonical, quire-verification-contracts, quire-walk) to sibling checkouts via a gitignored .cargo/config.toml; snapshots Cargo.lock to .cargo/Cargo.lock.pre-local; fails if cargo metadata fails or a patch is unused
make use-remote     # delete the patch config and restore Cargo.lock from that snapshot (no snapshot: lock untouched)
```

## Safety scaffolding

- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- Workspace Clippy lints deny undocumented unsafe blocks in every member crate.
- `rustfmt.toml` uses stable-channel 100-character formatting. CI fails on drift.

## Layout

```
crates/quire-contract-model/ # cycle-free semantic substrate and sole model source
src/lib.rs             # bounded-Kani crate root; model API lives in quire-contract-model
src/bin/               # conformance runner
tests/fixtures/        # test data (Kani playback, native rule model)
spec/                  # requirements artifacts
scripts/               # local tooling
```

`quire-contract-model` must not acquire QSL, observation, protocol, TL, or
`quire-contract-ir` dependencies. Bounded-Kani integrations belong in the root
package so the production graph remains acyclic. All Cargo gates must use
`--workspace`; a root-package-only result is incomplete.
