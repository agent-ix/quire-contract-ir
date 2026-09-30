---
id: SR-600
title: "gap analysis of PR 219 (ceremony sweep, spec and docs)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/**, tests/it/kani_replay.rs (coverage join only)"
review_set: subset
---
# SR-600: gap analysis of PR 219

## Summary

Ticket: IR-408. Plan completion: not assessed. This analysis checks that the deletions left every remaining requirement with its acceptance criteria and test backing. `quire coverage --scope . --strict` was run, and on an export of origin/main, where `make spec` stops at validate. It reports 155 backed rows on both. The unbacked set is the same 22 rows on both (FR-036, FR-037, FR-039, FR-344 and their ACs, FR-019-AC-5, TC-045, TC-055, TC-058, TC-222). Total rows drop from 185 to 178, all from deleted artifacts, and no backed row is removed. No requirement is left without acceptance criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | 17 tests still trace TC-221 and FR-031-AC-4, and the PR deleted the only spec text that explained them. Nothing stops a later FR-031-AC-4 from being silently backed by them | tests/it/kani_replay.rs:304 |

## Finding Detail

- FND-001: `tests/it/kani_replay.rs` has 17 `#[trace("TC-221", "FR-031-AC-4")]` tests (lines 304-852). On origin/main, the TM-002 "Withdrawn Test Cases" section and the FR-031 "Retired criteria" section recorded that these trace a withdrawn case and a retired AC. This PR deletes both sections. Coverage reports the same orphan traces on both sides, so no row is newly unbacked. However, FR-031 now declares only AC-1 and AC-5, and ADR-0056 Identifiers rule 2 forbids reissuing artifact IDs but says nothing about AC IDs. An author who later adds FR-031-AC-4 (or FR-037-AC-1..5) would get it backed by stale witness tests without noticing. Fix, without restoring a ledger: extend ADR-0056 Identifiers rule 2 (spec/decisions/ADR-0056-spec-layout-convention.md:133) to "An ID, including an acceptance-criterion ID, is never renumbered by a move and never reissued after deletion", and delete the 17 stale tags with `src/kani/witness.rs` in the code half (PR #218).

## Scope

- `FR-031-AC-1` (spec/contract/FR-031-bounded-kani-dispatch-and-terminal-map.md), examined: The shared dispatch index routes definedness/arithmetic, object/reference/graph, and collection/query work through distinct declared modules and rejects cross-family approximation.
- `FR-031-AC-5` (same), examined: Each outcome in the map's table maps to exactly its listed TerminalValue ... no outcome maps to Tested or Failed.
- `FR-037-AC-6` (spec/contract/FR-037-canonical-backend-replay-and-qualification.md), examined: Contract IR's public API names no replay envelope ... no Contract IR source calls a replay executor.
- `FR-034-AC-1..AC-5` (spec/contract/FR-034-assemble-output-package-atomically.md), examined: still backed by TC-043 tests, same as origin/main.
- `FR-029-AC-1` (spec/contract/FR-029-versioned-bounded-kani-profile.md), examined: backed by tests/it/kani_shared.rs:70 and kani_arithmetic.rs:104; src/kani/profile.rs has no Kani tool-version field, so the narrowed AC matches the code.
- `FR-039-AC-4` / `TC-055` (spec/interface/FR-039-root-crate-public-interface.md, spec/contract/TC-055-root-crate-public-interface.md), examined: TC-055 procedure now names the TC-041 and TC-042 corpora FR-039-AC-4 names; BridgeErrorCode and TC-038..040 exist nowhere in the tree.
- `NFR-005` (deleted), examined: had no acceptance criteria; rust-version = "1.98.1" remains in Cargo.toml and crates/quire-contract-model/Cargo.toml.
- `TM-002` (spec/contract-test-matrix.md), examined: every FR row cites only existing ACs and TCs.

## Verdict

Mergeable as-is from the gap-analysis side. No acceptance criterion lost its backing, and no requirement lost its criteria. The unbacked set matches origin/main exactly. FND-001 is a low follow-up that belongs with the code half (PR #218).
