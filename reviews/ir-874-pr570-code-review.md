---
id: SR-567880
title: "IR-874 PR 570 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; protocol_body.rs, flat_wire.rs, contract_package.rs, shared.rs, public-reader and model tests; PR #570 "
review_set: subset
---

## Summary

Reviewed repeat and await local member and reference checks at frozen PR #570. FR-038-AC-282/284/288, including causal, progress, binder and decision stages, remain Planned.

## Verdict

**FAIL** — An inverted await interval is admitted, and two refusal paths differ from the required diagnostics.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | An await with within.lower greater than within.upper is admitted; validate the interval and return ill_typed/type-mismatch at within. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:1527 |
| FND-002 | medium | A non-Boolean repeat guard is refused as malformed_wire during strict layout checking, rather than ill_typed/non-boolean-root at the guard. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:1519 |
| FND-003 | medium | An ineligible awaited event returns only code, cause and await node pointer; the required offending form and selected protocol profile are absent. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:1548 |

## Scope

- FR-038-AC-282 (examined): Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- FR-038-AC-284 (examined): The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- FR-038-AC-288 (examined): Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks: `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit` and `fence`. A choice or parallel branch list with fewer than two entries, repeated labels, a receive naming a non-send, an effect naming a non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint, an invalid join count or an ineligible memory or ordering spelling refuses with the FR-466 code, cause and locus, with
- FR-038-AC-290 (context_only): For each clause the reader constructs the QSpec FR-464 derived causal relation from control containment, sequencing, receive/send, effect/attempt and await edges, not source order or timestamps. A receive before its send or an await anchored at a later control refuses `invalid_package`/`definition-cycle` naming the cycle; a same pair on independent parallel branches admits. A repeat whose body can complete with no progress on a path refuses `unsupported_construct`/`declaration-form`, while a repeat with a real step admits with or without a maximum and loop proof.
- FR-464-AC-8 (context_only): An expression that reads a binder or capture outside its stage's list refuses `missing_declaration`/`missing-name` at the node whose slot holds it, retaining name, stage, owner and `read` (`forward`, `foreign` or `stage`); a read of `over` in an activation guard, a trigger guard, a retry relation, a recovery predicate, a choice or repeat guard, a `visible` entry, a `check` condition or a FIFO key refuses `ill_typed`/`operator-ineligible` before any stage-table refusal, while a protocol, registration or activation capture, an event constraint and `finish` may read it; a `finish` constraint read
- FR-466-AC-1 (examined): Each of the thirteen control operators is admitted with exactly its layout; the arguments reordered, a missing or surplus binding, an unknown operator spelling and an enumerated value outside its set (`memory`, `outstanding`, `policy`, `ordering`, `kind`) refuse `malformed_wire` or refuse `operation-member-mismatch`.
- FR-466-AC-5 (examined): A `repeat` is admitted with a maximum and an `exhausted` control, with a `proof`, with both, and with neither; `exhausted` without `maximum` and `maximum` without `exhausted` refuse `malformed_wire`, a body that does not necessarily step refuses `unsupported_construct`/`declaration-form` (FR-464), and `maximum: 0` is admitted.
- FR-466-AC-6 (examined): An `await` is admitted after an event node and after a compensation, awaiting a `receive`, an `effect` or an `event`; awaiting a `send` or an `attempt` refuses `unsupported_construct`/`declaration-form`; an anchor that is not a step control, or an `await` anchored at a control that follows it, refuses (`invalid_model_binding`/`malformed-declaration`, `invalid_package`/`definition-cycle`).

## Coverage

Rust style and repository conventions inspected; no CI workflow changes. Focused public-reader protocol tests passed 36/36; focused model layout test passed 1/1. No aggregate gate was run during review. The repeat progress and await causal checks are separately Planned and currently admit some invalid graphs; this PR must not claim full criterion completion.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The await local check now compares BigInt bounds and refuses the inverted interval with typed member, expected and actual details. |
| FND-002 | fixed | Repeat layout admits a literal candidate to local type validation, which now returns ill_typed/non-boolean-root at the guard. |
| FND-003 | fixed | The new typed detail carries the actual event form and selected protocol profile on the unsupported_construct/declaration-form refusal. |

## Disposition verdict

**PASS** — All original findings in this method are fixed. Remaining protocol stages and the full FR-038 acceptance criteria remain Planned.
