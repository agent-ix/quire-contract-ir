---
id: SR-3142
title: "Criterion strength review of the IR-690 structural inequality criteria"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir branch spec/ir690-structural-ne-domains (IR-690, specification only); FR-038-AC-197 through FR-038-AC-201; TC-048 structural inequality extension"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

## Summary

Ticket: IR-690. For each new criterion, asks whether a wrong implementation
could pass it and whether its TC-048 step supplies an independent oracle.
Judged manually over the specification text. Jev was not used.

- AC-197: decidable. It fails on a wrong application or occurrence, reordered
  or deduplicated entries. TC-048 step 1 requires expectations taken from the
  authored source, not from the accessor.
- AC-198: the criterion text is differential (ne equals eq), so an
  implementation equally wrong for both would pass it alone. TC-048 step 2
  closes this by requiring an independent enumeration from the authored input,
  and eq correctness is already backed by AC-179/AC-180. Adequate.
- AC-199: decidable, with exact variant, payload and first-defect order.
  Defensive mutations are labelled as not being admission evidence.
- AC-200: decidable. It names exact budget, one below and zero, and gives an
  independent visit count ("do not derive the expected visit count by invoking
  the implementation twice").
- AC-201: an Inspection with the owner seams named in TC-048 step 5. All of
  them exist in the model crate, so two inspectors would reach the same
  verdict.

## Examined units

- FR-038-AC-197 (examined)
- FR-038-AC-198 (examined)
- FR-038-AC-199 (examined)
- FR-038-AC-200 (examined)
- FR-038-AC-201 (examined)
- TC-048 structural inequality extension steps 1 to 5 and expected results (examined)
- FR-038-AC-177 (context_only)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-048 step 1 tells the tester to "request an absent occurrence" for structural.ne, but none of AC-197 through AC-201 states the expected outcome. AC-197 covers only success, and AC-199's refusal list names operand/domain refusals, not MissingOccurrence. The only stated oracle is AC-177's, which is about structural.eq. An implementation that returns a different variant or payload for a ne absent occurrence passes every new criterion. Add "an absent occurrence refuses MissingOccurrence with the supplied application and occurrence" to AC-197 (or AC-199). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3884; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3886; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:623-628 |

## Verdict

CONDITIONAL. Four of the five criteria can fail and have independent oracles.
FND-001 is a missing expected outcome for a case the procedure already
exercises, and needs one clause.
