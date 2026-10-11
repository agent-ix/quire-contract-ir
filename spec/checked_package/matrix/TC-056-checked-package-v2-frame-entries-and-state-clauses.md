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
---
# TC-056: CheckedPackage V2 frame entries, operation anchors and state clauses admit or refuse exactly

## Description

Verify FR-040-AC-1 through FR-040-AC-16: the V2 reader admits QSpec FR-340's
`FrameModifiesEntry` values, FR-342's operation anchor body, FR-341's state
clause and parameter bodies and exactly the fifteen `model` forms, and
refuses every other shape with the code, cause and locus those requirements
fix, in their reader order.

## Test Procedure

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

PLANNED/UNRUN (IR-770; FR-040-AC-14 through FR-040-AC-16): through the public
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

## Expected Results

The unmutated package admits. Each mutation returns exactly the code, cause
and RFC 6901 locus its criterion names and no package; a package with defects
in two steps reports the earlier step's defect.

## Status

FR-040-AC-1 through FR-040-AC-13 are implemented in
`tests/it/checked_package_v2_frame_entries.rs` and the entry eligibility
tests in `crates/quire-contract-model/src/checked_package/v2/frame.rs`.
FR-040-AC-14 through FR-040-AC-16 are PLANNED/UNRUN (IR-770), with no tagged
implementation tests yet.
