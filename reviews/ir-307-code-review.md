---
id: SR-624
title: "code review of PR 230 (same_type compares resolved operand types)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@c24c6a4f2fba713b35ea577969fa0a7c5f3c0082; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---
# SR-624: code review of PR 230

## Summary

Ticket: IR-307. Code review with the rust-review lane folded in. The PR changes one line in `check_operands`: the `same_type` arm now maps each operand through `operand_type_node` (the type node the operand resolves to) instead of `argument_type_id` (the operand's own reference target). It also adds one unit test. The normative rule is QSpec FR-322 (quire-specification origin/main, "Operation families, groups and constraints"): "`same_type` | The two named operands resolve to the same type node."

What was measured: the new test passes at the PR head. With the one-line fix reverted and the test kept, the test goes red on the admit half ("distinct parameters of one record type must be admitted"). `make ci` passes fmt-check, lint, test and corpus, and fails only at `make spec` with 22 unbacked rows. `make spec` on clean origin/main 3e7935f fails with the same 22 rows. PR #229 (head 65d29c5) merges cleanly with this head (`git merge-tree`), and the merged tree passes the full workspace test suite.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `same_type` is skipped, not checked, when either operand is a `literal` or an inline `application` term. FR-322 gives both a declared type (`literal.type`, `application.result_type`), but `operand_type_node` resolves only `reference` terms. So `enum.eq(x: A, <literal of enum B>)` and `quantity.add(x: m, <application of result s>)` are admitted. This predates the PR: the old `argument_type_id` path returned `None` for the same terms | crates/quire-contract-model/src/checked_package/v2/operations.rs:1390-1403 |
| FND-002 | low | No test pins what `same_type` does with an operand whose type cannot be resolved (a literal or a dangling reference). The refuse half of the new test does not tell the fix apart from the old code, since both refuse `a` against `c`. It only guards against a fix that drops the constraint | crates/quire-contract-model/src/checked_package/v2/operations.rs:2590-2662 |
| FND-003 | low | `argument_type_id` is misnamed and misdocumented. It returns the operand's own reference target, not a type-node id, and its doc says `check_mode_type`/`check_leaves` use it. After this PR its only caller is `operand_type_node`. The name is what led to IR-307 | crates/quire-contract-model/src/checked_package/v2/operations.rs:682-689 |

## Finding Detail

- FND-001: `operand_type_node` returns `None` for every non-`reference` term, and the `SameType` arm collects into `Option<Vec<_>>`, so a single unresolved operand skips the whole constraint. `argument_family` has the same gap, so the family checks skip these terms too. It is pre-existing and not introduced here. Recommend a follow-up ticket that resolves `literal` through `type` and inline `application` through `result_type`, in `operand_type_node` or a sibling helper, so both `same_type` and the family checks cover them.
- FND-002: Add a case with a parameter operand beside a `literal` operand, and assert the current skip (`Ok(None)`), or the refusal once FND-001 is fixed.
- FND-003: Rename it (for example `reference_target`) and correct the doc comment.

## Scope

- `check_operands` `SameType` arm (operations.rs:1047-1057), examined: it now compares resolved type nodes, as FR-322 requires.
- `operand_type_node` (operations.rs:1381-1403), examined: a type-shaped target is taken as itself, and otherwise the target's `semantic_type`. `bounded_domain` wrappers are not reduced, which is correct for "same type node": `Int[0,9]` and `Integer` are distinct nodes.
- `argument_type_id` (operations.rs:682-689), examined: it has no remaining direct use in a comparison (FND-003).
- `argument_family` (operations.rs:655-681), context_only.
- The new test `operation_defect_same_type_compares_operand_types_not_operand_nodes`, examined: it goes red when the fix is reverted.
- Rust lane: no new `unwrap`/`expect`/indexing panics, no `unsafe`, and clippy `-D warnings` and fmt-check are clean.
- PR #229 overlap in operations.rs, examined: it touches different hunks (the `reference_edge` arm and module docs), merges cleanly, and the merged tree tests green.

## Verdict

The fix is correct and minimal. It is the smallest change that makes `same_type` follow FR-322, and it introduces no new false refusal or false admit: for non-`reference` operands, behaviour is unchanged. Mergeable. FND-001 is pre-existing and should be deferred to a follow-up ticket. FND-002 and FND-003 are low.
