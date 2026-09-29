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

Verify FR-040-AC-1 through FR-040-AC-12: the V2 reader admits QSpec FR-340's
`FrameModifiesEntry` values, FR-342's operation anchor body, FR-341's state
clause and parameter bodies and exactly the fifteen `model` forms, and
refuses every other shape with the code, cause and locus those requirements
fix, in their reader order.

## Test Procedure

Build, from this repository's own public vocabulary, checked packages
holding frames, operation anchors, an invariant, a precondition and
parameter nodes. Read each and confirm it admits. Then
apply each single mutation the FR-040 criteria name and read each mutated
package: entry shapes and kinds, eligibility per member, misordered
`modifies`, a meaning-join defect with an
order defect, two defective frames, frame `semantic_type`, each removed
`model` form, each anchor binding and join, duplicate anchors, each clause
anchor, parameter and signature case, nested and misplaced clause
applications, a non-Boolean condition, each parameter body defect, each
occurrence role, and pairs of defects across the frame, state and operation
steps, and field entries on a source-declared object type and on a
`record_value_type` node with and without a `declaration`.

## Expected Results

The unmutated package admits. Each mutation returns exactly the code, cause
and RFC 6901 locus its criterion names and no package; a package with defects
in two steps reports the earlier step's defect.

## Status

Implemented in `tests/it/checked_package_v2_frame_entries.rs`
(FR-040-AC-1 to AC-12) and the entry eligibility tests in
`crates/quire-contract-model/src/checked_package/v2/frame.rs`.
