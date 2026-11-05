---
id: SR-2026110414560036
title: "IR-1345 PR 912 code-review review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/lower.rs; tests/conformance_qspec/carried_quantity.rs; FR-038-AC-73; TC-050"
review_set: subset
---

## Summary

Code/Rust review found one test-intent gap; the production match is exhaustive and admitted carried types are shape-checked by the reader. Reviewed frozen PR #912 head a6df76d14efdd129570a0d84d53ca012302c8dc9. Focused `cargo fmt --check` and `git diff --check` passed. Aggregate gate is assigned to the code lead after findings are fixed.

## Scope and evidence

| Unit | Path | Role | Excerpt |
| --- | --- | --- | --- |
| FR-038-AC-73 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | examined | Three distinct self-typed `scalar_type`/`carried_quantity` positions in QSpec FR-322's `Quantity<u, C>` form each lower: a record field with an admitted `integer_range` carrier, a parameter with an admitted `decimal_range` carrier, and a record field with an admitted `rational_range` carrier. |
| TC-050 quantity procedure | spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md | examined | lower three separate positive value positions declared at self-typed `scalar_type`/`carried_quantity` nodes |
| quantity_chain | crates/quire-contract-model/src/checked_package/v2/lower.rs | examined | \| ScalarTypeForm::CarriedQuantity |
| published_carried_quantity_positions_lower_without_covering_a_bare_unit | tests/conformance_qspec/carried_quantity.rs | examined | let carrier = all[carried]["carrier"].as_str().expect("published carrier"); |
| tc_050_a_unit_is_not_a_quantity_position_merely_by_being_reached | tests/it/checked_package_v2_require_bounds.rs | context_only | An application's `result_type` is not a position. |
| QSpec FR-322 quantity form | quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md | context_only | `Quantity<u, C>` \| `scalar_type`/`carried_quantity`, self-typed (family `quantity`) |

## Verdict

FAIL (change scope) — one medium finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The test takes each expected carrier from the same QSpec row it loads and checks only membership in lowered bounds. It never checks the admitted carrier forms. If q_dec and q_rat point at integer_range carriers, all three positive assertions still pass, leaving the decimal_range and rational_range obligations unproved. | tests/conformance_qspec/carried_quantity.rs:487-512; FR-038-AC-73 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The test fixes q_int, q_dec and q_rat to distinct published carrier IDs and checks each admitted carrier kind against the specified integer, decimal or rational form. |
