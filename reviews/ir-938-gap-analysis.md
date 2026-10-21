---
id: SR-14737
title: "IR-938 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package, crates/quire-contract-model/src/checked_package/v2, tests/it"
review_set: subset
---

## Summary

Reviewed acceptance criteria, trace bindings and public tests in PR #497 at head 069bbec05fa8373551c59f193e48d2e805bd8580. Examined FR-038-AC-21, FR-038-AC-299 through AC-302, and TC-230.

## Verdict

**CONDITIONAL — one adverse quantity type case is absent from TC-230.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-230 tests wrong-family types with integer, string and Boolean, but has no quantity literal typed by an admitted familyless type such as dimension. That missing control leaves the family-none early return in literal_defect undetected. | tests/it/checked_package_v2_operand_families.rs:202-246 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | A traced public TC-230 test now constructs a dimension-typed quantity literal and asserts IllTyped/OperatorIneligible at body/type; the coder observed the test red before the fix and green after it. |
