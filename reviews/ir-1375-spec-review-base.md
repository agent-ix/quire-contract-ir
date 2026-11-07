---
id: SR-2026110602354639
title: "IR-1375 spec-review/base review"
type: SpecReview
analysis: base
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

PASS — changed FR-353, FR-357, and FR-358 language is consistent and testable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
