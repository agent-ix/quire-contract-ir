---
id: SR-2026110101276
title: "IR-1265 PR 821 evidence review"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; IR-1265 PR #821; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Compared the authored Test method and TC-048 plan with the declared evidence-method catalog and published FM vector cases.

## Verdict

No evidence-method mismatch.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-431 FMA positive strict-reader admission: examined.
- FR-038-AC-432 FMA adverse strict-reader refusals: examined.
- TC-048 FMA by-reference plan and QSpec trace edges: examined.
- QSpec FR-322-AC-307, TC-233 FM-01..FM-04, and FM-01..FM-12 vector file: context_only.
- Current IR origin/main FR-038-AC-421..430 (IR-721 monitor plan): context_only.

## Evidence

quoin advise --json, rerun on rebased head 4c4c0b7d, reports authored Test and mismatch=false for both AC-431 and AC-432. TC-048 requires a complete package per published vector and a public-reader verdict. The planned rows have no implementation evidence yet by design; the unvectored rounding-mode mismatch remains explicitly unverified.

## Post-rebase verification

At PR head 4c4c0b7d, the FMA criteria and TC-048 verifies edges use unique FR-038-AC-431/432. No substantive requirement or test-plan text changed.
