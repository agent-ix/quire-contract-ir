---
id: SR-567881
title: "IR-874 PR 570 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; protocol_body.rs, flat_wire.rs, contract_package.rs, shared.rs, public-reader and model tests; PR #570 head dd393e25828c52a13230370da4cafeaad9b41b0b"
review_set: subset
---

## Summary

Reviewed repeat and await local member and reference checks at frozen PR #570 head dd393e25828c52a13230370da4cafeaad9b41b0b. FR-038-AC-282/284/288, including causal, progress, binder and decision stages, remain Planned.

## Verdict

**CONDITIONAL** — Accepted repeat and await variants need direct public-reader evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Public-reader tests admit repeat only with maximum and exhausted, and with neither; proof-present, maximum zero, and each unpaired direction lack evidence. | tests/it/checked_package_v2_protocol_body.rs:801 |
| FND-002 | medium | Public-reader tests exercise an event anchor and event control only; a compensation anchor and receive/effect events lack admission evidence. | tests/it/checked_package_v2_protocol_body.rs:848 |

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

Plan completion: not assessed. Quoin matrix reports FR-038-AC-282/284/288 tagged (2/4/11 bindings), with no run evidence. Reverse-gap and stub inspection found the two evidence gaps above. Optional repository-wide semantic review was not requested; changed behavior was manually compared with FR-464/466 and IR FR-038. The remaining protocol stages are Planned, not discharged by the tags.
