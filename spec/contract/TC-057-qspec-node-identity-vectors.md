---
id: TC-057
title: "QSpec's published node-identity vectors re-derive through the V2 reader"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---
# TC-057: QSpec's published node-identity vectors re-derive through the V2 reader

## Description

Verify FR-038-AC-40 against the independent oracle QSpec publishes,
the `vectors`, `operation_vectors`, `invalid_mutations` and
`operation_mutations` arrays of
`proposals/checked-package-v2/node-identity-vectors.json`, read at run time
from the checkout `QSPEC_DIR` names. Nothing of QSpec is copied into this
repository, so the digests the test compares against are computed by another
producer and the test cannot become a tautology.

## Test Procedure

Under `make qspec-vectors`, read the file's arrays from `QSPEC_DIR`:

- `vectors` (nominal preimages) and `operation_vectors` (application-node
  preimages): re-derive each `sha256` through this crate's nominal and
  application preimage encoders. Build one package per domain package version
  the nominal vectors' model owners name, since one lock selects one version
  of a domain package, and read each through the V2 reader. Check the
  operation vectors `positive-operation-identities.json` carries against that
  fixture, add the rest to it as nodes with their laws selected, and read it.
- `invalid_mutations` and `operation_mutations`: apply each entry's RFC 6902
  `patch` to its `base` vector's preimage, key the node under its
  `retained_sha256`, place it in a package and read it; for a `stale_key`
  operation mutation, also confirm that the patched preimage's digest equals
  the `rekeyed_as` vector's `sha256`. Any other operation mutation is first
  read under its retained key and shown to refuse, then keyed under its
  patched preimage's own digest and read, so a stale key does not pre-empt
  the refusal the entry expects.

Count the entries replayed from each array. Run once with `QSPEC_DIR` unset.
The `frame_mutations` array is replayed by TC-056.

## Expected Results

Every re-derived digest equals its recorded `sha256` and the package admits.
Every mutation refuses with its `expected_code` and, where the entry gives
one, its `expected_cause`, and no mutated package is exposed. The count
replayed from each array equals the count published in it. With `QSPEC_DIR`
unset the test skips, and `make qspec-vectors` fails on the skip rather than
passing.

## Status

Implemented in `tests/it/checked_package_v2_node_identity_vectors.rs`
(`qspec_node_identity_vectors`), run by `make qspec-vectors`. It replays 17
`vectors`, 21 `operation_vectors`, 13 `invalid_mutations` and 24
`operation_mutations`.
