---
id: SR-2026103043
title: "IR-1194 PR 744 TC-058 inventory repair review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #744 gate repair 8bb3d732...1f8439f3; spec/model/functional/FR-019-rust-library-interface.md Public items row and FR-019-AC-5"
review_set: subset
---

## Summary

The gate-repair delta adds one implemented Public items row for the Exact type projection API. Its eight type names equal the crate-root re-exports in `crates/quire-contract-model/src/lib.rs` and the V2 module re-exports in `checked_package/v2/mod.rs`; FR-352 owns those items. TC-058's `expected_items_from` parses this row as a `checked_package` entry. The delta does not change an acceptance criterion, production API, test, or CI gate. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**PASS** — The inventory repair is accurate and the final aggregate gate may rerun on 1f8439f3991a7b4d86e4823e24747436170fbbd7. Full FR-352 record-view and QSL producer acceptance remain open outside this partial PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- **FR-019-AC-5 (examined):** The default-feature model crate-root public items must exactly match the implemented Public items table; TC-058 must fail on an added or missing item.
- **FR-352 (context only):** Owns the Exact type projection and identity-witness public types.

## Evidence

The single added row is at `spec/model/functional/FR-019-rust-library-interface.md:227`. All eight names are in `crates/quire-contract-model/src/lib.rs:190-194` and `src/checked_package/v2/mod.rs:85-90`. `tests/it/public_interface.rs:873-904` extracts backtick-delimited names from the table; the row has no planned marker and is counted under `checked_package`. Targeted Quire validation and `git diff --check` passed. No full aggregate gate was rerun by this reviewer. Plan completion: not assessed.

## Focused Check

`cargo test -p quire-contract-ir --test it tc_058_model_root_exports_exactly_the_documented_public_items` passed (1 test) on 1f8439f3991a7b4d86e4823e24747436170fbbd7.
