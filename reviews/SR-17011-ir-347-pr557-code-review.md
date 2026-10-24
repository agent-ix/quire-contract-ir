---
id: SR-17011
title: "IR-347 PR 557 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir#557@59d855c0418db8f707c2084d5644dad2b8427b86 vs origin/main 7075edc; src/kani/mod.rs, src/kani/arithmetic.rs (deleted), src/kani/collections.rs (deleted), src/kani/objects.rs (deleted), tests/it/kani_shared.rs, tests/it/public_interface.rs, tests/it/main.rs, tests/it/kani_arithmetic.rs (deleted), tests/it/kani_collections.rs (deleted), tests/it/kani_objects.rs (deleted)"
review_set: subset
---
# SR-17011: IR-347 PR 557 code review

## Summary

Ticket: IR-347. Scope is `git diff origin/main...HEAD` at frozen head
`59d855c0418db8f707c2084d5644dad2b8427b86`, merge base `7075edc`. The Rust lane runs as its own
artifact (SR-17012), as the dispatch brief requests.

Checks run by the reviewer at the frozen head, in the worktree's own `target/`:

- `cargo test --locked --test it tc_055`: 3 passed.
- `cargo test --locked --doc -p quire-contract-ir kani`: 57 passed, including the 22 new
  `compile_fail,E0432` probes in `src/kani/mod.rs`.
- `make fmt-check`: clean. `make lint`: clean.
- The deleted `kani_*` integration suites were checked against `kani_shared.rs`. FR-029-AC-1,
  FR-030-AC-1, FR-030-AC-2 and FR-031-AC-1 keep TC-042 backing in
  `tests/it/kani_shared.rs` (lines 71, 92, 228, 429).
- Codegen origin/main `4eccd68` names none of the removed IR items through `quire_contract_ir`.
  It uses its own `CollectionQuery`, `QueryKind`, `GraphLowering` and `GraphRequest`, so the
  removal does not break the downstream build.

Examined units: FR-039-AC-1, FR-039-AC-2, FR-039-AC-3, FR-039-AC-4 and FR-037-AC-6 (all
examined), and FR-031-AC-1 (examined: three-family dispatch kept). FR-030-AC-5 and FR-030-AC-4
were read for context only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `tc_055_negative_kani_corpus_returns_typed_errors_without_panics` is tagged FR-039-AC-4 ("every public `kani` function over untrusted input"), but it is a new hand-picked subset and does not run the TC-042 negative corpus. It omits the wire readers, which are the main untrusted-input path: the `Deserialize` impls of `KaniProfile`, `KaniOutcome`, `CapabilityDisposition`, `FiniteInput` and `ModuleDescriptor`. It also omits `KaniProfile::new`'s `MissingSelectionField` and `InvalidMatrix` refusals. TC-055 says to "Run the TC-041 and TC-042 negative corpora under `catch_unwind`", and the matrix now marks FR-039 ✅ on this test alone | tests/it/kani_shared.rs:520-648 |

## Verdict

**PASS WITH FINDINGS.** The removal itself is correct. The three family modules, their
exports and their suites are gone. The kani root keeps exactly the 20 FR-039 items, and every
"Items codegen owns" and "Items QSL owns" name has a compile-fail probe that passes. The
inventory test parses FR-039's table, so the spec cannot drift from it silently. FND-001 is a
coverage overclaim, not a live panic. The TC-042 tests in `kani_shared.rs` already exercise the
malformed-wire and profile negatives, and libtest would surface a panic there. Either bind those
TC-042 negatives to FR-039-AC-4 and TC-055, or extend the TC-055 corpus to the wire readers and
the other `KaniProfile::new` refusals. Either fix keeps AC-4 at full strength.

## Dispositions

Round 1. Checked against fix commit `c28a120bb41f1dea550006fc55b79dd692419bc7`, which follows reviewed head `59d855c0`. Focused checks at that head: `cargo test --locked --test it -- kani_shared:: tc_055` (17 passed), `make fmt-check` clean, `make lint` clean.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The TC-042 negative tests now carry `TC-055` and `FR-039-AC-4` and run under `catch_unwind`. They cover `KaniProfile::new` (exact `UnsupportedProfile`, `MissingSelectionField` and `InvalidMatrix` values), the `KaniProfile`, `CapabilityDisposition` and `KaniOutcome` wire readers, `classify`, `validate`, and `DispatchIndex` `new` and `resolve`. A new test, `tc_055_malformed_finite_input_and_descriptor_wire_do_not_panic`, covers the `FiniteInput` and `ModuleDescriptor` readers (tests/it/kani_shared.rs:71-535) |
