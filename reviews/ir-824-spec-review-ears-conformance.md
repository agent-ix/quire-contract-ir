---
id: SR-11833
title: "EARS conformance review of IR-824 FR-038-AC-252 (PR #420)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (reaches paragraph SHALL statement, FR-038-AC-252)"
review_set: subset
---

## Summary

Ticket: IR-824. The new SHALL ("The reader SHALL therefore admit
`quire.op.model.reaches` over any such end with the same node shape") is a
single, conditioned, testable obligation. AC-252 uses direct assertions with
exact codes, causes and pointers and is testable, but it bundles several
independent outcomes.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | low      | FR-038-AC-252 is compound: admission over three multiplicities, the subtype operand, the unbounded/ordered refusal, the different-type refusal, the operand-1 refusal, three ordering cases and the result-type refusal sit in one criterion, so one tagged test can mark all of them backed. This mirrors the single QSpec FR-322-AC-57 and the FR-038 sibling style (AC-172, AC-243), hence low; splitting admission, refusals and ordering into separate ACs would let the matrix show which part is backed. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4424 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 1840592 split AC-252 into AC-252 (admission and supersession), AC-253 (operand 1 and check order), AC-254 (direction, RN-27), AC-255 (unbounded/ordered, RN-23/24/26), AC-256 (other type, RN-25) and AC-257 (result node), each with its own matrix row. |
