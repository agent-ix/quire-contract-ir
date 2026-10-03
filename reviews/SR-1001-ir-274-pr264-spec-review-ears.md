---
id: SR-1001
title: "requirement-statement shape review of PR 264"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@184e6668bb8722f03c68a6b9924586593233118d; new and amended statements and acceptance criteria in FR-011, FR-012, FR-013, FR-016, FR-020, FR-032, FR-033, FR-034, STD-003 and FR-038 (git diff origin/main...HEAD)"
review_set: subset
---
# SR-1001: requirement-statement shape review of PR 264

## Summary

Ticket: IR-274. This repo writes acceptance criteria as direct assertions, not
"shall" statements, and states requirements as prose or as "shall" bullets
(FR-032 to FR-034). The new ACs (FR-013-AC-5, FR-016-AC-5 to AC-8,
FR-020-AC-3, FR-032-AC-6, FR-033-AC-6, FR-034-AC-6 and AC-7, FR-038-AC-89 to
AC-94) each name a stimulus and an observable outcome. The new FR-032 and
FR-033 bullets are single obligations with a stated trigger. `quire validate`
reports no new grammar finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-034's second new bullet packs one obligation (spell each `u64` as a decimal string), its rationale ("RFC 8785 spells no integer past 2^53 exactly ...") and a derived claim about other requirements ("so no encode of identity material refuses for an integer") into one "shall" item. Split the obligation from the rationale and the cross-reference | spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md:66-74 |

## Verdict

Acceptable. One low compound statement.

## Dispositions

Reviewed at agent-ix/quire-contract-ir@553736cfce62fe6949915559c471d4a7f5349d21 (main still 7d7716d; not rebased onto #263, whose FR-038, TC-048 and checked_package tests.md hunks conflict textually, expected rebase work). Proof re-run: no schemas/, corpus or code file in `git diff origin/main --stat`; `make corpus` exit 0; workspace tests all pass (204 + 101, 0 failed); validate passes, grammar 1 (FR-014 baseline), 215 ACs, strict 23 unbacked, coverage rows 237.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 553736c |
