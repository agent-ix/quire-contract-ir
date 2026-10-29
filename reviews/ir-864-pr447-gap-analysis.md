---
id: SR-12391
title: "IR-864 gap-analysis of IR PR #447"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs, crates/quire-contract-model/src/checked_package/v2/relationship_end.rs, crates/quire-contract-model/src/checked_package/v2/structural.rs, tests/it/checked_package_v2_model_members.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-346
    type: references
---

# IR-864 gap-analysis of IR PR #447

## Summary

Ticket IR-864. Independent and `quire coverage --scope. --strict` outputs are byte-for-byte identical: 527/584 backed, 39 unbacked, zero contradicted. The strict exit remains 1 solely on the same 39 existing unbacked rows. `quire matrix --scope. --format json` has 550 criterion rows at both revisions, with the same statements, methods, statuses and binders after source line and column positions are excluded. The 12 removed doc citations were already represented by adjacent `#[trace]` claims; no binder was lost.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| crates/quire-contract-model/src/checked_package/v2/mod.rs::tc_225_exactly_one_kind_has_an_abstraction_relation_body | examined | `#[trace("TC-225", "FR-346-AC-2")]` |
| crates/quire-contract-model/src/checked_package/v2/model_members/tests/intake_retention.rs::tc_048_intake_retains_actual_nested_declaration_not_owner_or_missing_target | examined | `#[trace("TC-048", "FR-038-AC-174", "FR-038-AC-188")]` |
| crates/quire-contract-model/src/checked_package/v2/relationship_end.rs::tc_048_admitted_retention_is_read_and_outside_package_equality | examined | `#[trace("TC-048", "FR-038-AC-266")]` |
| crates/quire-contract-model/src/checked_package/v2/relationship_end.rs::tc_048_post_admission_mutations_of_an_admitted_application_refuse_unresolved_end | examined | `#[trace("TC-048", "FR-038-AC-265")]` |
| crates/quire-contract-model/src/checked_package/v2/structural.rs::tc_048_structural_integer_grammars_are_exact | examined | `#[trace("TC-048", "FR-038-AC-22")]` |
| crates/quire-contract-model/src/checked_package/v2/structural.rs::tc_048_application_join_collects_targets_and_member_declarations_only | examined | `#[trace("TC-048", "FR-038-AC-23")]` |
| tests/it/checked_package_v2_model_members.rs::tc_048_relationship_end_reads_forward_and_inverse_ends_of_unequal_bounds | examined | `#[trace("TC-048", "FR-038-AC-262")]` |
| tests/it/checked_package_v2_model_members.rs::tc_048_relationship_end_returns_the_declared_bounds_role_and_derived_type | examined | `#[trace("TC-048", "FR-038-AC-263")]` |
| tests/it/checked_package_v2_model_members.rs::tc_048_relationship_end_refuses_every_node_that_is_not_a_reaches_application | examined | `#[trace("TC-048", "FR-038-AC-264")]` |
| tests/it/checked_package_v2_model_members.rs::tc_048_retaining_the_relationship_end_changes_neither_equality_nor_work | examined | `#[trace("TC-048", "FR-038-AC-266")]` |
| tests/it/checked_package_v2_model_members.rs::tc_048_relationship_end_api_is_nine_fields_and_a_closed_error | examined | `#[trace("TC-048", "FR-038-AC-267")]` |
| tests/it/checked_package_v2_model_members.rs::tc_048_relationship_end_is_not_a_field_and_exports_no_selected_population_item | examined | `#[trace("TC-048", "FR-038-AC-268")]` |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No new trace or criterion gap (placeholder) | - |

## Verdict

PASS for the changed evidence: the deletion does not reduce criterion coverage or alter a computed binder.

## Coverage

Plan completion: not assessed. Semantic review: not opted in. Strict coverage retains the repository's 39 pre-existing unbacked rows and no contradicted statuses.
