---
id: SR-14730
title: "IR-940 spec-review-ears-conformance review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; IR-940; spec/checked_package/functional/FR-054-admitted-scalar-operand-projection.md; spec/checked_package/matrix/TC-458-admitted-scalar-operands.md; spec/model/functional/FR-019-rust-library-interface.md; crates/quire-contract-model/src/checked_package/v2; tests/it/admitted_scalar_operands.rs"
review_set: subset
---

## Summary

Examined FR-054 requirement grammar; no defects found in this lens.

## Verdict

**PASS** — Examined FR-054 requirement grammar; no defects found in this lens.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reviewed PR #496 head 276a6d40cc0c2fbb68f1f0efd7343a9b22b1d64c against main d41e521f97aee80f81ebce42fbe97d88792891be. Examined FR-054-AC-1 through FR-054-AC-5, TC-458, FR-019 public inventory, source and tagged tests. Plan completion: not assessed.

Targeted `quire validate --summary` reports FR-054 grammar-clean with zero grammar findings.

## Reviewed units

- FR-054-AC-1 (examined): For a reader-admitted positive scalar application with a graph reference followed by an inline integer literal, a public typed `ContractPackageV1` query returns positions 0 and 1 in that order, the exact child key and admitted sources at 0, and the exact signed literal value qualified by the application occurrence and position at 1, with no fabricated literal node.
- FR-054-AC-2 (examined): For an admitted application whose two argument positions reference the same represented child, the query returns two distinct ordered entries with positions 0 and 1 and the same authentic child key and sources; reached-closure set order and deduplication do not alter them.
- FR-054-AC-3 (examined): For admitted applications with unbounded or outside-native-range own facts, the query still returns the complete positional identity vector and distinguishes those facts from finite ranges. It supplies no inferred or result-fallback interval and does not change the interval selected by an existing consumer.
- FR-054-AC-4 (examined): A missing, foreign, non-lowered or non-application key, invalid role/ordinal/source correspondence, ineligible operation or inconsistent represented child returns typed absence, disposition, refusal or error with no selected operand list; an independently valid represented occurrence of the same node remains structurally valid and does not prove unique-original-expression ownership.
- FR-054-AC-5 (examined): A separate consumer test compiles against public owning types and reads the preceding cases from reader-admitted six-field packages without a copied schema, consumer JSON field reads or a CheckedPackageV2 generation fallback. Repeated queries leave the wire, identity, request disposition, FR-049 result context and forward closure unchanged; this accessor alone makes no full scalar replay claim.
- TC-458 (examined): Build the positive through the existing QSL increment source and ordinary FR-035 lowering, then read that same six-field package.
- FR-019 (context_only): The public items table declares the model crate public API.
