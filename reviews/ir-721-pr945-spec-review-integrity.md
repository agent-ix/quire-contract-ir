---
id: SR-2026110705000812
title: "Integrity review of quire-contract-ir#945 tick-monitor chain spec"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR #945 head cc5034a; spec/checked_package/functional/FR-361, FR-357, FR-358; spec/assurance/AD-010; spec/checked_package/matrix/TC-471, tests.md; spec/model/functional/FR-019; spec/model/matrix/tests.md; spec/spec.md"
review_set: subset
---
# Integrity review of quire-contract-ir#945

## Summary

Ticket: IR-721. Checked internal consistency, completeness and testability of
FR-361, FR-357-AC-16/17, FR-358-AC-10 and AD-010 against the existing closure
code and sibling driver #129 / RT #209. Five gaps leave coders with unknowns or
make exact-limit tests disagree across repositories.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-361 places the authority checks "after the existing clock correspondence step and before any producer row join", but the existing read joins producer rows (`read_reached_model_facts`: declarations, fields, tuples, source units, enum rows) before the clock step. The order is unsatisfiable without reordering the read, which changes first-refusal precedence of FR-357-AC-2; the spec must state the new step order. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:120-122; crates/quire-contract-model/src/checked_package/v2/monitor_plan/selected_type_closure.rs:615-652 |
| FND-002 | medium | Receipt byte custody is contradictory: Ownership says the receipt borrows the row, Charging says IR charges one handle plus one storage unit per byte "before the copy", and AD-010 P4 / driver FR-019 P4 also charge the driver for copying the same bytes (then P6 copies again into Layout). Exact-limit measurements (FR-361-AC-4, driver FR-019-AC-2) will disagree depending on which reading the coder takes. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:143-145,152; spec/assurance/AD-010-tick-monitor-admission-lowering-and-execution.md:131 |
| FND-003 | medium | `profile()` ("selection identity text") and the clock-binding `selected_profile_identity` member have no stated encoding; the admitted selection is `CheckedSelection{role, definition{authority, identity}}`, so identity-only, authority/identity or a JSON object are all readings. RT compares these bytes exactly, so two implementations differ. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md:94,99-104; crates/quire-contract-model/src/checked_package/shared.rs:1538-1543 |
| FND-004 | medium | FR-357-AC-16 refuses a bad `key_node` "with a typed cause" but names no `SelectedMonitorTypeClosureErrorCode` variant, and states no exact-allowance success / one-below refusal for the added key-join charges (strict bar: exact-limit admit, one-under refuse everywhere). | spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md:340 |
| FND-005 | low | FR-358-AC-10 states only the denied case for `AdapterRow`, `AuthorityRow` and `PreparationOutput`; it does not state that the exact allowance admits with that locus. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md:173 |

## Verdict

Not mergeable until FND-001..004 are fixed; FND-005 is a nit.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
