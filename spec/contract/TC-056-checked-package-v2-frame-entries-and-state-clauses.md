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

Verify FR-040-AC-1 through FR-040-AC-13: the V2 reader admits QSpec FR-340's
`FrameModifiesEntry` values, FR-342's operation anchor body, FR-341's state
clause and parameter bodies and exactly the fifteen `model` forms, and
refuses every other shape with the code, cause and locus those requirements
fix, in their reader order.

## Test Procedure

Build, from this repository's own public vocabulary, checked packages
holding frames, operation anchors, an invariant, a precondition and a
postcondition, and parameter nodes. Read each and confirm it admits. Then
apply each single mutation the FR-040 criteria name and read each mutated
package: entry shapes and kinds, eligibility per member, misordered
`modifies`, a meaning-join defect with an
order defect, two defective frames, frame `semantic_type`, each removed
`model` form, each anchor binding and join, duplicate anchors, each clause
anchor, parameter and signature case, nested and misplaced clause
applications, a non-Boolean condition, each parameter body defect, each
occurrence role, and pairs of defects across the frame, state and operation
steps. Field-name, anchor-operation and clause-signature resolution
against a domain package (own, inherited, redefined, undeclared, another
member kind's name, ambiguous, unselected version, systems context, and the
`self`/result/parameter types) replay QSpec TC-280's `frame_field_cases`,
`anchor_cases` and `clause_signature_cases` from
`model-member-type-vectors.json` through the reader's frame and state steps
rather than authoring them again. Under `make qspec-vectors`, read QSpec's published V2 fixtures that
carry these nodes from `QSPEC_DIR`, and replay every entry of the
`frame_mutations` array of `node-identity-vectors.json` in place on the
frame node of QSpec's `fixtures/positive-all-families.json`, as QSpec's
`proposals/checked-package-v2/README.md` describes, counting the entries
replayed.

## Expected Results

The unmutated package and QSpec's published fixtures admit. Each authored
mutation returns exactly the code, cause and RFC 6901 locus its criterion
names and no package; each `frame_mutations` entry refuses with its
`expected_code`, `expected_cause` and `expected_locus_digest`, and the count
replayed equals the count published; a package with defects in two steps reports the earlier step's
defect.

## Status

Implemented. Authored cases: `tests/it/checked_package_v2_frame_entries.rs`
(FR-040-AC-1, 2, 4 to 8, 10 to 12) and the entry eligibility tests in
`crates/quire-contract-model/src/checked_package/v2/frame.rs`. QSpec TC-280
replays: `crates/quire-contract-model/src/checked_package/v2/operations/model_member_vectors.rs`
(FR-040-AC-3, 7, 9). QSpec fixtures and the 30 `frame_mutations`:
`qspec_frame_mutations_and_published_fixtures` (FR-040-AC-13). Field-name
resolution for a `record_value_type` declaring node is pending a QSpec
ruling and is not tested.
