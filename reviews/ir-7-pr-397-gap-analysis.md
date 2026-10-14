---
id: SR-11033
title: gap-analysis of IR-7 PR 397 fairness declaration test
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; tests/it/checked_package_v2_temporal.rs
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-038
  type: references
---

## Summary

Diff-scoped planless gap analysis of FR-038-AC-103: the computed matrix binds the new test and its assertions exercise the production fairness resolver through the public reader.

## Examined units

- `FR-038-AC-103` (examined): a fairness member whose `declaration` and `name` resolve to an operation of a `model`/`object_type` declaration node admits (FR-370 "Fairness resolution" steps 1 to 3, in that order), one whose `name` is no operation of that node refuses `missing_declaration`/`missing-name` with path `/semantic_graph/nodes/{fairness node}/body/operation/member/name` and the `declaration` target's key as locus, and one whose `declaration` names a node that is not a `model`/`object_type` declaration (a `scalar_type` node, a `model`/`value_type` node, or a key that names no node) refuses `invalid_model_binding`/`
- `tc_048_wrong_declaring_nodes_cannot_name_a_fairness_operation` (examined): A selected systems interface is an authentic model declaration, but it cannot declare the operation named by a temporal fairness member. A value node cannot declare one either (QSpec FR-370-AC-11, TC-303 step 9). ///
- `FR-370-AC-11` (context_only): A fairness member naming an operation its `object_type` declaring node only inherits is admitted and resolves to the most derived redefinition; a name matching two effective operations refuses `ambiguous_declaration`/`ambiguous-name`; a declaring node no selected owner matches refuses `missing_declaration`/`missing-selection`; and a `declaration` naming a `systems_interface` model node or a non-model node refuses `invalid_model_binding`/`malformed-declaration` at the target.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the changed test. No new defect or trace inflation found.

## Coverage

Computed `quire matrix --scope . --format json`: FR-038-AC-103 is `tagged`; the new test appears as a binder at line 1955. Focused `cargo test --locked --workspace --test it tc_048_wrong_declaring_nodes_cannot_name_a_fairness_operation -- --nocapture` passed 1/1. Plan completion: not assessed. This verdict covers the PR diff, not the full repository.
