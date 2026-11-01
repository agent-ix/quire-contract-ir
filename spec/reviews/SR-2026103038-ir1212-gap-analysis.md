---
id: SR-2026103038
title: "gap-analysis review of PR #758 row types"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/exact_projection.rs; spec/model/functional/FR-019-rust-library-interface.md; IR-1212"
review_set: subset
---

## Summary

Scoped the computed Test Matrix and reverse code-to-spec check to the changed row types and FR-019 public inventory. FR-019-AC-5 is tagged to TC-058 and its focused test passes. FR-354 runtime-read criteria remain untagged future work outside this row-only PR. No applicable AssuranceProfile was found under `spec/`.

## Coverage

- `SelectedTupleDeclarationRow` (examined, `crates/quire-contract-model/src/checked_package/v2/exact_projection.rs`): pub struct SelectedTupleDeclarationRow {
    /// The tuple declaration's admitted structural node.
    pub structural_node: CheckedNodeId,
    /// The selected domain document identity.
    pub selected_document: Box<str>,
    /// The tuple's original identity in that document.
    pub original_identity: Box<str>,
    /// The tuple key issued by the QSL checking/model stage.
    pub node_key: NodeKey,
}
- `SelectedSourceUnitRow` (examined, `crates/quire-contract-model/src/checked_package/v2/exact_projection.rs`): pub struct SelectedSourceUnitRow {
    /// The declared unit's admitted structural node.
    pub structural_node: CheckedNodeId,
    /// Authority of the selected raw source.
    pub source_authority: Box<str>,
    /// Identity of the selected raw source.
    pub source_identity: Box<str>,
    /// Original qualified declaration segments in that source.
    pub qualified_declaration: Vec<Box<str>>,
    /// The declared unit identity issued by the QSL checking stage.
    pub unit_id: UnitId,
}
- `SelectedEnumDeclarationRow` (examined, `crates/quire-contract-model/src/checked_package/v2/exact_projection.rs`): pub struct SelectedEnumDeclarationRow {
    /// The enum declaration's admitted structural node.
    pub structural_node: CheckedNodeId,
    /// Authority of the selected raw source.
    pub source_authority: Box<str>,
    /// Identity of the selected raw source.
    pub source_identity: Box<str>,
    /// Original qualified declaration segments in that source.
    pub qualified_declaration: Vec<Box<str>>,
    /// The enum key issued by the QSL checking stage.
    pub node_key: NodeKey,
}
- `SelectedEnumMemberRow` (examined, `crates/quire-contract-model/src/checked_package/v2/exact_projection.rs`): pub struct SelectedEnumMemberRow {
    /// The enum member's admitted structural node.
    pub structural_node: CheckedNodeId,
    /// Authority of the selected raw source declaring its enum.
    pub source_authority: Box<str>,
    /// Identity of the selected raw source declaring its enum.
    pub source_identity: Box<str>,
    /// The selected enum declaration's admitted structural node.
    pub declaration_node: CheckedNodeId,
    /// Original qualified identity of the selected enum declaration.
    pub declaration_qualified_name: Vec<Box<str>>,
    /// The original member case in that enum
- `checked_package/v2/mod.rs` (examined, `crates/quire-contract-model/src/checked_package/v2/mod.rs`): SelectedEnumDeclarationRow, SelectedEnumMemberRow, SelectedExactDeclarationForm,
- `quire-contract-model/lib.rs` (examined, `crates/quire-contract-model/src/lib.rs`): RelationForm, ScalarTypeForm, SelectedEnumDeclarationRow, SelectedEnumMemberRow,
- `FR-019` (examined, `spec/model/functional/FR-019-rust-library-interface.md`): | `checked_package` (V2 Exact type projection) | types `ExactTypeProjection`, `ExactTypeProjectionError`, `ExactTypeProjectionErrorCode`, `SelectedEnumDeclarationRow`, `SelectedEnumMemberRow`, `SelectedExactDeclarationForm`, `SelectedExactDeclarationRow`, `SelectedExactFieldRow`, `SelectedExactIdentities`, `SelectedExactIdentity`, `SelectedRecordFieldView`, `SelectedRecordView`, `SelectedSourceUnitRow`, `SelectedTupleDeclarationRow` | FR-352, FR-354 |
- `FR-019-AC-5` (examined, `spec/model/functional/FR-019-rust-library-interface.md`): The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands; exporting a planned item fails the inventory until its row is marked implemented. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path. The retired `PACKAGE_DOMAIN_V2` export is absent.
- `FR-354-AC-1` (context_only, `spec/checked_package/functional/FR-354-project-admitted-runtime-context.md`): With the FR-352 and additional QSL-stage rows for selected tuples and source-owned units, a public IR-model read supplies complete record, tuple and object declarations, effective fields, ancestry and units from which QSL constructs the expected `TypeEnvironment`; no source or JSON is reparsed. Before those additional producer rows exist, a tuple or source-owned unit request refuses rather than claiming a complete view.
- `FR-354-AC-6` (context_only, `spec/checked_package/functional/FR-354-project-admitted-runtime-context.md`): With QSL-stage enum declaration and member rows, an ordered and an unordered selected enum each yield exact case order, member identity and position sufficient for `EnumMemberIndex`; absent, duplicate, mismatched-case or foreign member witnesses refuse before any partial index.

Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the scoped row-only PR. No full Cargo or CI gate was run by this reviewer.

## Post-review inventory recheck

At PR head `42e2e8895f4e6f322b1354e32b87f97df339d90e`, FR-019 removes the unimplemented `CheckedTermDataV2` and optional encoder argument from the implemented public-item row, names them as planned IR-1219 work, and inventories the exported `nominal_runtime` module. All seven focused TC-058 tests pass with `conformance-cli`; the existing clean scoped result stands. No prior finding requires a disposition.
