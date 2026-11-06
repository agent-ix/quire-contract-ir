---
id: SR-2026110602354627
title: "IR-1366 PR 935 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1366; runtime_type_projection, nominal_reads, charged model_members, reached Exact witness helpers, FR-360, TC-469"
review_set: subset
---

## Summary

Ticket: IR-1366. Rust lane included. Searched existing footprint helpers and slot implementations, then traced all changed runtime/nominal/helper paths. Three defects require a same-PR fix.

## Scope

- role: examined; id: FR-360-AC-1; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: Each public runtime type projection, monitor-closure, runtime-reachable nominal-read, and selected unit-definition entry point requires the caller's mutable `NativeJoinBudget`; budget-aware helper variants charge allocations to that same budget. They do not route storage through `model_members::Budget` / `WorkMeter`, create a local allowance, or offer an unbudgeted runtime entry point or compatibility wrapper.
- role: examined; id: FR-360-AC-2; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: Every inspected runtime declaration, closure node or edge, nominal row, identity component, and selected unit edge charges work before inspection. Every IR-owned temporary or retained allocation, clone, copied name/type, formatted unit string, and runtime-reachable `element_type` / `slot_type` result charges FR-358 logical storage to the caller's budget before allocation or copying. Exact-limit work and storage succeed; one unit below either measured requirement refuses before the first unpaid operation.
- role: examined; id: FR-360-AC-3; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: A storage-limited refusal occurs before the denied allocation and returns a bounded typed budget refusal with the first unpaid resource and amount, no partial runtime view, and no uncharged owned diagnostic payload. A work-limited refusal reports the first unpaid work locus without advancing the denied charge. Any earlier admitted charges remain visible and the refused operation has no success delta.
- role: examined; id: FR-360-AC-4; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: At sufficient limits, representative source and model runtime type reads, selected nominal reads, and selected unit-definition reads return the same complete values, original identities, order, and semantic refusal causes as the existing behavior. Equal input, rows, and limits produce equal values, cumulative charges, deltas, or first refusals for the original and a clone of the admitted package.
- role: examined; id: FR-360-AC-5; path: spec/checked_package/functional/FR-360-charge-runtime-type-projection-with-shared-budget.md; excerpt: Runtime projection and monitor-closure paths use budget-aware variants for `DomainModel::element_type` / `slot_type` and other helpers, charging every reachable `BTreeSet` / `Vec`, cloned name/member type, formatted unit string, and other allocation to the caller's `NativeJoinBudget` before allocation or copy. This ticket does not add storage accounting to `WorkMeter` / `model_members::Budget` or widen `model_fields`, `model_operations`, semantic-operation, state, `step_relation`, or `comparison_operands` APIs; these serve separate paths and are not called by the audited runtime/closure path. 

Changed production files examined: runtime_type_projection.rs; nominal_reads.rs; model_members.rs; exact_projection.rs and exact_projection/complete_witness.rs. Changed test files examined: model_members/tests.rs; checked_package_v2_model_fields.rs and operation_census.rs; nominal_reads.rs. TC-469 procedure and expected results examined. FR-358/FR-359/FR-354 and repository README/CLAUDE/AGENTS are context only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | runtime_node_storage duplicates the FR-358 node footprint already implemented by native_join_storage::node. Both independently encode package, path-name and nested-handle units; a footprint-rule correction now requires two edits. Reuse the existing helper instead of keeping a second accounting rule. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1032 |
| FND-002 | high | slot_type_runtime repeats the multiplicity-to-collection decision table in slot_type (lines 1640-1654), including exact-one, open-upper refusal and bounds construction. A change to slot typing now requires parallel semantic edits and can make runtime disagree with other readers. Keep the explicitly required charged variant, but share one decision implementation as element_type already does. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1689 |
| FND-003 | medium | Every budget refusal erases the native locus: exhausted sets at/form to None, charge/charge_work discard their at/form arguments, and NativeJoinDeniedCharge stores only kind/amount. For example the existing long tuple-row work refusal now returns no indication of which row failed. This contradicts FR-360 Outputs and Behavior requiring the first refused locus and a non-allocating native locus representation. Preserve a borrowed or fixed-size locus without allocating after denial; update the tests currently asserting only None. | crates/quire-contract-model/src/checked_package/v2/runtime_type_projection.rs:302 |

## Verdict

FAIL. Rust lane included. Searched existing footprint helpers and slot implementations, then traced all changed runtime/nominal/helper paths. Three defects require a same-PR fix.

## Coverage

Plan completion: not assessed

No Cargo rebuild or aggregate gate was run by this reviewer. Inspected existing receipts: one helper ownership test, four Exact helper tests, 85 model/runtime tests and 12 nominal tests pass; workspace Clippy, formatter and two-document specification validation pass. Final combined-tree gate remains lead-owned and pending. The static matrix command completed within its 45-second bound; no evidence store was used. Optional gap-analysis semantic fan-out was not requested and was skipped; mandatory code/test alignment was performed in the Rust lane. No whole-repository assurance claim is made.
