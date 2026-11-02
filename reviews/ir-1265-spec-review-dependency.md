---
id: SR-2026110101274
title: "IR-1265 PR 821 dependency review"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; IR-1265 PR #821; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Checked the added QSpec references and TC-048 verification edges for direction and prerequisites.

## Verdict

No dependency-analysis finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-421 FMA positive strict-reader admission: examined.
- FR-038-AC-422 FMA adverse strict-reader refusals: examined.
- TC-048 FMA by-reference plan and QSpec trace edges: examined.
- QSpec FR-322-AC-307, TC-233 FM-01..FM-04, and FM-01..FM-12 vector file: context_only.
- Current IR origin/main FR-038-AC-421..430 (IR-721 monitor plan): context_only.

## Evidence

QSpec FR-322-AC-307 and FM vectors already exist at eac40c3a; the IR requirement depends on that published contract. TC-048 verifies the IR criteria. The duplicated target IDs are separately recorded by base/integrity review and must be corrected before the edges can be used.
