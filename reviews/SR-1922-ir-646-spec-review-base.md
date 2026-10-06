---
id: SR-1922
title: IR-646 PR 304 spec-review/base review
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir@770969577284fc1b2b063805f1d20cf51e751125; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md
review_set: subset
---

## Summary

Ticket: IR-646. The TC-048 expected-result text contradicts the revised AC-89 golden boundary.

## Verdict

**CONDITIONAL** — The TC-048 expected-result text contradicts the revised AC-89 golden boundary.

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-89 | examined | Every identity digest of an unchanged wire shape recomputes to the value an in-repo fixture recorded before the move to `quire-canonical`: the owner-free nominal fixture keeps exact recorded node keys, every lowered `ir_id`, lowered package `package_id` and canonical bytes. Nodes newly bearing a mandatory owner under AC-153 are checked for owner/projection equality and key recomputation in TC-228; their pre-owner golden is inapplicable. QSL-638's emitter output supplies the authoritative owner-bearing golden in a subsequent conformance row, which remains explicitly planned until that output |
| FR-038-AC-153 | examined | A declared record, tuple and function each carries its source owner in the node and equal identity projection; QSpec's two-owner packages for `Point` and recursive `List` admit with distinct `Point` ids, distinct `List` group labels and member ids, and equal builtin `Integer` and application-keyed `three` ids under the two owners. An undeclared model declaration and clause function each carries its model owner, while an anonymous type, `source_locus` node and application-keyed node carry no owner. |
| FR-038-AC-154 | examined | Omitting a required node or projection owner, inserting `null`, using the wrong owner kind, adding `version` to `ModelOwner`, or placing an owner on an owner-free node refuses `malformed_wire` at the node or projection object that lacks a required owner and at the present `owner` otherwise, before identity validation. A well-shaped projection owner differing from its node's owner refuses `stale_dependency` at the first differing projection value. The reader does not reconstruct an omitted owner from an occurrence, source map or lock. |
| FR-038-AC-155 | examined | Before structural key re-derivation, a `SourceOwner` absent from `lock.sources` refuses `missing_declaration`/`missing-selection` at its node's `node_id`; a source-map region of its `declaration` occurrence naming another source pair refuses `invalid_package`/`invalid-value` at that source-map entry even when both pairs are lock-selected. A `ModelOwner` whose `identity` is unselected, whose `node` names no declaration, or whose `node` names the wrong kind refuses `missing_declaration`/`missing-selection` at its node's `node_id`, even if nothing reaches that node: a relationship cannot back |
| TC-048 | examined | For the owner-free nominal fixture, recompute every node key and package_id and compare each with the digest the fixture recorded before the move; lower every node and compare each ir_id, lowered package_id and canonical bytes. |
| TC-228 | examined | Give an unreachable clause function an absent operation member, then a field as its owner node; as positive controls, use an operation member's clause and an object type's invariant. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-048 still expects every positive fixture to retain its recorded pre-owner package id, while its revised procedure and FR-038-AC-89 limit the old golden to unchanged wire shapes. Owner-bearing mixed fixtures necessarily get new package ids. Restrict this Expected Results sentence to the owner-free nominal fixture and specify recomputation for owner-bearing fixtures. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:106 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed db4487f514562739017f8914b03c14d76d0d698c | The TC-048 expected result now restricts the pre-owner package-id golden to the owner-free nominal fixture and describes owner-bearing identity projection and planned QSL-638 golden. |
