---
id: SR-202610310121
title: "Gap analysis of IR-1222 model enum stage-row tests PR 766"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 766 diff at 387681d0; FR-354, FR-354-AC-6, FR-019-AC-5"
review_set: subset
---

## Summary

Mapped the changed model enum tests to FR-354-AC-6 using the computed Quoin Test Matrix. The new functions bind the criterion, but each asserts only stage-row joining. Existing source-owned nominal enum tests cover an ordered/unordered read; no model-owned complete enum-index read is present in this PR.

## Verdict

FAIL. Both new FR-354-AC-6 trace tags claim the complete criterion without testing its ordered/unordered `EnumMemberIndex` result and complete partial-index refusal.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-354-AC-6 tag claims complete enum-index evidence, but this test returns stage rows only and checks no ordered flag or authored position. | tests/it/checked_package_v2_model_members.rs:653 |
| FND-002 | high | FR-354-AC-6 tag claims complete adverse enum-index evidence, but this test checks only row-join refusals and never requests a complete index. | tests/it/checked_package_v2_model_members.rs:696 |

## Evidence

The tests correctly exercise the admitted model row join. The full criterion additionally requires ordered and unordered selected enum views with exact case order, member identity and position, and no partial index. These tests never call a complete enum-index read. Remove the FR-354-AC-6 tags from these partial tests; keep their behavioral assertions. The future IR-1212 read can bind the full criterion. Plan completion: not assessed.

## Coverage

The computed Quoin matrix binds FR-354-AC-6 to the two changed model enum tests and three existing source enum tests. FR-019-AC-5 is bound to four existing public-interface tests. Repository-wide static status is 712 tagged, 143 untagged, 22 method-without-symbol, and one tagged-by-ignored-test; the unrelated inherited gaps are outside this PR diff.

Plan completion: not assessed

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head ead9d58d: the first model enum stage-row test no longer claims FR-354-AC-6; it remains a focused behavior test. After excerpt: `#[test]\nfn model_enum_stage_rows_join_selected_declaration_and_exact_members()`. |
| FND-002 | fixed | Round 1, head ead9d58d: the second model enum stage-row test no longer claims FR-354-AC-6; it remains a focused adverse join test. After excerpt: `#[test]\nfn model_enum_stage_rows_refuse_foreign_missing_duplicate_and_wrong_case()`. |
