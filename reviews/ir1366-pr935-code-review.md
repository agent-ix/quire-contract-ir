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


## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The post-witness tuple/source-unit index insertion loops update ctx.row_locus but never activate it with locate before storage_node or charge_storage. exact_read restores the last preflight row context, so with two valid tuple rows a storage denial on row 0 insertion reports row 1; source-unit insertions have the same defect. Activate the current indexed row and static insertion form before the first storage charge, and test denial on an earlier row after a successful multi-row witness join. | crates/quire-contract-model/src/checked_package/v2/runtime_type_projection.rs:906 |
| FND-005 | medium | Locus capture performs an uncharged identity lookup before admitting the work it annotates: charge calls locate, which searches package.node_index before budget.charge. Zero-work projection therefore still performs BTreeMap node comparisons; nominal_reads Budget::charge/storage similarly call locate/index.get before charging. This adds unbounded identity-comparison work before the first unpaid inspection. Carry direct admitted graph positions/current row indices, or admit the lookup work before executing it without losing the denied locus. | crates/quire-contract-model/src/checked_package/v2/runtime_type_projection.rs:360 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: Removed runtime_node_storage; all three quantity copies call the existing native_join_storage::node. |
| FND-002 | fixed | Round 1: slot_type and slot_type_runtime both delegate to slot_type_inner, which owns the one multiplicity decision table. |
| FND-003 | fixed | Round 1: Added Copy NativeJoinLocus source indices/static forms, denial capture and constant-time denied_node resolution. This replaces total locus erasure; newly introduced attribution/work issues are separately recorded as FND-004 and FND-005. |
| FND-004 | still-open | Round 1: New finding: insertion charges retain the prior active row instead of the current row. |
| FND-005 | still-open | Round 1: New finding: locus lookup runs before the charged inspection is admitted. |
| FND-004 | fixed | Round 2: Both post-witness index insertion loops now activate the current TupleRow/SourceUnitRow and insertion form before storage_node. |
| FND-005 | fixed | Round 2: ProjectionBudget and nominal Budget no longer carry an identity-map locator; charge/storage update only fixed metadata before charging. Nominal node_with_position pays identity traversal before the shared lookup. Graph and row positions are retained; unresolved declaration coordinates resolve honestly to None without iterator scans. |
| FND-006 | still-open | Round 2: New finding: the second term inspection retains the previously resolved child graph position. |

## Round 1 Verdict

FAIL. FND-001 through FND-003 are statically addressed; FND-004 and FND-005 remain open. The new fixed-size representation and direct denied_node resolution allocate nothing; top-level runtime and exact_read restore previous active locus on Result success/refusal paths. New tests cover shared multiplicity values, first-denial preservation, zero limits, row preflight work denial and default context restoration, but do not exercise post-witness earlier-row insertion attribution. New focused checks are queued and were not run by this reviewer. Earlier 102 passing receipts cover the earlier candidate only and do not establish these fixes. Lead-owned final combined-tree gate remains pending.


## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | A dimension with two or more terms retains the previous child locus for the next parent-term charge. node_with_position changes active_locus to the first child; the next iteration charges the second term exponent against dimension.node_id without restoring the parent position. If work ends after the first child lookup/pending push, runtime reports GraphNode(first_child) instead of the parent dimension whose next term was denied. Reactivate the parent dimension before every term inspection and assert the resolved denied node in a multi-term dimension boundary test. | crates/quire-contract-model/src/checked_package/v2/nominal_reads.rs:501 |

## Round 2 Verdict

FAIL. FND-001 through FND-005 are statically addressed; FND-006 remains open. Fixed-size declaration ordinals truthfully identify unresolved input requests; denied_node performs only slice/graph direct indexing and never reconstructs identities. New insertion tests require a successful two-row witness join and assert earlier-row storage denial, preventing the previous stale insertion context. The nominal worklist retains graph positions and precharges new metadata vectors. Active runtime/Exact context restoration remains in place. New focused regression checks are still pending; no runtime PASS is claimed for the fixes. Earlier 102 receipts apply only to the earlier candidate. Lead-owned final combined-tree gate remains pending.
