---
id: TC-056
title: "CheckedPackage V2 frame entries, operation anchors and state clauses admit or refuse exactly"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: verifies
  - target: ix://agent-ix/quire-specification/FR-340
    type: references
  - target: ix://agent-ix/quire-specification/FR-341
    type: references
  - target: ix://agent-ix/quire-specification/FR-342
    type: references
  - target: ix://agent-ix/quire-spec-language/TC-1043
    type: references
---
# TC-056: CheckedPackage V2 frame entries, operation anchors and state clauses admit or refuse exactly

## Description

Verify FR-040-AC-1 through FR-040-AC-39: the V2 reader admits QSpec FR-340's
`FrameModifiesEntry` values, FR-342's operation anchor body, FR-341's state
clause and parameter bodies and exactly the fifteen `model` forms, and
refuses every other shape with the code, cause and locus those requirements
fix, in their reader order.

## Test Procedure

Planned process and state-machine seam (FR-040-AC-2, AC-5, AC-7 through
AC-9, AC-30, AC-38 and AC-39): build a selected `Flow` process with own and
inherited fields and own `go` operation, its `SubFlow` subtype inheriting
`go`, and a selected `Gate` state machine with own and inherited fields and
own `toggle` operation. Independently derive checked-package keys for a
frame and anchor per declaring type. Admit both `semantic_type` values,
`every-object` field entries on each, a process `creates` and `deletes`, and
one `parameter` scoped delete matching each own operation's reference
parameter. Admit a process or state-machine field entry in an object-type
frame. Refuse state-machine `creates` and `deletes`, an unknown field, a
`receiver` field scope and `receiver` scoped delete on each non-object frame,
and wrong-kind frame `semantic_type`, with exact codes, causes and loci from
FR-040. Pair each meaning-join defect with a canonical-order defect to
verify precedence. A process or state-machine context anchors its own
operation with both anchor and frame `semantic_type` equal to that context;
`(SubFlow, go)` when `go` is only inherited refuses at the operation binding,
and `(Flow, go)` remains the declaring anchor. Check same-context equal-frame
sharing and different-context frame nonsharing. A context target naming no
node or absent from the anchor's declared dependencies refuses
`missing_declaration`/`missing-name`; a declared record, event, persistence
or systems context refuses `invalid_model_binding`/`malformed-declaration`,
both before operation or scoped-delete checking.
Admit invariant and constraint clauses binding `self: Reference<C>` and
operation clauses binding `self`, result and own operation parameters for
both new contexts. A bad reference parameter of a valid process or
state-machine anchor refuses at the frame node in the state step. This
replaces the former unresolved invalid-context admission oracle of AC-30;
the earlier text remains available in revision history. Use QSpec
FR-340-AC-12/19/20 and FR-342-AC-2/7/8/9 as the independent oracle.

Build, from this repository's own public vocabulary, checked packages
holding frames, operation anchors, an invariant, a precondition, a
postcondition and parameter nodes. Read each and confirm it admits. Then
apply each single mutation the FR-040 criteria name and read each mutated
package: entry shapes and kinds, eligibility per member, misordered
`modifies`, a meaning-join defect with an
order defect, two defective frames, frame `semantic_type`, each removed
`model` form, each anchor binding and join, duplicate anchors, each clause
anchor, parameter and signature case, nested and misplaced clause
applications, a non-Boolean condition, each parameter body defect, each
occurrence role, and pairs of defects across the frame, state and operation
steps, and field entries on a source-declared object type. Over a lock
selecting a domain package document built here (`Order` with a field and
`scaled(Integer): Integer`, a subtype, and a type inheriting one field name
from two supertypes) and its model declaration nodes, read field entries,
anchor operations and clause parameter lists that resolve, name nothing,
name the other member kind, are ambiguous, are only inherited, are keyed
under an unselected domain package, or bind the wrong parameters.

IMPLEMENTED (IR-770; FR-040-AC-14 through FR-040-AC-16): through the public
V2 reader, select an authentic record owner with a bounded text field and an
object with a field typed by that record. Read a frame naming both the object
field and the record's own field and assert that each resolves against its
own effective-field owner. Change only the record field name to an unknown
identifier and assert `missing_declaration`/`missing-name` at that frame
entry; separately project the unknown field in an expression and assert
FR-322's `ill_typed`/`operator-ineligible` at its member name. Exercise an
operation name, two exposed fields of one name, an unselected or wrong-kind
record owner, and a source-declared object control.

Build selected nested and inherited record fields, a redefinition, an
optional record field, a bounded sequence of records and finite recursion
through an optional field. Assert that each derived inner type retains the
selected record declaration node and never becomes an object reference.
Compare the reader's effective field outcomes with independently authored
selected-domain declarations, not a copy of QSpec's vector bytes. Pair a
record name defect with a canonical-order defect, a lower-key frame defect,
and a pre-frame graph, owner or stale-key defect to assert the specified
first refusal. Read the selected owner table at the exact work limit and one
unit less, then repeat the existing object, relationship, anchor and clause
controls to establish their unchanged outcomes.

