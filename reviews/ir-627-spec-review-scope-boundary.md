---
id: SR-1554
title: "Scope-boundary review of quire-contract-ir PR #295 (IR-627 anonymous structural node bodies)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@984c283099ce117b5ab7cba2b8f03fe3d6e5e5bc; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (section 'Anonymous structural node bodies and keys', FR-038-AC-123..130), spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md"
review_set: subset
---

## Summary

Ticket: IR-627. Checked where the PR draws the line between the decided part, the gated part and
other owners, against the consumer the section names (quire-contract-codegen) and the trust root of the
derived type. The decided check reaches only nodes that a model-owned member's `result_type` names.
CG reads ranges from other nodes. The gate holds back forms that need no new preimage. The trust
assumption is not stated, and declared structural nodes fall outside both parts.

## Verdict

Changes requested: one high and two medium findings. The decided/gated split is honest for forms
with no published preimage, and the gated ACs are correctly marked PLANNED with no test. The open
questions route to an upstream owner and do not copy files. SR-1553 FND-001 covers which owner is the
right one.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The gate holds back a hole that can be closed now. CG takes ranges from `bounded_domain` nodes that no model-owned `result_type` names: scalar operand bounds (`check_parameters`/`checked_bounds`, src/oracle/scalar/mod.rs:882, src/kani/generate/scalar.rs:220-260) and state-field domains through the object body's member target (src/kani/generate/frame.rs:775-807). AC-123..127 never reach these nodes. AC-128 gates every re-derivation on Q1..Q4, yet for anonymous `integer_range`, `collection_bounds`, the Boolean/Integer scalars, Reference, Option and the collections, IR already derives the key from the published QSL FR-092/FR-094 preimage with the same function. That needs no new preimage. AC-128's form list also leaves out the scalar and composite forms, so an Option or collection node whose body is redirected stays unchecked even after the gate lifts | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1792-1809, :2452 |
| FND-002 | medium | The trust root of the "derived type" is unstated. The bounds come from the selected domain document, which the caller's evidence supplies under the `model_selections` digest. The check is sound only while that evidence is trusted. AC-123's threat model (package rewritten, `package_id` recomputed) has no row where the tamperer also re-points `model_selections` at a document declaring `Int[0, 10]` and re-keys the node to match. The spec should state the assumption and pin the expected refusal (missing evidence) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1776-1795, :2447 |
| FND-003 | medium | Declared structural nodes are in no requirement. A `bounded_domain`, record, tuple or function carrying a `declaration` and a SourceOwner is also keyed by `quire.structural-node/v1` (QSL FR-092) and is not re-derived either (identity.rs:244-283). The gated text covers "every anonymous structural node" only, so a named range type such as `type Small = Int[0, 9]`, or a declared record, can carry a tampered body with no planned requirement or tracked follow-up | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1796-1800, :2452 |

## Dispositions

Round 1, reviewed at ab860cac3a7162c5eea073d99bb5bcf5433e1237.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ab860cac3a7162c5eea073d99bb5bcf5433e1237 |
| FND-002 | fixed | ab860cac3a7162c5eea073d99bb5bcf5433e1237 |
| FND-003 | fixed | ab860cac3a7162c5eea073d99bb5bcf5433e1237 |
