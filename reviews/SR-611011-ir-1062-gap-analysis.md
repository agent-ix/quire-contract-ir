---
id: SR-611011
title: "Gap analysis of IR-1062 PR 616"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/{functional/FR-068-join-selected-process-and-persistence-declarations.md,matrix/TC-463-selected-process-and-persistence-owners.md}; tests/it/checked_package_v2_{model_fields,model_members,owners,refinement}.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-068
    type: references
---

## Summary

Reviewed FR-068-AC-1..5 and TC-463 against PR #616's public tests and selected-document reader. The trace tags name the intended criteria, but two required assertions are hollow or absent.

## Verdict

**CONDITIONAL** — the healthy selected-document proof and the FR-154 field-refusal locus are not established by the new tests. Code review separately records the reader acceptance defect.

## Coverage

Plan completion: not assessed. Manually checked changed test trace tags and the FR-068 criteria because host backoff prohibited the repository-wide Quoin matrix/spec sweep. Optional repository-wide semantic review was not requested. The public tests for process field/operation and reference admission, selected owner mismatch, record Text projection and clause-edge wrong-kind refusal were examined.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The positive selected-owner test uses meaning-invalid Flow, Machine and Store documents, so it cannot prove admission of a real selected domain document. | tests/it/checked_package_v2_refinement.rs:62 |
| FND-002 | medium | Five record field wrong-kind cases can pass on a different selection refusal because they do not assert the refused field identity and typeRef member path. | tests/it/checked_package_v2_model_fields.rs:1637 |

## Detail

FND-001: FR-068-AC-1 and TC-463 require real selected Semantic IR documents. `selected_form_document` creates Flow without identityFields or steps, Machine without operations/states/transitions, and Store without operations/persists; the test does not establish the promised real positive case. Correct source declarations and assert the required retained identity after each canonical reread.

FND-002: `selected_record_field_refuses_each_non_value_object_meaning_at_intake` asserts only `invalid_model_binding`/`malformed-declaration` and `/lock/model_selections/0`. All five `Other` declarations are minimal stubs that can independently cause that same selection-level result. The test never asserts that `Address.street` at `Field.typeRef` was refused. Give each Other meaning a valid declaration and assert retained declaration identity plus `Field.typeRef` locus, so the test fails if the record field check is removed.

## Dispositions

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed | The selected-document fixture supplies a process correlation field and step, a state-machine operation/state/transition, and a persistence operation and object target. The public reader test checks each owner key after both canonical rereads. |
| FND-002 | fixed | The five public wrong-kind cases now supply complete Other declarations. The internal `record_wrong_kind_type_refs_retain_field_identity_and_type_ref_path` test asserts `Address/street` and `fields[0].typeRef` for each meaning; the public refusal remains at the selected lock row by the reader contract. |
