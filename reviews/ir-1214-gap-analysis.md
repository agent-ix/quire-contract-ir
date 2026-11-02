---
id: SR-202611010119
title: "follow-up gap analysis of IR-1214 PR #797"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@883da283; changed dispatch projection and tests in PR #797; QSpec FR-322-AC-290/296; IR FR-354-AC-4"
review_set: subset
---

## Summary

Ticket: IR-1214. Recomputed the repository Test Matrix at PR head `883da283` with `quire matrix --scope /home/peter/dev/worktrees/qsl-1397-ir-witness --format json`. FR-354-AC-4 remains tagged by the same three tests; the new adverse non-Boolean `PureFunction` mutation is inside the existing positive dispatch test and refuses at the exact effective-precondition pointer. The broader matrix reports 126 untagged criteria outside this PR's dispatch-projection subset; no new untagged criterion was found in the changed behavior. Plan completion: not assessed. Optional semantic review was skipped.

## Reviewed units

- **FR-354-AC-4**, `spec/checked_package/functional/FR-354-project-admitted-runtime-context.md`: matrix reports `tagged`, with bindings to the three tests listed below.
- **FR-322-AC-296**, QSpec FR-322: examined candidate witness requirement and exact refusal locus; TC-233 is tagged to this external owner requirement.
- `tests/it/checked_package_v2_model_members.rs`: examined `checked_dispatch_sidecar_retains_empty_and_bounded_candidate_sets` (including the non-Boolean pure-function substitution), `checked_dispatch_sidecar_refuses_wrong_witness_and_duplicate_key`, and `checked_dispatch_candidate_cannot_borrow_another_rows_selected_operation`.
- Production dispatch validation in `crates/quire-contract-model/src/checked_package/v2/dispatch_projection.rs`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No gaps found in the changed dispatch-projection obligations | - |

## Coverage

- Plan completion: not assessed.
- FR-354-AC-4 remains tagged by the nonempty-order/mapping test, malformed-witness and duplicate-key test, and cross-row witness-substitution test. The non-Boolean pure-function adverse case is included in the first test and refuses at `/checked_term_data/dispatch_tables/0/candidates/0/effective_precondition`.
- TC-233 retains FR-322-AC-296 coverage; the candidate witness mutation refuses at the specified `/checked_term_data/dispatch_tables/0/candidates/0/selected_operation` locus.
- The 126 untagged criteria in the full repository matrix are outside the changed dispatch-projection subset.

## Verdict

PASS for the reviewed subset. The changed dispatch behavior has criterion-backed tests, and no new code-to-spec gap or hollow test evidence was found.
