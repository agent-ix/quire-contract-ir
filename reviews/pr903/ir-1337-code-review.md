---
id: SR-2026110414560023
title: Rust/code review of retained record-field read
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/model_fields.rs,
  spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/model/functional/FR-019-rust-library-interface.md,
  tests/it/checked_package_v2_model_fields.rs
review_set: subset
---

## Summary

Ticket: IR-1337. The public accessor reads the existing selected record table, preserves admitted identity, optionality and nested member type, and keeps Exact projection refusal. The changed integration tests exercise values and typed errors; no defect found.

## Reviewed scope

- FR-038-AC-467 (examined, spec/checked_package/functional/FR-038-consume-checked-package-v2.md): For an admitted selected `Receipt` model record whose effective field refers to an `Other` object declaration present in the selected document but absent from the selected graph, `model_record_fields(Receipt)` returns that field's authentic selected IR identity, name, optional flag and `CheckedMemberType::Reference(Other)` without requiring an Exact projection of `Other`; `project_exact_type(Receipt)` still refuses `UnresolvedSelectedDeclaration`. The accessor reads the retained table without a second derivation or admission charge.
- FR-038-AC-468 (examined, spec/checked_package/functional/FR-038-consume-checked-package-v2.md): For effective record fields with nested `Option` and `Collection` member types, the accessor returns each field in effective declaration order with its exact derived kind, element and bounds; an optional String field whose derived type is `Option(String)` and a required field declared with `Option(Integer)` remain distinguishable by their optional flags. `field(name)` returns the same field as `fields()` and returns `None` for an absent name. A field with no derived member type remains in the returned list with `member_type` `None`.
- FR-038-AC-469 (examined, spec/checked_package/functional/FR-038-consume-checked-package-v2.md): `model_record_fields` returns distinct typed `UnknownNode`, `NotModelRecordType` and `AmbiguousField(name)` errors for an absent graph node, an admitted non-record node and an admitted selected record with conflicting effective field names, respectively; each returns no partial field list. Equal calls and a cloned admitted package return equal values, and none of these calls panics.
- FR-019 (examined, spec/model/functional/FR-019-rust-library-interface.md): The V2 reader API inventory adds CheckedModelRecordField, CheckedModelRecordFields, and CheckedModelRecordFieldsError.

## Verdict

**PASS** for the changed PR surface. Focused coder checks passed; final aggregate gate belongs to the code lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
