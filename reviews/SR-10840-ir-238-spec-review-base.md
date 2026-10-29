---
id: SR-10840
title: "Base review of IR-238 typed V2 envelope specification"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; FR-051, TC-060, FR-019 public inventory"
review_set: subset
---

## Summary

Reviewed the three-file diff against the IR wire, QSL producer, QSpec FR-322/FR-272, and local matrix and index. The API can carry caller-minted identity and caller-selected limits without adding identity semantics. Three registration and verification defects remain.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-051 | examined | When a producer has assembled the typed members of a complete checked-package V2 artifact and minted its semantic package identity, `quire_contract_model` SHALL encode the closed envelope with its private V2 wire type. |
| FR-051-AC-1 | examined | A default-feature external consumer supplies the seven typed members and explicit `Limits` through `quire_contract_model::encode_checked_package_v2`; its bytes have exactly the eight FR-322 envelope members, including IR's fixed `contract_version`, and equal the canonical bytes of the same private wire assembled in an IR unit test. |
| FR-051-AC-2 | examined | For an otherwise valid package, encoding with a ceiling equal to its canonical length returns the complete bytes; one byte lower returns `quire_canonical::Error` and no bytes. |
| FR-051-AC-3 | examined | An external producer mints `package_id` from the supplied preimage, calls the encoder, and passes the result to `CheckedPackageV2::read`; the reader admits the valid fixture. |
| TC-060 | examined | Verify FR-051-AC-1 through FR-051-AC-3 through the public model-crate interface, canonical encoder, and existing V2 reader. |
| FR-019 | examined | The public items table adds only `encode_checked_package_v2` to the V2 reader row. |
| QSpec FR-322 and FR-272 | context_only | FR-322 owns the closed checked-package envelope and identity; FR-272 owns typed diagnostic causes. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-060 is reserved by active IR-759 and already used in its reviewed draft; assign a fresh IR test-case ID and update all FR-051 references. | spec/checked_package/matrix/TC-060-typed-v2-envelope-encoding.md:2 |
| FND-002 | medium | FR-051 and its test case are absent from `spec/checked_package/matrix/tests.md`, and FR-051 is absent from `spec/spec.md`; add planned status and index registration so coverage readers can find them. | spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md:17 |
| FND-003 | medium | TC-060 tests a synthetic external consumer but never requires QSL to replace its local `WireV2`; the duplicate envelope can remain after every AC passes. Add a QSL adoption acceptance check or identify the separately tracked consumer gate. | spec/checked_package/matrix/TC-060-typed-v2-envelope-encoding.md:19 |
| FND-004 | medium | The reader round-trip controls change identity fields but never mutate the envelope shape (missing, extra, or duplicate member), so the malformed-envelope refusal requested by IR-238 is not exercised. Add one shape mutation and assert FR-038's typed refusal. | spec/checked_package/matrix/TC-060-typed-v2-envelope-encoding.md:28 |

## Verdict

Changes required before the specification can serve as the handoff gate. The external build boundary is representable as an integration crate or compile probe. Exact and one-byte-low ceilings and independent caller-minted digest tests discriminate the intended byte and identity behavior. The current live IR-759 ticket is in Specify, and its Linear comment reserves TC-060; this external text was checked against its reviewed draft, not treated as authority over this review.

## Dispositions

Reviewed fix. Each outcome refers to the original finding above.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The TC file, frontmatter and all FR-051 references now use TC-451. No other local IR artifact uses TC-451; the active IR-759 draft retains its TC-060 reservation. |
| FND-002 | fixed | `spec/checked_package/matrix/tests.md` now carries planned FR-051 and TC-451 rows, and `spec/spec.md` links both artifacts. |
| FND-003 | fixed | FR-051 and TC-451 explicitly assign actual QSL emitter adoption and local `WireV2` removal to QSL-766. Linear's structured IR-238 `blocks` QSL-766 relation confirms the dependency, and QSL-766 has separate producer acceptance criteria. |
| FND-004 | fixed | FR-051-AC-3 and TC-451 require missing, unknown and duplicate top-level member mutations with FR-038's distinct typed refusals. |
