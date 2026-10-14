---
id: SR-10841
title: "Integrity review of IR-238 typed V2 envelope specification"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; FR-051 and TC-060 borrowed encoding contract"
review_set: subset
---

## Summary

Reviewed the new borrowed encoder signature against the actual owned private wire, canonical serializer, and QSL's borrowed wire. The specification leaves an expensive or duplicative implementation choice unresolved.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-051 | examined | The encoder SHALL accept typed members so the producer need not reproduce that wire type or serialize an untyped object map. |
| FR-051-AC-1 | examined | A default-feature external consumer supplies the seven typed members and explicit `Limits` through `quire_contract_model::encode_checked_package_v2`. |
| FR-051-AC-2 | examined | For an otherwise valid package, encoding with a ceiling equal to its canonical length returns the complete bytes; one byte lower returns `quire_canonical::Error` and no bytes. |
| FR-051-AC-3 | examined | Source inspection finds no package-id derivation, validation, secondary wire schema, or producer byte-limit constant in the new encoder. |
| TC-060 | examined | Compile an external default-feature consumer that supplies borrowed typed preimage, package identity, lock, graph, source map, capabilities and diagnostics. |
| FR-019 | context_only | The model-crate public inventory adds one encoder function and no public wire or builder. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The prescribed borrowed inputs cannot directly construct the existing owned `CheckedPackageWireV2`; implementation must clone potentially large graph and body values or duplicate the eight-field envelope in a borrowed serializer. Require a single authoritative private wire encoding path with an explicit no-clone/resource contract, and test or inspect it. | spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md:29 |

## Verdict

Caller-supplied package identity is sound for a byte-construction API because FR-038's reader remains the admission authority. Caller-supplied `quire_canonical::Limits` provides an explicit byte ceiling. The representation and resource obligation needs clarification before implementation.

## Dispositions

Reviewed fix commit 8d6789ce2fd77bc988107a2efc6ad248f5745200.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-051 now requires one private generic top-level declaration instantiated with owned reader, raw-term borrowed intake and typed borrowed encoder members; derived serialization and no clone are explicit. TC-451 inspects all three instantiations. A focused serde 1.0.228 and serde_json 1.0.151 `RawValue` probe compiled and ran with that generic derived Serialize/Deserialize shape, borrowed raw-term intake, owned decode and borrowed typed encode. |
