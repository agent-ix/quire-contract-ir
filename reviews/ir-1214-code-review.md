---
id: SR-202611010118
title: "follow-up code review of IR-1214 PR #797"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@883da283; PR #797; changed dispatch candidate carrier, typed validation, and tests"
review_set: subset
---

## Summary

Ticket: IR-1214. Reviewed current PR #797 head `883da283`, including required candidate-level `selected_operation`, canonical encoding, equality validation, nonempty dispatch admission, model-owned function IDs outside the emitted root package, and the final Boolean-result validation for `PureFunction` preconditions. The non-Boolean mutation refuses at the exact effective-precondition pointer. No applicable AssuranceProfile was found. Focused integration tests, formatting, and workspace library Clippy were run through `locked-build.sh` and passed.

## Reviewed units

- **FR-322-AC-296**, QSpec FR-322: examined strict candidate witness equality and exact mismatch refusal path.
- **FR-322-AC-290**, QSpec FR-322: examined the producer-owned `(selected_operation, selection_key)` mapping and handle membership.
- **FR-354-AC-4**, `spec/checked_package/functional/FR-354-project-admitted-runtime-context.md`: examined candidate order, precondition handles, and refusal behavior.
- `crates/quire-contract-model/src/checked_package/v2/checked_term_data.rs`, `dispatch_projection.rs`, and `mod.rs`.
- `tests/it/checked_package_v2_model_members.rs` and `tests/it/checked_package_v2_term_data_writer.rs`, including the adverse integer-returning precondition substitution.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. Candidate witnesses are required, encoded canonically, compared with the parent operation witness before admission, and mismatches refuse at the candidate `selected_operation` pointer. Valid nonempty candidates are admitted in producer order. Dependency model handles resolve through the admitted graph index and retain function-kind validation; requiring their package to equal the outer emitted package would reject the QSL producer's model-owned handles. No correctness, resource-bounding, or test-oracle defect was found in the reviewed delta.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | Effective-precondition validation accepts any `PureFunction` handle without checking its result type; an integer-returning pure function can therefore be admitted as a Boolean guard. | crates/quire-contract-model/src/checked_package/v2/dispatch_projection.rs:174-180 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | The current validation checks only the function form. The fixture's `PureFunction` precondition returns Boolean, but a non-Boolean `PureFunction` remains admissible; require a Boolean semantic type and add an adverse mutation test. |
| FND-002 | fixed | `dispatch_projection.rs` now admits `PureFunction` only when its `semantic_type` resolves to `ScalarType(Boolean)`, while retaining `Predicate` support. The new adverse test substitutes an integer-returning body and checks refusal at the exact `effective_precondition` pointer. Focused dispatch tests pass 3/3. |
