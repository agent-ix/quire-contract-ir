---
id: SR-2026110101272
title: "IR-1265 PR 821 base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1265 PR #821; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Reviewed the new FMA strict-reader criteria and TC-048 plan against QSpec FR-322-AC-307 and FM-01..FM-12, then compared criterion IDs with current IR main.

## Verdict

Two high ID collisions block merge until the criteria and TC-048 relationships are renumbered on a rebase.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-421 is already assigned to IR-721 monitor-plan admission on current main; the FMA criterion and its TC-048 link would collide | FR-038:5349; origin/main FR-038:5499; TC-048:32 |
| FND-002 | high | FR-038-AC-422 is already assigned to IR-721 monitor-plan refusal rules on current main; the FMA criterion and its TC-048 link would collide | FR-038:5350; origin/main FR-038:5500; TC-048:34 |

## Coverage

- FR-038-AC-431 FMA positive strict-reader admission: examined.
- FR-038-AC-432 FMA adverse strict-reader refusals: examined.
- TC-048 FMA by-reference plan and QSpec trace edges: examined.
- QSpec FR-322-AC-307, TC-233 FM-01..FM-04, and FM-01..FM-12 vector file: context_only.
- Current IR origin/main FR-038-AC-421..430 (IR-721 monitor plan): context_only.

## Evidence

QSpec origin/main eac40c3a defines generic call3 FMA and the published two admitted/ten refused vectors. The FMA prose matches those outcomes and records the unvectored rounding-mode disagreement without assigning it an outcome. IR origin/main 2aba7d2f already has AC-421..430 from IR-721; the next free numbers at review time are 431/432. Scoped Quire validation passes on the frozen branch but cannot detect a future merge collision.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Rebasing onto IR main retained the IR-721 AC-421 and renumbered the FMA admission criterion and TC-048 verifies edge to unique AC-431. |
| FND-002 | fixed | Rebasing onto IR main retained the IR-721 AC-422 and renumbered the FMA refusal criterion and TC-048 verifies edge to unique AC-432. |

## Current Verdict

At PR head 4c4c0b7d, the two ID collisions are resolved. The FMA requirement text and by-reference plan are otherwise unchanged; no new findings arose.
