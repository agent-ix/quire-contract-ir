---
id: SR-202611010111
title: "IR-1240 PR 792 FR-048 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-1240 PR #792; spec/model/functional/FR-048-derive-structural-and-group-candidates.md"
review_set: subset
---

## Summary

Checked the amended FR-048 member-slot census against closed model enums, existing mapper order, related reader rules, and criterion traceability. The requirement is consistent and testable; the pending nested-member evidence is recorded in the base review.

## Verdict

**PASS** — each closed operation-member variant has the right number and name of identity slots, and the stated order is deterministic.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-048 Behavior, examined: Field, Position, Element, Operation, TypeArgument, RelationshipEnd, Fairness, ProfileOperator, StateClause, and TemporalInterval align with `CheckedOperationMember`; fairness variants align with `CheckedFairnessTarget`.
- FR-048-AC-6, examined: whole-ID callback semantics do not contradict the existing no-cascade and per-slot mapping rule.
- FR-048-AC-7, examined: member-before-result-before-arguments order matches the intended stack schedule for Application terms; the typed mirrors are named without a second callback.
- FR-048-AC-8, context_only: malformed output remains subject to the normal reader refusal and iterative bound.

## Evidence

Model types: `crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:119-229`; current mapper: `typed_terms.rs:1294-1423,1682-1705`; node order: `v2/mod.rs:351`. The changed FR-048 document validates. No applicable AssuranceProfile was found under `spec/`.
