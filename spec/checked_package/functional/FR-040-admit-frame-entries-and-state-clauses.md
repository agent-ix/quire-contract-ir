---
id: FR-040
title: "Admit V2 frame entries, operation anchors and state clause bodies"
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
# FR-040: Admit V2 frame entries, operation anchors and state clause bodies

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
`FrameBody`, `FrameDeletesScopedEntry`, `OperationAnchorBody`, `StateClauseBody`, `ParameterBody`,
`BodyBindingRules`) are normative; this requirement states what the Contract
IR reader does with them and adds no shape of its own.

## Inputs

Untrusted V2 package bytes and the evidence FR-038 already takes, including
each selected domain package document the model-owned member resolution reads.

## Outputs

An admitted package whose frame nodes expose their declaring object type and
their `modifies`, `creates`, `deletes` and `deletes_scoped` sets (each
`modifies` field entry carrying its scope), whose operation anchors expose
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
"deletes", "deletes_scoped"}`, all four members required and each possibly
empty. `creates` and `deletes` are arrays of node keys
([FR-038](./FR-038-consume-checked-package-v2.md)); a `deletes` entry is
type-wide. `modifies` is an array of `FrameModifiesEntry`, each exactly one
of:

- `{"kind": "relationship", "declaration"}`, where `declaration` is the node
  key of a `relation`/`relationship` node;
- `{"kind": "field", "declaration", "name", "scope"}`, where `declaration` is
  the node key of the `model`/`object_type` or `model`/`record_value_type`
  node that declares or inherits the field, `name` is the field's identifier
  and `scope` is exactly `receiver` (the write is granted on the operation's
  receiver only) or `every-object` (on every object whose effective type has
  the field).

`deletes_scoped` is an array of `FrameDeletesScopedEntry`, each exactly one of
`{"kind": "receiver"}` (`deletes self`) and `{"kind": "parameter", "name"}`
(`deletes p`, `name` being a reference parameter's name, an identifier).

An entry of any other shape, a `field` entry with no `scope` or a `scope`
other than `receiver` or `every-object`, a `relationship` entry carrying
`scope`, a `deletes_scoped` entry of any other shape, and a frame body
without `deletes_scoped` or with a member not named here each refuse as
`invalid_semantic_graph` with no cause: at that entry for an entry, at the
frame node's `body` for a missing or further member. A member array that
repeats an entry refuses the same way, located at the second occurrence of
the repeated entry (the first index whose entry equals an earlier one), as
the reader's repeat check locates it. These are schema-level
refusals of the node-body validation that runs before the frame step, so they
outrank every defect the frame step and the state step report. Every
occurrence of a frame node has role `generated`.

The frame's scoped members enter its node identity: a `scope` or a
`deletes_scoped` entry changed without re-keying the frame node is the stale
node key refusal (`invalid_package`/`stale-node-key`) of the stage that
precedes the frame step.

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
FR-038's model-owned member resolution when the declaring node is a selected
model declaration node. For a `model`/`object_type` or
`model`/`record_value_type` declaration node, the reader SHALL index the
selected document's owner under its authentic self-typed model declaration
node key, recover the owner by that key and form, and resolve the name among
the owner's exposed effective field members, own and inherited. Redefinitions
and ambiguity use the same selected-domain effective-member rules as object
fields. A record field's type remains the selected record declaration node
through a nested record, an inherited field, `Option`, a bounded `Sequence`
and finite recursive record structure; the reader SHALL NOT substitute an
object `Reference` node or discard the record owner while deriving a member
type. A source-declared object type carrying a `declaration` retains its
existing joined and ordered but unresolved name behavior. A record field
entry without a recoverable selected record declaration SHALL NOT be admitted
as an unresolved name.

Operation members are not field candidates. An unselected or wrong-kind
record owner refuses `missing_declaration`/`missing-selection`, and a stale
record declaration key refuses `invalid_package`/`stale-node-key`, at
FR-322's derived-shape owner/key stage before the frame step. Once that
stage admits the declaring node, no effective field of the entry's name
refuses `missing_declaration`/`missing-name`, and two or more refuse
`ambiguous_declaration`/`ambiguous-name`, each as a FR-340 frame-entry
refusal at the entry. An unknown name in an expression `field` projection
remains FR-322's separate `ill_typed`/`operator-ineligible` refusal at the
operation's member name. Owner indexing and effective-field lookup SHALL
use the existing selected-model read stage and work meter, with no new
validation pass or changed first-refusal order. The frame step SHALL retain
its existing meaning-join-before-canonical-order and ascending-frame-key
precedence, and non-record entries retain their existing results.

A `modifies` entry's order key is its `declaration` digest, compared as
unsigned bytes, then its `name` as UTF-8 bytes, then its `scope` with
`every-object` before `receiver`; a relationship entry's name is empty and it
has no scope. `modifies` shall be strictly ascending by that key; a misordered
array refuses as `invalid_semantic_graph`. `deletes_scoped` shall hold the
`receiver` entry first, then the `parameter` entries ascending by `name` as
UTF-8 bytes; a misordered array refuses as `invalid_semantic_graph`, a
canonical-order defect located like the order defect of any other member.

Two `field` entries with equal `declaration` and `name` and different scopes
refuse as `invalid_model_binding`/`conflicting-binding`. The same pair with
one scope listed twice is a repeated entry, a schema-level refusal above, and
the scope component of the order key therefore places no admitted entry. IR
reading, in the manner of FR-346's conflicting bindings: a refusal carries
one path and one locus (FR-038 "Locating refusals") and no member that could
carry a second entry, so the refusal's locus is the two entries' declaring
node key and its path is that of the entry that sorts second by the order key,
the `receiver` one, since `every-object` sorts first. That entry is the same
entry in whatever order the two are listed (its array index follows its
position), which is why the rule is not "the later entry in array order",
whose entry would change with the listing. The other entry is not carried:
FR-340 says the refusal names both entries, and carrying the first needs a
refusal member this repository does not have, which is a QSpec question as
FR-346 states it for its own conflicts.

One frame node reports exactly one defect: any meaning-join defect outranks a
canonical-order defect, except that a `conflicting-binding` defect ranks after
every other meaning-join defect of the frame, so it is reported only when the
frame holds no other one, and it outranks any canonical-order defect, so a
frame whose two entries conflict is never reported as misordered, in whatever
order the two are listed; ties break by position, `semantic_type`, then
`modifies`, then `creates`, then `deletes`; remaining ties within one member
break by ascending order key. `deletes_scoped` holds no meaning-join defect in
the frame step, which checks its shape and order only. Frames are checked in
ascending `node_id` digest order and the first defective frame is reported.

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
inherits as `invalid_model_binding`/`malformed-declaration`); and every
`parameter` entry of the `frame` target's `deletes_scoped` names a reference
parameter of the resolved operation, else `missing_declaration`/`missing-name`
located at the frame node (QSpec FR-342 check 5). A `receiver` entry needs no
parameter, and a parameter is a reference parameter when its type is
`Reference<T>` for an object type `T`. Once every
anchor's own joins hold, a second anchor naming the same (`context`,
`operation`) pair, or the same `frame` target as another anchor, refuses as
`ambiguous_declaration`/`ambiguous-name` at the second anchor in ascending
node-id digest order; this is what the reader does today. QSpec FR-342-AC-6
admits two anchors of one `context` that reference one `frame` node, as
operations with equal frames do, and FR-040-AC-28 supersedes the
same-`frame` refusal when it is implemented: the scoped-deletes check then
runs against each anchor's operation in ascending node-id digest order, the
first failure being reported. The `operation` name is resolved only when the
`context` target is a model declaration node, as for field entries; any other
`context` admits its `operation` name unresolved. IR reading, which QSpec
FR-342 check 5 does not state: the scoped-deletes check needs the resolved
operation's parameters, so it does not run for an anchor whose `operation` is
unresolved (FR-040-AC-30).

### State clause body

A `state`/`state_clause` node's `body` is one `application` with operator
class `state_clause` and `operation.identity` `quire.op.state.clause`, whose
`operation.member` is `{"kind": "state_clause", "clause"}` with `clause` one
of `invariant`, `precondition` or `postcondition`, `laws` empty, `mode`
`null` and `leaves` empty, and whose three arguments are, in order, an
`aggregate` of one or more `reference` terms (the parameters), a `reference`
(the anchor) and the Boolean condition. Every occurrence has role `claim`,
and the node's `semantic_type` is the `scalar_type`/`boolean` node. A
`quire.op.state.clause` application standing as the body root of any node that
is not `state`/`state_clause` refuses as `ill_typed`/`operator-ineligible` at
the node that holds it, and one nested in a node body or another term refuses
`malformed_wire` at the nested application at strict wire validation (merged
QSpec FR-341-AC-6 and FR-322 "Body grammar"; FR-038 "The flat wire").

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
   then `result`, then the operation's parameters. `self` is checked for
   every clause. The result and the operation's parameters are checked only
   when the operation resolves, that is when `C` is a model declaration node;
   a clause anchored on any other context is checked for `self` alone. A
   result or parameter type is compared by the node key of the resolved
   member's slot type.

A clause node's own `semantic_type` is not checked beyond the catalog's
`result_type` for `quire.op.state.clause` being the `scalar_type`/`boolean`
node.

A `value`/`parameter` node's `body` is an `aggregate` of exactly two
bindings in order: `name`, a `text` literal whose value is an identifier, and
`level`, an `integer` literal. Every occurrence has role `expression`. A body
of any other shape refuses at the node's `body`.

An operation whose catalog entry carries the `reference_edge` constraint
(`quire.op.model.reaches_field`) is admitted exactly when, in order: operand 0
is `Reference<D>` for the `field` member's declaring model node `D`; the
member's name resolves on `D` to a field declared by object type `T` (`D` or
one of its supertypes); that field's type is `Reference<T>`,
`Option<Reference<T>>` or a bounded `Sequence<Reference<T>>`; and operand 1 is
`Reference<B>` for an object type `B` of the same selected document that is
`T` or a declared subtype of `T` (conformance one way, from `B` to `T`: a
supertype of `T` is not admitted). Any other application refuses as
`ill_typed`/`operator-ineligible` at the operand, the member declaration or
the member name at fault; a declaration keyed under an unselected domain package
refuses as `missing_declaration`/`missing-selection` at the member
declaration, and an ambiguous name as `ambiguous_declaration`/`ambiguous-name`
at the member name.

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
operation anchor (including the scoped-deletes check of its frame's
`deletes_scoped`), then every state clause, and reports the first defect.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-040-AC-1 | A `modifies` array holding a `relationship` entry and a `field` entry, each naming an eligible declared dependency, admits; an all-empty frame body of four empty members admits; a bare node key, an entry of another `kind`, a `field` entry without `name`, a `relationship` entry carrying `name`, and an entry with an extra member each refuse as `invalid_semantic_graph` with no cause, located at that entry. | Test (TC-056) |
| FR-040-AC-2 | A relationship entry naming an object type, a field entry declared on a process, a relationship or a systems node, and a `creates` entry naming a relationship each refuse as `invalid_model_binding`/`malformed-declaration` at the entry's declaring node key; an entry naming no declared dependency of the frame, or no node of the package, refuses as `missing_declaration`/`missing-name`. | Test (TC-056) |
| FR-040-AC-3 | A field entry on a model declaration object type resolves among its exposed effective fields, own and inherited; an undeclared name and an operation's name refuse as `missing_declaration`/`missing-name`, a name two supertypes both expose as `ambiguous_declaration`/`ambiguous-name`, and a declaring node keyed under an unselected domain package as `missing_declaration`/`missing-selection`, each at the entry. A field entry on a source-declared object type admits with its name unresolved; selected record value type declarations are covered by FR-040-AC-14 through FR-040-AC-16. | Test (TC-056) |
| FR-040-AC-4 | Two field entries of one declaring node in descending name order refuse as `invalid_semantic_graph`; a frame carrying a meaning-join defect and a canonical-order defect reports the meaning-join refusal regardless of member or array position; with two defective frames, the lower node-id frame's own defect is reported. | Test (TC-056) |
| FR-040-AC-5 | A frame whose `semantic_type` names a node that is not `model`/`object_type` refuses as `invalid_model_binding`/`malformed-declaration` at its `semantic_type` before any entry defect; a frame occurrence with a role other than `generated` refuses as `invalid_semantic_graph` with no cause, located at that occurrence's `role`. | Test (TC-056) |
| FR-040-AC-6 | Each of the fifteen `model` forms admits as a node form; `field_declaration`, `operation_declaration`, `clause_member_declaration` and any other `model` form refuse as `invalid_semantic_graph` with no cause, located at the node's `semantic_form`. | Test (TC-056) |
| FR-040-AC-7 | An operation anchor with the three bindings in order admits. Reordered bindings, a non-text `operation`, a missing binding, and an extra binding each refuse as `invalid_semantic_graph` with no cause, located at the anchor's `body`; an occurrence role other than `anchor` refuses the same way, located at that occurrence's `role`. A `context` naming a frame, a `frame` naming an object type, and a `frame` target missing from `dependencies` refuse as `invalid_model_binding`/`malformed-declaration`, `invalid_model_binding`/`malformed-declaration` and `missing_declaration`/`missing-name` at the target; an anchor whose `semantic_type` is not its `context` refuses as `invalid_model_binding`/`malformed-declaration` at the anchor. An `operation` the model declaration `context` declares resolves; an undeclared name and a field's name refuse as `missing_declaration`/`missing-name`, an operation the context only inherits as `invalid_model_binding`/`malformed-declaration`, and a context keyed under an unselected domain package as `missing_declaration`/`missing-selection`, each at the anchor's `operation` binding. A duplicate (`context`, `operation`) pair refuses as `ambiguous_declaration`/`ambiguous-name` at the second anchor in ascending node-id digest order; a shared `frame` target refuses the same way (FR-040-AC-28 supersedes that last clause when implemented). | Test (TC-056) |
| FR-040-AC-8 | An invariant, a precondition and a postcondition in QSpec FR-341's shape each admit; an invariant anchored at an operation anchor and a precondition anchored at an object type refuse as `invalid_model_binding`/`malformed-declaration` at the anchor's target; a parameter reference to a node that is not `value`/`parameter` refuses the same way at its target. Another operation identity, operator class, member kind or clause value, a first argument that is not an `aggregate` of `reference` terms, and a second argument that is not a `reference` each refuse as `invalid_semantic_graph` with no cause, located at the clause node's `body`; an occurrence role other than `claim` refuses the same way, located at that occurrence's `role`. | Test (TC-056) |
| FR-040-AC-9 | For `Order.scaled(n: Integer): Integer`, a precondition binding `[Reference<Order>, Integer]` and a postcondition binding `[Reference<Order>, Integer, Integer]` admit; a precondition missing the operation parameter, a postcondition missing the result, a parameter of another type, a `self` that is a `Reference` to another object type than its anchor's, and an invariant binding a second parameter each refuse as `ill_typed`/`operator-ineligible` at the clause node's first argument. | Test (TC-056) |
| FR-040-AC-10 | A `quire.op.state.clause` application standing as the body root of a node that is not `state`/`state_clause` refuses as `ill_typed`/`operator-ineligible` at the holding node, and one nested in a node body or another term refuses `malformed_wire` at the nested application (FR-038-AC-114); a non-Boolean condition refuses as `ill_typed`/`operator-ineligible` at the operation step. | Test (TC-056) |
| FR-040-AC-11 | A `value`/`parameter` body of `name` then `level` admits; reordered bindings, a non-text `name`, a non-integer `level`, and a missing binding each refuse as `invalid_semantic_graph` with no cause, located at the node's `body`; an occurrence role other than `expression` refuses the same way, located at that occurrence's `role`. | Test (TC-056) |
| FR-040-AC-12 | A package carrying a frame defect and a state defect reports the frame defect; one carrying an anchor defect and a clause defect reports the anchor defect; one carrying a state defect and an operation defect reports the state defect. | Test (TC-056) |
| FR-040-AC-13 | A `quire.op.model.reaches_field` application over `Reference<D>` operands admits when its field resolves on `D` to a `Reference<T>`, `Option<Reference<T>>` or bounded `Sequence<Reference<T>>` of the field's owner `T` and operand 1 references `T` or a subtype of `T`, including one two levels down and an inherited edge. A set, an unbounded sequence, a reference to another type and a non-reference field, an undeclared name, an operand 0 that does not name the declaring node, an operand 1 that is a supertype of `T`, unrelated or an object refuse as `ill_typed`/`operator-ineligible` at the operand, member declaration or member name at fault; an ambiguous name refuses as `ambiguous_declaration`/`ambiguous-name` and a declaring node keyed under an unselected domain package as `missing_declaration`/`missing-selection`. | Test (TC-056) |
| FR-040-AC-14 | IMPLEMENTED (IR-770). With an authentic selected record owner `Address` declaring a bounded text `street` field and an object `Account` declaring an `address: Address` field, the reader admits a frame with distinct `(Account, address)` and `(Address, street)` entries and resolves both against their selected owners. Replacing only `street` with `ghost` on the same valid selected record refuses `missing_declaration`/`missing-name` at that frame entry; an operation's name is not a field candidate, two exposed fields of one name refuse `ambiguous_declaration`/`ambiguous-name`, and a record node with no selected owner refuses `missing_declaration`/`missing-selection` before field resolution. A separate expression projection of unknown `Address.ghost` retains FR-322's `ill_typed`/`operator-ineligible` refusal at its member name. | Test (TC-056) |
| FR-040-AC-15 | IMPLEMENTED (IR-770). A selected record field whose type is another record, including an inherited field on a record subtype, retains the named record declaration node as its member type. Optional, bounded-sequence and finite-recursive record paths retain that same owner-bearing record node at the innermost element; none becomes `Reference<Address>` or acquires object identity. The effective view honors redefinitions and reports an ambiguous exposed field name rather than choosing one. | Test (TC-056) |
| FR-040-AC-16 | IMPLEMENTED (IR-770). Selected record owner indexing and effective-field derivation are charged at the existing selection row under the `work` limit; an exact limit admits and one unit less returns typed `incomplete` at that row. A record field meaning-join defect outranks canonical-order defects in its frame, lower node-id frames retain precedence, and pre-frame graph, owner and stale-key refusals retain precedence over a record name defect. Existing source-declared object, relationship, operation-anchor and state-clause outcomes are unchanged. | Test (TC-056) |
| FR-040-AC-17 | PLANNED / UNRUN (IR-816). A frame body carrying `modifies` field entries of scope `receiver` and `every-object` and a `deletes_scoped` of one `receiver` entry and `parameter` entries admits with each entry's scope and kind exposed; a frame of four empty members admits. | Test (TC-056) |
| FR-040-AC-18 | PLANNED / UNRUN (IR-816). A `field` entry with no `scope`, a `field` entry with a `scope` other than `receiver` or `every-object`, and a `relationship` entry carrying `scope` each refuse `invalid_semantic_graph` with no cause, located at that entry, before the frame step. | Test (TC-056) |
| FR-040-AC-19 | PLANNED / UNRUN (IR-816). A frame body with no `deletes_scoped` member refuses `invalid_semantic_graph` with no cause, located at the frame node's `body`, before the frame step. | Test (TC-056) |
| FR-040-AC-20 | PLANNED / UNRUN (IR-816). A `deletes_scoped` entry of a `kind` other than `receiver` or `parameter`, a `parameter` entry with no `name`, an entry carrying a further member, and an array repeating an entry each refuse `invalid_semantic_graph` with no cause before the frame step, located at the offending entry and, for a repeat, at the second occurrence of the repeated entry; a missing or further member of the frame body is located at the body (FR-040-AC-19). | Test (TC-056) |
| FR-040-AC-21 | PLANNED / UNRUN (IR-816). A `deletes_scoped` that does not hold the `receiver` entry first and the `parameter` entries ascending by name as UTF-8 bytes refuses `invalid_semantic_graph` with no cause, located at the frame node; `receiver`, `parameter a`, `parameter b` admits and `parameter a`, `receiver`, `parameter b` refuses. | Test (TC-056) |
| FR-040-AC-22 | PLANNED / UNRUN (IR-816). The `modifies` entries of one declaring node ascend by name as UTF-8 bytes; entries (`step`, `receiver`) then (`value`, `every-object`) admit and the reverse order refuses `invalid_semantic_graph` with no cause, located at the frame node. | Test (TC-056) |
| FR-040-AC-23 | PLANNED / UNRUN (IR-816). Two `field` entries sharing a declaration and name and differing in scope refuse `invalid_model_binding`/`conflicting-binding` with the declaring node key as the locus and, as the path, that of the entry that sorts second by the order key (`every-object` before `receiver`, so the `receiver` entry), whichever order the two are listed in; the other entry is not carried. | Test (TC-056) |
| FR-040-AC-24 | PLANNED / UNRUN (IR-816). A frame holding a `conflicting-binding` defect and a canonical-order defect reports the `conflicting-binding` and not the `invalid_semantic_graph` of the misordered array. | Test (TC-056) |
| FR-040-AC-25 | PLANNED / UNRUN (IR-816). A frame holding a `conflicting-binding` defect and any other meaning-join defect reports the other meaning-join defect. | Test (TC-056) |
| FR-040-AC-26 | PLANNED / UNRUN (IR-816). A `scope` or a `deletes_scoped` entry changed in a frame node left under its old key refuses `invalid_package`/`stale-node-key` before the frame step. | Test (TC-056) |
| FR-040-AC-27 | PLANNED / UNRUN (IR-816). A `parameter` entry of a frame's `deletes_scoped` naming no reference parameter of the operation an anchor referencing the frame resolves refuses `missing_declaration`/`missing-name`, located at the frame node, in the state step after that anchor's own joins hold, and not in the frame step. | Test (TC-056) |
| FR-040-AC-28 | PLANNED / UNRUN (IR-816). Two anchors of one `context` referencing one `frame` node both admit, and the frame's `parameter` entries are checked against each anchor's operation, the first failure in ascending anchor node-id digest order being reported. When implemented this criterion supersedes the last clause of FR-040-AC-7: the existing test tagged FR-040-AC-7 (`tc_056_operation_anchors_admit_their_three_bindings_and_joins`, case "a shared frame"), which asserts the refusal, changes to assert the admission, and no other outcome of AC-7 changes. | Test (TC-056) |
| FR-040-AC-29 | PLANNED / UNRUN (IR-816). A package carrying a frame-step defect and a `parameter` entry that fails the state step's scoped-deletes check reports the frame-step defect. | Test (TC-056) |
| FR-040-AC-30 | PLANNED / UNRUN (IR-816). An anchor whose `context` is not a model declaration node, and so whose `operation` is admitted unresolved, is not checked for scoped deletes: a frame `parameter` entry naming no parameter admits in the state step for it. | Test (TC-056) |

## Dependencies

[FR-038](./FR-038-consume-checked-package-v2.md) owns the reader, its
refusal codes and pointers, the `creates`/`deletes` eligibility and the
model-owned member resolution this requirement reuses. QSpec FR-340, FR-341
and FR-342 own the normative shapes and refusal order, and FR-322 the
operation catalog and reader order they extend. Lowering an admitted frame's
grants, scoped or not, to `kani::modifies` places is a later lowering requirement, not this
one.
