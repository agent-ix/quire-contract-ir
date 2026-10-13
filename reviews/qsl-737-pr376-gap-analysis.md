---
id: SR-8691
title: "QSL-737 quire-contract-ir PR 376 gap analysis: FR-347 criteria to TC-448 tests"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@d0b83ff93fdea368f23de95da2b781fd9f7b9e6c; spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md; tests/it/checked_package_v2_operation_arguments.rs; crates/quire-contract-model/src/checked_package/v2/"
review_set: subset
---

## Summary

Ticket: QSL-737. This is a planless gap analysis of FR-347-AC-1 to AC-14
against the TC-448 tests in `tests/it/checked_package_v2_operation_arguments.rs`.
Plan completion: not assessed. Reviewer-only: no build. Coverage comes from
the coder's logs and earlier snapshots, not a new run.

Mapping (one test per criterion; each tagged `/// Trace:` plus
`#[trace("TC-448", "FR-347-AC-n")]`):

| AC | Test | Would it fail if the change were reverted? |
| --- | --- | --- |
| AC-1 | `tc_448_a_child_and_a_grandchild_record_are_admitted_at_a_parent_parameter` | yes: TaxedMoney and GiftTaxedMoney admit only by conformance |
| AC-2 | `tc_448_an_unrelated_record_is_refused_at_its_argument` | negative control, code/cause/path pinned |
| AC-3 | `tc_448_an_ancestor_never_conforms_to_a_descendant` | yes, both ways: Gift at `settle` admits, and the ancestor refusals catch a symmetric `conforms` |
| AC-4 | `tc_448_every_other_parameter_type_is_compared_exactly` | catches any widening to Int ranges, Reference, Set or Option |
| AC-5 | `tc_448_ancestry_is_read_from_the_selected_document_only` | source record and foreign document refusals at `arguments/1` |
| AC-6 | `tc_448_the_first_offending_argument_in_position_order_is_reported` | position order and arity first |
| AC-7 | `tc_448_the_receiver_and_the_result_type_stay_exact` | receiver at `arguments/0`, result at `result_type` |
| AC-8 | `tc_448_conformance_charges_every_reachable_supertype_edge_once` | yes: deltas 2/2/4 by bisection over a padded, identical node set; limit total-1 gives `incomplete`/work; the row count at `/lock/model_selections/0` grows by 2 |
| AC-9 | `tc_448_a_record_field_is_read_over_a_conforming_operand` | yes: the exact record read refused at `name` before this change |
| AC-10 | `tc_448_a_case_arm_body_conforms_to_the_result_record` | yes: a TaxedMoney arm at Money admits |
| AC-11 | `tc_448_a_union_payload_conforms_to_the_declared_record` | yes: child payloads admit; `type-mismatch` at the node |
| AC-12 | `tc_448_a_body_root_conforms_to_the_declared_record` | yes: the refusals exist only through `body_root_defect` |
| AC-13 | `tc_448_function_call_arguments_follow_the_callee_parameters` | yes: the arity and type refusals are new |
| AC-14 | `tc_448_a_malformed_parameters_binding_is_refused_at_the_callee` | yes: five shapes at `arguments/0` |

Every refusal oracle pins the code, the cause and the full pointer. The
work-accounting values were checked against `DomainModel::ancestors`, which
charges one unit per popped supertype edge and expands an ancestor once:
GiftTaxedMoney costs 2 at Money and also at TaxedMoney, and D costs 4 at A
(C, A, B, A). The `settle` control (TaxedMoney exact, whose own walk would
cost 1) pins "an exact match charges none": without the exact shortcut its
delta would be 1, not 2.

Coverage baseline: the CI log written at the PR head (worktree at d0b83ff,
clean) ends `42 unbacked row(s)`, with FR-347 14/14. The earlier snapshots
taken for spec PR 375 show main before 375 at 42, and PR 375's head, which
then had 8 ACs, at 52 (42 + 8 + the FR-347 functional row + the TC-448 row).
PR 375 merged with 14 ACs, which makes main 220a2e3 42 + 14 + 2 = 58. So the
coder's "58 to 42" holds. The remaining 42 are the same rows as before PR 375.
`make ci` stops at `make spec` on main as well, so the strict-coverage stop
is not new.

## Verdict

PASS WITH FINDINGS (one low). All 14 criteria are backed by tests that
would fail if the behaviour were reverted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No test pins how far the binder and payload widening goes. Every new case uses a `record_value_type`. No case keeps a `value`/`parameter` typed at a `model`/`object_type` node, or a union payload naming one, refused. If `parameter_defect` or `is_type_node` accepted any `model` node, the suite would still pass. | crates/quire-contract-model/src/checked_package/v2/structural.rs:359-371; crates/quire-contract-model/src/checked_package/v2/operations.rs:1486-1495 |
