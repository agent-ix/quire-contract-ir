---
id: SR-202611010119
title: "follow-up gap analysis of IR-1214 PR #797"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@27c8cf7c; changed dispatch projection and tests in PR #797; QSpec FR-322-AC-290/296; IR FR-354-AC-4"
review_set: subset
---

## Summary

Ticket: IR-1214. Computed the repository Test Matrix with `quire matrix --scope /home/peter/dev/worktrees/qsl-1397-ir-witness --format json`. The changed IR obligation FR-354-AC-4 is tagged by three tests. The broader matrix reports 126 untagged criteria outside this PR's dispatch-projection subset; this review found no new untagged criterion in the changed behavior. Plan completion: not assessed. Optional semantic review was skipped.

## Reviewed units

- **FR-354-AC-4**, `spec/checked_package/functional/FR-354-project-admitted-runtime-context.md`: matrix reports `tagged`, with bindings to the three tests listed below.
- **FR-322-AC-296**, QSpec FR-322: examined candidate witness requirement and exact refusal locus; TC-233 is tagged to this external owner requirement.
- `tests/it/checked_package_v2_model_members.rs`: examined `checked_dispatch_sidecar_retains_empty_and_bounded_candidate_sets`, `checked_dispatch_sidecar_refuses_wrong_witness_and_duplicate_key`, and `checked_dispatch_candidate_cannot_borrow_another_rows_selected_operation`.
- Production dispatch validation in `crates/quire-contract-model/src/checked_package/v2/dispatch_projection.rs`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No gaps found in the changed dispatch-projection obligations | - |

## Coverage

- Plan completion: not assessed.
- FR-354-AC-4 is tagged by the nonempty-order/mapping test, malformed-witness and duplicate-key test, and cross-row witness-substitution test.
- TC-233 retains FR-322-AC-296 coverage; the candidate witness mutation refuses at the specified `/checked_term_data/dispatch_tables/0/candidates/0/selected_operation` locus.
- The 126 untagged criteria in the full repository matrix are outside the changed dispatch-projection subset.

## Verdict

PASS for the reviewed subset. The changed dispatch behavior has criterion-backed tests, and no new code-to-spec gap or hollow test evidence was found.
