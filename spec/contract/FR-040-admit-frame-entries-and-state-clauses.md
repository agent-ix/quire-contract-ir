---
id: FR-040
title: "Admit V2 frame modifies entries, operation anchors and state clause bodies"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-340
    type: references
  - target: ix://agent-ix/quire-specification/FR-341
    type: references
  - target: ix://agent-ix/quire-specification/FR-342
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---
# FR-040: Admit V2 frame modifies entries, operation anchors and state clause bodies

## Description

When the `quire.checked-package/v2` reader ([FR-038](./FR-038-consume-checked-package-v2.md))
reads a `state`/`frame`, `state`/`operation_anchor`, `state`/`state_clause`
or `value`/`parameter` node, the reader shall admit that node's body exactly
as QSpec FR-340, FR-342 and FR-341 define it, and shall refuse every other
body of those nodes with the refusal those requirements fix, in their reader
order. QSpec FR-341 here is the checked-package state clause body,
`spec/objects/interfaces/FR-341-checked-package-state-clause-body.md`;
FR-340 is `spec/objects/interfaces/FR-340-checked-package-frame-location.md`
and FR-342 is
`spec/objects/interfaces/FR-342-checked-package-operation-anchor-body.md`.
The QSpec text and the published
`proposals/checked-package-v2/schema.json` (`FrameModifiesEntry`,
`FrameBody`, `OperationAnchorBody`, `StateClauseBody`, `ParameterBody`,
`BodyBindingRules`) are normative; this requirement states what the Contract
IR reader does with them and adds no shape of its own.

## Inputs

Untrusted V2 package bytes and the evidence FR-038 already takes, including
each selected domain package document the model-owned member resolution reads.

## Outputs

An admitted package whose frame nodes expose their declaring object type and
their `modifies`, `creates` and `deletes` sets, whose operation anchors expose
their context, operation and frame, and whose state clauses expose their
clause kind, parameters, anchor and condition; or one typed refusal with its
cause and RFC 6901 locus. No partial package.

## Behavior

### Model forms

The reader shall admit exactly the fifteen `model` semantic forms QSpec's
`ModelNode` enumerates: `model_import`, `object_type`, `value_type`,
`variant_type`, `record_value_type`, `event_type`, `state_machine`,
`process`, `persistence_interface`, `namespace`, `systems_interface`,
`systems_part`, `systems_port`, `systems_connection` and
`systems_allocation`. A field, an operation and a clause member are named by
their declaring node and their name, never by a `model` node of their own, so
any other `model` form refuses as `invalid_semantic_graph`.

### Frame body

