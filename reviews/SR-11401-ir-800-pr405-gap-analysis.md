---
id: SR-11401
title: "IR-800 PR 405 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@0768eb1b35f1736b785479e8efcb4c688e9181ee (base 84734511996c8fff2574037397f49c44d3efdd05); spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; tests/it/checked_package_v2_operand_families.rs; tests/it/checked_package_v2_model_fields.rs; tests/it/checked_package_v2_model_members.rs; crates/quire-contract-model/src/checked_package/v2"
review_set: subset
---

## Summary

Ticket IR-800, PR quire-contract-ir#405. A planless gap analysis of the diff against FR-038-AC-242..245 and the native String/UUID/Timestamp section of TC-048. It includes a test-oracle-strength lane: for each new test, can it fail for the right reason, and would an unrelated refusal also make it pass? `quoin matrix --json` (quoin 0.28.3) reports all four criteria as `tagged`. The binders are the six new tests; evidence status is "no run evidence", because the matrix binds no run record. The focused runs in this review passed.

## Examined units

- FR-038-AC-242: native String/UUID/Timestamp fields, parameters and results admit, with distinct self-typed scalar nodes and matching member types. String never resolves as `text` and never requires a text profile. Each literal keeps its exact canonical payload. (examined)
- FR-038-AC-243: exactly six native `eq`/`ne` identities. Each refuses a foreign operand at argument 0 and, separately, at argument 1. `integer.lt` and `integer.add` over each native family refuse at `arguments/0`. (examined)
- FR-038-AC-244: a `text_profile` law refuses `operation-law-mismatch` and a mode refuses `operation-mode-mismatch` on every native comparison. A String operand in `text.eq` refuses `operation-mode-type-mismatch` at the mode value. (examined)
- FR-038-AC-245: a wrong-family literal type refuses at `type` before its payload is checked. Noncanonical String, UUID and Timestamp payloads refuse `invalid-value` at `value`. Endpoints and nil/max admit. Keys are recomputed. (examined)
- FR-038 statement item 22 (closed `CheckedMemberType` variant list). (examined)
- TC-048 section "Native String, UUID and Timestamp admission". (examined)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038 item 22 still lists the closed `CheckedMemberType` variants without the new String, Uuid and Timestamp variants that the PR adds to the public enum | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2718 |
| FND-002 | low | AC-242..245 and the TC-048 section still read PLANNED/UNRUN (IR-800) and claim no evidence, though tagged, passing tests now exist | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4338 |
| FND-003 | low | The work-limit control's bisection treats any `Refused` result as the threshold. It never asserts that the refusal at the threshold budget is the invalid-value refusal | tests/it/checked_package_v2_operand_families.rs:487 |
| FND-004 | low | The String-in-`text.eq` control places String only at argument 0, so a guard that reads only `arguments[0]` would still pass | tests/it/checked_package_v2_operand_families.rs:586 |
| FND-005 | low | The native comparison controls use Timestamp payloads 0 and 1 only. TC-048 names 0, 1 and -1 for the equal/unequal comparison payloads | tests/it/checked_package_v2_operand_families.rs:269 |

## Verdict

**CONDITIONAL.** Every examined criterion has a tagged test that asserts the exact code, cause, pointer and node locus, and the tests pass. Trace bindings are correct:

- `selected_native_fields_keep_distinct_member_types` and `selected_native_parameters_and_results_keep_their_type_nodes` bind to AC-242.
- `native_scalar_comparisons_keep_distinct_types_and_operands` binds to AC-242 and AC-243.
- `native_comparisons_reject_text_laws_and_modes` and `text_comparison_rejects_string_at_its_mode_value` bind to AC-244.
- `native_literal_payloads_are_canonical_after_type_validation` and `native_literal_work_limit_precedes_later_payload_defect` bind to AC-245.

Oracle strength holds for the main controls:

- The foreign-operand mutations assert the changed argument's own pointer, so a one-sided check fails at the other position.
- The literal type-before-payload control pairs a wrong type with a malformed value.
- Every mutation re-keys through `settle`, so a stale-key refusal cannot stand in.
- The catalog comparison reads the dependency's own catalog bytes.

The medium finding is a spec statement the code now exceeds. It and the four lows are open.

## Coverage

Plan completion: not assessed. Semantic review: scoped to FR-038-AC-242..245, their TC-048 procedure, and the new tests and code in the diff. Test-oracle strength is reviewed as a lane of this analysis, because the SpecReview `analysis` vocabulary has no separate test-oracle value.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | b747d65f7571590fe8e234ff86f471fb4e778070: item 22 now lists String, Uuid and Timestamp. Observation, not a finding against this PR: Record and Variant are still missing from the list, which predates this PR |
| FND-002 | fixed | b747d65f7571590fe8e234ff86f471fb4e778070: AC-242..245 read "IMPLEMENTED (IR-800; tagged reader tests passed)", matching the AC-225..227 convention, and TC-048 drops the "claims none" text. All four criteria have tagged tests that pass at b747d65 |
| FND-003 | fixed | b747d65f7571590fe8e234ff86f471fb4e778070: asserts the exact invalid_package/invalid-value refusal at /body/value for budget `high`, Incomplete with limit_kind Work and limit high-1 below it, and high < upper |
| FND-004 | fixed | b747d65f7571590fe8e234ff86f471fb4e778070: text_comparison_rejects_string_at_its_mode_value now loops String over argument positions 0 and 1 |
| FND-005 | still-open | b747d65 changed the Timestamp comparison pair from ("0", "1") to ("-1", "1"). -1 is now covered, but 0 is no longer an admitted comparison payload. TC-048 names 0, 1 and -1, so the controls still cover only two of the three |
| FND-005 | fixed | 572ff4ecbabfc1fb221db4c13da506ee93ead31a: the Timestamp cases are now ("-1", "1") and ("0", "1"), so 0, 1 and -1 are each admitted comparison operands under both eq and ne (round 2) |
