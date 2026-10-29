---
id: SR-12720
title: IR-875 PR 464 code-review review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md; spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md; spec/model/functional/FR-019-rust-library-interface.md; spec/checked_package/matrix/tests.md; tests/it/checked_package_v2_envelope.rs"
review_set: subset
---

## Summary

Ticket IR-875. Review of PR #464. Diff-scoped document and code-alignment review found two cross-document conformance defects.

## Examined units

- **FR-051-AC-1** (`examined`, `spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md`): A default-feature external consumer constructs `CheckedPackageIdentityPreimageRefV2` from borrowed selections and graph nodes, supplies it with the other six typed members and explicit `Limits` through `quire_contract_model::encode_checked_package_v2`, and receives exactly the eight FR-322 envelope members, including IR's fixed `contract_version`. The bytes equal the canonical bytes of the same private wire assembled in an IR unit test. The consumer cannot import `CheckedPackageWireV2` or the function from `quire_contract_ir`; the encoder's prior owned-preimage signature is absent.
- **FR-051-AC-2** (`examined`, `spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md`): For an otherwise valid package, encoding with a ceiling equal to its canonical length returns the complete bytes; one byte lower returns `quire_canonical::Error` and no bytes. A canonical-invalid typed value likewise returns the encoder error and no bytes.
- **FR-051-AC-3** (`examined`, `spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md`): An external producer mints `package_id` by canonically hashing the borrowed preimage view, calls the encoder, and passes the result to `CheckedPackageV2::read`; the reader admits the valid fixture. Changing only the supplied package digest or identity preimage leaves the corresponding bytes as supplied and lets the reader refuse according to FR-038. Removing a required envelope member, adding an unknown member, or duplicating a member yields FR-038's matching typed malformed-wire refusal. Source inspection finds one private generic top-level wire declaration instantiated for owned read, raw-te
- **FR-051-AC-4** (`examined`, `spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md`): For empty, nested, and deeply nested node bodies, and for present and absent optional projection fields, the borrowed view and equivalent owned preimage have byte-identical canonical encodings and identical typed `package_id` digests. The borrowed encoder's complete envelope equals a private-wire envelope containing the owned preimage under the same limits; source occurrences appear in the graph but not either identity projection. A ceiling equal to the envelope length succeeds, one byte less returns an error and no bytes, and a canonical-invalid body returns the same typed error on both paths
- **TC-451** (`examined`, `spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md`): Verify FR-051-AC-1 through FR-051-AC-4 through the public model-crate
interface, canonical encoder, and existing V2 reader. Its IR-focused controls
for AC-1 through AC-3 are implemented; the borrowed-view AC-4 is planned for
IR-875 slice 2. QSL producer adoption remains separate.
- **FR-019-AC-5** (`examined`, `spec/model/functional/FR-019-rust-library-interface.md`): The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists, checked by a public-item inventory that fails on an added or missing item; no item in the table is reachable through a `quire_contract_ir` path.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-051 adds the public CheckedPackageIdentityPreimageRefV2 view, but FR-019’s exact Public items table still lists only encode_checked_package_v2 for the V2 encoder. A correct implementation would violate FR-019-AC-5 and TC-451’s inventory comparison until the table includes the view. | spec/model/functional/FR-019-rust-library-interface.md:184,218 |
| FND-002 | high | TC-451 says AC-1 through AC-3 are implemented, and checked_package/matrix/tests.md still marks FR-051/TC-451 complete and lists only AC-1..3. The changed AC-1 and AC-3 now require the borrowed view, while current code/tests accept and hash only the owned preimage; AC-4 is missing from the matrix row. Mark changed clauses planned and update that row so the existing trace tags cannot imply current conformance. | spec/checked_package/matrix/TC-451-typed-v2-envelope-encoding.md:15-18 |

## Coverage

`quire validate` passes both changed documents. Review-head `quire coverage --scope . --strict` reports 533/603 backed, 40 unbacked and 0 contradicted; AC-4 remains planned in this spec-only PR. Existing AC-1/3 tags were checked against their revised wording.

## Verdict

FAIL: address the findings above before merging.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | FR-019 now lists the borrowed view as planned, but FR-019-AC-5 still requires the default-feature exports to equal the Public items table exactly and rejects missing items. The view is absent on this spec-only head, so the table and AC remain contradictory. |
| FND-002 | fixed | TC-451 and both matrix rows now distinguish owned controls from planned borrowed-view obligations. |
| FND-001 | fixed | FR-019-AC-5 and TC-058 now compare the exact implemented Public items table entries and explicitly exclude planned entries until code lands; the borrowed view remains visibly planned in the table. |
