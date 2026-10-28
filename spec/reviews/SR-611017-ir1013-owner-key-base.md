---
id: SR-611017
title: "Base review of FR-058 owner-bound claim input"
type: SpecReview
analysis: base
scope: "spec/checked_package/functional/FR-058-admitted-clause-execution-point.md; spec/checked_package/matrix/TC-462-admitted-clause-execution-point.md"
review_set: base
---
# Base review of FR-058 owner-bound claim input

## Summary

The changed requirement and test procedure preserve all ten acceptance-criterion rows and the four refusal codes. CG and QSL source paths carry node and occurrence separately, so the proposed owner-bound key is available to consumers. The public-call shape ambiguity was corrected in the same PR; the scoped review passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | RESOLVED in the same PR: FR-058 Inputs now requires one third argument `claim: (&CheckedNodeId, &CheckedOccurrence)`, and TC-462's public compile control asserts exactly the package receiver, selected clause id and this tuple. AC-6's three-input wording and AC-5's exhaustive reference types remain literal; the same-valued other-node case is still tested. | FR-058-AC-3; FR-058-AC-5; FR-058-AC-6; FR-058-AC-10; TC-462 |