Implemented (FR-040-AC-17 through FR-040-AC-30): authored, from this
repository's own public vocabulary, the in-repo `Account` fixture with
`balance` and `total` fields, and
the selected `Order` fixture with a `scaled` operation whose named `n`
parameter has Integer type and a `reset` operation with no parameters.
Oracle: quire-spec-language TC-1043 (wire vectors W1 to W6 and reader rows X1 to
X9), referenced by id and re-authored here, never copied. Read frames, each
carrying all four members and its anchor, in these cases: a receiver entry and
an every-object entry with `deletes_scoped` of `receiver` (W1); a frame with
`parameter a`, `parameter b` after `receiver` (W2) and the empty frame (W3);
a `field` entry without `scope` and with another `scope` (X1, X2); a body
without `deletes_scoped` (X3); `deletes_scoped` in a wrong order (X4, X5) and
with a repeated entry; `modifies` entries in wrong name or scope order; two
entries for one field with different scopes, listed in either order (X8, X9),
alone (asserting the same entry, the `receiver` one, is located in both listings), beside a misordered array and beside another meaning-join defect; an
edited `scope` or `deletes_scoped` with the frame node left under its old key; a
`parameter` entry naming the non-reference `scaled` parameter (`n`) and one
naming no parameter of `scaled` (X6), with the frame step admitting and the
state step
refusing at the frame node; two anchors of one context sharing one frame node,
where one operation has the named parameter and the other has not; an anchor whose context is not a model declaration node, with a `parameter` entry naming no parameter, admitted in the state step; and a frame
defect beside a failing `parameter` entry. Assert the order of schema, graph,
frame step (meaning-join with `conflicting-binding` last, then canonical
order), state step.

Build both an unpointed protocol anchor and point-qualified `pre` and
`post` anchors for the same checked operation. Build invariant and constraint
clauses with `initialization` and `handler` points and operation clauses with
`pre` and `post` points. Retain valid structural node ids and the package name from this
repository's own public vocabulary, read each, then write each admitted package and round-trip the retained
point or absence through the public writer and reader. Mutate one field at a time: point variant, required name, extra member,
applicability to clause kind, operation name, and clause/anchor
point agreement. Mutate valid point bodies under retained structural ids and separately test malformed or duplicate ids.
Pair each point defect with an earlier graph-shape, frame, anchor or signature
defect and a later operation-catalog defect to test first-refusal order.
For source-name agreement, run a fresh QSL producer vector from its
authoritative checkout with an invariant or constraint bound to a named
initialization or handler. Compare the produced point name to the producer's
checked source anchor name and then read the package, asserting the IR reader
retains that name exactly. A producer mismatch fails the QSL-origin vector;
it is not treated as an IR reader refusal because the wire has no independent
source-anchor name to compare.
Assert the exact code, cause and RFC 6901 locus of the first IR refusal, and no
partial package. Cover FR-040-AC-31 through FR-040-AC-36. This is planned
until tagged reader tests and the full reviewed-code gate pass.

## Expected Results

The unmutated package admits. Each mutation returns exactly the code, cause
and RFC 6901 locus its criterion names and no package; a package with defects
in two steps reports the earlier step's defect. The IR-816 cases return the
codes, causes, paths and loci FR-040-AC-17 through FR-040-AC-30 name; a
`conflicting-binding` is reported over a misordered array and after any other
meaning-join defect; a scoped-deletes `parameter` defect is a state-step
`missing_declaration`/`missing-name` at the frame node and is never reported
ahead of a frame-step defect.

## Status

FR-040-AC-1 through FR-040-AC-13 have existing tests for their former
object-type cases; the corrected process/state-machine cases are planned
under AC-38/39 and are not credited as implemented. FR-040-AC-28 supersedes
the former shared-frame refusal of FR-040-AC-7. Their tests are in
`tests/it/checked_package_v2_frame_entries.rs` and the entry eligibility
tests in `crates/quire-contract-model/src/checked_package/v2/frame.rs`.
FR-040-AC-31 through FR-040-AC-36 are planned under IR-1040; no implemented
coverage is claimed. FR-040-AC-17 through FR-040-AC-30 are implemented by tagged reader tests in
`tests/it/checked_package_v2_frame_entries.rs`. FR-040-AC-14 through FR-040-AC-16 are implemented by the tagged public-reader
record frame and expression cases in `tests/it/checked_package_v2_frame_entries.rs`
and `tests/it/checked_package_v2_model_fields.rs`, and the selected-row and
effective-type unit cases in `model_members/tests.rs` and `model_fields/tests.rs`.
