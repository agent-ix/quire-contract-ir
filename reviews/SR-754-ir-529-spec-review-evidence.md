---
id: SR-754
title: "evidence review of PR 251 (IR-529 artifact references)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; FR-038-AC-46 through FR-038-AC-64 verification column; TC-048 Artifact references procedure; spec/checked_package/matrix/tests.md FR-038 and TC-048 rows; FR-019-AC-5 and TC-058 as context"
review_set: subset
---
# SR-754: evidence review of PR 251

## Summary

Ticket: IR-529. Every new AC is verified by `Test (TC-048)`, and TC-048 gains an
"Artifact references" procedure covering each AC. For the wire-level ACs (46 to 59 and
63), a Test through the package reader is the right method. The matrix rows mark AC-46
to AC-64 🚧 planned (IR-530), with the reason stated.

`quire coverage --strict` does not list FR-038-AC-46 to AC-64 as unbacked, because the
TC-048 row's existing symbols back them at row level. This is the same pre-existing
mechanism as AC-45, and the 🚧 marker is what keeps it honest.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038-AC-60, AC-61 and AC-62 are public-API shape claims (member sets, absence of `CheckedRevision`) under a strict-reader property TC. FR-019-AC-5 and TC-058 already own the public-item inventory, which fails on an added or missing item, so AC-61 duplicates it via the FR-019 table edit. Inspection, or a TC-058 trace, fits better than TC-048. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:854-856; spec/model/functional/FR-019-rust-library-interface.md:152 |
| FND-002 | low | FR-038-AC-64 (a catalog `law_roles` entry with an extra member fails the catalog read, and that is not a package refusal) is to be shown by TC-048 "reading the operation catalog with one entry carrying `revision`". The catalog is a `const` from quire-verification-contracts, parsed once into a `OnceLock` that panics on invalid bytes. There is no entry point over other bytes today. TC-048 should say the evidence is a unit test of the catalog parser over supplied bytes, and what the failure is: a parse error, not a panic. | crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs:39-40,156-160; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:141-143 |

## Verdict

The evidence is adequate. Two low findings, on where the API-shape ACs and AC-64 are
verified.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | FR-019-AC-5 now also claims member sets: CheckedArtifactRef has exactly authority and identity, CheckedSourceRef four members, CheckedArtifactLocator three, and CheckedRevision is absent. Its only verifier, TC-058, was not updated. Its procedure, expected results and cases row inventory item names alone and never check an item's members. Absence of CheckedRevision is covered by the inventory; the member sets are verified by nothing. Add a member-set step to TC-058 (procedure, expected results and the cases row in spec/model/matrix/tests.md). | spec/model/matrix/TC-058-model-crate-public-interface.md:18-36 |

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f (delta from 0b106a64ea261a86c0f2f64eb9ab8f31f546d39d; base origin/main e80ea70ab8874676de47ac1b7fb439497ced7991 unchanged).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |
| FND-002 | fixed | 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f |

Round 2, reviewed at agent-ix/quire-contract-ir@a4811eab16f0f89208c20ca172291d31344d8f0c (delta from 9f76810d4bfd9b6ac4590f49d15bc2484fdc6a5f; base unchanged).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-003 | fixed | a4811eab16f0f89208c20ca172291d31344d8f0c |
