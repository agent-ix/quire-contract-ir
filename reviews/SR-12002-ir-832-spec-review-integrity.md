---
id: SR-12002
title: "IR-832 spec integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir (PR #425); spec/model/functional/FR-019-rust-library-interface.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Ticket: IR-832. Checked cross-document consistency: the FR-038 inventory bullet, the section, the AC rows, the tests.md FR-038 summary row, the TC-048 row, the per-AC matrix rows, the TC-048 procedure and the FR-019 public-items row. Also checked that the spec still agrees with the tests that read it.

Reproduced at the PR head, reusing the repo's existing target dir:
`cargo test --test it tc_058_model_root_exports_exactly_the_documented_public_items` FAILS. `tests/it/public_interface.rs` `include_str!`s FR-019 and asserts that `exported_items() == expected_items()`. The right-hand side now has `CheckedParameterDeclaration` and `CheckedParameterDeclarationError`, and no code exports them. The PR's only CI checks are CLA, so the break would land on `main` unnoticed.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-019 V2 parameter declaration row | examined | spec/model/functional/FR-019-rust-library-interface.md | `checked_package` (V2 parameter declaration) - types `CheckedParameterDeclaration`, `CheckedParameterDeclarationError` - FR-038 |
| FR-019-AC-5 | context_only | spec/model/functional/FR-019-rust-library-interface.md | The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists, checked by a public-item inventory that fails on an added or missing item ... |
| FR-019 planned-API convention | context_only | spec/model/functional/FR-019-rust-library-interface.md | FR-052 and FR-053 specify additional public model APIs that remain planned until they are implemented and exported; their owning requirements retain those API obligations. |
| tests.md FR-038 summary row | examined | spec/checked_package/matrix/tests.md | ... FR-038-AC-257, FR-038-AC-258 through FR-038-AC-261 - TC-048, TC-050, ... |
| tests.md TC-048 row | examined | spec/checked_package/matrix/tests.md | ... FR-038-AC-257, FR-038-AC-258, FR-038-AC-259, FR-038-AC-260, FR-038-AC-261 ... |
| tests.md per-AC rows AC-258..261 | examined | spec/checked_package/matrix/tests.md | FR-038 - FR-038-AC-258 - TC-048 - PLANNED/UNRUN (IR-832). ... |
| FR-038 inventory bullet | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A typed accessor on the admitted package for a `value`/`parameter` node's declared name and level, `parameter_declaration`; see "Typed parameter declaration" (FR-038-AC-258 through FR-038-AC-261, IR-832). |
| TC-048 "Typed parameter declaration" | examined | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md | PLANNED/UNRUN (IR-832); no test is tagged yet. ... The compiled public inventory of the two new types is covered by TC-058. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FR-019 Public items row lists two types no code exports. FR-019-AC-5's test reads that table, so merging this "spec-only" PR makes `tc_058_model_root_exports_exactly_the_documented_public_items` fail on `main` (reproduced). FR-019's own convention keeps planned APIs out of the table until they are exported. Move the row into the IR-832 code PR. | spec/model/functional/FR-019-rust-library-interface.md:183; tests/it/public_interface.rs:188-190 |
| FND-002 | low | The tests.md FR-038 summary row adds AC-258..261 to its AC list, but its status cell does not mark them PLANNED/UNRUN, as it does for AC-239..241. Only the per-AC rows say so. | spec/checked_package/matrix/tests.md:21 |

## Verdict

FAIL until FND-001 is fixed. Every other link is consistent: the inventory bullet, the section, four AC rows with Test (TC-048), the TC-048 coverage row, four per-AC rows and the TC-048 procedure. The PR is not effectively spec-only: it changes the outcome of an existing tagged test.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 : FR-019 is byte-identical to origin/main again. FR-038 and TC-048 say the exporting code change adds the row. `cargo test --test it tc_058_` now passes 4/4, including `tc_058_model_root_exports_exactly_the_documented_public_items`. |
| FND-002 | fixed | Round 1 : the FR-038 summary status cell now opens with "🚧 AC-258 through AC-261 (IR-832) typed parameter declaration are PLANNED/UNRUN." |
