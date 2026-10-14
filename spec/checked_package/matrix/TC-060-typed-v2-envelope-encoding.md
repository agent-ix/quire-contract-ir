---
id: TC-060
title: "Typed checked-package V2 envelope encoding"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: verifies
---
# TC-060: Typed checked-package V2 envelope encoding

## Description

Verify FR-051-AC-1 through FR-051-AC-3 through the public model-crate
interface, canonical encoder, and existing V2 reader. This case is planned;
no implementation or test result is claimed by the specification change.

## Test Procedure

Compile an external default-feature consumer that supplies borrowed typed
preimage, package identity, lock, graph, source map, capabilities and diagnostics
to `encode_checked_package_v2` with explicit `quire_canonical::Limits`.
Compare its output with the canonical encoding of IR's private wire in a unit
test, and inspect the decoded top-level member set and fixed version. Compile
negative probes importing `CheckedPackageWireV2` from the model crate and the
encoder from the root crate. Check the exact byte ceiling and one byte below;
pass a typed value containing a canonical-invalid number. None of the failing
calls may return bytes.

Mint the package identity independently from the preimage in the external
consumer and read the resulting valid bytes through `CheckedPackageV2::read`
with its required evidence. Then alter only the supplied digest, and separately
alter only the preimage, encode each, and assert the reader's typed refusal.
Inspect the new encoder source for identity derivation, admission, another wire
shape, or a byte-limit constant. Compare the default-feature public export set
with [FR-019](../../model/functional/FR-019-rust-library-interface.md)'s
Public items table.

## Expected Results

The output is the exact closed FR-322 V2 envelope, the public and negative
compile probes enforce the intended boundary, canonical refusals return no
bytes, and only the matching preimage and digest admit. The public inventory
contains exactly one new function and no wire or builder type.
