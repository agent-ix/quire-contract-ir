---
id: SR-2026110414560032
title: "IR-1345 PR 911 code-review review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md; spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Spec-only diff. Checked changed artifacts for copied files, tracking pins, code or gate changes, and cross-repo shape restatement. The QSpec FR-322 reference selects the authoritative producer form without adding an independently owned schema. No applicable AssuranceProfile was found under spec/.

## Scope and evidence

| Unit | Path | Role | Excerpt |
| --- | --- | --- | --- |
| FR-038 quantity policy | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | examined | A position declared at an admitted `carried_quantity` with its admitted bounded numeric carrier is bounded at that position and does not raise `requires_bound` for its underlying unit. |
| FR-038-AC-73 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | examined | A record field and a parameter each declared at a self-typed `scalar_type`/`carried_quantity` in QSpec FR-322's `Quantity<u, C>` form, with an admitted bounded numeric carrier, each lower. |
| TC-050 quantity procedure | spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md | examined | From the published QSpec FR-322 `Quantity<u, C>` form, lower a record whose field is declared at a self-typed `scalar_type`/`carried_quantity` with an admitted bounded numeric carrier, and lower a parameter declared at the same form. |
| TC-050 summary | spec/checked_package/matrix/tests.md | examined | a quantity at a record field, parameter or collection element declared directly at a `unit` or `compound_unit` requires a bound, while one declared at QSpec FR-322's admitted `carried_quantity` with a bounded numeric carrier lowers |

## Verdict

PASS — no code-review finding in this spec-only diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
