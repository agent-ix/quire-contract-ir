---
id: SR-611003
title: "IR-874 PR 614 STD-515 spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md FR-038-AC-292; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; QSpec FR-466-AC-14/16 and TC-948 DF7g/DF7h"
review_set: subset
---

## Summary

Reviewed the FR-038-AC-292 and TC-048 additions in PR #614 at b201f727bf402e1cc4694360ff735886dac5448c against QSpec 9bcbc625. The IR criterion distinguishes a field projection from a bare record-binder read and preserves scope, model-state, bare-read and decision-fact precedence. TC-048 names the four slots, the `opened` and `got` cases, both mixed operand orders and exact refusal location/detail.

## Examined units

- **FR-038-AC-292** (`examined`): “A bare read of any record binder, including the protocol activation binder, with no field projection refuses `ill_typed`/`operator-ineligible` at that bare read with the slot detail after scope and model-state checks and before decision-fact checks; it never fabricates a field or `wrong_snapshot` detail.”
- **TC-048** (`examined`): “In each of those four slots, read the activation binder `opened` bare inside a Boolean-rooted expression; repeat with a bare receive record binder. Require `ill_typed`/`operator-ineligible` at the bare-read node with the slot detail after scope and model-state checks.”
- **QSpec FR-466-AC-14/16** (`context_only`): field facts, bare binder reads and the stated stage precedence.
- **QSpec TC-948 DF7g/DF7h** (`context_only`): published four-slot bare read and both-order mixed guard oracles.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Verdict

**PASS** for the changed specification. The criterion's “any record binder” scope is broader than its representative test procedure, as intended; the implementation and test gap for record-typed captures is in SR-610062/610063. The two changed Markdown documents pass targeted `quire validate`. No applicable AssuranceProfile was found in this repository's spec tree. This is the independent PR spec review, not an enforced workflow acceptance.
