---
id: SR-202611010118
title: "Integrity review of IR-1226 PR 809"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-354-project-admitted-runtime-context.md"
review_set: subset
---

## Summary

Checked consistency and completeness of the added operation projection against the retained model signature and proposed producer fact. The acceptance criterion leaves signature agreement unstated.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-11 needs a disagreement case for ordered parameter names/types and optional result between retained and producer operation signatures | FR-354-AC-11; FR-354:51-60,90-99 |

## Verdict

Changes requested. This is the same underlying defect as base FND-001, recorded in the integrity lens so both methods retain their own artifact.
