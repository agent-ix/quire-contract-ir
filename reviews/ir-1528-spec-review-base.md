---
id: SR-2026111219210500000003
title: "spec-review of IR-1528 FR-038 Operation-body carrier and FR-019 inventory row"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #1018 at 306ded5e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Operation-body carrier section, consumer list item, OperationBodies stage row, FR-038-AC-541); spec/model/functional/FR-019-rust-library-interface.md (V2 operation bodies inventory row, FR-019-AC-28)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---

## Summary

Ticket: IR-1528. Base spec review of the PR's spec edits against QSpec FR-322 "Operation-body carrier" and FR-322-AC-397 at f268d62f, read by reference. The author ran no spec validation, so this review ran it: `quire validate --scope . <both changed files>` exits 0. Its only warning, `ac:non-canonical-shape` at FR-038 line 6405, is on an existing row this PR did not touch. `quire matrix --format tsv` was run to confirm which ids are minted. The section states the has_body derivation explicitly (a body clause function node owned by the operation member), which is the right IR statement of FR-322's "the selected document gives the operation a body". The FR-019 inventory row and FR-019-AC-28 are accurate and testable. Criterion-strength analysis (Jev) was not run; its can-fail judgment is folded into this review.

## Examined scope

- FR-038 Operation-body carrier section (examined): the statement of admission, joins, derivation, candidate rule, key order, coverage and encoder input.
- FR-038 consumer list item (examined): accurate.
- FR-038 PackageReadStage `OperationBodies` row (examined): matches the enum and its ordering.
- FR-038-AC-541 (examined): "A default-feature producer passes a borrowed `OperationBodyRow` slice as `CheckedPackageSidecarsRef::operation_bodies` and the strict reader returns the same rows ... OBR-21 is the exception: the package's owner join refuses the foreign-owner function node first."
- FR-019 V2 operation bodies inventory row (examined): matches the exported `OperationBody` and `OperationBodyRow`, `operation_body_rows`, `operation_body` and the encoder field.
- FR-019-AC-28 (examined): testable and can fail.
- QSpec FR-322-AC-397 (context_only).

## Verdict

Validation passes. There are three defects. FND-001 strands the new AC, and FND-002 misstates coverage; both are small text fixes in this PR. FND-003 mirrors QSpec FR-322's candidate rule, which contradicts QSpec's own DT-01 polymorphic dispatch vector. It needs a QSpec correction, and the code-review high finding depends on the same ruling.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-541 is appended to a headerless table fragment after a blank line, below the main AC table, so quire never mints it: it is absent from `quire matrix` output, while FR-038-AC-540 above the gap is minted. The fragment's existing rows FR-038-AC-492 and FR-038-AC-493 (from IR-1370, #937) are stranded the same way. Move all three into the main acceptance table. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6555-6557 |
| FND-002 | medium | The section says the member holds "exactly one row for each operation member that carries an `operation_effects` row". That ties coverage to a different optional carrier, which may be absent. FR-322 and the code (`has_uncovered_operation`) cover every operation member declared by an admitted `object_type`, `process` or `state_machine` declaring node of a selected document, whether or not `operation_effects` is present. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5382-5383 |
| FND-003 | medium | The section requires the row's `body` to "equal the `operation_body` of every `checked_term_data` dispatch candidate carrying the row's operation". That mirrors QSpec FR-322's rule, keyed on `selected_operation`. Every candidate of one dispatch table carries the same family `selected_operation` with a distinct `operation_body`, one per selection key (QSpec DT-01), so the rule cannot hold for polymorphic dispatch. The join should go through the candidate's `selection_key`, not `selected_operation`. Raise it against QSpec FR-322 and amend this statement once that ruling lands. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5392-5393 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 3f7cc94a moved FR-038-AC-492, FR-038-AC-493 and FR-038-AC-541 into the main acceptance table. `quire matrix --format tsv` now lists all three. |
| FND-002 | fixed | Coverage is now stated as every operation member declared by a selected `model`/`object_type`, `model`/`process` or `model`/`state_machine` declaration, independent of `operation_effects`. |
| FND-003 | still-open | The code now joins through the selection key (8e4d0cf4), but the FR-038 SHALL statement still says "the `operation_body` of every `checked_term_data` dispatch candidate carrying the row's operation" (FR-038 line 5392-5394). The IR spec now contradicts its own implementation. The join is described only in the non-normative matrix row. Amend the statement to the join the code implements (and to the effective-declaration join, if code-review FND-005 is taken), citing STD-661. |
