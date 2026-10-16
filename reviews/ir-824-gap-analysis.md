---
id: SR-11836
title: "Gap analysis of IR-824 reaches over finite multi-valued ends (PR #422)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-252..257 and FR-038-AC-172 against crates/quire-contract-model/src/checked_package/v2/operations.rs and tests/it/checked_package_v2_model_members.rs"
review_set: subset
---

## Summary

Ticket: IR-824. PR agent-ix/quire-contract-ir#422 at 3a79b1f. Plan
completion: not assessed (planless). Each criterion was traced to its tagged
test and checked by mutation in a throwaway worktree.

- FR-038-AC-252 (examined): `tc_048_reaches_finite_multivalued_ends_keep_the_single_valued_node_shape`, both roles, `[0,1]`, `[1,1]`, `{0,3}` unique and not, `{2,3}`; red under the Reference|Option restriction and under a required multiplicity member.
- FR-038-AC-253 (examined): `tc_048_reaches_checks_operand_one_against_receiver_before_edge_defects`; subtype admits, unrelated refuses at `arguments/1` alone and with each edge defect; red when operand 1 is checked last or against the destination type.
- FR-038-AC-254, AC-255, AC-256 (examined): `tc_048_reaches_refusal_order_is_direction_then_multiplicity_then_type`; red for direction after multiplicity, type before multiplicity, unbounded admitted, ordered admitted, and operand 1 against destination type (RN-25 case).
- FR-038-AC-257 (examined): `tc_048_reaches_multivalued_end_requires_boolean_result`; see FND-001.
- FR-038-AC-172 (context_only): row unchanged, supersession pointer to AC-252 retained; its remaining assertions in `tc_048_a_selected_self_relationship_reaches_through_either_named_end` are intact.

No production code without an owning requirement; no stubs. `quire coverage
--strict`: 47 unbacked / 0 contradicted at head and 47 / 0 at origin/main
(no regression; AC-252..257 are not among the unbacked rows). No hash, digest
or pin introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-257's test replaces the Boolean result with `Reference<Order>` (fixture node 3), which is not the navigation result type of the `[1,3]` end under test (`Set<Reference<Order>>[1,3]`). A reader that also accepts the end's own navigation type as a `reaches` result (mutation: return admitted when `result_type` equals `navigation_type`'s key) passes every TC-048 model-member test, so AC-257's "names a navigation result type" refusal is not pinned. | tests/it/checked_package_v2_model_members.rs:1572-1604 |

## Verdict

Mergeable after FND-001 is fixed: author the multi-valued end's own
`Set<Reference<Order>>[l,u]` (and a `Bag` variant) type node as the result in
the AC-257 test, recomputing identities, and assert the `body/result_type`
refusal.
