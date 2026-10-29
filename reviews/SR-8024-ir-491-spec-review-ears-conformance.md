---
id: SR-8024
title: "spec-review/ears-conformance review of IR-491 inline-application correction"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; IR-491"
review_set: subset
---

## Summary

The changed table row specifies a concrete selection and admission precondition. The changed AC and TC step use observable outcomes and retain the earlier failure ordering.

Examined units:
- `FR-038` (examined): | `structural.eq`, `structural.ne` | `operand:0` | The first operand's declared type: a `literal`'s `type`, a `reference`'s target when it is type-shaped, or otherwise that target node's `semantic_type`. A reference to an expression node with an application at its body root selects that node's `semantic_type`, which must equal the body's `result_type`; an application cannot occur inline in `arguments`. Its family must already fit `structural_kind`, and `same_type` must hold. |
- `FR-038-AC-228` (examined): PLANNED/UNRUN (IR-491). Through `CheckedPackageV2::read`, otherwise valid `structural.eq` and `structural.ne` packages whose first operand is, separately, a reference to a structural value, a reference directly to its type node, a reference to an expression node with an application at its body root whose `result_type` equals that node's `semantic_type`, or an option-typed `none` literal, each derive the leaves of the selected declared type. An all-integer record admits with `leaves: []`, and a record with one pinned text field admits with exactly that field's selected-law and mode leaf, while
- `TC-048 step 1` (examined): 1. For AC-228, use `structural.eq` and `.ne` over an all-integer record and
   a record with one `Text[0, 8; nfc]` field. Independently author the empty
   and one-field leaf lists. Supply the first operand as a `reference` to a
   value node, a `reference` directly to the structural type, a `reference`
   to an expression node whose body root is an `application` with
   `result_type` equal to that node's `semantic_type`, and an option-typed
   `none` literal for an option comparison; keep each second operand the same
   declared type. For the application-bearing expression, use an admitted


## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
