---
id: SR-10846
title: "Integrity review — IR-759 source-obligation correspondence"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; FR-052, TC-452, FR-019 addition, spec.md index"
review_set: subset
---

## Summary

Identifiers, reference links, FR-019 inventory placement and the spec index validate. Two behaviors lack a determinate contract: completeness of required inventories, and the typed disposition for conflicting assertion facts/provenance. No applicable AssuranceProfile was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The spec requires an incomplete-inventory error without defining how inventory completeness is established | FR-052 |
| FND-002 | medium | Conflicting assertion identities require a typed rejection but no conflict reason is specified or directly tested | FR-052-AC-3, TC-452 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-052 now defines completeness evidence, complete-empty inventories, owner/scope matching, and typed incomplete failures. |
| FND-002 | fixed | FR-052 names `ConflictingRelation` and `ConflictingProvenance`; AC-3 and TC-452 now exercise each separately. |
