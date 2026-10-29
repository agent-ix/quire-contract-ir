---
id: SR-612093
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

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | `tc_935_rows()` now spells the exact T/D/U values, but it independently defines the three QSpec-owned selection rows in IR test code, contrary to TC-044's explicit no-copy/no-redefinition clause. An owner-sourced vector or owner decision is still needed. |
| FND-002 | fixed | The successful path compares each lowered typed node's full serialized value against the corresponding originating checked-package node by requested key. |
| FND-003 | fixed | `assert_tc_935_failed_work` checks typed `Failed`, empty lowered/dependencies/semantic graph, exact retained lock/package id/profile reference, and decimal-string wire limit/consumed for mismatch and one-under cases. |
| FND-001 | fixed | QSpec publishes authoritative JSON; `vectors` reads it from `QUIRE_SPECIFICATION_DIR`, `row` resolves T/D/U from `selections`, and the local literal rows are removed. |

## Review ID correction

This artifact replaces the same PR review artifact formerly identified as `SR-612090`. The old ID collided with another review on merged main and is superseded. Findings and disposition history above are unchanged.

## Owner-vector and rebase review

At frozen PR rebased on main, the changed scope adds `tests/conformance_qspec/main.rs` and `tests/conformance_qspec/selection_vectors.rs`. The latter reads `proposals/checked-package-v2/contract-package-selection-vectors.json` through `QUIRE_SPECIFICATION_DIR` in the named QSpec conformance target; no selection row values or fixture file are copied into IR. QSpec current main carries the same file bytes as merged owner. Its owner test checks the JSON against TC-935's rows and case prose. The target has `test = false`, is compiled by `make ci` through `conformance-qspec-compile`, and is executed by `make conformance-qspec`. Both new `#[trace("TC-935", "FR-035-AC-9")]` tests call the real admitted V2 reader, lowering API and wire adapter. No code or gate narrowing appears in this PR diff. Final aggregate and named QSpec conformance execution remain the code lead's gate.

## Current verdict

**PASS** at the reviewed PR head: latest disposition for FND-001 is fixed, and FND-002/FND-003 remain fixed. The original conditional verdict and findings are retained as the review history.
