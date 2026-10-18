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

The public `CheckedPackageIdentityPreimageRefV2<'a>` view borrows the preimage's
version, edition, profile selections, definition selections, model selections,
required features, dependency selections, and ordered identity projection. Its
projection is a borrowed slice of `CheckedSemanticNodeV2` values: encoding
selects the same identity fields as `CheckedNodeProjectionV2` and borrows each
node's `body` without copying it. Source occurrences are excluded from the
projection. The view can select nodes independently of the graph supplied to
the envelope, so an inconsistent preimage remains representable for reader
refusal tests. The owned `CheckedPackageIdentityPreimageV2` and
`CheckedNodeProjectionV2` remain reader-facing types.

The public function `encode_checked_package_v2` takes
`&CheckedPackageIdentityPreimageRefV2<'_>`, `&CheckedSemanticId` (the producer's
`package_id`), `&CheckedPackageLockV2`, `&CheckedSemanticGraphV2`,
`&[CheckedSourceMapEntry]`, `&[CheckedCapability]`, and
`&CheckedDiagnosticsV2`, followed by an explicit `quire_canonical::Limits`.
These are the seven non-version envelope members of the private
`CheckedPackageWireV2`, in its existing declaration order. The function takes
no source text, compiler state, model document, admitted package, or closure.
This signature replaces the owned-preimage parameter; there is no overload or
compatibility encoding path.

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
  types; they are not an output envelope. The encoder SHALL borrow the graph,
  preimage and other supplied members without cloning them. The envelope
  declaration SHALL generate the serde reader/intake struct and canonical
  `Encode` field dispatch from its one top-level field list. The encoder SHALL
  dispatch fixed-depth members through `Writer::serialize`. The encoder SHALL
  dispatch members containing arbitrary-depth JSON terms through their existing
  iterative `Encode` implementations. Each supplied member appears exactly once
  under its FR-322 name; no extra member or independently maintained serializer
  or field list appears.
- `CheckedPackageIdentityPreimageRefV2` SHALL encode the same field names,
  field order, optional-field omission, and canonical body bytes as an owned
  `CheckedPackageIdentityPreimageV2` made from the same selected nodes. Owned
  and borrowed projection encoding SHALL dispatch through one shared field
  encoder; the borrowed path SHALL NOT collect owned projections or clone node
  bodies. The model crate SHALL implement canonical `Encode` for the borrowed
  view so producers can hash that preimage and encode the complete envelope
  without allocating a second body per node.
- The function SHALL retain the caller's preimage and package identity as
  supplied. It shall not derive or rewrite `identity_preimage`, hash or replace
  `package_id`, reorder graph nodes or selections, synthesize diagnostics, or
  admit the result. Source compilation, the package identity algorithm, the
  canonical byte ceiling, and a reader round-trip remain producer decisions.
- If canonical encoding refuses any supplied value or reaches the caller's
  ceiling, the function SHALL return the encoder's typed error and no partial
  bytes. It shall not inspect display text to classify an error.

The replacement typed encode function and borrowed preimage view are the
producer-facing public surface. The existing private wire type remains private.
A public builder would add construction
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
| FR-051-AC-1 | A default-feature external consumer constructs `CheckedPackageIdentityPreimageRefV2` from borrowed selections and graph nodes, supplies it with the other six typed members and explicit `Limits` through `quire_contract_model::encode_checked_package_v2`, and receives exactly the eight FR-322 envelope members, including IR's fixed `contract_version`. The bytes equal the canonical bytes of the same private wire assembled in an IR unit test. The consumer cannot import `CheckedPackageWireV2` or the function from `quire_contract_ir`; the encoder's prior owned-preimage signature is absent. | Test (TC-451) |
| FR-051-AC-2 | For an otherwise valid package, encoding with a ceiling equal to its canonical length returns the complete bytes; one byte lower returns `quire_canonical::Error` and no bytes. A canonical-invalid typed value likewise returns the encoder error and no bytes. | Test (TC-451) |
| FR-051-AC-3 | An external producer mints `package_id` by canonically hashing the borrowed preimage view, calls the encoder, and passes the result to `CheckedPackageV2::read`; the reader admits the valid fixture. Changing only the supplied package digest or identity preimage leaves the corresponding bytes as supplied and lets the reader refuse according to FR-038. Removing a required envelope member, adding an unknown member, or duplicating a member yields FR-038's matching typed malformed-wire refusal. Source inspection finds one private generic top-level wire declaration instantiated for owned read, raw-term borrowed intake and typed borrowed encode; no clone of borrowed members, package-id derivation, admission, third envelope schema, or producer byte-limit constant is added. | Test (TC-451) |
| FR-051-AC-4 | For empty, nested, and deeply nested node bodies, and for present and absent optional projection fields, the borrowed view and equivalent owned preimage have byte-identical canonical encodings and identical typed `package_id` digests. The borrowed encoder's complete envelope equals a private-wire envelope containing the owned preimage under the same limits; source occurrences appear in the graph but not either identity projection. A ceiling equal to the envelope length succeeds, one byte less returns an error and no bytes, and a canonical-invalid body returns the same typed error on both paths. Source inspection finds one shared projection-field encoder and no retained owned projection or `Value::clone` on the borrowed encoding path. | Test (TC-451) |

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