A `state`/`frame` node's `body` is `{"term": "frame", "modifies", "creates",
"deletes"}`, all three members required. `creates` and `deletes` are arrays
of node keys ([FR-038](./FR-038-consume-checked-package-v2.md)). `modifies` is
an array of `FrameModifiesEntry`, each exactly one of:

- `{"kind": "relationship", "declaration"}`, where `declaration` is the node
  key of a `relation`/`relationship` node;
- `{"kind": "field", "declaration", "name"}`, where `declaration` is the node
  key of the `model`/`object_type` or `model`/`record_value_type` node that
  declares or inherits the field and `name` is the field's identifier.

An entry of any other shape refuses as `invalid_semantic_graph` at that
entry. Every occurrence of a frame node has role `generated`.

Where QSpec fixes a case at the schema level (a body, entry or binding of
the wrong shape, or an occurrence of the wrong role for its node form), the
reader refuses it as `invalid_semantic_graph` with no cause, located at the
offending entry, at the node's `body` for a body or binding shape, and at the
occurrence's `role` for a role.

The frame's `semantic_type` shall name a `model`/`object_type` node; one
naming no node refuses as `missing_declaration`/`missing-name` and one naming
another node as `invalid_model_binding`/`malformed-declaration`, located at
the `semantic_type`. An entry whose `declaration` is not among the frame
node's `dependencies`, or names no node of the package, refuses as
`missing_declaration`/`missing-name`; an entry naming a dependency its kind
cannot carry refuses as `invalid_model_binding`/`malformed-declaration`. The
locus of an entry's refusal is its declaring node key. None of the five
systems forms is an admitted declaring node.

A field entry's `name` resolves among the fields of its declaring node by
FR-038's model-owned member resolution: for a `model`/`object_type` node, the
exposed effective field members of that object type, own and inherited; for a
`model`/`record_value_type` node, the fields its record declaration declares.
Operation members are not candidates. An unrecovered owner refuses with that
resolution's own refusal (`missing_declaration`/`missing-selection` or
`invalid_package`/`stale-node-key`), no field of that name as
`missing_declaration`/`missing-name`, and two or more as
`ambiguous_declaration`/`ambiguous-name`.

A `modifies` entry's order key is its `declaration` digest, compared as
unsigned bytes, then its `name` as UTF-8 bytes, a relationship entry's name
being empty. `modifies` shall be strictly ascending by that key; a misordered
array refuses as `invalid_semantic_graph`. One frame node reports exactly one
defect: any meaning-join defect outranks a canonical-order defect; ties break
by position, `semantic_type`, then `modifies`, then `creates`, then
`deletes`; remaining ties within one member break by ascending order key.
Frames are checked in ascending `node_id` digest order and the first
defective frame is reported.

### Operation anchor body

A `state`/`operation_anchor` node's `body` is an `aggregate` of exactly three
bindings in order: `context`, a `reference`; `operation`, a `text` literal
whose value is an identifier; and `frame`, a `reference`. Every occurrence has
role `anchor`. For one anchor the reader checks, in order, and reports the
first failure: the `context` target is a declared `model`/`object_type`
dependency; the `frame` target is a declared `state`/`frame` dependency (each
`missing_declaration`/`missing-name` when not a declared dependency,
`invalid_model_binding`/`malformed-declaration` when of another meaning,
located at the target); the anchor's and the frame's `semantic_type` both
equal the `context` target (else `invalid_model_binding`/
`malformed-declaration` at the anchor); and the `operation` name resolves
among the context's exposed operation members, own and inherited, with redefinitions applied, to
one the context itself declares (an unrecovered owner with the resolution's own refusal, no such
operation as `missing_declaration`/`missing-name`, two or more as
`ambiguous_declaration`/`ambiguous-name`, an operation the context only
inherits as `invalid_model_binding`/`malformed-declaration`). Once every
anchor's own joins hold, a second anchor naming the same (`context`,
`operation`) pair, or the same `frame` target as another anchor, refuses as
`ambiguous_declaration`/`ambiguous-name` at the second anchor in ascending
node-id digest order.

### State clause body

A `state`/`state_clause` node's `body` is one `application` with operator
class `state_clause` and `operation.identity` `quire.op.state.clause`, whose
`operation.member` is `{"kind": "state_clause", "clause"}` with `clause` one
of `invariant`, `precondition` or `postcondition`, `laws` empty, `mode`
`null` and `leaves` empty, and whose three arguments are, in order, an
`aggregate` of one or more `reference` terms (the parameters), a `reference`
(the anchor) and the Boolean condition. Every occurrence has role `claim`,
and the node's `semantic_type` is the `scalar_type`/`boolean` node. A
`quire.op.state.clause` application nested in another term, or standing as
the body root of any node that is not `state`/`state_clause`, refuses as
`ill_typed`/`operator-ineligible` at the node that holds it.

For one clause node the reader checks, in order, and reports the first
failure:

1. **Anchor.** The anchor target is a declared dependency, else
   `missing_declaration`/`missing-name`; for an invariant it is a
   `model`/`object_type` node and for a precondition or postcondition a
   `state`/`operation_anchor` node, else `invalid_model_binding`/
   `malformed-declaration`, located at the anchor's target.
2. **Parameters.** Each parameter reference, in aggregate order, targets a
   declared dependency, else `missing_declaration`/`missing-name`, that is a
   `value`/`parameter` node, else `invalid_model_binding`/
   `malformed-declaration`, located at the offending target.
3. **Signature.** The parameters are, in order, `self` of type
   `Reference<C>`; for a postcondition, the resolved operation's result when
   it has one; and for a precondition or postcondition, the resolved
   operation's parameters in declared order. `C` is the anchor object type
   for an invariant and the anchor's `context` otherwise, and an invariant
   binds `self` alone. A different count or type refuses as
   `ill_typed`/`operator-ineligible` at the clause node, checking `self`,
   then `result`, then the operation's parameters.

A `value`/`parameter` node's `body` is an `aggregate` of exactly two
bindings in order: `name`, a `text` literal, and `level`, an `integer`
literal. Every occurrence has role `expression`.

The operation catalog shall hold `quire.op.state.clause` with operator class
`state_clause`, operands `aggregate`, `object` and `boolean`, result `clause`
(declared Boolean), no laws, no mode and the `state_clause` member kind. A
non-Boolean condition and a `result_type` naming a non-Boolean node each
refuse as `ill_typed`/`operator-ineligible` at the operation step.

### Reader order

The frame step runs after every graph-shape, stale-key and declaration
refusal. The state step runs after the frame step and before the operation
step; within it the reader checks, each in ascending node-id digest order,
the placement of every `quire.op.state.clause` application, then every
operation anchor, then every state clause, and reports the first defect.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-040-AC-1 | A `modifies` array holding a `relationship` entry and a `field` entry, each naming an eligible declared dependency, admits; an all-empty frame body admits; a bare node key, an entry of another `kind`, a `field` entry without `name`, a `relationship` entry carrying `name`, and an entry with an extra member each refuse as `invalid_semantic_graph` with no cause, located at that entry. | Test (TC-056) |
| FR-040-AC-2 | A relationship entry naming an object type, a field entry declared on a process, a relationship or a systems node, and a `creates` entry naming a relationship each refuse as `invalid_model_binding`/`malformed-declaration` at the entry's declaring node key; an entry naming no declared dependency of the frame, or no node of the package, refuses as `missing_declaration`/`missing-name`. | Test (TC-056) |
| FR-040-AC-3 | A field entry resolves among the exposed effective fields of its declaring object type, own and inherited, and among the declared fields of a record value type; a field entry on an object type for a field typed with a record value type and a field entry on that record value type's own node are distinct entries and both admit in one frame; an undeclared name and an operation's name refuse as `missing_declaration`/`missing-name`; a name two supertypes both expose refuses as `ambiguous_declaration`/`ambiguous-name`; a declaring node keyed under an unselected domain package version refuses as `missing_declaration`/`missing-selection`. | Test (TC-056) |
| FR-040-AC-4 | Two field entries of one declaring node in descending name order refuse as `invalid_semantic_graph`; a frame carrying a meaning-join defect and a canonical-order defect reports the meaning-join refusal regardless of member or array position; with two defective frames, the lower node-id frame's own defect is reported. | Test (TC-056) |
| FR-040-AC-5 | A frame whose `semantic_type` names a node that is not `model`/`object_type` refuses as `invalid_model_binding`/`malformed-declaration` at its `semantic_type` before any entry defect; a frame occurrence with a role other than `generated` refuses as `invalid_semantic_graph` with no cause, located at that occurrence's `role`. | Test (TC-056) |
| FR-040-AC-6 | Each of the fifteen `model` forms admits as a node form; `field_declaration`, `operation_declaration`, `clause_member_declaration` and any other `model` form refuse as `invalid_semantic_graph` with no cause, located at the node's `semantic_form`. | Test (TC-056) |
| FR-040-AC-7 | An operation anchor with the three bindings in order admits. Reordered bindings, a non-text `operation`, a missing binding, and an extra binding each refuse as `invalid_semantic_graph` with no cause, located at the anchor's `body`; an occurrence role other than `anchor` refuses the same way, located at that occurrence's `role`. A `context` naming a frame, a `frame` naming an object type, and a `frame` target missing from `dependencies` refuse as `invalid_model_binding`/`malformed-declaration`, `invalid_model_binding`/`malformed-declaration` and `missing_declaration`/`missing-name` at the target; an anchor whose `semantic_type` is not its `context` refuses as `invalid_model_binding`/`malformed-declaration` at the anchor. Operation resolution: `(Order, "scaled")` resolves to `Order`'s operation and `(Sub, "size")` to `Sub`'s redefinition; `(Sub, "reset")`, which `Sub` only inherits, and an operation on a `systems_interface` node refuse as `invalid_model_binding`/`malformed-declaration`; an undeclared name and a field's name refuse as `missing_declaration`/`missing-name`; a context keyed under an unselected domain package version refuses as `missing_declaration`/`missing-selection`. A duplicate (`context`, `operation`) pair and a shared frame refuse as `ambiguous_declaration`/`ambiguous-name` at the second anchor in ascending node-id digest order. | Test (TC-056) |
| FR-040-AC-8 | An invariant, a precondition and a postcondition in QSpec FR-341's shape each admit; an invariant anchored at an operation anchor and a precondition anchored at an object type refuse as `invalid_model_binding`/`malformed-declaration` at the anchor's target; a parameter reference to a node that is not `value`/`parameter` refuses the same way at its target. Another operation identity, operator class, member kind or clause value, a first argument that is not an `aggregate` of `reference` terms, and a second argument that is not a `reference` each refuse as `invalid_semantic_graph` with no cause, located at the clause node's `body`; an occurrence role other than `claim` refuses the same way, located at that occurrence's `role`. | Test (TC-056) |
| FR-040-AC-9 | For `Order.scaled(n: Integer): Integer` the precondition's parameter types `[Reference<Order>, Integer]` and the postcondition's `[Reference<Order>, Integer, Integer]` admit, and for `Order.reset()` with no result `[Reference<Order>]` admits; a missing parameter, a missing result, a `self` of another object type, a parameter of another type and an invariant binding a second parameter each refuse as `ill_typed`/`operator-ineligible` at the clause node. | Test (TC-056) |
| FR-040-AC-10 | A `quire.op.state.clause` application nested in another term, and one standing as the body root of a node that is not `state`/`state_clause`, refuse as `ill_typed`/`operator-ineligible` at the holding node; a non-Boolean condition refuses as `ill_typed`/`operator-ineligible` at the operation step. | Test (TC-056) |
| FR-040-AC-11 | A `value`/`parameter` body of `name` then `level` admits; reordered bindings, a non-text `name`, a non-integer `level`, and a missing binding each refuse as `invalid_semantic_graph` with no cause, located at the node's `body`; an occurrence role other than `expression` refuses the same way, located at that occurrence's `role`. | Test (TC-056) |
| FR-040-AC-12 | A package carrying a frame defect and a state defect reports the frame defect; one carrying an anchor defect and a clause defect reports the anchor defect; one carrying a state defect and an operation defect reports the state defect. | Test (TC-056) |
| FR-040-AC-13 | Read at run time from the checkout `QSPEC_DIR` names and never copied into this repository: every entry of the `frame_mutations` array of QSpec's `proposals/checked-package-v2/node-identity-vectors.json`, substituted in place into the frame node of QSpec's published `fixtures/positive-all-families.json` as that node's `dependencies`, `modifies`, `creates` and `deletes` (each bare digest as a `NodeRef`, and an entry's `second_frame` inserted verbatim as a second frame node), refuses with its `expected_code`, `expected_cause` and `expected_locus_digest`, and the count replayed equals the count published; and QSpec's published V2 fixtures that carry frame, operation anchor, state clause and parameter nodes admit. Both run under `make qspec-vectors`. | Test (TC-056) |

## Dependencies

[FR-038](./FR-038-consume-checked-package-v2.md) owns the reader, its
refusal codes and pointers, the `creates`/`deletes` eligibility and the
model-owned member resolution this requirement reuses. QSpec FR-340, FR-341
and FR-342 own the normative shapes and refusal order, and FR-322 the
operation catalog and reader order they extend. Lowering an admitted frame's
grants to `kani::modifies` places is a later lowering requirement, not this
one.
