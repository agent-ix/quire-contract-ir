---
id: SR-2026110602354638
title: "IR-1375 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/equality_projection.rs; spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md; spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md; spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md"
review_set: subset
---

## Summary

IR-1375 PR #939 frozen diff review. Ticket IR-1375. Coder reports focused formatting, workspace check, equality tests 15/15, root integration 1/1, model Clippy, and targeted Quire validation pass.

## Examined scope

- FR-353 admission and refusal wording (examined): shared work, storage and typed refusals.
- FR-357 equality handoff wording and AC-8 (examined): one caller budget through the selected-facts path.
- FR-358-AC-7 (examined): The admitted equality read requires the same caller-owned `NativeJoinBudget` and charges selected decisions, owned identities, row material, QSV work and typed refusal storage before the corresponding inspection or allocation.
- Public signature, callers, tests and code changed by PR #939 (examined).

## Verdict

FAIL — FR-358-AC-7 has a materialization refusal storage gap; tests cover ordinary and zero-storage refusals, but no bounded nonzero materialization cause.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The materialization refusal path is not backed by a test that compares its owned cause size with the charged diagnostic storage; a nonzero budget may return an undercharged payload. | crates/quire-contract-model/src/checked_package/v2/equality_projection.rs:1035 |
