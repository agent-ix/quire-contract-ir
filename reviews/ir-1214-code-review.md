---
id: SR-202611010118
title: "follow-up code review of IR-1214 PR #797"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@27c8cf7c; PR #797; changed dispatch candidate carrier, typed validation, and tests"
review_set: subset
---

## Summary

Ticket: IR-1214. Reviewed current PR #797 head `27c8cf7c`, including required candidate-level `selected_operation`, canonical encoding, equality validation, nonempty dispatch admission, and the latest allowance for model-owned function IDs outside the emitted root package. The handles must still resolve in the admitted graph and have the required function kind. No applicable AssuranceProfile was found. Focused integration tests, formatting, and workspace library Clippy were run through `locked-build.sh` and passed.

## Reviewed units

- **FR-322-AC-296**, QSpec FR-322: examined strict candidate witness equality and exact mismatch refusal path.
- **FR-322-AC-290**, QSpec FR-322: examined the producer-owned `(selected_operation, selection_key)` mapping and handle membership.
- **FR-354-AC-4**, `spec/checked_package/functional/FR-354-project-admitted-runtime-context.md`: examined candidate order, precondition handles, and refusal behavior.
- `crates/quire-contract-model/src/checked_package/v2/checked_term_data.rs`, `dispatch_projection.rs`, and `mod.rs`.
- `tests/it/checked_package_v2_model_members.rs` and `tests/it/checked_package_v2_term_data_writer.rs`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. Candidate witnesses are required, encoded canonically, compared with the parent operation witness before admission, and mismatches refuse at the candidate `selected_operation` pointer. Valid nonempty candidates are admitted in producer order. Dependency model handles resolve through the admitted graph index and retain function-kind validation; requiring their package to equal the outer emitted package would reject the QSL producer's model-owned handles. No correctness, resource-bounding, or test-oracle defect was found in the reviewed delta.
