---
id: SR-12729
title: IR-875 PR 464 code-slice ears-conformance review
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md; spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md; spec/model/functional/FR-019-rust-library-interface.md; spec/model/matrix/TC-058-model-crate-public-interface.md; crates/quire-contract-model/src/checked_package/v2/encode.rs; crates/quire-contract-model/src/checked_package/v2/mod.rs; crates/quire-contract-model/src/lib.rs; tests/it/checked_package_v2_envelope.rs; tests/it/checked_package_v2_canonical_encoding.rs"
review_set: subset
---

## Summary

Ticket IR-875. Diff-scoped non-author review of the combined PR. No new normative SHALL statement is introduced by the code-slice status edits; the amended requirement remains actor, action and outcome specific.

## Examined units

- **FR-051** (`examined`, `spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md`): The public `CheckedPackageIdentityPreimageRefV2<'a>` view borrows the preimage's
version, edition, profile selections, definition selections, model selections,
required features, dependency selections, and ordered identity projection. Its
projection is a borrowed slice of `CheckedSemanticNodeV2` values: encoding
selects the same identity fields as `CheckedNodeProjectionV2` and borrows each
node's `body` without copying it. Source occurrences are excluded from the
projection. The view can select nodes independently of the graph supplied to
the envelope, so an inconsistent preimage remains representable for reader
refusal tests. The owned `CheckedPackageIdentityPreimageV2` and
`CheckedNodeProjectionV2` remain reader-facing types.
- **FR-051-AC-1** (`examined`, `spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md`): A default-feature external consumer constructs `CheckedPackageIdentityPreimageRefV2` from borrowed selections and graph nodes, supplies it with the other six typed members and explicit `Limits` through `quire_contract_model::encode_checked_package_v2`, and receives exactly the eight FR-322 envelope members, including IR's fixed `contract_version`. The bytes equal the canonical bytes of the same private wire assembled in an IR unit test. The consumer cannot import `CheckedPackageWireV2` or the function from `quire_contract_ir`; the encoder's prior owned-preimage signature is absent.
- **FR-051-AC-2** (`examined`, `spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md`): For an otherwise valid package, encoding with a ceiling equal to its canonical length returns the complete bytes; one byte lower returns `quire_canonical::Error` and no bytes. A canonical-invalid typed value likewise returns the encoder error and no bytes.
- **FR-051-AC-3** (`examined`, `spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md`): An external producer mints `package_id` by canonically hashing the borrowed preimage view, calls the encoder, and passes the result to `CheckedPackageV2::read`; the reader admits the valid fixture. Changing only the supplied package digest or identity preimage leaves the corresponding bytes as supplied and lets the reader refuse according to FR-038. Removing a required envelope member, adding an unknown member, or duplicating a member yields FR-038's matching typed malformed-wire refusal. Source inspection finds one private generic top-level wire declaration instantiated for owned read, raw-term borrowed intake and typed borrowed encode; no clone of borrowed members, package-id derivation, admission, third envelope schema, or producer byte-limit constant is added.
- **FR-051-AC-4** (`examined`, `spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md`): For empty, nested, and deeply nested node bodies, and for present and absent optional projection fields, the borrowed view and equivalent owned preimage have byte-identical canonical encodings and identical typed `package_id` digests. The borrowed encoder's complete envelope equals a private-wire envelope containing the owned preimage under the same limits; source occurrences appear in the graph but not either identity projection. A ceiling equal to the envelope length succeeds, one byte less returns an error and no bytes, and a canonical-invalid body returns the same typed error on both paths. Source inspection finds one shared projection-field encoder and no retained owned projection or `Value::clone` on the borrowed encoding path.
- **FR-019-AC-5** (`examined`, `spec/model/functional/FR-019-rust-library-interface.md`): The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists at the tested revision. The public-item inventory fails on an added or missing item; no item in the table is reachable through a `quire_contract_ir` path.
- **TC-451** (`examined`, `spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md`): Verify FR-051-AC-1 through FR-051-AC-4 through the public model-crate
interface, canonical encoder, and existing V2 reader. Its IR-focused controls
for AC-1 through AC-3 and the borrowed-view AC-4 are implemented in IR.
QSL producer adoption remains separate.
- **NFR-006** (`context_only`, `spec/checked_package/non-functional/NFR-006-v2-producer-memory.md`): ## Statement

When a producer encodes a `quire.checked-package/v2` artifact, the Contract IR
identity-projection encoding path SHALL permit it to borrow each semantic graph
node body without retaining a second owned `serde_json::Value` body per node.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

`quire coverage --scope . --strict` reports 534/603 backed, 40 inherited unbacked and 0 contradicted. FR-051-AC-1 through AC-4 are tagged. Plan completion: not assessed. Independent focused Cargo test: queued on the shared heavy-build lock at review authoring time.

## Verdict

PASS for this diff-scoped method. The separate evidence findings in SR-12710 and SR-12714 still require correction.
