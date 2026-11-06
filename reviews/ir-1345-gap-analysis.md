---
id: SR-2026110414560132
title: IR-1345 PR 927 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/model/functional/FR-019-rust-library-interface.md; tests/conformance_qspec/carried_quantity.rs;
  IR-1345
review_set: subset
---

## Summary

The changed trace tags cover admission, bounds refusal, direct unit RequiresBound, and operation admission. The identity and compound-unit obligations have no direct test assertion. Plan completion: not assessed.

## Coverage

- `FR-038-AC-485` (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): The strict reader admits QSpec FR-322-AC-384's self-typed bounded_quantity with ordered unit, min and max bindings, exact canonical rational endpoints, structural identity and dependencies; it refuses a reversed or noncanonical interval as invalid_package/invalid-derived-shape.
- `FR-038-AC-486` (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): Under a bounds-required profile, a bounded-quantity parameter lowers while a parameter at a bare unit or compound unit still returns requires_bound. The strict reader admits bounded operands to quantity add, convert and lt with existing dimension and mode rules.
- `FR-019-AC-26` (examined, `spec/model/functional/FR-019-rust-library-interface.md`): A default-feature consumer matches ScalarTypeForm::BoundedQuantity after strict admission and reads the typed unit, min and max bindings of a QSpec FR-322 bounded quantity, preserving its exact endpoints and structural identity.
- `FR-322-AC-384` (context_only, `quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md`): Equal bounds through an alias produce the same node; changing the unit or either bound produces a distinct node. Bare unit and compound-unit parameter forms remain RequiresBound.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Published BQ-02 and BQ-03 identity cases have no value assertion | tests/conformance_qspec/carried_quantity.rs:499 |
| FND-002 | medium | Compound-unit RequiresBound case is not exercised | tests/conformance_qspec/carried_quantity.rs:543 |

## Verdict

**CONDITIONAL** for this method.

Plan completion: not assessed.

## Dispositions

| Round | FND | Outcome | Evidence |
| --- | --- | --- | --- |
| 1 | FND-001 | fixed | published_bounded_quantity_unit_and_alias_identity asserts BQ-02 changes structural identity and BQ-03 forwards both parameter types to one bounded node, with a bounded-node census of two. |
| 1 | FND-002 | fixed | published_bounded_quantity_shape_and_bounds_policy requests a compound-unit parameter under require_bounds and asserts RequiresBound names its exact structural ID. |
