---
id: SR-3143
title: "Gap analysis of the IR-690 structural inequality composite operand extension"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir branch spec/ir690-structural-ne-domains (IR-690, specification only); FR-038-AC-177 through FR-038-AC-182; FR-038-AC-197 through FR-038-AC-201; TC-048"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Ticket: IR-690. Planless gap analysis scoped to the composite accessor
criteria, using `quire matrix --format tsv` at the head. FR-038-AC-197 through
AC-200 are untagged and AC-201 is method-without-symbol (Inspection). That
matches their PLANNED/UNRUN status, and the matrix rows say they are unbacked.
AC-177 through AC-182 remain tagged by the IR-651 tests. The change itself
introduces no stub and claims no evidence it does not have.

There is one coverage gap. The public "known ineligible identity" case of
AC-181 is backed only by an assertion that structural.ne returns
IneligibleOperator. Under the amended error table that assertion describes
superseded behavior, and no tagged test exercises a catalogued identity
outside structural.eq/structural.ne.

## Examined units

- FR-038-AC-177 through FR-038-AC-182 matrix rows and binders (examined)
- FR-038-AC-197 through FR-038-AC-201 matrix rows (examined)
- tests/it/checked_package_v2_composite_operands.rs external error consumer test (examined)
- crates/quire-contract-model/src/checked_package/v2/composite_operands.rs eligibility check (context_only)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-181's IneligibleOperator payload clause shows as covered only through the structural.ne assertion in the external error consumer test (tagged AC-181/AC-182). Once the amended table makes ne eligible, that assertion must change. No other tagged test exercises a known catalogued identity outside eq/ne at the composite accessor, so AC-181's public ineligible-identity case would be unbacked while it stays counted as tagged. The TC-048 AC181 procedure already names the replacement input. Record that this binding is retargeted in the IR-690 code step, as AC-199 also requires, so the tagged count does not inflate meanwhile. | tests/it/checked_package_v2_composite_operands.rs:1351-1386; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3868; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:595-598 |

## Coverage

- Criteria in scope: 11. Tagged: 6 (AC-177 to AC-182). Untagged: 4 (AC-197 to AC-200, PLANNED). Method-without-symbol: 1 (AC-201, Inspection, PLANNED).
- `quire coverage --strict` at head: 31 unbacked rows. Five are AC-197 to AC-201, which are new and PLANNED. The other 26 are in files this change does not touch.
- Semantic review: skipped (specification-only change; no code under review).
- Plan completion: not assessed

## Verdict

CONDITIONAL. The change adds no regression in what is tagged. FND-001 is a
pending binding retarget that the matrix rows should state.
