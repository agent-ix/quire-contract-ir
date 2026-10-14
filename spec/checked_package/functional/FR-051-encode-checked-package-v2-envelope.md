---
id: FR-051
title: "Encode the typed checked-package V2 envelope"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-specification/FR-272
    type: references
---
# FR-051: Encode the typed checked-package V2 envelope

## Description

When a producer has assembled the typed members of a complete checked-package V2 artifact and minted its semantic package identity, `quire_contract_model` SHALL encode the closed envelope with its private V2 wire type.
The encoder SHALL accept typed members so the producer need not reproduce that wire type or serialize an untyped object map.
This is a byte-construction API, not an admission or identity-derivation API.

## Inputs

The public function `encode_checked_package_v2` takes borrowed
`&CheckedPackageIdentityPreimageV2`, `&CheckedSemanticId` (the producer's
`package_id`), `&CheckedPackageLockV2`, `&CheckedSemanticGraphV2`,
`&[CheckedSourceMapEntry]`, `&[CheckedCapability]`, and
`&CheckedDiagnosticsV2`, followed by an explicit `quire_canonical::Limits`.
These are the seven non-version envelope members of the private
`CheckedPackageWireV2`, in its existing declaration order. The function takes
no source text, compiler state, model document, admitted package, or closure.

## Outputs

`Result<Vec<u8>, quire_canonical::Error>`: complete canonical V2 bytes or the
canonical encoder's typed error with no bytes. The function has no public
envelope value or builder result.

## Behavior

- The function SHALL instantiate one crate-private generic envelope
  declaration with borrowed typed member types,
  `contract_version` set by IR to `CHECKED_PACKAGE_V2`, and the seven supplied
  typed members, then SHALL encode it through `quire_canonical` under the
  caller's `Limits`. The reader SHALL use the same declaration instantiated with owned member types. The intake SHALL use it with its existing raw-term borrowed member types.
  The refactor shall replace the current separate top-level field lists in
  `v2/mod.rs` (`CheckedPackageWireV2`) and `v2/intake.rs` (`BorrowedWire`)
  with this one private declaration. The intake's nested `BorrowedPreimage`,
  `BorrowedGraph`, and `BorrowedDiagnostics` remain its delayed raw-term decode
  types; they are not an output envelope. That path SHALL neither clone the
  graph, preimage or other supplied members nor add a third independently
  maintained envelope serializer or field list. The one field declaration
  SHALL generate its serde reader/intake struct and its canonical
  `Encode` field dispatch. Fixed-depth members SHALL use
  `Writer::serialize`; members containing arbitrary-depth JSON terms SHALL
  use their existing iterative `Encode` implementations. Each supplied member
  shall appear exactly once under its FR-322 name; no extra member shall appear.
- The function SHALL retain the caller's preimage and package identity as
  supplied. It shall not derive or rewrite `identity_preimage`, hash or replace
  `package_id`, reorder graph nodes or selections, synthesize diagnostics, or
  admit the result. Source compilation, the package identity algorithm, the
  canonical byte ceiling, and a reader round-trip remain producer decisions.
- If canonical encoding refuses any supplied value or reaches the caller's
  ceiling, the function SHALL return the encoder's typed error and no partial
  bytes. It shall not inspect display text to classify an error.

The typed encode function is the entire new public surface. The existing
private wire type remains private. A public builder would add construction
states without removing any input or obligation from this single encoding call.
`CheckedPackageV2` remains a reader-admitted value and is not constructed by
this function. The reader in [FR-038](./FR-038-consume-checked-package-v2.md)
still decides whether the produced bytes are valid.

**Separate QSL producer adoption dependency (QSL-766).** That QSL-owned change to
`qsl-package/src/emit.rs` must replace its local `WireV2` with this function
and verify an actual QSL-emitted package through IR's reader. Passing this IR
requirement's synthetic external-consumer controls does not claim that QSL has
adopted the API or removed its local wire type.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-051-AC-1 | A default-feature external consumer supplies the seven typed members and explicit `Limits` through `quire_contract_model::encode_checked_package_v2`; its bytes have exactly the eight FR-322 envelope members, including IR's fixed `contract_version`, and equal the canonical bytes of the same private wire assembled in an IR unit test. The consumer cannot import `CheckedPackageWireV2` or the function from `quire_contract_ir`. | Test (TC-451) |
| FR-051-AC-2 | For an otherwise valid package, encoding with a ceiling equal to its canonical length returns the complete bytes; one byte lower returns `quire_canonical::Error` and no bytes. A canonical-invalid typed value likewise returns the encoder error and no bytes. | Test (TC-451) |
| FR-051-AC-3 | An external producer mints `package_id` from the supplied preimage, calls the encoder, and passes the result to `CheckedPackageV2::read`; the reader admits the valid fixture. Changing only the supplied package digest or identity preimage leaves the corresponding bytes as supplied and lets the reader refuse according to FR-038. Removing a required envelope member, adding an unknown member, or duplicating a member yields FR-038's matching typed malformed-wire refusal. Source inspection finds one private generic top-level wire declaration instantiated for owned read, raw-term borrowed intake and typed borrowed encode; no clone of borrowed members, package-id derivation, admission, third envelope schema, or producer byte-limit constant is added. | Test (TC-451) |

## Dependencies

The envelope shape and identity semantics are QSpec `ix://agent-ix/quire-specification/FR-322`.
[FR-038](./FR-038-consume-checked-package-v2.md) owns the private wire and
reader. [FR-019](../../model/functional/FR-019-rust-library-interface.md)
owns the model-crate public inventory. QSpec
`ix://agent-ix/quire-specification/FR-272` owns typed diagnostic causes;
this encoder only carries the already typed `CheckedDiagnosticsV2` member.
QSL producer adoption is the separate dependent QSL-766 change owned by the QSL
repository; its actual emitter and reader round-trip are outside this IR API
contract and its local acceptance evidence.
