---
id: SR-2026110414560033
title: "IR-1345 PR 911 spec-review/base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md; spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Reviewed the corrective quantity policy against QSpec FR-322's published carried quantity wire form and TC-050. No applicable AssuranceProfile was found under spec/. The positive rule names three admitted numeric carrier forms, but the acceptance evidence does not discriminate them.

## Scope and evidence

| Unit | Path | Role | Excerpt |
| --- | --- | --- | --- |
| FR-038 quantity policy | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | examined | A position declared at an admitted `carried_quantity` with its admitted bounded numeric carrier is bounded at that position and does not raise `requires_bound` for its underlying unit. |
| FR-038-AC-73 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | examined | A record field and a parameter each declared at a self-typed `scalar_type`/`carried_quantity` in QSpec FR-322's `Quantity<u, C>` form, with an admitted bounded numeric carrier, each lower. |
| TC-050 quantity procedure | spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md | examined | From the published QSpec FR-322 `Quantity<u, C>` form, lower a record whose field is declared at a self-typed `scalar_type`/`carried_quantity` with an admitted bounded numeric carrier, and lower a parameter declared at the same form. |
| TC-050 summary | spec/checked_package/matrix/tests.md | examined | a quantity at a record field, parameter or collection element declared directly at a `unit` or `compound_unit` requires a bound, while one declared at QSpec FR-322's admitted `carried_quantity` with a bounded numeric carrier lowers |
| QSpec FR-322 quantity form | quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md | context_only | `Quantity<u, C>` \| `scalar_type`/`carried_quantity`, self-typed (family `quantity`) |

## Verdict

FAIL — AC-73 and TC-050 do not require a positive control for each published carrier form.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-73 and TC-050 leave integer_range, decimal_range and rational_range carrier coverage unspecified; an implementation accepting only one form could pass the stated controls. Require an admitted carried_quantity value position for each carrier form. | FR-038-AC-73; TC-050 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-73 and TC-050 now require distinct admitted carried quantity positions for integer_range, decimal_range and rational_range, each with a lower outcome. |
