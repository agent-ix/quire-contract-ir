---
id: SR-057
title: "Risk and complexity review of issue 64 native temporal TL correspondence"
type: SpecReview
analysis: risk-complexity
scope: "FR-026 and its external semantic authorities"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: reviews
---
# SR-057: Risk and complexity review of issue 64

## Summary

FR-026 remains high technical risk because it claims exact semantic
correspondence across independently versioned native and TL contracts. Snapshot
`558c4dc` bounds that risk with immutable identities, authority-bound positions
and progress, closed support tables, explicit correction relations, and
fail-closed admission.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6451 | high | **Mitigated:** equal formula bytes or equal Booleans could mask different clock, capture, history, position, closure, or settlement semantics. The correspondence and join now bind exact content revisions and TC-039 carries semantic discriminators and one-axis mutations. | FR-026 Formula construction; Valuation; Progress and result joining; TC-039 | missing-requirement |
| FND-6452 | high | **Mitigated:** a foreign or replayed valuation/progress assertion could settle the wrong scope. Observation positions are injectively bound to FR-025 evaluation identities, and progress binds clock, subject, both scopes, source set, interval, and history boundary. | FR-026 Valuation; Public v1 records | missing-requirement |
| FND-6453 | medium | **Mitigated:** late contradictions could silently restamp settled history. Selected views distinguish original, superseding, and invalidating relations and validate predecessor, corrected input, and contradicted premise while retaining old bytes. | FR-026 Progress, closure and result joining | missing-requirement |
| FND-6454 | medium | **Controlled:** the state space across profiles, positions, bounds, contracts, result axes, and corrections is large. Closed enums, deterministic precedence, 94 allocated cause codes, two operation diagnostics, and a bounded TC-039 corpus constrain implementation. | FR-026 Public v1 records and cause allocation; TC-039 | missing-requirement |

## Risk Register

| Req | Technical risk | Volatility | Primary control |
|---|---|---|---|
| FR-026 | high | medium | immutable public selections, authority-bound identities, exact differential vectors, fail-closed readers, dependency-ordered implementation |

## Result

**PASS after specification mitigation.** High risk remains a reason to require
the planned differential and mutation evidence, not permission to prototype
around a missing authority.
