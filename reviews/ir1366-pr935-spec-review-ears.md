---
id: SR-2026110602354630
title: "IR-1366 PR 935 spec-review-ears review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; IR-1366; runtime_type_projection, nominal_reads, charged model_members, reached Exact witness helpers, FR-360, TC-469"
review_set: subset
---

## Summary

Ticket: IR-1366. Scoped deterministic validation reports 2/2 grammar-clean documents and zero grammar findings. The requirement names Contract IR, uses an event trigger and measurable work/storage responses. No semantic EARS defect found.

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
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. Scoped deterministic validation reports 2/2 grammar-clean documents and zero grammar findings. The requirement names Contract IR, uses an event trigger and measurable work/storage responses. No semantic EARS defect found.

## Coverage

Plan completion: not assessed

No Cargo rebuild or aggregate gate was run by this reviewer. Inspected existing receipts: one helper ownership test, four Exact helper tests, 85 model/runtime tests and 12 nominal tests pass; workspace Clippy, formatter and two-document specification validation pass. Final combined-tree gate remains lead-owned and pending. The static matrix command completed within its 45-second bound; no evidence store was used. Optional gap-analysis semantic fan-out was not requested and was skipped; mandatory code/test alignment was performed in the Rust lane. No whole-repository assurance claim is made.


## Round 1 Evidence Status

Static fix review does not change this method's original scoped conclusion. No global matrix recomputation or Cargo rebuild was performed. New regression receipts remain pending; prior 102 tests apply only to the earlier candidate. The lead final combined-tree gate remains pending. See the code-review round 1 new findings before merge.


## Round 2 Evidence Status

The static scope and earlier method conclusion remain qualified as before. No global matrix recomputation or Cargo checks were run by the reviewer. New regression results and the final combined-tree gate remain pending. Code-review FND-006 requires a same-PR correction before merge.
