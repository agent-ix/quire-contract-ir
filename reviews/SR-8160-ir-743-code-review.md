---
id: SR-8160
title: code-review review of IR-743 ContractPackage
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir;;
  crates/quire-contract-model/src/checked_package/v2/contract_package.rs, crates/quire-contract-model/src/lib.rs,
  spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md,
  spec/model/functional/FR-019-rust-library-interface.md
review_set: subset
---

## Summary

Ticket: IR-743. Independent code-review pass on the exact frozen candidate.

## Verdict

**FAIL** — 2 finding(s).

## Examined units

- `contract_package.rs` (`examined`): let wire: Self = serde_path_to_error::deserialize(&value).map_err(|error| { malformed(decoder_pointer(JsonPointer::root(), error.path()), CheckedPackageRefusalCause::WrongValueKind) })?;
- `lib.rs` (`examined`): ContractBodyIncompleteV1, ContractBodyRefusalV1, ContractDependencies, ContractDependencyInput, ContractDispositionV1, ContractFeatureRecordV1, ContractNodeSourcesV1, ContractPackageReadResult, ContractPackageV1, ContractPackageWireV1
- `FR-035-AC-5` (`examined`): One call emits exactly one FR-330 `quire.contract-package/v1` value with the six required properties and one key-ordered `semantic_graph` node collection; repeated equivalent construction orders have identical canonical semantic identity. The digest is computed from exactly `contract_version`, `package`, `semantic_graph` and `feature_accounting`: changing an included member changes it, changing only valid `source_map` locations preserves it, and invalid or incomplete `source_map` correspondence refuses admission.
- `FR-019-AC-5` (`examined`): The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists, checked by a public-item inventory that fails on an added or missing item; no item in the table is reachable through a `quire_contract_ir` path.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Unknown nested enum strings are reported as wrong-value-kind. For example, source_map.nodes[0].occurrences[0].role="bogus" passes shape checking, then Serde rejects the enum and the generic error mapper returns WrongValueKind at that member; FR-330 requires invalid-value for an out-of-domain string. | crates/quire-contract-model/src/checked_package/v2/contract_package.rs:564 |
| FND-002 | high | New public ContractPackage exports are absent from FR-019’s exact crate-root inventory. TC-058 fails for the candidate head, so the stable public surface and its acceptance criterion disagree. | crates/quire-contract-model/src/lib.rs:128 |

## Coverage

Review limited to the candidate diff and named acceptance units. Full gate is owned by the author; reviewer ran no Cargo suites.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
| FND-002 | fixed | Fixed in the reviewed PR. |

## Disposition verdict

**PASS (round 1, scoped)** — Every original finding was fixed by the listed commits. No new finding arose from the fix diff. Full gates are owned by the author.
