---
id: SR-2026110414560035
title: "IR-1345 PR 911 spec-review/ears-conformance review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md; spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

The edited FR-038 quantity policy has a named condition and observable admitted/refused results. Quire's full-spec grammar summary reports 219/221 clean documents and an unchanged FR-038 non-canonical-shape advisory outside this diff. No applicable AssuranceProfile was found under spec/.

## Scope and evidence

| Unit | Path | Role | Excerpt |
| --- | --- | --- | --- |
| FR-038 quantity policy | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | examined | A position declared at an admitted `carried_quantity` with its admitted bounded numeric carrier is bounded at that position and does not raise `requires_bound` for its underlying unit. |
| FR-038-AC-73 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | examined | A record field and a parameter each declared at a self-typed `scalar_type`/`carried_quantity` in QSpec FR-322's `Quantity<u, C>` form, with an admitted bounded numeric carrier, each lower. |

## Verdict

PASS — no EARS defect in the changed quantity statement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
