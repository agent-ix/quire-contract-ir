---
id: SR-612100
title: IR-1077 frame seam base review
type: SpecReview
analysis: base
scope: 'agent-ix/quire-contract-ir; PR #650 aeb07e6c; FR-040 and TC-056'
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-040
  type: reviews
- target: ix://agent-ix/quire-contract-ir/TC-056
  type: reviews
---

## Summary

Ticket: IR-1077. Reviewed PR #650 aeb07e6c against QSpec 18211795, FR-340-AC-12/19/20, FR-342-AC-2/7/8/9, FR-341-AC-11/12 and IR FR-061-AC-14/15/16.

## Verdict

FAIL: findings below.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The unchanged precedence prose excludes every deletes_scoped meaning-join defect, but new AC-38 requires receiver-scoped delete on a process/state-machine frame to refuse as a frame meaning-join defect before canonical order; QSpec FR-340-AC-20 confirms it. State where this defect ranks, including the deletes_scoped tie position. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:200 |
| FND-002 | high | AC-30 says a context outside the three allowed forms always refuses invalid_model_binding/malformed-declaration. A missing or undeclared context instead requires missing_declaration/missing-name under QSpec FR-342-AC-2 and the FR-040 anchor prose. Restrict the malformed-declaration oracle to a declared wrong-form target and retain the missing-target branch. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:424 |

## Coverage

- FR-040-AC-2, role: examined. Excerpt: “A relationship entry naming an object type, a field entry declared on a relationship or systems node, a `creates` entry naming a relationship or state machine, and a `deletes` entry naming a state machine each refuse as `invalid_model_binding`/`malformed-declaration` at the entry's declaring node key; a field entry on a process or state machine and a `creates` or `deletes` entry naming a process admit when their other joins hold. An entry naming no declared dependency of the frame, or no node of the package, refuses as `missing_declaration`/`missing-name`.”
- FR-040-AC-5, role: examined. Excerpt: “A frame whose `semantic_type` names a `model`/`object_type`, `model`/`process` or `model`/`state_machine` declaring its operation admits that type join; a frame whose `semantic_type` names any other node refuses as `invalid_model_binding`/`malformed-declaration` at its `semantic_type` before any entry defect. A frame occurrence with a role other than `generated` refuses as `invalid_semantic_graph` with no cause, located at that occurrence's `role`.”
- FR-040-AC-7, role: examined. Excerpt: “An operation anchor with the three bindings in order admits. Reordered bindings, a non-text `operation`, a missing binding, and an extra binding each refuse as `invalid_semantic_graph` with no cause, located at the anchor's `body`; an occurrence role other than `anchor` refuses the same way, located at that occurrence's `role`. A `context` naming a frame, a `frame` naming an object type, and a `frame` target missing from `dependencies` refuse as `invalid_model_binding`/`malformed-declaration`, `invalid_model_binding`/`malformed-declaration` and `missing_declaration`/`missing-name` at the targe”
- FR-040-AC-8, role: examined. Excerpt: “An invariant, a precondition and a postcondition in QSpec FR-341's shape each admit; an invariant anchored at an operation anchor and a precondition anchored at an object type, process or state machine refuse as `invalid_model_binding`/`malformed-declaration` at the anchor's target; a parameter reference to a node that is not `value`/`parameter` refuses the same way at its target. Another operation identity, operator class, member kind or clause value, a first argument that is not an `aggregate` of `reference` terms, and a second argument that is not a `reference` each refuse as `invalid_seman”
- FR-040-AC-9, role: examined. Excerpt: “For `Order.scaled(n: Integer): Integer`, a precondition binding `[Reference<Order>, Integer]` and a postcondition binding `[Reference<Order>, Integer, Integer]` admit; a precondition missing the operation parameter, a postcondition missing the result, a parameter of another type, a `self` that is a `Reference` to another declaring type than its anchor's, and an invariant binding a second parameter each refuse as `ill_typed`/`operator-ineligible` at the clause node's first argument.”
- FR-040-AC-17, role: examined. Excerpt: “An object-type frame body carrying `modifies` field entries of scope `receiver` and `every-object` and a `deletes_scoped` of one `receiver` entry and `parameter` entries admits with each entry's scope and kind exposed; a frame of four empty members admits.”
- FR-040-AC-30, role: examined. Excerpt: “A context target naming no node or absent from the anchor's declared dependencies refuses `missing_declaration`/`missing-name` at that target. A declared context target that is not a `model`/`object_type`, `model`/`process` or `model`/`state_machine` refuses `invalid_model_binding`/`malformed-declaration` at the context target before operation resolution or scoped deletes. A valid process or state-machine anchor with a frame `parameter` entry naming no reference parameter of its resolved own operation refuses `missing_declaration`/`missing-name` at the frame node in the state step. This correc”
- FR-040-AC-38, role: examined. Excerpt: “For selected process `Flow` and state machine `Gate`, a frame whose `semantic_type` is that declaring node admits `every-object` field entries naming own and inherited fields, and a `parameter` scoped-delete entry checked against its anchor operation. The process frame also admits `creates` and `deletes` entries naming a process; the state-machine entry of either kind refuses `invalid_model_binding`/`malformed-declaration` at the entry key. A missing process or state-machine field refuses `missing_declaration`/`missing-name` at the field entry. A field entry naming either form also admits in a”
- FR-040-AC-39, role: examined. Excerpt: “A process anchor `(Flow, "go")` and state-machine anchor `(Gate, "toggle")` each resolve an operation declared by that exact context, with anchor and frame `semantic_type` equal to it. An operation inherited by subtype process `SubFlow` is anchored at `Flow`; `(SubFlow, "go")` when only inherited refuses `invalid_model_binding`/`malformed-declaration` at the operation binding. Two own operations of one process or state machine with equal frames share a frame node, while anchors of two different declaring types cannot. A context that is a record value type, event type, persistence interface or ”
- FR-040, role: examined. Excerpt: “The frame step admits object-type, process, and state-machine declaring nodes with QSpec FR-340 meaning joins and precedence.”
- TC-056, role: examined. Excerpt: “Planned process and state-machine seam using independent QSpec FR-340/FR-342 oracle.”

## Assurance Context

No applicable AssuranceProfile identified for this spec-only seam.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in PR #650 head cb8e5f9b: corrected precedence and/or missing-context oracle matches QSpec. |
| FND-002 | fixed | Verified in PR #650 head cb8e5f9b: corrected precedence and/or missing-context oracle matches QSpec. |
