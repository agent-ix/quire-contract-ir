---
id: SR-2026110414560068
title: "IR-1348 PR 915 FR-038 base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 915; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1348. Reviewed the new FR-038 bounds exception and FR-038-AC-471 against QSpec FR-142-AC-6, FR-322's literal-node wire rule, the existing IR bounds policy and the emitted U12 square graph. No applicable AssuranceProfile was found.

## Verdict

FAIL pending FND-001. The positive U12 case and retention controls are testable, but one negative needs an explicit post-admission lowering seam or a different reader-admitted graph shape.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The nonliteral `value/literal` negative is not a reader-admitted package under QSpec FR-322, so the required `requires_bound` outcome has no stated executable route. | FR-038-AC-471; QSpec FR-322 |

## Evidence

FR-038-AC-471 starts with a reader-admitted QSL-shaped package and then says replacing its exponent node's body with a nonliteral reference while keeping form `literal` returns `requires_bound`. QSpec FR-322 "Node dependencies" states `value`/`literal` has empty dependencies and its body holds literals only. A reference body either violates that closed form or requires a target dependency, so strict admission can refuse before lowering. Name a private post-admission lowering seam for this form/body control, or use a reader-admitted nonliteral value form; keep the public positive and Integer-parameter controls.

## Coverage

- FR-038-AC-471, role: examined. Positive U12 square, retained keys/value/result type and three negative controls.
- FR-038 bounds-policy paragraph near line 4766, role: examined. Canonical literal own typing edge and nonliteral-body distinction.
- FR-038-AC-73, role: context_only. Bare unit and compound-unit positions retain `requires_bound`.
- QSpec FR-142-AC-6, role: context_only. Integer power yields the canonical compound identity.
- QSpec FR-322 "Node dependencies", role: context_only. A `value`/`literal` body holds literals only and has no dependencies.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: AC-471 uses a reader-admitted Integer parameter for the bound refusal and states that strict admission rejects a nonliteral `value`/`literal` body before lowering. |
