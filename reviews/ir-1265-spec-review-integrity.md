---
id: SR-2026110101273
title: "IR-1265 PR 821 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-1265 PR #821; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Checked atomicity, consistency, and unique identifiers of the two FMA criteria and the TC-048 plan against current main and QSpec.

## Verdict

The criterion identities collide with unrelated monitor-plan requirements on current main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-421 is already assigned to IR-721 monitor-plan admission on current main; the FMA criterion and its TC-048 link would collide | FR-038:5349; origin/main FR-038:5499; TC-048:32 |
| FND-002 | high | FR-038-AC-422 is already assigned to IR-721 monitor-plan refusal rules on current main; the FMA criterion and its TC-048 link would collide | FR-038:5350; origin/main FR-038:5500; TC-048:34 |

## Coverage

- FR-038-AC-421 FMA positive strict-reader admission: examined.
- FR-038-AC-422 FMA adverse strict-reader refusals: examined.
- TC-048 FMA by-reference plan and QSpec trace edges: examined.
- QSpec FR-322-AC-307, TC-233 FM-01..FM-04, and FM-01..FM-12 vector file: context_only.
- Current IR origin/main FR-038-AC-421..430 (IR-721 monitor plan): context_only.

## Evidence

Each proposed criterion is separately testable and the positive/adverse split is clear. The current-main AC-421/422 meanings are different, making both duplicated identities ambiguous for traceability and the TC-048 verifies edges. No other internal contradiction was found.
