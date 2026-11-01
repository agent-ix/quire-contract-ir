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

When a producer has assembled the typed members of a complete checked-package V2 artifact, `quire_contract_model` SHALL encode the closed envelope with its private V2 wire type. The encoder SHALL accept typed members so the producer need not reproduce that wire type or serialize an untyped object map. This is a byte-construction API; the strict reader alone decides admission.

## Inputs

The public `encode_checked_package_v2` function takes a package `name`, `&CheckedPackageLockV2`, `&CheckedSemanticGraphV2`, `CheckedPackageSourceCorrespondenceRef` (the source-map slice and optional refinement-step rows), `Option<&CheckedTermDataV2>` for the producer-captured equality, protocol-location and dispatch rows, `&[CheckedCapability]`, `&CheckedDiagnosticsV2`, and explicit `quire_canonical::Limits`. The package name is its identity; no package id, preimage, projection, digest or seal is accepted or constructed. The graph contains structural `CheckedNodeId` values `{package, path, ordinal}`. The function accepts no source text, compiler state, model document, admitted package or closure.

## Outputs

`Result<Vec<u8>, quire_canonical::Error>`: complete canonical V2 bytes or the canonical encoder's typed error with no bytes. The function has no public envelope value or builder result.

## Behavior

- The function SHALL instantiate one crate-private generic envelope declaration with `contract_version` fixed to `quire.checked-package/v2` and the six required non-version members, then encode it through `quire_canonical` under the caller's limits. It emits `refinement_step_rows` exactly when the source-correspondence input supplies it and `checked_term_data` exactly when the separate typed input is `Some`. A `Some` value with three empty arrays remains present; `None` omits the member. A present `dispatch_tables` row with an empty `candidates` array remains present and retains that producer-captured empty candidate set. The reader uses the same declaration instantiated with owned member types; intake uses it with borrowed raw-term member types. One top-level field list generates serde reader/intake structure and canonical `Encode` dispatch. Fixed-depth members use `Writer::serialize`; members holding arbitrary-depth terms use their iterative `Encode` implementations. No separate envelope schema or serializer is retained.
- The encoder SHALL borrow the graph, lock and other supplied members without cloning them. It SHALL preserve the supplied node order, complete bodies, source occurrences, diagnostics, capability values and sidecar array and candidate order. It SHALL never mint or compare an identity derived from the contents.
- A canonical-invalid typed value SHALL return the encoder's typed error and no partial bytes. The strict reader SHALL refuse an empty name under FR-038. An exact-length limit admits complete encoding; one byte less returns the typed limit error and no bytes.
- The reader SHALL validate the resulting package independently under FR-038 and QSpec FR-322, including sidecar joins and duplicate keys.
- The encoder SHALL NOT silently normalize, sort, rekey or repair graph nodes, selections or producer-captured sidecar facts. Successfully writing bytes does not claim admission.

The producer-facing API remains the one typed encode function. `CheckedPackageV2` remains a reader-admitted value, never constructed by this function. QSL's adoption of this API is a separate producer task; an IR synthetic external-consumer test does not claim QSL adoption.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-051-AC-1 | A default-feature external consumer calls `quire_contract_model::encode_checked_package_v2` with the six typed required non-version members, an optional borrowed `CheckedTermDataV2` and explicit limits, and receives exactly `contract_version`, `name`, `lock`, `semantic_graph`, `source_map`, `capability_report` and `diagnostics`, plus `refinement_step_rows` and `checked_term_data` exactly when their separate inputs are present. The bytes equal those of the same private wire assembled in an IR unit test; the private wire cannot be imported from `quire_contract_ir` or the model crate. | Test (TC-451) |
| FR-051-AC-2 | For an otherwise valid package, a limit equal to its canonical length returns complete bytes and one byte lower returns `quire_canonical::Error` and no bytes. A canonical-invalid typed value returns a typed encoder error and no bytes; bytes with an empty name are refused by the strict reader. | Test (TC-451) |
| FR-051-AC-3 | An external producer supplies a package name and structural node ids, encodes the envelope and passes it to `CheckedPackageV2::read`; the reader admits the valid fixture. Removing a required member, adding an unknown member or duplicating a member yields FR-038's exact typed refusal. Source inspection finds one private generic top-level wire declaration used by owned read, raw-term intake and typed borrowed encode; no package-id derivation, admission, second envelope schema or producer byte-limit constant is added. | Test (TC-451) |
| FR-051-AC-4 | Empty, nested and deeply nested node bodies and present or absent optional fields are serialized once and retain their full values and node order. The optional checked-term-data member is omitted for `None` and present with all three empty arrays for `Some(empty)`; neither case is normalized into the other. Exact-length and one-byte-short limits give their specified outcomes. No owned graph projection, sidecar clone or body clone is introduced on the borrowed encode path. | Test |
| FR-051-AC-5 | Given shared typed checked-term-data inputs built from QSpec FR-322's equality, protocol-location and dispatch vectors, the public encoder emits all three arrays with canonical bytes matching the private envelope at the same limit. It preserves each array's producer order and a present dispatch row with an empty candidate set. At the complete sidecar envelope's exact byte limit it returns those bytes; one byte below it returns only the canonical limit error and no bytes. Neither writer success nor this writer criterion claims that the strict reader admits every sidecar section before its owning read implementation and corrected producer wire land. | Test |

## Dependencies

QSpec `ix://agent-ix/quire-specification/FR-322` owns the envelope shape and structural identity semantics. FR-038 owns the strict reader and private wire; FR-019 owns the model-crate public inventory. QSpec FR-272 owns typed diagnostic causes. QSL owns adoption in its emitter and its emitted-package round-trip evidence.
