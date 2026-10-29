---
id: SR-14727
title: "IR-940 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-940; spec/checked_package/functional/FR-054-admitted-scalar-operand-projection.md; spec/checked_package/matrix/TC-458-admitted-scalar-operands.md; spec/model/functional/FR-019-rust-library-interface.md; crates/quire-contract-model/src/checked_package/v2; tests/it/admitted_scalar_operands.rs"
review_set: subset
---

## Summary

The TC-458 positive public test uses an IR-authored JSON fixture, so it never exercises the required QSL increment source through ordinary lowering; extend the existing tests/conformance_qsl_ir769/main.rs:22-135 producer lane with the public scalar_operands query.

## Verdict

**CONDITIONAL** — The TC-458 positive public test uses an IR-authored JSON fixture, so it never exercises the required QSL increment source through ordinary lowering; extend the existing tests/conformance_qsl_ir769/main.rs:22-135 producer lane with the public scalar_operands query.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The TC-458 positive public test uses an IR-authored JSON fixture, so it never exercises the required QSL increment source through ordinary lowering; extend the existing tests/conformance_qsl_ir769/main.rs:22-135 producer lane with the public scalar_operands query. | tests/it/admitted_scalar_operands.rs:34 |

## Coverage

Reviewed PR #496 against main. Examined FR-054-AC-1 through FR-054-AC-5, TC-458, FR-019 public inventory, source and tagged tests. Plan completion: not assessed.

## Reviewed units

- FR-054-AC-1 (examined): For a reader-admitted positive scalar application with a graph reference followed by an inline integer literal, a public typed `ContractPackageV1` query returns positions 0 and 1 in that order, the exact child key and admitted sources at 0, and the exact signed literal value qualified by the application occurrence and position at 1, with no fabricated literal node.
- FR-054-AC-2 (examined): For an admitted application whose two argument positions reference the same represented child, the query returns two distinct ordered entries with positions 0 and 1 and the same authentic child key and sources; reached-closure set order and deduplication do not alter them.
- FR-054-AC-3 (examined): For admitted applications with unbounded or outside-native-range own facts, the query still returns the complete positional identity vector and distinguishes those facts from finite ranges. It supplies no inferred or result-fallback interval and does not change the interval selected by an existing consumer.
- FR-054-AC-4 (examined): A missing, foreign, non-lowered or non-application key, invalid role/ordinal/source correspondence, ineligible operation or inconsistent represented child returns typed absence, disposition, refusal or error with no selected operand list; an independently valid represented occurrence of the same node remains structurally valid and does not prove unique-original-expression ownership.
- FR-054-AC-5 (examined): A separate consumer test compiles against public owning types and reads the preceding cases from reader-admitted six-field packages without a copied schema, consumer JSON field reads or a CheckedPackageV2 generation fallback. Repeated queries leave the wire, identity, request disposition, FR-049 result context and forward closure unchanged; this accessor alone makes no full scalar replay claim.
- TC-458 (examined): Build the positive through the existing QSL increment source and ordinary FR-035 lowering, then read that same six-field package.
- FR-019 (context_only): The public items table declares the model crate public API.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | QSL main lowers the authored integer 1 in a + 1 to a represented literal node and graph reference, so the proposed TC-458 inline-literal producer assertion would fail on the current producer. QSL-940 tracks correcting that producer behavior. PR marks producer-origin evidence Planned and untagged in TC-458; the reader-admitted IR positive still exercises the specified inline shape. Defer only that producer origin evidence until QSL-940 is resolved. |
