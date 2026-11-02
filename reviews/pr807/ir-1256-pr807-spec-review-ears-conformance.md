---
id: SR-202611010312
title: "spec-review/ears-conformance review of IR-1256 PR 807"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/model/functional/FR-019-rust-library-interface.md; IR-1256"
review_set: subset
---

## Summary

Checked the changed FR-038 requirement statements for named reader, concrete response and appropriate conditional grammar. Scoped Quire validation showed only an unchanged criterion warning. No applicable AssuranceProfile was found under `spec/`.

## Coverage

- `FR-038-AC-401` (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): planned until IR-1256 lands. The strict reader admits QSpec `CQ-W01` through `CQ-W04`: decimal and integer carried-quantity nodes are anonymous, self typed and of family `quantity`; the integer carrier with `exact` and with `nearest-even` produces distinct structural nodes. The public `ScalarTypeForm::CarriedQuantity` and admitted body expose the exact unit, carrier and rounding without evaluating a quantity.
- `FR-038-AC-402` (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): planned until IR-1256 lands. The strict reader refuses `CQ-W05` through `CQ-W08` as `invalid_package`/`invalid-derived-shape` at the carried node's `node_id`: a dimension in the unit slot (`RM-CQ-UNIT`), a rational or unbounded decimal carrier (`RM-CQ-CARRIER`), and a decimal carrier whose rounding differs (`RM-CQ-MODE`). Additional missing, extra or reordered bindings and a path/body mismatch receive the same typed refusal after applicable earlier wire or graph checks. No refused node yields an admitted public type.
- `FR-038-AC-403` (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): planned until IR-1256 lands. The strict reader admits `CQ-W09`, `CQ-W10` and `CQ-W15`: conversion from a unit-typed quantity or a carried quantity into a carried result of the same dimension with the exact result-pinned mode. It refuses `CQ-W11` and `CQ-W12` as `invalid_package`/`operation-mode-type-mismatch` at the application for a differing or null mode, `CQ-W13` as `ill_typed`/`operator-ineligible` there for a different dimension, and `CQ-W14` with that code and cause for a carried operand of `quantity.add`. Other quantity arithmetic, comparison and `collection.sum.quantity` also reject a carried operand.
- `FR-038-AC-404` (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): planned until IR-1256 lands. TC-048 reads `proposals/checked-package-v2/quantity-carrier-vectors.json` from the selected QSpec checkout by reference and applies `CQ-W01` through `CQ-W15` to complete IR package reads, checking admission or exact typed code, cause and primary node or application locus. The test records the selected checkout revision in evidence and stores no copy of the upstream vector or schema.
- `FR-038-AC-405` (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): planned until IR-1256 lands. `CQ-E01` through `CQ-E20` are referenced as downstream evaluation controls only: the IR strict reader admits their valid carried type and conversion wire independently of the evaluator's completed, refused or incomplete outcome. The reader never emits `inexact_conversion`, `decimal_out_of_domain` or `integer_out_of_domain` as a package-read result for a valid conversion application.
- `FR-019-AC-20` (examined, `spec/model/functional/FR-019-rust-library-interface.md`): planned until IR-1256 lands. A default-feature external consumer exhaustively matches `ScalarTypeForm::CarriedQuantity` in the public closed vocabulary, reads an admitted carried quantity node's exact typed unit, carrier and rounding body bindings, and distinguishes integer-carrier exact and nearest-even nodes by their structural ids. The implemented inventory includes the extended vocabulary only when the IR-1256 reader lands; no evaluation outcome type enters this row.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for this method on the frozen head.
