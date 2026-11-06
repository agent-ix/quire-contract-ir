---
id: SR-2026110602354629
title: "IR-1366 PR 935 spec-review-base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1366; runtime_type_projection, nominal_reads, charged model_members, reached Exact witness helpers, FR-360, TC-469"
review_set: subset
---

## Summary

Ticket: IR-1366. FR-360 and TC-469 define inputs, outputs, exact boundaries, denied-charge semantics, clone determinism and explicit API exclusions. No new hashes, pins or tracking records. No applicable AssuranceProfile exists. Base checklist is clean within the two changed documents.

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

PASS. FR-360 and TC-469 define inputs, outputs, exact boundaries, denied-charge semantics, clone determinism and explicit API exclusions. No new hashes, pins or tracking records. No applicable AssuranceProfile exists. Base checklist is clean within the two changed documents.

## Coverage

Plan completion: not assessed

No Cargo rebuild or aggregate gate was run by this reviewer. Inspected existing receipts: one helper ownership test, four Exact helper tests, 85 model/runtime tests and 12 nominal tests pass; workspace Clippy, formatter and two-document specification validation pass. Final combined-tree gate remains lead-owned and pending. The static matrix command completed within its 45-second bound; no evidence store was used. Optional gap-analysis semantic fan-out was not requested and was skipped; mandatory code/test alignment was performed in the Rust lane. No whole-repository assurance claim is made.


## Round 1 Evidence Status

Static fix review does not change this method's original scoped conclusion. No global matrix recomputation or Cargo rebuild was performed. New regression receipts remain pending; prior 102 tests apply only to the earlier candidate. The lead final combined-tree gate remains pending. See the code-review round 1 new findings before merge.


## Round 2 Evidence Status

The static scope and earlier method conclusion remain qualified as before. No global matrix recomputation or Cargo checks were run by the reviewer. New regression results and the final combined-tree gate remain pending. Code-review FND-006 requires a same-PR correction before merge.


## Round 3 Evidence Status

All six code findings are now statically fixed with no new finding. New regression checks are running and their actual results remain pending; no runtime PASS is claimed here. Original scoped matrix qualification and pre-existing global gaps remain unchanged. The lead final combined-tree gate remains pending.


## Final Focused Evidence and Current Verdict

PASS for the reviewed specification scope. All six code findings have a latest fixed disposition; no new finding remains. This current evidence update supersedes the earlier round notes saying that regression execution was pending.

Actual current-production regression receipts were read in full: /tmp/ir1366-round3-focused.sh and its head log identify the reviewed production candidate; the set-e script completed workspace checks with conformance-cli enabled. Tests passed: 2 runtime helper tests, 3 native-budget tests, 4 Exact helper tests, 86 model/runtime integration tests, and 13 nominal integration tests (108 total; zero failed/ignored in these selections). These include shared slot multiplicities, fixed first-denial metadata, earlier-row tuple/unit insertion attribution, unresolved declaration coordinates, context restoration, and the multi-term parent-dimension denial regression. Workspace Clippy with warnings denied and the integration-target compile check passed. That Clippy invocation omitted --all-targets; broader lint coverage remains pending with the lead/coder. This is focused execution evidence, not the final combined-tree gate.

All five FR-360 criteria were tagged in the earlier computed matrix. Its repository-wide pre-existing gaps remain qualified and were not recomputed or resolved here; no global repository assurance PASS is claimed. Plan completion: not assessed.

Merge readiness remains conditional on committing the latest versions of all six scratch review artifacts and the lead-owned final combined-tree gate, including the required all-targets coverage. No reviewer build, source edit, push or merge was performed.


## Narrow Rebase Integration Review

PASS for the narrow rebase/integration scope. All six findings remain fixed and no new finding was found. Range-diff shows the three fix commits and retained review artifacts carry unchanged; the only additional PR change adapts runtime-mode restoration assertions to the new upstream charged native-denial behavior. The shared-rule, parent-dimension, direct-coordinate and precharge implementations are unchanged. Upstream native_budget_denial remains the single ordinary budget-denial constructor, charging the complete diagnostic floor before ownership and preserving the original first unpaid charge when its diagnostic storage is unaffordable.

The revised mode-restoration oracle is substantive: it requires a runtime refusal inside admitted_tuple_row, then reuses that budget with abundant storage for an ordinary join and asserts an owned requested locus, additional admitted diagnostic storage, matching denial metadata and no leaked runtime coordinate. A leaked scoped Exact mode returns compact with no requested node and fails this oracle. The observed_restored_native_mode assertion requires that branch to execute. The separate no-storage ordinary-denial case correctly expects compact payloads under the new baseline; it is no longer relied on alone as proof of mode restoration.

Inspected actual rebase-focused script, tested-head receipt, range-diff and execution logs. Passing selections total 169: 2 runtime helpers, 3 native-budget unit tests, 5 Exact helper tests, 86 model/runtime integration tests, 13 nominal tests, 9 complete-witness tests, 33 owner tests and 18 union tests. The native_join_budget integration filter runs zero standalone tests and is not included in that total. Workspace Clippy, feature-matched integration-target compilation and formatter receipts pass. The focused script uses --locked --workspace with conformance-cli. No reviewer build or global matrix rerun was performed. No excluded scratch/old dependency commit is in the candidate ancestry.

Current scoped review verdict: PASS, subject to the lead-owned final combined-tree gate after the latest six artifacts are committed. Earlier pending lint/evidence notes are historical; the current rebase receipts replace them for this candidate. Repository-wide matrix gaps remain qualified and unchanged; this is no global assurance claim.
