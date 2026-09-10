---
id: SR-037
title: "Integrity review of issue 64 native temporal TL correspondence"
type: SpecReview
analysis: integrity
scope: "StR-001, FR-012, FR-023, FR-025, TM-002, ADR-0053"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-037: Integrity review of issue 64

## Summary

The integrity lens checked completeness, consistency, atomicity, external
assumptions, and testability. The remediated requirement has one interpretation:
it decides exact correspondence and refuses every unqualified alternative.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-371 | high | **Closed:** ADR-0053 still describes an unaccepted FRETish source candidate, which could conflict with the native-only owner ruling. FR-025 now states that native Quire is authoritative and issue #57 is output-only. | ADR-0053; FR-025 Description/Dependencies | wrong-requirement |
| FND-372 | high | **Closed:** the initial version claim lacked an exact profile value and result form. FR-025 now fixes `quire.contract.native-temporal-tl-correspondence/v1`, both tagged variants, and their fields. | FR-025 Description/Behavior; FR-025-AC-8 | missing-requirement |
| FND-373 | medium | **Closed:** the capability combinations and endpoint/state boundaries were distributed across prose and could yield divergent implementations. A closed support table and expanded TC-038 population now make the combinations testable. | FR-025 capability table; TM-002 TC-038 | wrong-requirement |

## Completeness and Traceability

| Stakeholder | Requirement | Verification | State |
|---|---|---|---|
| StR-001 | FR-025 | FR-025-AC-1 through FR-025-AC-8 / TC-038 | planned and dependency-blocked |

FR-025 has explicit inputs, outputs, behavior, dependencies, refusal classes,
and verification. It adds no NFR: deterministic identity and portability remain
within the existing semantic-contract qualities, while resource bounds are
inherited from FR-023.

## Consistency and Atomicity

- FR-025 owns one behavior: decide and identify exact native-temporal-to-TL
  correspondence. Predicate checking, temporal evaluation, observation
  authority, FRETish output, evidence retention, and release decisions remain
  separate.
- No fallback is permitted while issue #63 is absent; the eventual resolution
  chain is explicit.
- The two decision variants are mutually exclusive and invalid input produces
  neither, eliminating a third implicit success state.
- The clauses are observable through structural output, deterministic refusal
  dimensions, diagnostics, and differential TC-038 vectors.

## Result

**PASS after remediation.** No conflicting or non-testable interpretation
remains in the reviewed scope.
