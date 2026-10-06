---
id: SR-2107
title: "Dependency analysis \u2014 IR-661 FR-038 new upstream relationship edges"
type: SpecReview
analysis: dependency
scope: agent-ix/quire-contract-ir@c50050da5f20d4af41ab2dd6ea573d5e4b9abacc; spec/checked_package/functional/FR-038-consume-checked-package-v2.md relationships frontmatter (five new edges); prior 705cef3f7a07370f111f19e8e7d86e1e07fd31d7; base 1540b3b6c0e4d167fe1ed9116c296e45e7dff258; Ticket IR-661
review_set: subset
---

## Summary

Scoped to the five `references` edges round 1 added to FR-038 (QSpec FR-152, FR-154, FR-043; FCD FR-094, FR-095), the only new `relationships:` edges in the diff. Every target exists at the merged revisions (QSpec 60630b0, FCD 68c0acb), and none of the five targets' frontmatter has an edge back into quire-contract-ir, so the edges add no cycle. All five are merged upstream enablement that FR-038 consumes: they are prerequisites already satisfied and do not block IR-661's feature order.

Ticket: IR-661. Method: `spec-review/spec-dependency-analysis`. Reviewer model `claude-opus-5-5`, run `ae1a4470-06bc-44c7-90df-81c70be53842`.

## Verdict

**PASS**: no dependency defect. Classification: FR-152, FR-154, FR-043, FCD FR-094 and FR-095 are Enablement (merged upstream contracts); FR-038's IR-661 amendment is Feature. Edges: each of the five points into FR-038, and the topological order is upstream first (already merged), then the FR-038 amendment. Cycles: none. Observation, not a defect: the edges use `references`, the same verb FR-038 already uses for QSpec FR-322 and QSL FR-094, although FR-152 and FR-043 are now applied normatively by reference; whether to adopt `depends_on` is a repository-wide convention question.

## Scope Examined

- `FR-038#frontmatter-new-edges` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:10-26
- `FR-038#navigation-by-reference` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1071-1078
- `FR-038#reaches-by-reference` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1086-1093
- `QSpec FR-152 frontmatter` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/functional/type-model/FR-152-bind-systems-model-structures.md:2-14
- `QSpec FR-154 frontmatter` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/functional/type-model/FR-154-admit-domain-package-model.md:2-19
- `QSpec FR-043 frontmatter` (context_only) agent-ix/quire-specification@60630b0d3d5e9cca048d1c314675db3bf9c6e4f2:spec/functional/foundation/FR-043-evaluate-finite-graph-relations.md:2-11
- `FCD FR-094 frontmatter` (context_only) agent-ix/filament-core-data@68c0acba2390eb1593cc003d0a3e144638540a86:spec/functional/FR-094-lower-relationships-operations-and-clauses.md:2-19
- `FCD FR-095 frontmatter` (context_only) agent-ix/filament-core-data@68c0acba2390eb1593cc003d0a3e144638540a86:spec/functional/FR-095-mint-package-identity-and-provenance.md:2-18

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
