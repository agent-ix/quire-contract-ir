---
id: SR-1003
title: "evidence review of PR 264 (verification methods of the new criteria)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@184e6668bb8722f03c68a6b9924586593233118d; verification column and matrix rows for FR-013-AC-5, FR-016-AC-5 to AC-8, FR-020-AC-3, FR-032-AC-6, FR-033-AC-6, FR-034-AC-6, FR-034-AC-7, FR-038-AC-89 to AC-94; TC-017, TC-018, TC-043, TC-048"
review_set: subset
---
# SR-1003: evidence review of PR 264

## Summary

Ticket: IR-274. Every new AC names a test case (TC-016/TC-018, TC-017, TC-018,
TC-043 or TC-048) and is marked planned in its matrix, so no implemented status
is overstated. The FR-034-AC-6 package-identity step tested through a
ceiling parameter is a sound unit seam. A public route is not guaranteed to
produce package material over the request limit (author decision e).
TC-048's procedure section covers AC-89 to AC-94. TC-017's row lists its
planned cases. TC-043's coverage design adds the exact and one-over identity
cases.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-016-AC-6, FR-016-AC-7 (first clause), FR-038-AC-90 (source clause), FR-038-AC-91 and FR-038-AC-92 are verified by reading crate source, but are marked "Test". This repo marks a source-reading check "Inspection" elsewhere (FR-013-AC-2). Merged FR-038-AC-80 also uses "Test", so the repo is inconsistent; choose one method for source scans | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1411-1413 |

## Verdict

Acceptable. The missing lowering-ceiling AC is recorded in SR-999 FND-006.

## Dispositions

Reviewed at agent-ix/quire-contract-ir@553736cfce62fe6949915559c471d4a7f5349d21 (main still 7d7716d; not rebased onto #263, whose FR-038, TC-048 and checked_package tests.md hunks conflict textually, expected rebase work). Proof re-run: no schemas/, corpus or code file in `git diff origin/main --stat`; `make corpus` exit 0; workspace tests all pass (204 + 101, 0 failed); validate passes, grammar 1 (FR-014 baseline), 215 ACs, strict 23 unbacked, coverage rows 237.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | The new source-reading criteria now uniformly use Test (TC-...), matching merged FR-038-AC-80; FR-013-AC-2's Inspection is pre-existing and outside this PR. Consistent within the PR's scope, so no change is needed here. |
