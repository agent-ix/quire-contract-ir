---
id: SR-1559
title: "Base spec review of quire-contract-ir PR #296 (IR-628 typed model object fields accessor)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@7ffa956c25fe491767cb790d8168e958929000e4; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Public items bullet, section 'Typed accessor for a model object type's fields', FR-038-AC-136..143), spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md, spec/checked_package/matrix/tests.md, spec/core/matrix/tests.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: IR-628. Base checklist over `git diff origin/main...HEAD` of PR #296 (five spec files, no
code, no CI or workflow edits, no file copied from QSL or QSpec; re-measured). The PR text and author
report were treated as claims and re-measured against the code at the reviewed sha:

- `CheckedPackageV2` (`crates/quire-contract-model/src/checked_package/v2/mod.rs:383`) holds only
  `wire`, `kinds` and `bytes`; its public methods are `read`, `package_id`, `identity_preimage`,
  `lock`, `graph`, `node_kinds`, `source_map`, `capability_report` and `diagnostics`. No accessor
  returns a field, member type or bound. `MemberType`, `IntegerBounds`, `DomainModel`,
  `DomainModel::resolve`/`slot_type`/`field_type`, `MemberType::node_key`, `ModelOwners` and
  `ModelOwners::recover` exist and are `pub(super)` (`model_members.rs`). Confirmed.
- Admission builds `LockAdmission.models: Vec<DomainModel>` in `validate_lock`
  (`mod.rs:810`, `882`) and drops it when `validate` returns. Retaining it is feasible
  (`DomainModel` is owned, `Clone`, `Eq`). Every `model_selections` row needs evidence:
  `admit_document` refuses a missing document `missing_import`/`missing-selection`
  (`model_members.rs:1043`). The same bytes without evidence therefore never admit, so the
  accessor never meets a package with no model. "Takes no evidence argument" is true. The model is
  the evidence document bound by the lock digest, as the IR-627 Trust root paragraph records.
- `IntegerBounds` is `{lower: i128, upper: i128}`. A bound is read as `as_i64` for a JSON number or
  `str::parse::<i128>` for a string (`model_members.rs:1495-1513`). A bound outside `i128` fails to
  parse, so the value type is "not bound as `Int[lo, hi]`" and the field type is `None`. Nothing is
  truncated.
- AD-006 (`spec/assurance/AD-006-codegen-consumption-seam.md:83`) records no `#[non_exhaustive]`
  on any public enum. FR-019's Public items table has the row `checked_package` (V2 reader)
  (`spec/model/functional/FR-019-rust-library-interface.md:151`).
- `make spec` at the merge base `bf36cda`: 37 unbacked rows (`--strict`). At head: 46. The nine new
  rows are exactly FR-038-AC-136..143 and TC-227, all planned. `quire validate` is clean at head.
  The IR-627 numbering on main runs to FR-038-AC-135 and TC-226, and neither AC-136..143 nor
  TC-227 existed on main. The FR-038 matrix range, the TC-227 summary and cases rows, the Checked
  package index row and the StR-001 row all agree.

Analyses run: base (this file), integrity (SR-1560), scope-boundary (SR-1561), ears-conformance
(SR-1562), failure-domain (SR-1563). criterion-strength was not run: the installed
`spec-criterion-strength-analysis` skill (quoin 0.28.2) says it is blocked on its Jev client and
must not be run yet. Its strength observations are folded into SR-1560 (FND-002, FND-003, FND-004)
and SR-1563 (FND-002). SR-1564 was reserved for it and is unused.

## Verdict

Changes requested overall: high findings here (FND-001) and in SR-1560 (FND-001). Clean units: ID
format and uniqueness, the coverage delta (exactly the new planned rows), the matrix and index rows,
no CI edits, no copied files. The claims about the code (no accessor today, `pub(super)` resolution,
the model dropped at admission, `i128` bounds with no truncation, the functions reused) were
re-measured and are true.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-142 (and TC-227's procedure) verify closedness by a source-text scan for `#[non_exhaustive]`, and verify that the five names appear in FR-019's Public items table and its inventory, which is a scan of spec text. Item 14 adds an inventory entry obligation. The installed spec-review checklist ("Hash / Digest / Pin Antipattern": no test case scans source or spec text; no inventory) makes any such hit high. The AC-142 compile fixtures already discharge the property: an exhaustive match with no wildcard, compiled from `tests/it` (an external crate), fails if an enum is `#[non_exhaustive]`, and a struct literal that must fail to compile proves the private fields. Remove the scan and table clauses. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2079, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2794, spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md:37 |
| FND-002 | low | IR-628-Q1's rationale ("the key derivation can be refused by the encoder at the byte limit, which would make the accessor fallible per field") does not fit item 9: the `Reference` variant already carries an encoder-derived key (`element_type` derives it with `declaration_key(...).ok()`, and `ModelOwners::new` has derived every such key under the same limit at admission). The question is open for the owner, but the stated reason against it is not the real trade-off. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2150 |


## Dispositions

Round 1, reviewed at bd47f6aa58501e5d88afff2657f634f9f2b4f382.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
| FND-002 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
