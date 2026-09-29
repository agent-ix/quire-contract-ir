---
id: TC-053
title: "CheckedPackage V2 frame-body eligibility, precedence and visit order"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-053: CheckedPackage V2 frame-body eligibility, precedence and visit order

## Description

Verify FR-038-AC-12 through FR-038-AC-15: the closed `creates`/`deletes`
entry eligibility, the `missing_declaration`/`invalid_model_binding` refusal
split and its cause, the meaning-join-over-order and member-then-order-key
refusal precedence, and the ascending-node-id-digest visit order across
multiple frame nodes. The `modifies` entry shape, its eligibility and QSpec's
published `frame_mutations` vectors are FR-040's and TC-056's.

## Test Procedure

Enumerate every `(member, tag, form)` triple the closed `CheckedNodeKind`
taxonomy can produce and assert each against the frame member's own
eligibility predicate, so that for `creates` and `deletes` the four eligible
triples (`model`/`object_type` and `model`/`process` in each) and every other
triple are checked, not sampled. Using the in-repo `v2_all_families()`
fixture built from this crate's own public vocabulary, reassign its frame
body so its declared `object_type` and `process` dependencies sit in
`creates` and `deletes` respectively and read the package. Then read one
locally authored package per rule: an entry naming a digest outside the
frame's own `dependencies`, once for a real node elsewhere in the graph and
once for no node at all; a frame carrying a meaning-join defect beside a
canonical-order defect; and two defective frames.

## Expected Results

The eligibility check admits exactly the four `creates`/`deletes` triples and
refuses every other `creates`/`deletes` triple; the fixture, with `process` in
`creates` and `object_type` in `deletes`, admits. An entry naming no declared
dependency, whether or not the digest resolves to a node elsewhere, refuses as
`missing_declaration`/`missing-name` at that entry; the meaning-join defect is
reported over the co-occurring order defect; and the lower node-id frame's own
defect is reported over the higher one's.

## Status

Implemented in `tests/it/checked_package_v2_frame_bodies.rs` and in the
eligibility enumeration test in
`crates/quire-contract-model/src/checked_package/v2/mod.rs`, one
representative authored case per rule. That enumeration still asserts the
two `modifies` triples FR-040 replaces (`relation`/`relationship` and
`model`/`field_declaration` as bare node keys); those assertions move to
TC-056 when the reader adopts FR-040.
