---
id: TC-451
title: "Typed checked-package V2 envelope encoding"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: verifies
---
# TC-451: Typed checked-package V2 envelope encoding

## Description

Verify FR-051-AC-1 through FR-051-AC-4 through the public model-crate
interface, canonical encoder, and existing V2 reader. Its IR-focused controls
for AC-1 through AC-3 and the borrowed-view AC-4 are implemented in IR.
QSL producer adoption remains separate.

## Test Procedure

Compile an external default-feature consumer that constructs
`CheckedPackageIdentityPreimageRefV2` over borrowed selections and graph nodes
and supplies that view, package identity, lock, graph, source map, capabilities and diagnostics
to `encode_checked_package_v2` with explicit `quire_canonical::Limits`.
Compare its output with the canonical encoding of IR's private wire in a unit
test, and inspect the decoded top-level member set and fixed version. Compile
negative probes importing `CheckedPackageWireV2` from the model crate and the
encoder from the root crate, and check that its old owned-preimage signature no
longer compiles. Check the exact byte ceiling and one byte below;
pass a typed value containing a canonical-invalid number. None of the failing
calls may return bytes.

Mint the package identity independently from the borrowed preimage in the external
consumer and read the resulting valid bytes through `CheckedPackageV2::read`
with its required evidence. Then alter only the supplied digest, and separately
alter only the preimage, encode each, and assert the reader's typed refusal.
Mutate the valid encoded bytes once each by removing a required top-level
member, adding an unknown top-level member, and duplicating a top-level member;
assert the reader's typed missing-, unknown-, and duplicate-member refusal,
with no admitted package. Inspect the private generic envelope declaration:
its one top-level field list must serve the owned reader, raw-term borrowed
intake and typed borrowed encoder, generating serde decode and canonical
`Encode` dispatch from the same declaration. Check that fixed-depth members
use `Writer::serialize` and arbitrary-depth terms use their existing iterative
`Encode` implementations. Encode a deeply nested diagnostic detail on a small
stack to guard that boundary. The intake's nested raw-term types remain
intake-only. Find no clone of supplied members, identity derivation, admission,
third envelope schema or producer byte-limit constant. Compare the default-feature public export set
with [FR-019](../../model/functional/FR-019-rust-library-interface.md)'s
Public items table.

For one empty projection, one nested body, and one body nested deeply enough
to exercise the iterative encoder on a small stack, encode the borrowed view
and an equivalent owned preimage and compare their complete canonical bytes
and typed `package_id` digests. Compare the borrowed encoder's complete envelope
with the private wire holding the owned preimage under the same `Limits`. Cover
present and absent recursion group,
nominal preimage, declaration, and owner fields; put source occurrences on a
graph node and verify they are absent from both identity projections. Compare
the exact-length and one-byte-short ceilings and a canonical-invalid body on
both paths; neither refusal returns partial bytes. Inspect the shared
projection-field encoder and confirm the borrowed path neither clones a body
nor retains an owned projection. The integrated producer RSS reduction remains
PLANNED under NFR-006 and TC-455 until a QSL candidate is measured.

## Expected Results

The output is the exact closed FR-322 V2 envelope, the public and negative
compile probes enforce the intended boundary, canonical refusals return no
bytes, and only the matching preimage and digest admit. The three malformed
envelope mutations refuse with their distinct typed causes. The public inventory
contains the replacement function and borrowed preimage view, with no public
wire or builder type. Owned and borrowed preimages produce identical canonical
bytes and identities for the selected cases. QSL producer adoption and the
integrated memory result are separate and are not counted as passing from
these IR controls.
