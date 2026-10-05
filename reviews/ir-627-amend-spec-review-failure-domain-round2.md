---
id: SR-1588
title: "failure-domain review, disposition round 2, of quire-contract-ir PR #298 (IR-627 final skip predicate)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@4f953d9434d0eae651de39a2934543c9e16b34cd; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Decided: skip; Stated soundness limit; Consumer consequence; FR-038-AC-145 to AC-149)"
review_set: subset
---

## Summary

Ticket: IR-627. Re-attack of the final predicate at 4f953d9. The predicate skips
a node only when all of these hold:
- its wire form is one of the six skippable forms;
- it carries a label;
- it lies on a cycle of the names graph (IR's cycle graph less `dependencies`);
- every node of its component carries that label.

Each attack, and what stops or bounds it:

| Attack | Result |
|---|---|
| Lone label | Closed: the node is on no cycle, so it is verified. |
| Self-dependency | Closed: `dependencies` is not in the names graph. Without the label, FR-038-AC-18 refuses it first. |
| Label on `integer_range`, `boolean`, `integer` or `reference` | Closed: these forms are always verified. |
| Forged cycle through skippable forms only | Admitted, as the stated limit (AC-148). |
| Retag of an `Int[0, 1000]`-keyed node to `collection_bounds` | Admitted, as the stated limit (AC-148), deferred to AC-150. |

The retag limit is acceptable. A consumer that dispatches on form reads no range
from the retagged node. CG's planned IR-628 route reads member types from the
retained table built from the domain document, not from node bodies. The field
read's `result_type` check (operations.rs:1868) compares keys only, so the
retag row's admission holds.

AC-145, AC-147, AC-148 and AC-149 follow from the predicate.

One residual precision gap remains, recorded below. AC-146 row 3 is still open
under SR-1587 FND-002.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | low | The "Consumer consequence" paragraph names only a `collection_bounds` count as reaching a consumer through a forged group. A skipped collection's body is not checked for closedness (item 3 is part of the skipped stage), so a tamperer can also add a body `reference` to another, genuinely keyed `integer_range` node. An element range then reaches a consumer that reads the element through the graph. The limit paragraph covers this as "the element type of an `option` or a collection", but the consumer paragraph should say it too. Note that under a bounds-required profile both routes hit FR-038-AC-41, because the forged group labels a `composite_type`. | FR-038 Consumer consequence, FR-038 Stated soundness limit | wrong-requirement |

## Dispositions

Round 3, reviewed at 861f997a66b00602a662925bb2be68f63f8da17e.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 861f997: the Consumer consequence now names the element-range route (a skipped collection's body re-pointed, or given a second reference, at another genuinely keyed range) and says FR-038-AC-41 catches both routes under a bounds-required profile and neither under any other. |
