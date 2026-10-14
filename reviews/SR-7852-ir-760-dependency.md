---
id: SR-7852
title: "Dependency analysis — IR-760 selected population specification"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-019-rust-library-interface.md, spec/model/functional/FR-053-selected-population-binding.md, spec/model/matrix/TC-453-selected-population-binding.md, spec/spec.md"
review_set: subset
---

## Summary

FR-053 explicitly depends on reader admission and the public inventory, but it also relies on QSL’s checked population lowering contract without a structured cross-repository edge.

## Verdict

**CONDITIONAL** — The boundary can be made explicit with the reported clarification.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The QSL lowering contract is only named as prose, not as a dependency edge | spec/model/functional/FR-053-selected-population-binding.md:108-115 |
