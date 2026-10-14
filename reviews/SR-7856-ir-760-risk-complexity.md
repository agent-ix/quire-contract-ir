---
id: SR-7856
title: "Risk-complexity analysis — IR-760 selected population specification"
type: SpecReview
analysis: risk-complexity
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-019-rust-library-interface.md, spec/model/functional/FR-053-selected-population-binding.md, spec/model/matrix/TC-453-selected-population-binding.md, spec/spec.md"
review_set: subset
---

## Summary

The projection depends on a cross-repository source encoding and authenticated occurrence/type association; direct end-to-end QSL fixture verification is planned in TC-453. The base review records the concrete source-association gap.

## Verdict

**CONDITIONAL** — The boundary can be made explicit with the reported clarification.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The highest-risk association is stated but lacks an adverse-source control | spec/model/functional/FR-053-selected-population-binding.md:102 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-2 and TC-453 now require a valid re-keyed package whose parameter semantic_type points to a second population node while the selected node-occurrence extent remains unchanged. |
### Disposition evidence

FND-001 after excerpt: `Two population nodes with the same selected object type but distinct source occurrences retain their own maxima and provenance. A valid package that re-points a parameter's semantic_type at the other population node and re-derives affected keys still returns the extent of the node-occurrence pair named by the selector; the query never follows that parameter pointer. Selecting the other node with the first occurrence returns MissingOccurrence unless that exact pair is source-mapped.`

