---
id: SR-12172
title: "IR-861 gap analysis of IR PR #441"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@243009d0d25f271521b9a372fe30bfc4f8bde109 (PR #441); git diff origin/main...HEAD (spec only: FR-038 Verification cells, TC-048 'Selected variant types'); computed matrix (quire matrix --scope . --format tsv) and trace tags for the 38 edited FR-038 criteria, and the AC-236..238 tests in tests/it/checked_package_v2_model_members.rs and tests/it/checked_package_v2_enum_order.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-861 gap analysis of IR PR #441

## Summary

Ticket: IR-861. This is a planless gap analysis of a spec-only PR. Plan completion
was not assessed. The PR changes no production or test code, so it adds no
reverse gap and no stub.

The computed matrix at the head (quire 0.36.2, engine 0.50.2) gives these
results for the 38 edited criteria:

- 34 compute `tagged`.
- AC-193, AC-194 and AC-201 compute `method-without-symbol`, because they are
  Inspection.
- AC-195 computes `untagged`.

Where a tag names a TC, it names TC-048. The AC-236..238 tests carry only
`/// Trace: FR-038-AC-23x`, with no TC id. That gap is pre-existing and was
already recorded as SR-12122 FND-002, so it is not reported again here.

I checked whether the AC-236..238 tests assert what the criteria and the new
TC-048 section say:

- AC-236: `selected_variant_types_field_parameter_and_result` admits a package
  that uses the variant as a field, a parameter and a result, under an Option
  and a bounded Sequence. It asserts that each field's typed member names the
  variant. `selected_variant_requires_its_exact_model_owner` asserts
  `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/5/node_id`
  for an absent owner and for a different existing owner.
- AC-237: four UTF-8 cases are admitted. The test asserts distinct keys across
  the cases and across declarations. `enum.eq` and `enum.ne` are admitted, and
  `enum.lt` and `structural.eq` are refused at `arguments/0`.
- AC-238: the empty declaration is refused at `/lock/model_selections/0`, the
  absent case at `.../nominal_identity_preimage/case`, and the stale document
  at `/lock/model_selections/0/digest`. An ordinary enum with the case `Gold
  Plus` is refused `invalid_semantic_graph`.

Each assertion fails if the reader is wrong, and none is tautological. The
computed matrix agrees with reviews/ir-777-gap-analysis.md.

## Coverage

Plan completion: not assessed

- Edited criteria: 38. 34 are tagged, 3 are method-without-symbol (AC-193, 194,
  201) and 1 is untagged (AC-195).
- `quire coverage --strict`: 43 unbacked on origin/main, 39 on the head, with 0
  contradicted on both. The four rows that changed (AC-193, 194, 195, 201) are
  explained in SR-12171.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038-AC-236 tests | examined | tests/it/checked_package_v2_model_members.rs:541-583,692-721 | /// Trace: FR-038-AC-236 selected_variant_types_field_parameter_and_result; selected_variant_requires_its_exact_model_owner |
| FR-038-AC-237 tests | examined | tests/it/checked_package_v2_model_members.rs:474-539,627-664 | /// Trace: FR-038-AC-237 selected_variant_accepts_exact_utf8_cases; selected_variant_case_keys_bind_exact_case_and_declaration; selected_variant_admits_enum_equality_and_refuses_order_or_structural_equality |
| FR-038-AC-238 tests | examined | tests/it/checked_package_v2_model_members.rs:666-690,723-747, tests/it/checked_package_v2_enum_order.rs:282-297 | /// Trace: FR-038-AC-238 selected_variant_refuses_absent_case_and_empty_declaration; selected_variant_cases_are_bound_to_the_selected_document_digest; ordinary_scalar_enum_still_refuses_nonidentifier_cases |
| Matrix status, 38 edited criteria | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4720-4832 | tagged 34; method-without-symbol AC-193, 194, 201; untagged AC-195 |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS: the gap analysis is clean. The PR introduces no code gap, and each of
the 38 rows it binds to TC-048 has tagged tests or an Inspection procedure
behind it. The one exception is AC-195, whose Clippy evidence is UNRUN; its
own cell says so, and SR-12171 FND-002 records it.
