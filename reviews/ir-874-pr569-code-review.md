---
id: SR-567878
title: "IR-874 PR 569 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs; PR #569 head 786722d469368b1cebf99490e9bd67a3f0b157a7"
review_set: subset
---

## Summary

Reviewed the first ordered FR-466 control prefix at frozen PR #569 head 786722d469368b1cebf99490e9bd67a3f0b157a7. The rest of FR-038-AC-282/284/288 remains Planned.

## Verdict

**CONDITIONAL** — The tightened choice layout makes two existing model tests fail because their fixture has no cases.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Two existing model tests now fail: their choice fixture has zero cases, while the validator correctly requires at least two. Update the QSpec frame fixture to valid sorted cases. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:2030 |

## Scope

- FR-038-AC-282 (examined): Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- FR-038-AC-284 (examined): The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- FR-038-AC-288 (examined): Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks: `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit` and `fence`. A choice or parallel branch list with fewer than two entries, repeated labels, a receive naming a non-send, an effect naming a non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint, an invalid join count or an ineligible memory or ordering spelling refuses with the FR-466 code, cause and locus, with
- FR-466-AC-1 (examined): Each of the thirteen control operators is admitted with exactly its layout; the arguments reordered, a missing or surplus binding, an unknown operator spelling and an enumerated value outside its set (`memory`, `outstanding`, `policy`, `ordering`, `kind`) refuse `malformed_wire` or refuse `operation-member-mismatch`.
- FR-466-AC-2 (examined): A `sequence` holds its children in execution order and may hold none; reordering two children changes the node key; a child that is not a `control` node refuses `invalid_model_binding`/`malformed-declaration` at the target, and a child absent from `dependencies` `missing_declaration`/`missing-name`.
- FR-466-AC-3 (examined): A `choice` is admitted with an owning role, a `visible` set and at least two labelled cases sorted by label; one case, equal labels, cases out of label order, a guard that reads model state and a guard that reads `over` refuse (`malformed_wire`, `ambiguous_declaration`/`ambiguous-name`, `invalid_semantic_graph`, `ill_typed`/`operator-ineligible` at the read); a non-Boolean guard refuses `ill_typed`/`non-boolean-root`; the guards' exhaustiveness is not re-proved.
- FR-466-AC-4 (examined): A `parallel` is admitted for each join policy; `all` naming every branch, `any` and `predicate` naming at least one and `quorum` with `1 <= count <=` the joined branches; a join naming a missing branch label refuses `missing_declaration`/`missing-name` at the label, `all` omitting a branch and `quorum` outside its range `invalid_package`/`invalid-value`, and a repeated or out-of-order joined label `invalid_semantic_graph`; branches are sorted ascending by label, and a branch list out of that order is `invalid_semantic_graph`.

## Coverage

Rust style and repository conventions reviewed; no CI workflow changed. Focused public-reader tests are running independently. Aggregate gates remain for the final reviewed head.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The QSpec frame fixture now supplies two sorted choice cases; both previously failing model tests pass at cee96d69. |

## Disposition verdict

**PASS at cee96d69a0e5aec656b14368abff1aa71496ede3** — FND-001 is fixed. The unimplemented FR-466 operators and protocol stages remain Planned.
