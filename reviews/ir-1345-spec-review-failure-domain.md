---
id: SR-2026110414560030
title: "IR-1345 PR 909 spec-review/failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md"
review_set: subset
---

## Summary

Checked position locality, closure coexistence, domain base-chain termination, requested-node and annotation exceptions. No further failure-domain finding beyond the AC coverage gap recorded in base. Review of frozen head 3f4054a915ee343fa880abb3d993dfa5a38cc4f1. No applicable AssuranceProfile under spec/.

## Scope and evidence

| Unit | Path | Role | Excerpt |
| --- | --- | --- | --- |
| FR-038 quantity policy | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | examined | A position declared at a bounded_domain whose base chain ends at a unit or compound_unit is bounded and does not raise requires_bound for that underlying type. |
| FR-038-AC-73 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | examined | A record field declared at a bounded_domain over a unit lowers; a second field declared directly at that unit in the same closure still refuses naming the unit. |
| TC-050 quantity procedure | spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md | examined | Lower a record whose field is declared at a bounded_domain over a unit node. Lower a record with both a field declared directly at that unit and a field declared at the domain over it. |
| TC-050 quantity expected | spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md | examined | The domain-typed record lowers. The mixed record returns requires_bound naming its directly used unit, despite the domain elsewhere in its closure. |
| FR-038-AC-39 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | context_only | A literal.type annotation is not a value position. |
| FR-038-AC-40 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | context_only | A bounded_domain elsewhere in the closure does not bound a separately declared unbounded type position. |

## Verdict

PASS — no additional defect in this method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
