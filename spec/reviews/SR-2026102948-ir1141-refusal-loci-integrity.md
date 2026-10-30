---
id: SR-2026102948
title: "Integrity review of IR-1141 PR #711 refusal loci"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR #711 FR-038, FR-035 and FR-062 diff"
review_set: subset
---

## Summary

The list shape, first-primary rule, byte-stream absence and graph/document-pointer separation are independently testable. The conflicts below prevent a consistent implementation across the shared public type. They are cross-criterion consistency defects, not grounds to narrow an AC.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-369 contradicts FR-038's pre-existing repeated-dependency two-locus rule. | FR-038-AC-33; FR-038-AC-369 |
| FND-002 | high | Removing fields from shared CheckedPackageRefusal contradicts FR-035-AC-18 and FR-062-AC-4; update those criteria in place while preserving the behavior. | FR-035-AC-18; FR-062-AC-4; FR-038-AC-368 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At d7076090, FR-038 Behavior and AC-369 carry repeated dependency identity into the two-loci list. |
| FND-002 | fixed | At d7076090, FR-035 and FR-062 requirements use the shared loci shape and retain their separate exact-pointer and terminal-duplicate controls. |
