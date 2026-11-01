---
id: FR-355
title: "Read the complete effective model operation census"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: references
---
# FR-355: Read the complete effective model operation census

## Description

When a consumer requests the effective operations of an admitted model context,
Contract IR SHALL return the complete selected declaration census or a typed
refusal under the caller's work provision. The census SHALL include operations
without an executable body, source clause, operation anchor or captured call.
It SHALL read the retained selected model declarations and use FR-322's effective
member and most-derived-redefiner rules, without inferring operations from bodies.

## Inputs

An admitted `CheckedPackageV2`, one admitted operation-bearing model context node,
and a finite caller work provision. Eligible contexts are object types, processes,
state machines, persistence interfaces and systems interfaces. A declaration
identity alone is not evidence that its node is admitted in this package.

## Outputs

A read-only census in ascending effective operation name order. Each operation
borrows its admitted declaration identity and parameter names, retains its actual
admitted declaring node and the selected context, and exposes parameter domains in
producer order and a separately optional result domain using `CheckedMemberType`.
An unrepresentable domain remains explicitly unavailable; it SHALL NOT delete the
operation or become an unbounded integer. Result absence and a present result with
an unavailable domain SHALL remain distinguishable. The census grants no executable
body, dispatch, successor, boundedness or proof credit.

## Behavior

The read SHALL first distinguish a foreign package context, an unknown node and
a node of the wrong model form. It SHALL charge context lookup, selected declaration
visits, ancestry/member/redefinition resolution and each projected binder before
exposing the census. Exhaustion SHALL report the configured work limit and no
partial table. An authoritative empty census SHALL succeed under a sufficient
provision and remain distinct from a refusal.

Every effective operation name SHALL be resolved. An ambiguous name SHALL refuse
the whole census rather than disappear from an otherwise successful list. Inherited
and redefining operations SHALL retain the original selected member identity and
its actual declaring node; missing or ambiguous declaring-node joins SHALL refuse
rather than manufacture a structural identity or select by display name. No
selected-stage Exact identity is inferred from a retained model member identity.

The public read SHALL borrow retained declarations rather than create a second
operation model, parse JSON, reconstruct QSL scope or recheck source. Existing
FR-354 callable/body/frame and per-call dispatch reads remain separate: consumers
must use them independently when they require execution or dispatch evidence.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-355-AC-1 | A package admitted through the public reader with two declared operations and no operation anchors or bodies returns both identities and an authoritative empty census for a different empty context. Adding or removing an unreferenced declaration changes the census independently of bodies. | Test |
| FR-355-AC-2 | Parameters retain producer name/order and exact Boolean, bounded-integer and collection member domains; a separately present result retains its domain. Absent results and present unavailable domains remain distinguishable, without omitting operations. | Test |
| FR-355-AC-3 | Inheritance, a diamond and a most-derived redefinition retain each effective operation once with its original member and actual declaring node. An ambiguous effective name, absent declaring node or duplicate declaring-node join returns its distinct typed refusal and no partial census. | Test |
| FR-355-AC-4 | Foreign, unknown and wrong-kind contexts have distinct typed refusal codes. A sufficient work provision returns the full table and consumed work; one below the required provision refuses with that configured limit, and a zero provision refuses before census exposure. | Test |
| FR-355-AC-5 | A downstream public consumer can read the census and existing callable/dispatch APIs separately through the model crate facade. The public inventory names the new read types, which borrow retained declaration facts and expose canonical member-domain types without a copied parser, mirror operation model or added dependency. | Test |

## Dependencies

FR-038 owns strict public admission and selected model evidence; FR-019 owns the
model crate facade and inventory; FR-354 owns separate executable callable, frame
and dispatch joins. CG FR-048 consumes the census as an input to admitting a complete
finite transition domain and remains responsible for method generation and results.
