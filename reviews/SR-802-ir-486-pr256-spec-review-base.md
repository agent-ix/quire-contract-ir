---
id: SR-802
title: "spec review of PR 256 matrix rows (IR-486)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@c832ea54d6528369ac8de8f554d8b0d0bcd5d51b; spec/checked_package/matrix/tests.md"
review_set: subset
---
# SR-802: spec review of PR 256 matrix rows

## Summary

Ticket: IR-486. The only spec file this PR changes is `spec/checked_package/matrix/tests.md`,
in two rows, FR-038 and TC-048. No requirement text changes. AC-70 to AC-72 move into the
implemented list. The "planned, AC-43 asserts the refusal" note is removed from both rows.
The TC-048 row names the new unit tests and `tests/it/checked_package_v2_recursive_leaves.rs`.
The other PRs' planned text (AC-45, AC-46 to AC-61, AC-62 to AC-64, AC-65 to AC-69) is
byte-identical to origin/main. The other rows are untouched.

Merge interaction. Open PRs #250 (IR-505) and #253 (IR-530 and IR-503) edit the same two
physical lines, 15 and 23, and #250 also edits lines 13 and 28. Whichever of these merges
second or third gets a textual conflict on lines 15 and 23. The edits change different
clauses of the same prose, so they are compatible, but the conflict needs a hand merge
rather than a rerere. Each PR also carries its own FR-038 backed count in prose.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The FR-038 row still says "`quire coverage --strict` still counts them backed with no contradiction (42 of 70 FR-038 rows)". At this head, quire counts 45/70 (measured; origin/main is 42/70). #253's reviewer flagged the same stale count. The count is a hand-kept record that every PR touching FR-038 makes stale, and it conflicts on every merge. Drop the count, rather than updating it, and leave the number to `quire coverage` | spec/checked_package/matrix/tests.md:15 |

## Verdict

Approve the row changes. Fix FND-001 with the next touch of the row. Dropping the count
also removes one recurring merge conflict with #250 and #253.

## Dispositions

Round 1 at e44305c9dd30c38062d7bac75af5c2a7e2c223c0. The hand-kept count '(42 of 70 FR-038 rows)' is deleted from the FR-038 row and no other count was added. The word diff of tests.md is that one deletion only. No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed e44305c9dd30c38062d7bac75af5c2a7e2c223c0 |
