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
| FND-006 | fixed | Round 3: Every sibling term reactivates its already-paid parent dimension position before exponent work; child lookup may then change the context without leaking into the next sibling. |

## Round 1 Verdict

FAIL. FND-001 through FND-003 are statically addressed; FND-004 and FND-005 remain open. The new fixed-size representation and direct denied_node resolution allocate nothing; top-level runtime and exact_read restore previous active locus on Result success/refusal paths. New tests cover shared multiplicity values, first-denial preservation, zero limits, row preflight work denial and default context restoration, but do not exercise post-witness earlier-row insertion attribution. New focused checks are queued and were not run by this reviewer. Earlier 102 passing receipts cover the earlier candidate only and do not establish these fixes. Lead-owned final combined-tree gate remains pending.


## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | A dimension with two or more terms retains the previous child locus for the next parent-term charge. node_with_position changes active_locus to the first child; the next iteration charges the second term exponent against dimension.node_id without restoring the parent position. If work ends after the first child lookup/pending push, runtime reports GraphNode(first_child) instead of the parent dimension whose next term was denied. Reactivate the parent dimension before every term inspection and assert the resolved denied node in a multi-term dimension boundary test. | crates/quire-contract-model/src/checked_package/v2/nominal_reads.rs:501 |

## Round 2 Verdict

FAIL. FND-001 through FND-005 are statically addressed; FND-006 remains open. Fixed-size declaration ordinals truthfully identify unresolved input requests; denied_node performs only slice/graph direct indexing and never reconstructs identities. New insertion tests require a successful two-row witness join and assert earlier-row storage denial, preventing the previous stale insertion context. The nominal worklist retains graph positions and precharges new metadata vectors. Active runtime/Exact context restoration remains in place. New focused regression checks are still pending; no runtime PASS is claimed for the fixes. Earlier 102 receipts apply only to the earlier candidate. Lead-owned final combined-tree gate remains pending.


## Round 3 Verdict

CONDITIONAL pending execution evidence and the lead final gate. All six findings are statically fixed; no new finding was found in this round. The production delta adds only a fixed-size parent graph-position activation before every dimension term charge, without any identity lookup, allocation or key clone. The multi-term integration test first requires ample successful runtime projection, then exhausts every work boundary and identifies the unique sixteen-unit second exponent charge, asserting the resolved parent instead of the prior child and matching preserved first-denial metadata. Its positive matched-boundary count prevents a silently skipped oracle. The unrelated test fixture now borrows its row instead of moving it. New focused regression execution is still running; this review does not claim runtime PASS. The original 102-test receipts remain earlier-candidate evidence only. Final combined-tree validation is lead-owned and pending.


## Final Focused Evidence and Current Verdict

PASS for the reviewed changed scope. All six code findings have a latest fixed disposition; no new finding remains. This current evidence update supersedes the earlier round notes saying that regression execution was pending.

Actual current-production regression receipts were read in full: /tmp/ir1366-round3-focused.sh and its head log identify the reviewed production candidate; the set-e script completed workspace checks with conformance-cli enabled. Tests passed: 2 runtime helper tests, 3 native-budget tests, 4 Exact helper tests, 86 model/runtime integration tests, and 13 nominal integration tests (108 total; zero failed/ignored in these selections). These include shared slot multiplicities, fixed first-denial metadata, earlier-row tuple/unit insertion attribution, unresolved declaration coordinates, context restoration, and the multi-term parent-dimension denial regression. Workspace Clippy with warnings denied and the integration-target compile check passed. That Clippy invocation omitted --all-targets; broader lint coverage remains pending with the lead/coder. This is focused execution evidence, not the final combined-tree gate.

All five FR-360 criteria were tagged in the earlier computed matrix. Its repository-wide pre-existing gaps remain qualified and were not recomputed or resolved here; no global repository assurance PASS is claimed. Plan completion: not assessed.

Merge readiness remains conditional on committing the latest versions of all six scratch review artifacts and the lead-owned final combined-tree gate, including the required all-targets coverage. No reviewer build, source edit, push or merge was performed.


## Narrow Rebase Integration Review

