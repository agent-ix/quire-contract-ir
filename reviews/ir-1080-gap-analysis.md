---
id: SR-612090
title: "Gap analysis of IR-1080 PR #644"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1080; PR #644; crates/quire-contract-model/src/checked_package/v2/lower.rs, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs, tests/conformance_qsl_ir769/main.rs, tests/conformance_qsl_ir779/main.rs, tests/it/admitted_guard_context.rs, tests/it/admitted_result_context.rs, tests/it/checked_package_v2_abstraction_relation.rs, tests/it/checked_package_v2_dependency_reference.rs, tests/it/checked_package_v2_flat_wire.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_identity_digests.rs, tests/it/checked_package_v2_lowering.rs, tests/it/checked_package_v2_parameters.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_require_bounds.rs, tests/it/checked_package_v2_temporal.rs, tests/it/checked_package_v2_union.rs, tests/it/complete_v1_checked_package.rs, tests/it/complete_v1_contract_package.rs"
review_set: subset
---

## Summary

Planless acceptance-chain audit found that the new tests claim FR-035-AC-9/TC-935 but miss required authoritative vector and output checks. The implementation is not demonstrated to satisfy every AC clause.

## Verdict

**CONDITIONAL** — Three evidence gaps require correction in PR #644.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-935 T vector is replaced by the fixture’s D selection; exact owner vectors are not exercised | tests/it/checked_package_v2_lowering.rs:116 |
| FND-002 | medium | Successful rows assert only Lowered, not the originating node’s typed semantic value | tests/it/checked_package_v2_lowering.rs:160 |
| FND-003 | medium | Failed/Work work-boundary cases omit the required serialized counters and empty output controls | tests/it/checked_package_v2_lowering.rs:226 |

## Coverage

Examined FR-035-AC-9: "The typed profile exposes `required_source_selections: Vec<CheckedSelection>` and the IR adapter conforms to QSpec FR-330-AC-10 and TC-935: input refusals use `InvalidInput { node_id }`/`invalid_input`, work stops use existing `Failed`/`Work` with unchanged key/limit/consumed counters, and successful checks continue the existing FR-038 meter without reset. The adapter preserves exact source provenance and emits only `profile.reference`, with no duplicated requirement wire field or new refusal payload."

Examined TC-044 typed source-selection adapter section against authoritative QSpec FR-330-AC-10/TC-935. Read all 19 changed files: crates/quire-contract-model/src/checked_package/v2/lower.rs, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs, tests/conformance_qsl_ir769/main.rs, tests/conformance_qsl_ir779/main.rs, tests/it/admitted_guard_context.rs, tests/it/admitted_result_context.rs, tests/it/checked_package_v2_abstraction_relation.rs, tests/it/checked_package_v2_dependency_reference.rs, tests/it/checked_package_v2_flat_wire.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_identity_digests.rs, tests/it/checked_package_v2_lowering.rs, tests/it/checked_package_v2_parameters.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_require_bounds.rs, tests/it/checked_package_v2_temporal.rs, tests/it/checked_package_v2_union.rs, tests/it/complete_v1_checked_package.rs, tests/it/complete_v1_contract_package.rs.

Plan completion: not assessed. Optional repository-wide semantic review was not requested; this PR review nonetheless checked the changed test-to-AC intent. Full gate and final coverage are owned by the code lead.
