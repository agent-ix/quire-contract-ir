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

An admitted package whose frame nodes expose their declaring object type,
process or state machine and their `modifies`, `creates`, `deletes` and
`deletes_scoped` sets (each
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
  the node key of the `model`/`object_type`, `model`/`record_value_type`,
  `model`/`process` or `model`/`state_machine` node that declares or inherits
  the field, `name` is the field's identifier and `scope` is exactly
  `receiver` (the write is granted on the operation's
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

The frame's scoped members are retained in its complete body. A valid body edit under a retained structural id reaches the frame step; malformed members refuse at their own body or frame rule.

Where QSpec fixes a case at the schema level (a body, entry or binding of
the wrong shape, or an occurrence of the wrong role for its node form), the
reader refuses it as `invalid_semantic_graph` with no cause, located at the
offending entry, at the node's `body` for a body or binding shape, and at the
occurrence's `role` for a role.

The frame's `semantic_type` shall name the `model`/`object_type`,
`model`/`process` or `model`/`state_machine` node declaring its operation; one
naming no node refuses as `missing_declaration`/`missing-name` and one naming
another node as `invalid_model_binding`/`malformed-declaration`, located at
the `semantic_type`. An entry whose `declaration` is not among the frame
node's `dependencies`, or names no node of the package, refuses as
`missing_declaration`/`missing-name`; an entry naming a dependency its kind
cannot carry refuses as `invalid_model_binding`/`malformed-declaration`. The
locus of an entry's refusal is its declaring node key. None of the five
systems forms is an admitted declaring node.

A `creates` or `deletes` entry may name an object type or process, but not a
state machine. A `field` entry may name an object type, record value type,
process or state machine, including one distinct from the frame's declaring
type. A `field` entry with `scope: receiver` on a process or state-machine
frame refuses `invalid_model_binding`/`malformed-declaration` at the entry's
declaring node key; a `deletes_scoped` receiver entry on either frame refuses
the same at the frame node. These receiver forms remain admitted on an
object-type frame. The refusals are frame meaning-join defects under the
ordering below.

A field entry's `name` resolves among the fields of its declaring node by
FR-038's model-owned member resolution when the declaring node is a selected
model declaration node. For a `model`/`object_type`, `model`/`record_value_type`,
`model`/`process` or `model`/`state_machine` declaration node, the reader SHALL
index the
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
record owner refuses `missing_declaration`/`missing-selection`, and an unresolved record declaration reference refuses by its independent owner or graph rule before the frame step. Once that
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

A `modifies` entry's order key is its `declaration` structural node id, compared componentwise under QSpec FR-322, then its `name` as UTF-8 bytes, then its `scope` with
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
`modifies`, then `creates`, then `deletes`, then `deletes_scoped`; remaining
ties within one member break by ascending order key. On a process or
state-machine frame, a `deletes_scoped` receiver entry has the meaning-join
defect above, at the frame node. A `parameter` entry's name is checked against
each anchor's resolved operation in the later state step. Shape and canonical
order of `deletes_scoped` remain frame checks. Frames are checked in
structural `node_id` order and the first defective frame is reported.

### Operation anchor body

A `state`/`operation_anchor` node's `body` is an `aggregate` of three
required bindings in order: `context`, a `reference`; `operation`, a `text` literal
whose value is an identifier; and `frame`, a `reference`. It may carry an
optional fourth `execution_point` binding after `frame`. Every occurrence has
role `anchor`. For one anchor the reader checks, in order, and reports the
first failure: the `context` target is a declared `model`/`object_type`,
`model`/`process` or `model`/`state_machine` dependency; the `frame` target
is a declared `state`/`frame` dependency (each
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
anchor's own joins and point applicability hold, a second anchor naming the same (`context`,
`operation`) pair with the same execution point (including both absent) refuses as
`ambiguous_declaration`/`ambiguous-name` at the second anchor in ascending
structural node-id order. QSpec FR-342-AC-6
admits two anchors of one `context` that reference one `frame` node, as
operations with equal frames do. An inherited-only operation is anchored
at its declaring type: a subtype process or state machine naming an operation
it only inherits refuses at the operation binding. Anchors of different
declaring types cannot share a frame because its `semantic_type` equals the
anchor context. The scoped-deletes check
runs against each anchor's operation in structural node-id order, the
first failure being reported. The `operation` name is resolved only when the
`context` target is one of the three admitted model declaration forms, as
for field entries. Any other context refuses before operation resolution or
the scoped-deletes check (FR-040-AC-30).

The optional fourth binding's `value` is the bare closed FR-322
`ExecutionPoint` object (`{kind, name}` or `{kind, operation}`), not a
`CheckedTerm` and not an application argument. The anchor-body decoder shall
read this member as a dedicated typed point alongside the three existing
term-valued bindings; it shall not invent a `term` tag or put the point into
the generic expression-term grammar. Its wire shape is checked against
QSpec's `OperationAnchorBody`/`ExecutionPoint` before graph and state checks.
A wrong binding count, position or non-point fourth value refuses
`invalid_semantic_graph` with no cause at the anchor `body` before the state
step. IR reading of QSpec's shape/semantic tension: an object present in the
fourth position but failing the closed `ExecutionPoint` schema (unknown kind,
missing or extra member, empty name) is a schema-level
`invalid_semantic_graph` with no cause at that binding's `value`; only a
well-formed point of the wrong applicable kind or operation name reaches the
state step and refuses `invalid_model_binding`/`malformed-declaration` at the
anchor node. A state-clause member's point has the same split: a point failing
the closed schema refuses `invalid_semantic_graph` with no cause at
`body/operation/member/execution_point` before state, while a well-formed but
inapplicable or disagreeing point reaches the state step.

An anchor's optional fourth binding has exactly the closed FR-322
`ExecutionPoint` shape. It is `pre{operation}` or `post{operation}`, with
`operation` equal to the resolved `operation` binding name; a clause-bound
anchor's point equals every clause that references it, while a protocol-only
anchor may omit the point. A well-formed `initialization`/`handler` point or a well-formed
`pre`/`post` point with a disagreeing operation name refuses
`invalid_model_binding`/`malformed-declaration` at the anchor node in the
state step. The fourth binding and the state-clause member point are retained in their complete bodies. The reader validates their shapes and agreement independently of the structural id. A valid point-only body edit may retain that id; no stale-key refusal precedes the state step.

### State clause body

The admitted `CheckedApplicationOperation::state_clause_kind()` public accessor
SHALL return the closed `StateClauseKind` of its `state_clause` member only for
an admitted `quire.op.state.clause` operation. It SHALL return `None` for an
operation of another identity or member shape. The accessor reads the retained
typed admission result; it neither exposes the operation's raw record nor
re-parses its JSON. FR-038 owns the other public operation views.

A `state`/`state_clause` node's `body` is one `application` with operator
class `state_clause` and `operation.identity` `quire.op.state.clause`, whose
`operation.member` is `{"kind": "state_clause", "clause", "execution_point"?}` with `clause` one
of `invariant`, `constraint`, `precondition` or `postcondition` and an optional
`execution_point`, `laws` empty, `mode` `null` and `leaves` empty, and whose three arguments are, in order, an
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
   `missing_declaration`/`missing-name`; for an invariant or constraint it is a
   `model`/`object_type`, `model`/`process` or `model`/`state_machine`
   node and for a precondition or postcondition a
   `state`/`operation_anchor` node, else `invalid_model_binding`/
   `malformed-declaration`, located at the anchor's target.
2. **Parameters.** Each parameter reference, in aggregate order, targets a
   declared dependency, else `missing_declaration`/`missing-name`, that is a
   `value`/`parameter` node, else `invalid_model_binding`/
   `malformed-declaration`, located at the offending target.
3. **Signature.** The parameters are, in order, `self` of type
   `Reference<C>`; for a postcondition, the resolved operation's result when
   it has one; and for a precondition or postcondition, the resolved
   operation's parameters in declared order. `C` is the anchor object type,
   process or state machine for an invariant or constraint and the anchor's
   `context` otherwise; an
   invariant or constraint binds `self` alone. A different count or type refuses as
   `ill_typed`/`operator-ineligible` at the clause node, checking `self`,
   then `result`, then the operation's parameters. `self` is checked for
   every clause. The result and the operation's parameters are checked only
   when the operation resolves on the admitted model declaration context. A
   result or parameter type is compared by the node key of the resolved
   member's slot type.

4. **Execution point.** A present point already passed FR-322's closed
   schema check before the state step; an unknown variant, missing or extra
   member, or empty name refused `invalid_semantic_graph` with no cause at
   `/semantic_graph/nodes/{i}/body/operation/member/execution_point` there.
   Here `initialization{name}` or `handler{name}` applies to an invariant or
   constraint, and `pre{operation}` or `post{operation}` to a precondition or
   postcondition. For an operation clause, the `operation` name equals the
   resolved operation name and its point, including absence, equals its
   `operation_anchor` point. For an invariant or constraint, the reader
   retains the nonempty name unchanged; the V2 package has no independently
   selected source anchor name to compare. QSL owns that source agreement,
   verified by the fresh producer vector in TC-056. A well-formed point of an
   inapplicable kind, a mismatched operation name, or a clause/anchor point
   mismatch refuses `invalid_model_binding`/`malformed-declaration` at
   `/semantic_graph/nodes/{i}` in the state step.

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

The frame step runs after every graph-shape and declaration
refusal. The state step runs after the frame step and before the operation
step; within it the reader checks, each in structural node-id order,
the placement of every `quire.op.state.clause` application, then every
operation anchor (including the scoped-deletes and point checks), then tuple
uniqueness among anchors whose own checks passed, then every state clause,
then every `state`/`transition` node under
[FR-067](./FR-067-admit-boolean-state-transition.md), and reports the first
defect. The state step checks the placement of a
`quire.op.state.transition` application before the anchor checks, as QSpec
FR-341 requires.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-040-AC-1 | A `modifies` array holding a `relationship` entry and a `field` entry, each naming an eligible declared dependency, admits; an all-empty frame body of four empty members admits; a bare node key, an entry of another `kind`, a `field` entry without `name`, a `relationship` entry carrying `name`, and an entry with an extra member each refuse as `invalid_semantic_graph` with no cause, located at that entry. | Test (TC-056) |
| FR-040-AC-2 | A relationship entry naming an object type, a field entry declared on a relationship or systems node, a `creates` entry naming a relationship or state machine, and a `deletes` entry naming a state machine each refuse as `invalid_model_binding`/`malformed-declaration` at the entry's declaring node key; a field entry on a process or state machine and a `creates` or `deletes` entry naming a process admit when their other joins hold. An entry naming no declared dependency of the frame, or no node of the package, refuses as `missing_declaration`/`missing-name`. | Test (TC-056) |
| FR-040-AC-3 | A field entry on a model declaration object type resolves among its exposed effective fields, own and inherited; an undeclared name and an operation's name refuse as `missing_declaration`/`missing-name`, a name two supertypes both expose as `ambiguous_declaration`/`ambiguous-name`, and a declaring node keyed under an unselected domain package as `missing_declaration`/`missing-selection`, each at the entry. A field entry on a source-declared object type admits with its name unresolved; selected record value type declarations are covered by FR-040-AC-14 through FR-040-AC-16. | Test (TC-056) |
| FR-040-AC-4 | Two field entries of one declaring node in descending name order refuse as `invalid_semantic_graph`; a frame carrying a meaning-join defect and a canonical-order defect reports the meaning-join refusal regardless of member or array position; with two defective frames, the lower node-id frame's own defect is reported. | Test (TC-056) |
| FR-040-AC-5 | A frame whose `semantic_type` names a `model`/`object_type`, `model`/`process` or `model`/`state_machine` declaring its operation admits that type join; a frame whose `semantic_type` names any other node refuses as `invalid_model_binding`/`malformed-declaration` at its `semantic_type` before any entry defect. A frame occurrence with a role other than `generated` refuses as `invalid_semantic_graph` with no cause, located at that occurrence's `role`. | Test (TC-056) |
| FR-040-AC-6 | Each of the fifteen `model` forms admits as a node form; `field_declaration`, `operation_declaration`, `clause_member_declaration` and any other `model` form refuse as `invalid_semantic_graph` with no cause, located at the node's `semantic_form`. | Test (TC-056) |
| FR-040-AC-7 | An operation anchor with the three bindings in order admits. Reordered bindings, a non-text `operation`, a missing binding, and an extra binding each refuse as `invalid_semantic_graph` with no cause, located at the anchor's `body`; an occurrence role other than `anchor` refuses the same way, located at that occurrence's `role`. A `context` naming a frame, a `frame` naming an object type, and a `frame` target missing from `dependencies` refuse as `invalid_model_binding`/`malformed-declaration`, `invalid_model_binding`/`malformed-declaration` and `missing_declaration`/`missing-name` at the target; an anchor whose `semantic_type` is not its `context` refuses as `invalid_model_binding`/`malformed-declaration` at the anchor. An `operation` declared by an object type, process or state-machine `context` resolves; an undeclared name and a field's name refuse as `missing_declaration`/`missing-name`, an operation the context only inherits as `invalid_model_binding`/`malformed-declaration`, and a context keyed under an unselected domain package as `missing_declaration`/`missing-selection`, each at the anchor's `operation` binding. A duplicate (`context`, `operation`) pair refuses as `ambiguous_declaration`/`ambiguous-name` at the second anchor in structural node-id order; a shared `frame` target refuses the same way (FR-040-AC-28 supersedes the former shared-frame refusal). | Test (TC-056) |
| FR-040-AC-8 | An invariant, a precondition and a postcondition in QSpec FR-341's shape each admit; an invariant anchored at an operation anchor and a precondition anchored at an object type, process or state machine refuse as `invalid_model_binding`/`malformed-declaration` at the anchor's target; a parameter reference to a node that is not `value`/`parameter` refuses the same way at its target. Another operation identity, operator class, member kind or clause value, a first argument that is not an `aggregate` of `reference` terms, and a second argument that is not a `reference` each refuse as `invalid_semantic_graph` with no cause, located at the clause node's `body`; an occurrence role other than `claim` refuses the same way, located at that occurrence's `role`. | Test (TC-056) |
| FR-040-AC-9 | For `Order.scaled(n: Integer): Integer`, a precondition binding `[Reference<Order>, Integer]` and a postcondition binding `[Reference<Order>, Integer, Integer]` admit; a precondition missing the operation parameter, a postcondition missing the result, a parameter of another type, a `self` that is a `Reference` to another declaring type than its anchor's, and an invariant binding a second parameter each refuse as `ill_typed`/`operator-ineligible` at the clause node's first argument. | Test (TC-056) |
| FR-040-AC-10 | A `quire.op.state.clause` application standing as the body root of a node that is not `state`/`state_clause` refuses as `ill_typed`/`operator-ineligible` at the holding node, and one nested in a node body or another term refuses `malformed_wire` at the nested application (FR-038-AC-114); a non-Boolean condition refuses as `ill_typed`/`operator-ineligible` at the operation step. | Test (TC-056) |
| FR-040-AC-11 | A `value`/`parameter` body of `name` then `level` admits; reordered bindings, a non-text `name`, a non-integer `level`, and a missing binding each refuse as `invalid_semantic_graph` with no cause, located at the node's `body`; an occurrence role other than `expression` refuses the same way, located at that occurrence's `role`. | Test (TC-056) |
| FR-040-AC-12 | A package carrying a frame defect and a state defect reports the frame defect; one carrying an anchor defect and a clause defect reports the anchor defect; one carrying a state defect and an operation defect reports the state defect. | Test (TC-056) |
| FR-040-AC-13 | A `quire.op.model.reaches_field` application over `Reference<D>` operands admits when its field resolves on `D` to a `Reference<T>`, `Option<Reference<T>>` or bounded `Sequence<Reference<T>>` of the field's owner `T` and operand 1 references `T` or a subtype of `T`, including one two levels down and an inherited edge. A set, an unbounded sequence, a reference to another type and a non-reference field, an undeclared name, an operand 0 that does not name the declaring node, an operand 1 that is a supertype of `T`, unrelated or an object refuse as `ill_typed`/`operator-ineligible` at the operand, member declaration or member name at fault; an ambiguous name refuses as `ambiguous_declaration`/`ambiguous-name` and a declaring node keyed under an unselected domain package as `missing_declaration`/`missing-selection`. | Test (TC-056) |
| FR-040-AC-14 | IMPLEMENTED (IR-770). With an authentic selected record owner `Address` declaring a bounded text `street` field and an object `Account` declaring an `address: Address` field, the reader admits a frame with distinct `(Account, address)` and `(Address, street)` entries and resolves both against their selected owners. Replacing only `street` with `ghost` on the same valid selected record refuses `missing_declaration`/`missing-name` at that frame entry; an operation's name is not a field candidate, two exposed fields of one name refuse `ambiguous_declaration`/`ambiguous-name`, and a record node with no selected owner refuses `missing_declaration`/`missing-selection` before field resolution. A separate expression projection of unknown `Address.ghost` retains FR-322's `ill_typed`/`operator-ineligible` refusal at its member name. | Test (TC-056) |
| FR-040-AC-15 | IMPLEMENTED (IR-770). A selected record field whose type is another record, including an inherited field on a record subtype, retains the named record declaration node as its member type. Optional, bounded-sequence and finite-recursive record paths retain that same owner-bearing record node at the innermost element; none becomes `Reference<Address>` or acquires object identity. The effective view honors redefinitions and reports an ambiguous exposed field name rather than choosing one. | Test (TC-056) |
| FR-040-AC-16 | IMPLEMENTED (IR-770). Selected record owner indexing and effective-field derivation are charged at the existing selection row under the `work` limit; an exact limit admits and one unit less returns typed `incomplete` at that row. A record field meaning-join defect outranks canonical-order defects in its frame, lower node-id frames retain precedence, and pre-frame graph and owner refusals retain precedence over a record name defect. Existing source-declared object, relationship, operation-anchor and state-clause outcomes are unchanged. | Test (TC-056) |
| FR-040-AC-17 | An object-type frame body carrying `modifies` field entries of scope `receiver` and `every-object` and a `deletes_scoped` of one `receiver` entry and `parameter` entries admits with each entry's scope and kind exposed; a frame of four empty members admits. | Test (TC-056) |
| FR-040-AC-18 | A `field` entry with no `scope`, a `field` entry with a `scope` other than `receiver` or `every-object`, and a `relationship` entry carrying `scope` each refuse `invalid_semantic_graph` with no cause, located at that entry, before the frame step. | Test (TC-056) |
| FR-040-AC-19 | A frame body with no `deletes_scoped` member refuses `invalid_semantic_graph` with no cause, located at the frame node's `body`, before the frame step. | Test (TC-056) |
| FR-040-AC-20 | A `deletes_scoped` entry of a `kind` other than `receiver` or `parameter`, a `parameter` entry with no `name`, an entry carrying a further member, and an array repeating an entry each refuse `invalid_semantic_graph` with no cause before the frame step, located at the offending entry and, for a repeat, at the second occurrence of the repeated entry; a missing or further member of the frame body is located at the body (FR-040-AC-19). | Test (TC-056) |
| FR-040-AC-21 | A `deletes_scoped` that does not hold the `receiver` entry first and the `parameter` entries ascending by name as UTF-8 bytes refuses `invalid_semantic_graph` with no cause, located at the frame node; `receiver`, `parameter a`, `parameter b` admits and `parameter a`, `receiver`, `parameter b` refuses. | Test (TC-056) |
| FR-040-AC-22 | The `modifies` entries of one declaring node ascend by name as UTF-8 bytes; entries (`step`, `receiver`) then (`value`, `every-object`) admit and the reverse order refuses `invalid_semantic_graph` with no cause, located at the frame node. | Test (TC-056) |
| FR-040-AC-23 | Two `field` entries sharing a declaration and name and differing in scope refuse `invalid_model_binding`/`conflicting-binding` with the declaring node key as the locus and, as the path, that of the entry that sorts second by the order key (`every-object` before `receiver`, so the `receiver` entry), whichever order the two are listed in; the other entry is not carried. | Test (TC-056) |
| FR-040-AC-24 | A frame holding a `conflicting-binding` defect and a canonical-order defect reports the `conflicting-binding` and not the `invalid_semantic_graph` of the misordered array. | Test (TC-056) |
| FR-040-AC-25 | A frame holding a `conflicting-binding` defect and any other meaning-join defect reports the other meaning-join defect. | Test (TC-056) |
| FR-040-AC-26 | Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recompute identity from body content or refuse a retained structural id solely because a valid body changed. This id is retained and never reused. | Withdrawn |
| FR-040-AC-27 | A `parameter` entry of a frame's `deletes_scoped` naming no reference parameter of the operation an anchor referencing the frame resolves refuses `missing_declaration`/`missing-name`, located at the frame node, in the state step after that anchor's own joins hold, and not in the frame step. | Test (TC-056) |
| FR-040-AC-28 | Two anchors of one `context` referencing one `frame` node both admit, and the frame's `parameter` entries are checked against each anchor's operation, the first failure in ascending anchor structural node-id order being reported. This criterion supersedes the last clause of FR-040-AC-7: the test tagged FR-040-AC-7 (`tc_056_operation_anchors_admit_their_three_bindings_and_joins`, case "a shared frame") asserts admission; no other outcome of AC-7 changes. | Test (TC-056) |
| FR-040-AC-29 | A package carrying a frame-step defect and a `parameter` entry that fails the state step's scoped-deletes check reports the frame-step defect. | Test (TC-056) |
| FR-040-AC-30 | A context target naming no node or absent from the anchor's declared dependencies refuses `missing_declaration`/`missing-name` at that target. A declared context target that is not a `model`/`object_type`, `model`/`process` or `model`/`state_machine` refuses `invalid_model_binding`/`malformed-declaration` at the context target before operation resolution or scoped deletes. A valid process or state-machine anchor with a frame `parameter` entry naming no reference parameter of its resolved own operation refuses `missing_declaration`/`missing-name` at the frame node in the state step. This corrects the earlier unresolved-context admission oracle to match QSpec FR-342 checks 1 and 5; it does not admit an additional context kind. | Test (TC-056) |
| FR-040-AC-31 | PLANNED/UNRUN (IR-1141). An anchor admits its three required bindings with no point or a fourth execution_point binding after frame holding the closed pre or post object. Reordering the fourth binding or adding another binding refuses invalid_semantic_graph at the exact offending member. The reader retains the point in the complete body; it does not rederive the node id from that body. | Test (TC-056) |
| FR-040-AC-32 | PLANNED/UNRUN (IR-1141). An anchor with an unknown point variant, missing or extra point member, or empty name refuses `invalid_semantic_graph` with no cause at `/semantic_graph/nodes/{i}/body/members/3/value` before state. A well-formed `initialization` or `handler` point or point whose operation differs from the checked operation refuses `invalid_model_binding`/`malformed-declaration` at `/semantic_graph/nodes/{i}` in state. A duplicate `(context, operation, point)` tuple refuses `ambiguous_declaration`/`ambiguous-name` at the second anchor by ascending structural node id; the same context and operation with distinct accepted points or one absent point admit. This refines AC-7's pair uniqueness without changing its no-point duplicate outcome. | Test (TC-056) |
| FR-040-AC-33 | A state clause admits an optional `execution_point` member with each closed FR-322 variant: `initialization{name}` and `handler{name}` on an invariant or constraint, `pre{operation}` and `post{operation}` on a precondition or postcondition. The reader admits an omitted point as an explicit absence and never infers one from clause kind or anchor; the producer determines whether to emit one. A point-only body edit changes the retained body and accessor result under the same valid structural id; an accepted point round-trips unchanged. A `constraint` is checked like an invariant. This extends AC-8's three listed admitted clause kinds. | Test (TC-056) |
| FR-040-AC-34 | An unknown clause point variant, missing or extra point member, or empty name refuses `invalid_semantic_graph` with no cause at `/semantic_graph/nodes/{i}/body/operation/member/execution_point` before state. A well-formed point inapplicable to the clause kind, an operation name different from the resolved operation, or a point unequal to the referenced operation anchor's point (including present on only one side) refuses `invalid_model_binding`/`malformed-declaration` at `/semantic_graph/nodes/{i}` in state. Invariant and constraint point names are retained exactly; source-name agreement is QSL-owned and verified through a fresh producer vector. A point-only body edit is judged by point shape, applicability and clause-anchor agreement, without a stale-key refusal. | Test (TC-056) |
| FR-040-AC-35 | PLANNED/UNRUN (IR-1141). With simultaneous defects, graph shape precedes frame, frame precedes state; within state, placement precedes structural-id-ordered anchors and anchors precede clauses. Anchor checks precede duplicate (context, operation, point) detection, and clause signature checks precede point applicability and equality. The first applicable defect returns its exact code, cause and RFC 6901 locus with no package. | Test (TC-056) |
| FR-040-AC-36 | For an admitted package, writer/encoder output preserves the exact optional point object in each anchor body and clause application member, including variant and name; an absent point emits no `execution_point` member. Reading the writer output returns equal retained typed points (or equal absence), equal structural node ids and equal package name. Writer output never turns the anchor's bare point object into an expression term. | Test (TC-056) |
| FR-040-AC-37 | Implemented (IR-1082). Through the public V2 reader, each admitted invariant, constraint, precondition and postcondition application returns its exact `StateClauseKind` from `state_clause_kind()`; an admitted non-state-clause operation returns `None`. A malformed clause member is refused by admission and cannot produce a value through this accessor. The accessor returns a typed value without exposing a raw operation record. | Test (TC-056) |
| FR-040-AC-38 | For selected process `Flow` and state machine `Gate`, a frame whose `semantic_type` is that declaring node admits `every-object` field entries naming own and inherited fields, and a `parameter` scoped-delete entry checked against its anchor operation. The process frame also admits `creates` and `deletes` entries naming a process; the state-machine entry of either kind refuses `invalid_model_binding`/`malformed-declaration` at the entry key. A missing process or state-machine field refuses `missing_declaration`/`missing-name` at the field entry. A field entry naming either form also admits in an object-type frame. A `receiver` scoped field on a process or state-machine frame refuses `invalid_model_binding`/`malformed-declaration` at its declaring key; a `receiver` scoped delete refuses the same at the frame node, before canonical-order defects. Both remain admitted on an object-type frame. | Test (TC-056) |
| FR-040-AC-39 | A process anchor `(Flow, "go")` and state-machine anchor `(Gate, "toggle")` each resolve an operation declared by that exact context, with anchor and frame `semantic_type` equal to it. An operation inherited by subtype process `SubFlow` is anchored at `Flow`; `(SubFlow, "go")` when only inherited refuses `invalid_model_binding`/`malformed-declaration` at the operation binding. Two own operations of one process or state machine with equal frames share a frame node, while anchors of two different declaring types cannot. A context that is a record value type, event type, persistence interface or systems interface refuses at the context target. An invariant or constraint anchored at either admitted type binds only `self: Reference<C>`; preconditions and postconditions bind that `self` and the resolved own operation's result and parameters in the stated order. | Test (TC-056) |

## Dependencies

[FR-038](./FR-038-consume-checked-package-v2.md) owns the reader, its
refusal codes and pointers, the `creates`/`deletes` eligibility and the
model-owned member resolution this requirement reuses. QSpec FR-340, FR-341
and FR-342 own the normative shapes and refusal order, and FR-322 the
operation catalog and reader order they extend. Lowering an admitted frame's
grants, scoped or not, to `kani::modifies` places is a later lowering requirement, not this
one.
