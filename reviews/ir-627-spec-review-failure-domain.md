---
id: SR-1556
title: "Failure-domain review of quire-contract-ir PR #295 (IR-627 anonymous structural node bodies)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@984c283099ce117b5ab7cba2b8f03fe3d6e5e5bc; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-123..127), spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md"
review_set: subset
---

## Summary

Ticket: IR-627. Tamper scenarios were run against the decided check (AC-123..127). The direct cases
are closed: `max` 10, `max` 5000, a `min` tamper, a min/max swap, an off-by-one at the `i128`
extremes (a float comparison is correctly ruled out) and a `collection_bounds` `u` off by one. Each
is a catchable mutation, because the comparison is with bounds from the selected declaration, which
the tamperer cannot change without changing the selection digest (see SR-1554 FND-002). Indirect
identity confusion is not closed. The check compares only the `min`/`max` values of the node that
`result_type` names. It does not check that node's `semantic_type`, which node the literal `type`
names by key, or any node reached through a reference.

## Verdict

Changes requested: one high finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Indirect redirection stays admitted. (1) For `Option<Int[lo, hi]>`, `K<Int[lo, hi]>` and an operation parameter typed `Int[lo, hi]` (`dispatch_call` compares argument keys only, operations.rs:1840-1852), no AC applies: the Option or collection node keeps its key while its body reference is pointed at a genuinely keyed `Int[0, 10]` node. (2) For `K<E>[l, u]`, only `min`/`max` are compared, so the `collection_bounds` node's `semantic_type` can be re-pointed at another collection whose element range differs. (3) AC-124's "typed at the `scalar_type`/`integer` node" is matched by form rather than by the derived Integer key, and scalar keys are not re-derived. A consumer following these references still reads a range the declaration does not state | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1776-1792, :2447, :2448, :2450 |

## Dispositions

Round 1, reviewed at ab860cac3a7162c5eea073d99bb5bcf5433e1237.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ab860cac3a7162c5eea073d99bb5bcf5433e1237 |
