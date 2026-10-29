---
id: SR-567886
title: "IR-874 PR 572 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs; PR #572 "
review_set: subset
---

## Summary

The computed Quoin matrix tags FR-038-AC-282/284/288, but the changed tests do not execute valid attempt/effect or compensation-bearing event paths.

## Verdict

**CONDITIONAL — Two focused public-reader test gaps need closure on this PR.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new attempt and effect tests only assert refusals before a valid attempt can exist; no public-reader fixture admits an attempt with an operation anchor or an effect of that attempt, and no test reaches contract kind/anchor checks. | tests/it/checked_package_v2_protocol_body.rs:1649 |
| FND-002 | medium | The event compensation test names a role and proves only wrong-form refusal; no public-reader fixture admits a same-clause compensation target or rejects a foreign-clause compensation target. | tests/it/checked_package_v2_protocol_body.rs:1687 |

## Scope

- **FR-038-AC-282 (examined)**: Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- **FR-038-AC-284 (examined)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-288 (examined)**: Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks: `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit` and `fence`. A choice or parallel branch list with fewer than two entries, repeated labels, a receive naming a non-send, an effect naming a non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint, an invalid join count or an ineligible memory or ordering spelling refuses with the FR-466 code, cause and locus, with
- **FR-466-AC-1 (examined)**: Each of the thirteen control operators is admitted with exactly its layout; the arguments reordered, a missing or surplus binding, an unknown operator spelling and an enumerated value outside its set (`memory`, `outstanding`, `policy`, `ordering`, `kind`) refuse `malformed_wire` or refuse `operation-member-mismatch`.
- **FR-466-AC-8 (examined)**: An `attempt` is admitted with a role, an operation anchor, each memory ordering or none, and `contracts` that are the pre- and postconditions of that anchor; a `contracts` target anchored at another operation, an `invariant` clause and a non-clause node refuse `invalid_model_binding`/`malformed-declaration`; an empty `contracts` list is admitted. An `effect` is admitted of an `attempt` and refuses `invalid_model_binding`/`malformed-declaration` of any other control.
- **FR-466-AC-9 (examined)**: An `event` is admitted with and without a compensation; a `commit` and a `fence` are admitted; `related` entries name a relationship the clause lists with end terms of the endpoint types, and an unlisted relationship refuses `missing_declaration`/`missing-name` and an ill-typed end `ill_typed`/`type-mismatch`.

## Coverage

Plan completion: not assessed. Quoin matrix reports FR-038-AC-282/284/288 tagged (2/4/28 binders) with no run evidence in its store. Reverse-gap and stub inspection found no ownerless changed code or stub. The optional repository-wide semantic review was not requested; changed behavior was compared manually with FR-464/466 and IR FR-038. Full protocol criteria remain Planned.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The public reader now admits an operation-anchored attempt and an effect of that attempt, then checks wrong contract kind and anchor. |
| FND-002 | fixed | The public reader now admits an event with a listed same-clause compensation and refuses an otherwise shaped foreign target at the exact pointer. |

## Disposition verdict

**PASS ** — All original findings were fixed on this PR; the complete FR-038-AC-282/284/288 criteria remain Planned.
