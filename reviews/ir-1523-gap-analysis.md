---
id: SR-20261112044524
title: "gap-analysis of IR-1523 public CatalogReadError::new"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #1017 at 09210a7c; FR-019 public items table (context), FR-038 catalog-unavailable paragraph (context); crates/quire-contract-model/src/checked_package/v2/operation_catalog.rs"
review_set: subset
---

## Summary

Planless gap analysis of quire-contract-ir PR #1017. Ticket: IR-1523. Plan completion was not assessed.

The PR changes no spec. The constructor is new code, so this review checked that it has an owning requirement and that it conflicts with none. FR-019's public-items table lists `CatalogReadError` under `checked_package` (owners FR-038, FR-047, FR-062). Rows in that table list types, not their methods, so adding a method needs no spec row. FR-038 covers the catalog-unavailable result and `initialize_operation_catalog`, and says nothing that rules out a caller constructing the error. FR-019-AC-8 forbids unchecked constructors only for validated semantic values, and an error report is not one. No spec text conflicts with the change.

The ticket's acceptance was checked against the code, not taken from the ticket text. It asks that a downstream crate can construct the error and feed it through the Kani reader's catalog-unavailable path. The constructor is `pub` on a type re-exported at the crate root (lib.rs:151). CG's `OracleGenerationError::KaniReaderCatalogUnavailable` wraps `Box<quire_contract_model::CatalogReadError>`, so the driver can build the variant. No reader code changed, so reading behaves as before.

## Examined scope

- FR-019 public items, `checked_package` row (context_only): "functions `initialize_operation_catalog`, `read_checked_package`, `read_checked_package_with_budget`; types `CatalogReadError`, ..."
- FR-038 catalog paragraph (context_only): "An unreadable catalog returns `CatalogUnavailable`, retaining all prior admitted charges. The public `initialize_operation_catalog() -> Result<(), CatalogReadError>` checks the published catalog independently"
- FR-019-AC-8 (context_only): "validated fields are private, and no unchecked constructor or `From` conversion crosses that boundary."
- operation_catalog.rs:277-308 constructor and test (examined).

## Verdict

PASS. No gap. The constructor is owned by FR-019's existing type row and backed by a unit test. Building the driver's mapping test itself is quire-driver's work, under IR-1494.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
