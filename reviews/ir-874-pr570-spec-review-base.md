---
id: SR-567882
title: "IR-874 PR 570 FR-019 public API row review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-019-rust-library-interface.md:186; FR-019-AC-5; PR #570 "
review_set: subset
---

## Summary

Reviewed the added CheckedProtocolRefusalDetail entry in FR-019's V2 public API catalog against the model crate re-export and owning FR-038. No requirement statement or acceptance criterion changed.

## Verdict

**PASS** — The single catalog entry names the new exported type in the correct module row and preserves the existing API inventory criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- FR-019-AC-5 (examined): The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path.
- Public items table, V2 reader row (examined): `checked_package` (V2 reader) | types `CheckedCollectionKind`, `CheckedDeclaration`, `CheckedDependencySelection`, `CheckedDiagnosticCause`, `CheckedDiagnosticCode`, `CheckedDiagnosticStage`, `CheckedDiagnosticV2`, `CheckedDiagnosticsV2`, `CheckedDomainPackageRef`, `CheckedMemberType`, `CheckedModelField`, `CheckedModelFieldsError`, `CheckedModelObjectFields`, `CheckedNodeOwner`, `CheckedNodeProjectionV2`, `CheckedPackageIdentityPreimageV2`, `CheckedPackageLockV2`, `CheckedPackageV2`, `CheckedPackageV2ReadResult`, `CheckedProtocolRefusalDetail`, `CheckedSemanticGraphV2`, `CheckedSemanticNodeV2`; constants `CHECKED_PACKAGE_V2`, `DOMAIN_PACKAGE_DIGEST`, `PACKAGE_DOMAIN_V2` | FR-038, FR-040

## Coverage

The FR-019-AC-5 trace remains tagged by three public-interface tests (TC-058), with no bound run evidence in the local Quoin matrix. The changed document validates with Quire. No applicable AssuranceProfile was found in this repository. No new requirement statement, relationship, object or behavior was introduced, so no specialized spec analysis applies.
