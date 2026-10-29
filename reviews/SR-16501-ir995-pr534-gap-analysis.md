---
id: SR-16501
title: "IR-995 PR 534 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-269, FR-038-AC-273, FR-053-AC-2, FR-056-AC-7/8; selected_population.rs; operations.rs; related TC-048 and TC-453 tests"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---
# SR-16501: IR-995 PR 534 incremental gap analysis

## Summary

Ticket: IR-995. Plan completion: not assessed. The review reconciled the retained-kind caller change with its new test, existing admitted-package tests, and the actual authoritative scanner over both source roots with empty branch rules. No spec or matrix file changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new same-position test calls only `is_model_population(kinds[at])` on a locally supplied enum array. It never invokes the changed `check_relationship_member` caller or an admitted graph whose retained kind disagrees with raw spelling. Reverting the caller to `is_model_population` on raw tag/form (or using a different node's retained kind) leaves this test green; ordinary admitted-package tests also cannot distinguish the two sources because admission keeps spelling and kind aligned. Add a private caller-level disagreement control that asserts the relationship-member result when the linked node's raw spelling and same-position retained kind disagree. | crates/quire-contract-model/src/checked_package/selected_population.rs:494; crates/quire-contract-model/src/checked_package/v2/operations.rs:2445-2454 |

## Coverage

- FR-038-AC-269 and AC-273: existing TC-048 and QSpec conformance tests cover admitted population selection and non-population refusal. The new test's tags do not add caller-level evidence for the retained-kind dispatch.
- FR-053-AC-2: existing selected-population tests cover binding and occurrence separation; the new helper test does not exercise the selection API.
- FR-056-AC-7/8: the real QSL scanner (same binary, both roots, empty rules) reported 268 rows on main and 266 on this head. Exactly the two old `is_model_population` string decisions disappeared; no new rows appeared. Both reports remain red, so AC-8 remains open. The scanner delta independently supports the narrow source claim but does not make the new Rust test exercise its caller.
- Plan completion: not assessed. The final reviewed-head aggregate gate remains pending.

## Verdict

CONDITIONAL. The production typed dispatch and narrow scanner reduction are sound, but FND-001 leaves the new caller behavior without a test capable of detecting a raw-spelling regression.

## Dispositions

| ID | Round | Outcome | Evidence |
| --- | --- | --- | --- |
| FND-001 | 1 | fixed | The same-PR test invokes `check_relationship_member` on a private graph with actual checked relationship owners. Its valid population baseline and raw-scalar/retained-population case both return `None` with the same measured work; its raw-population/retained-IntegerRange case returns `IllTyped`/`OperatorIneligible` at the population member. The graph index is built after inserting both nodes and kinds, so the changed caller reads the tested position. The author's actual-caller bypass mutant made the negative assertion fail, and the restored source passed. The history-preserving merge to adds the disjoint IR-996 main patch; `operations.rs` is byte-identical to. |
