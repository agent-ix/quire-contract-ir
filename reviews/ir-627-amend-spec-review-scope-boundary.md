---
id: SR-1584
title: "scope-boundary review of quire-contract-ir PR #298 (IR-627 recursion-group amendment)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@a5e2e32ea87a3c649a827caf296ef294d243a583; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Stated soundness limit; Q4; FR-038-AC-138)"
review_set: subset
---

## Summary

Ticket: IR-627. Scope-boundary lens over what the amendment allocates to IR and
what it allocates to QSL, QSpec and quire-contract-codegen. The allocation to QSL
and QSpec is clean. Q4's remaining questions go to the right owners: the declared
member's `SourceOwner` goes to QSL FR-092 and QSpec FR-322, and graph-order versus
content-order ordinals goes to FR-092 "The group order". Every section the
amendment cites exists at fresh `origin/main` and says what the amendment claims:
FR-092 "Recursion groups", "An in-group node's preimage", "The group order" item 5,
"Groups that collide", "The `quire.structural-node/v1` preimage", vector G9,
QSpec FR-143 "Recursion rule", and `proposals/checked-package-v2/node-identity-preimage.schema.json`.
The codegen consequence statement allocates a burden across the boundary that
codegen cannot discharge as written.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | medium | The codegen consequence statement points at the wrong case and hands codegen an unbacked obligation. It says codegen "may rely on" AC-123 only for a node with no `recursion_group`, and names "a bounded collection or integer range inside a recursive record". (1) Under FR-038-AC-41, a `scalar_type` or `composite_type` node declaring a `recursion_group` already returns `requires_bound` under a bounds-required profile, so the legitimate recursive record is not what exposes codegen's field range. (2) The exposure that matters is a `bounded_domain` node that a tamperer labels (SR-1586 FND-001). `is_recursive_type` (lower.rs:728) ignores `bounded_domain`, so lowering passes it to codegen. (3) No codegen requirement or ticket obliges codegen to check `recursion_group` on every node it reads a range through, so the stated reliance has no owner. Fix the skip predicate first (SR-1586 FND-001). Then restate the consequence in terms of what remains unverified, cite AC-41, and either file the codegen obligation with its owner or drop it. | FR-038 Stated soundness limit (line 1899), FR-038-AC-41, crates/quire-contract-model/src/checked_package/v2/lower.rs:728 | wrong-requirement |

## Dispositions

Round 1, reviewed at 49b85f5d96d54b655660117c5fff54ba4973eade.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7f70b5c: The consumer consequence is rewritten: it cites AC-41 and is_recursive_type, places no obligation on codegen, and names IR-624 as the place for a downstream requirement. |
