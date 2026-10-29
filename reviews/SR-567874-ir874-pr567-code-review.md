---
id: SR-567874
title: "IR-874 PR 567 code and Rust review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/flat_wire.rs; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs; tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Ticket: IR-874. Diff-scoped Rust/code review of PR #567. Examined the three changed files and the FR-038-AC-283/284 and TC-048 obligations. The public reader invokes the declaration path pass after operator validation and before local member layout. Queue wire roles and declaration presence are checked before typed reading. The path traversal covers all five specified control containment forms in code.

## Examined units

- **FR-038-AC-283** (`examined`): "Role, channel, finish, terminal, control and compensation declarations use the fixed qualified-name segments of QSpec FR-464; queues have no declaration and generated occurrences, while other protocol members have declaration occurrences. A wrong path or occurrence role refuses at the node."
- **FR-038-AC-284** (`examined`): "The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest."
- **TC-048** (`examined`): "Use QSpec FR-464, FR-465 and FR-466 and the authored tables in TC-946, TC-947 and TC-948 as the current independent expected outcomes."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Public-reader path tests cover only sequence containment. Choice, parallel, repeat and await are new dispatch arms with no independent holder-path refusal and admission cases, so a wrong field selection or skipped edge in those arms can remain green. | tests/it/checked_package_v2_protocol_body.rs:238; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:844 |

## Verdict

CONDITIONAL. FND-001 leaves four new containment branches without direct acceptance evidence. No panic, wire-owner violation, duplicate-order defect or trace overclaim was found in the changed code. Later membership, target/binder joins, catalog join and source-produced vectors remain open under IR-874.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, `choice_parallel_repeat_and_await_hold_controls_at_their_own_paths` independently constructs all four holder layouts, admits each correct tree, and mutates every child path to assert the exact `invalid_semantic_graph` node refusal. The focused public-reader test passed. |

## Current verdict

PASS for the reviewed PR #567. The former work-meter regression was removed by comparing holder-relative path slices without new meter charges; the existing TC-226 exact read-work boundary remains an independent guard. Empty protocol occurrence lists now reach the existing source-map refusal path in `v2/mod.rs` while present wrong occurrence roles remain `malformed_wire` in flat-wire validation. Reviewer focused checks passed at this head: `choice_parallel_repeat_and_await_hold_controls_at_their_own_paths`, `tc_226_tree_whole_read_work_boundary`, `ir_v2_inventory_never_completes_a_node_without_source_occurrences`, and `ir987_scalar_and_result_results_bytes_and_work_match_admitted_fixture` (one test each). The reviewed PR delta passed the whitespace check. The remaining protocol membership, target/binder, catalog and producer-vector stages are outside this slice and stay open under IR-874.

## Final head check

PASS at PR #567. The only change after the prior fixed disposition is in the `protocol_clause_rejects_short_layout_at_wire_boundary` unit fixture. It now supplies the declaration required by the flat-wire protocol form before checking the intentionally short body. This removes the unrelated node-level `malformed_wire` refusal and preserves the intended `/semantic_graph/nodes/0/body` oracle. The exact model unit test passed at this head; `git diff --check` passed. FND-001 remains fixed. No new finding or disposition row was added. Hosted Rust Checks were in progress at review time.