PASS for the narrow rebase/integration scope. All six findings remain fixed and no new finding was found. Range-diff shows the three fix commits and retained review artifacts carry unchanged; the only additional PR change adapts runtime-mode restoration assertions to the new upstream charged native-denial behavior. The shared-rule, parent-dimension, direct-coordinate and precharge implementations are unchanged. Upstream native_budget_denial remains the single ordinary budget-denial constructor, charging the complete diagnostic floor before ownership and preserving the original first unpaid charge when its diagnostic storage is unaffordable.

The revised mode-restoration oracle is substantive: it requires a runtime refusal inside admitted_tuple_row, then reuses that budget with abundant storage for an ordinary join and asserts an owned requested locus, additional admitted diagnostic storage, matching denial metadata and no leaked runtime coordinate. A leaked scoped Exact mode returns compact with no requested node and fails this oracle. The observed_restored_native_mode assertion requires that branch to execute. The separate no-storage ordinary-denial case correctly expects compact payloads under the new baseline; it is no longer relied on alone as proof of mode restoration.

Inspected actual rebase-focused script, tested-head receipt, range-diff and execution logs. Passing selections total 169: 2 runtime helpers, 3 native-budget unit tests, 5 Exact helper tests, 86 model/runtime integration tests, 13 nominal tests, 9 complete-witness tests, 33 owner tests and 18 union tests. The native_join_budget integration filter runs zero standalone tests and is not included in that total. Workspace Clippy, feature-matched integration-target compilation and formatter receipts pass. The focused script uses --locked --workspace with conformance-cli. No reviewer build or global matrix rerun was performed. No excluded scratch/old dependency commit is in the candidate ancestry.

Current scoped review verdict: PASS, subject to the lead-owned final combined-tree gate after the latest six artifacts are committed. Earlier pending lint/evidence notes are historical; the current rebase receipts replace them for this candidate. Repository-wide matrix gaps remain qualified and unchanged; this is no global assurance claim.


## Final-Gate Discovery Correction Review

PASS for the narrow final-gate discovery corrections; all six original findings remain fixed and no new unresolved finding was found. The first native full-gate attempt failed with 1280 passing, 10 failing and one ignored integration test; an earlier whitespace preflight failure was separate. Neither failed attempt is a final gate PASS. Three failures exposed the missing declaration of the two actual public metadata types; seven were missing checker/scanner executables. After configuring the real tools, one actual helper dispatch violation remained and was corrected without an exemption.

FR-019 now explicitly inventories NativeJoinLocus and NativeJoinLocusSource under FR-360, and FR-360 documents their existing fixed-coordinate API. The exact public-inventory comparison and fail-closed checks remain intact; only the two legitimate names are added. Duration selection now uses the existing typed own_seconds_declared fact to construct exactly the owning-model or global resolved key. Foreign units sharing the suffix cannot match. Four borrowed parts form a stack array; the common native_join_storage::text_parts function computes checked/saturating logical text size with no heap allocation. Caller work and storage are admitted before concat or map lookup; one charged temporary key is then used. The common text footprint delegates to the same function, avoiding a duplicate formula. Nonruntime getter signatures and shared semantic implementation remain intact.

Actual newly completed receipts were inspected: the duration-green script uses --locked --workspace and conformance-cli with the real QSL_STRING_EDGE_BIN and QUIRE_LOCK_CHECKER. Its tested-head receipt matches the frozen candidate. All 143 selected tests pass: 3 runtime helpers, 3 native-budget tests, 5 Exact helpers, 6 authoritative string-edge tests, 25 public-interface tests, 2 lock-coherence tests, 86 model/runtime tests and 13 nominal tests. The added exact owner/global test checks own presence, absent-own with global available, global selection and absent-global with foreign/own decoys. The zero-storage key test failed on the prior scan implementation and now passes. Existing ownership-boundary tests exhaust every storage prefix. Clippy/fmt receipts pass. Scoped FR-019/FR-360 validation reports both documents grammar-clean. No global matrix rerun, reviewer build or source edit occurred.

Additional examined scope: spec/model/functional/FR-019-rust-library-interface.md closed export inventory; the new FR-360 metadata paragraph; public_interface exact inventory; duration helper, common text_parts footprint and owner/zero-storage tests. role: examined; excerpt: "The runtime refusal metadata types `NativeJoinLocus` and `NativeJoinLocusSource`".

Current scoped verdict: PASS. The latest six scratch artifacts must be copied and committed before the corrected final combined-tree gate. That full gate remains pending; existing repository-wide matrix gaps remain qualified, with no global assurance PASS claimed.
