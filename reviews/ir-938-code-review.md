---
id: SR-14736
title: "IR-938 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; spec/checked_package, crates/quire-contract-model/src/checked_package/v2, tests/it"
review_set: subset
---

## Summary

Reviewed Rust code and public-reader tests in PR #497 at head 069bbec05fa8373551c59f193e48d2e805bd8580. Examined FR-038-AC-21, FR-038-AC-299 through AC-302, and TC-230.

## Verdict

**CONDITIONAL — the unit-only quantity type rule has an untested familyless-type escape.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A quantity literal whose type resolves to no operand family skips literal_defect before the Quantity match. A valid familyless type such as scalar_type/dimension can therefore bypass the unit-only type rule instead of refusing ill_typed/operator-ineligible at type. | crates/quire-contract-model/src/checked_package/v2/operations.rs:360-367 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The Quantity variant now reaches the family match when type resolution has no family, which returns IllTyped/OperatorIneligible at the literal type. |
