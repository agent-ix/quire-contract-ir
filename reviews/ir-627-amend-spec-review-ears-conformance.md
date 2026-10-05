---
id: SR-1585
title: "ears-conformance review of quire-contract-ir PR #298 (IR-627 recursion-group amendment)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@a5e2e32ea87a3c649a827caf296ef294d243a583; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (item 1 of the decided rule; Decided: skip; Gated: in-group re-derivation; the stage-order paragraph)"
review_set: subset
---

## Summary

Ticket: IR-627. EARS lens over the amended and new requirement statements. Amended
item 1 ("When a node ... carries no `declaration` and no `recursion_group`, the
reader shall re-derive ...") is a well-formed event-driven statement. "Decided:
skip" ("shall skip ... if and only if the node carries a `recursion_group`") is a
well-formed state-driven statement with a closed predicate. Its soundness defect
is recorded in SR-1586 FND-001, not here. "Gated: in-group re-derivation" ("Once
the reader can derive ..., it shall re-derive ...") is a well-formed gated
statement. One edited statement is ambiguous.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | low | The stage-order statement now reads "charging one work unit per node, skipping a node that carries a `recursion_group` (FR-038-AC-136)". It does not say whether a skipped node is still charged its work unit. Two implementers would read it differently, and the result is observable at the work limit. AC-136 does not pin it either. Say "charging one work unit per node, skipped or not" (or the opposite), and add the boundary to an AC. | FR-038 stage-order paragraph (line 1953), FR-038-AC-136 | missing-requirement |

## Dispositions

Round 1, reviewed at 49b85f5d96d54b655660117c5fff54ba4973eade.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7f70b5c: Now reads 'charging one work unit per node, skipped or not', pinned by AC-149. AC-149's limit clause is weak: see SR-1587 FND-006. |
