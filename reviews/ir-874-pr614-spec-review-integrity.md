---
id: SR-611004
title: "IR-874 PR 614 STD-515 spec integrity analysis"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; FR-038-AC-292; TC-048; QSpec FR-466-AC-14/16 and TC-948 DF7g/DF7h"
review_set: subset
---

## Summary

The new FR-038-AC-292 clause gives an observable typed refusal and ordered stage placement for bare record reads; TC-048 supplies public-reader cases with a Boolean root to avoid an earlier non-Boolean-root refusal. The IR statement agrees with QSpec 9bcbc625 and does not narrow the existing field-projection, scope or model-state cases.

## Examined units

- **FR-038-AC-292** (`examined`): “A bare read of any record binder, including the protocol activation binder, with no field projection refuses `ill_typed`/`operator-ineligible` at that bare read with the slot detail after scope and model-state checks and before decision-fact checks.”
- **TC-048** (`examined`): four choice/repeat visible/guard slots for bare `opened` and `got`, both operand orders against `sent` field, exact path/locus/slot.
- **QSpec FR-466-AC-14/16 and TC-948 DF7g/DF7h** (`context_only`): independent stage, detail and adverse-case oracle.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Verdict

**PASS** for consistency, testability and scope of the changed text. The criterion explicitly covers every record binder, while TC-048 names representative activation and receive binders. The code/test gap for captures is a delivery finding, not a reason to narrow this AC.
