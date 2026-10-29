---
id: SR-612092
title: "Code and Rust review of IR-1080 PR #644"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1080; PR #644; crates/quire-contract-model/src/checked_package/v2/lower.rs, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs, tests/conformance_qsl_ir769/main.rs, tests/conformance_qsl_ir779/main.rs, tests/it/admitted_guard_context.rs, tests/it/admitted_result_context.rs, tests/it/checked_package_v2_abstraction_relation.rs, tests/it/checked_package_v2_dependency_reference.rs, tests/it/checked_package_v2_flat_wire.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_identity_digests.rs, tests/it/checked_package_v2_lowering.rs, tests/it/checked_package_v2_parameters.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_require_bounds.rs, tests/it/checked_package_v2_temporal.rs, tests/it/checked_package_v2_union.rs, tests/it/complete_v1_checked_package.rs, tests/it/complete_v1_contract_package.rs"
review_set: subset
---

## Summary

Reviewed all 19 changed Rust files, public profile API, typed row matching, first-charge and comparison accounting, refusal precedence, and repo conventions. No production code defect found. Final aggregate gate belongs to the code lead.

## Verdict

**PASS** — No scoped defect found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Examined FR-035-AC-9: "The typed profile exposes `required_source_selections: Vec<CheckedSelection>` and the IR adapter conforms to QSpec FR-330-AC-10 and TC-935: input refusals use `InvalidInput { node_id }`/`invalid_input`, work stops use existing `Failed`/`Work` with unchanged key/limit/consumed counters, and successful checks continue the existing FR-038 meter without reset. The adapter preserves exact source provenance and emits only `profile.reference`, with no duplicated requirement wire field or new refusal payload."

Examined TC-044 typed source-selection adapter section against authoritative QSpec FR-330-AC-10/TC-935. Read all 19 changed files: crates/quire-contract-model/src/checked_package/v2/lower.rs, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs, tests/conformance_qsl_ir769/main.rs, tests/conformance_qsl_ir779/main.rs, tests/it/admitted_guard_context.rs, tests/it/admitted_result_context.rs, tests/it/checked_package_v2_abstraction_relation.rs, tests/it/checked_package_v2_dependency_reference.rs, tests/it/checked_package_v2_flat_wire.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_identity_digests.rs, tests/it/checked_package_v2_lowering.rs, tests/it/checked_package_v2_parameters.rs, tests/it/checked_package_v2_recursive_leaves.rs, tests/it/checked_package_v2_require_bounds.rs, tests/it/checked_package_v2_temporal.rs, tests/it/checked_package_v2_union.rs, tests/it/complete_v1_checked_package.rs, tests/it/complete_v1_contract_package.rs.

Plan completion: not assessed. Optional repository-wide semantic review was not requested; this PR review nonetheless checked the changed test-to-AC intent. Full gate and final coverage are owned by the code lead.

## Review ID correction

This artifact replaces the same PR review artifact formerly identified as `SR-612089`. The old ID collided with another review on merged main and is superseded. Findings and disposition history above are unchanged.

## Owner-vector and rebase review

At frozen PR head `70f23389574af7a0880013fd3b0c2cb845d4e9ec` rebased on main `09d9edc7`, the changed scope adds `tests/conformance_qspec/main.rs` and `tests/conformance_qspec/selection_vectors.rs`. The latter reads `proposals/checked-package-v2/contract-package-selection-vectors.json` through `QUIRE_SPECIFICATION_DIR` in the named QSpec conformance target; no selection row values or fixture file are copied into IR. QSpec current main carries the same file bytes as merged owner commit `ae67d17d`. Its owner test checks the JSON against TC-935's rows and case prose. The target has `test = false`, is compiled by `make ci` through `conformance-qspec-compile`, and is executed by `make conformance-qspec`. Both new `#[trace("TC-935", "FR-035-AC-9")]` tests call the real admitted V2 reader, lowering API and wire adapter. No code or gate narrowing appears in this PR diff. Final aggregate and named QSpec conformance execution remain the code lead's gate.
