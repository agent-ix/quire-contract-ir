---
id: SR-2026110414560131
title: IR-1345 PR 927 code-review review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/model/functional/FR-019-rust-library-interface.md; tests/conformance_qspec/carried_quantity.rs;
  IR-1345
review_set: subset
---

## Summary

The new bounded shape and operation code was compared with QSpec and the Rust idiom checklist. The new canonical parser repeats the existing quantity parser in the same module family.

## Coverage

- `FR-038-AC-478` (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): The strict reader admits QSpec FR-322-AC-384's self-typed bounded_quantity with ordered unit, min and max bindings, exact canonical rational endpoints, structural identity and dependencies; it refuses a reversed or noncanonical interval as invalid_package/invalid-derived-shape.
- `FR-038-AC-479` (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): Under a bounds-required profile, a bounded-quantity parameter lowers while a parameter at a bare unit or compound unit still returns requires_bound. The strict reader admits bounded operands to quantity add, convert and lt with existing dimension and mode rules.
- `FR-019-AC-26` (examined, `spec/model/functional/FR-019-rust-library-interface.md`): A default-feature consumer matches ScalarTypeForm::BoundedQuantity after strict admission and reads the typed unit, min and max bindings of a QSpec FR-322 bounded quantity, preserving its exact endpoints and structural identity.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Quantity add accepts a bounded result type instead of the required ordinary unit | crates/quire-contract-model/src/checked_package/v2/operations.rs:814 |
| FND-002 | medium | Canonical rational parsing duplicates the existing quantity parser | crates/quire-contract-model/src/checked_package/v2/structural.rs:1046 |

## Verdict

**FAIL** for this method.

## Detail

FND-001: `check_bounded_dimensions` compares only the result unit dimension. For `quantity.add`, replacing the ordinary unit `result_type` with a bounded quantity of the same dimension leaves `result_mismatch` false. The catalog result field is documented as unchecked except for `inner`, and no other quantity-add result-kind check exists. QSpec FR-322-AC-384 and FR-038-AC-479 require the ordinary unit result.

FND-002: `structural.rs::canonical_rational` repeats the integer spelling, fraction splitting, positive denominator and gcd checks already in `operations.rs::canonical_quantity_magnitude` (line 661). Search over `checked_package/v2` found these two parsers for the same canonical magnitude rule. A single helper should return parsed values for the structural order check and serve the operation literal check.

## Dispositions

| Round | FND | Outcome | Evidence |
| --- | --- | --- | --- |
| 1 | FND-001 | fixed | operations.rs now requires CheckedNodeKind::ScalarType(ScalarTypeForm::Unit) for a bounded quantity.add result and the negative conformance test changes result_type to bounded and asserts IllTyped/OperatorIneligible. |
| 1 | FND-002 | fixed | operations.rs calls structural::canonical_rational and deletes its duplicate canonical_quantity_magnitude parser and BigInt imports. |
