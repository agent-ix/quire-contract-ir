---
id: SR-630
title: "code review of PR 233 (delete Contract IR's witness and replay modules)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@6a04978b0e34cb4d8e1a179a58ae52dc19fde2df; src/kani/mod.rs, src/kani/replay.rs, src/kani/witness.rs, tests/it/kani_replay.rs, tests/it/main.rs"
review_set: base
---
# SR-630: code review of PR 233

## Summary

Ticket: IR-347 (deletion slice). Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (origin/main ead7267, head 6a04978). The PR deletes `src/kani/witness.rs` (747 lines), `src/kani/replay.rs` (271), `tests/it/kani_replay.rs` (692), the two `mod` lines and two `pub use` blocks in `src/kani/mod.rs`, and `mod kani_replay;` in `tests/it/main.rs`. Nothing else in `src/` changes.

Gates run by the reviewer at 6a04978 with CARGO_TARGET_DIR inside the review worktree: `make fmt-check lint test corpus` exit 0 (170 + 58 + 2 tests passed, conformance corpus all match), `make deny` exit 0 (advisories, bans, licenses, sources ok; one-copy check ok). Codegen origin/main 8fcb51f `cargo check --all-targets` with `--config patch` onto this head: Finished, no errors.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Approve. Checked and correct:

- Kept surface: `FiniteInput`, `ValidatedFiniteInput`, `FiniteObject`, `FiniteReference`, `PopulationCompleteness`, `ResourceBounds`, `KaniOutcome`, `KaniOutcomeKind`, `KaniProviderRecord`, `KaniProviderResult`, `KaniProfile`, `CapabilityEntry`, `CapabilityDisposition`, `ProfileError`, `ProfileSelection`, dispatch types and the three family lowerings are still exported from `src/kani/mod.rs`.
- The deleted modules imported only `serde`, `std` and `super::{FiniteInput, KaniOutcome, KaniOutcomeKind}`; no dependency is orphaned in Cargo.toml and Cargo.lock is unchanged.
- No dangling reference: no `src/`, `crates/`, `tests/`, `schemas/`, `corpus/`, README or AGENTS text names a deleted item, module, file, or any of the 18 deleted `kani_witness_*` / `kani_replay_*` / `kani_native_replay_disagreement` refusal codes (none was registered in STD-001). Remaining mentions are FR-039's negative "Items QSL owns" list (needed by FR-039-AC-3), AD-001/FR-031 naming QSL's own types, and historical `spec/reviews/` and `reviews/` files.
- Kept code keeps its tests: `KaniOutcome::{proved, counterexample, non_success, proved_from_checks, boolean_claim, provider_record}` and `FiniteInput::validate` stay exercised in `tests/it/kani_shared.rs` and the three family suites. The deleted file tested kept items only as a vehicle for replay.
- No compatibility shim, re-export, deprecated alias or feature gate is left behind.
- Consumers: no `quire_contract_ir::kani` import of any deleted item in codegen main 8fcb51f (its `kani_witness_join.rs:16` doc comment only), QSL main 7d0497be, quire-protocol, quire-analyze, or any other checkout under the dev root; no `quire_contract_ir::*` glob import anywhere; QSL and quire-protocol reach IR only through `use quire_contract_ir as ir` with no `ir::kani` path.
