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

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-053 now has structured depends_on edges to QSL FR-094 and FR-084, with dependency ownership also stated in the specification. |
### Disposition evidence

FND-001 after excerpt: `  - target: ix://agent-ix/quire-spec-language/FR-094
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-084
    type: depends_on`

