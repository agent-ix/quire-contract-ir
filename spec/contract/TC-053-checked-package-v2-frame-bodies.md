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

Verify FR-038-AC-12 through FR-038-AC-15: the closed frame-body entry
eligibility table, the `missing_declaration`/`invalid_model_binding` refusal
split and its cause, the meaning-join-over-order and member-then-digest
refusal precedence, and the ascending-node-id-digest visit order across
multiple frame nodes.

## Test Procedure

Enumerate every `(member, tag, form)` triple the closed `CheckedNodeTag::ALL`
family/form taxonomy can produce and assert each against the frame member's
own eligibility predicate, so the six eligible triples and every other triple
are both checked, not sampled. Reassign the published all-families fixture's
frame body so its two already-declared `object_type` and `process`
dependencies sit in `creates` and `deletes` respectively (the two eligible
triples the fixture's own body does not already exercise) and read the
package. Replay every vendored `node-identity-vectors.json` `frame_mutations`
vector — substituting its `dependencies`, `modifies`, `creates` and `deletes`
into the fixture's one frame node in place, splicing in a `second_frame`
node verbatim where the vector carries one — asserting the refused code,
cause and locus digest for each, and that the number of vectors replayed
equals the number published.

## Expected Results

The eligibility check admits exactly the six declared triples and refuses
every other triple; the published fixture, with `process` in `creates` and
`object_type` in `deletes`, still admits. Each vendored vector refuses with
its recorded code, cause and locus: an ineligible declared entry as
`invalid_model_binding`/`malformed-declaration`; an entry naming no declared
dependency, whether or not that digest resolves to a real node elsewhere, as
`missing_declaration`/`missing-name`; a member array out of ascending digest
order, with no meaning-join defect present, as `invalid_semantic_graph` with
no cause, located at the frame node; two meaning-join defects in different
members resolve by member order regardless of digest, and two in the same
member resolve by ascending digest; and the vector carrying two defective
frame nodes refuses at the lower-digest node's own defect, never reaching or
comparing the other frame's. All 26 published vectors are replayed.
