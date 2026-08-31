---
id: SR-008
title: "Failure-domain review of the contract IR v0.1 foundation"
type: SpecReview
analysis: failure-domain
scope: "FR-011 through FR-020; CAC-001"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/CAC-001
    type: reviews
---
# Failure-domain review of the contract IR v0.1 foundation

## Summary

The requirements distinguish malformed wire input, unsupported versions,
ill-typed expressions, potential undefinedness, stale references, orphans,
conformance mismatches, and runner environment failure.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-008 | low | Stable diagnostic and runner-exit requirements cover each declared failure boundary without converting it to success. | FR-012–FR-020, NFR-003 |
