---
id: SR-007
title: "Scope-boundary review of the contract IR v0.1 foundation"
type: SpecReview
analysis: scope-boundary
scope: "spec/index.md, AD-001, AP-001, issue #11"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/11
    type: reviews
---
# Scope-boundary review of the contract IR v0.1 foundation

## Summary

The semantic source of truth is fully owned here. Runtime execution, generated
Rust, SMT queries, temporal logic, Quoin/Quire integration, and project
accreditation are explicitly external.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-007 | low | No parallel downstream expression or identity model is permitted; downstream behavior itself remains out of scope. | spec/index.md, AD-001 |
