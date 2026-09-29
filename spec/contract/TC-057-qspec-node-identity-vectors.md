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
`proposals/checked-package-v2/node-identity-vectors.json`, read at run time
from the checkout `QSPEC_DIR` names. Nothing of QSpec is copied into this
repository, so the digests the test compares against are computed by another
producer and the test cannot become a tautology.

## Test Procedure

Under `make qspec-vectors`, read every vector's `preimage` and `sha256`,
re-derive each digest through this crate's nominal and application preimage
encoders, and build one package holding every vector node and read it through
the V2 reader. Run once with `QSPEC_DIR` unset.

## Expected Results

Every re-derived digest equals its recorded `sha256` and the package admits.
With `QSPEC_DIR` unset the test skips, and `make qspec-vectors` fails on the
skip rather than passing.

## Status

Planned. The `function-call` vector already runs through the reader in
`tests/it/checked_package_v2_dependency_reference.rs`; the full vector set
does not.
