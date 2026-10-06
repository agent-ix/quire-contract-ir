---
id: SR-1921
title: IR-646 PR 304 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir@770969577284fc1b2b063805f1d20cf51e751125; crates/quire-contract-model/src/checked_package/v2/encode.rs,
  crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/owner.rs,
  spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md,
  spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md,
  tests/it/checked_package_v2_abstraction_relation.rs, tests/it/checked_package_v2_adr002_members.rs,
  tests/it/checked_package_v2_dependency_reference.rs, tests/it/checked_package_v2_frame_entries.rs,
  tests/it/checked_package_v2_identity_digests.rs, tests/it/checked_package_v2_model_fields.rs,
  tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_owners.rs,
  tests/it/checked_package_v2_parameters.rs, tests/it/checked_package_v2_reader.rs,
  tests/it/checked_package_v2_temporal.rs, tests/it/checked_package_v2_union.rs, tests/it/main.rs,
  tests/it/support/checked_package.rs
review_set: subset
---

## Summary

Ticket: IR-646. The computed Quoin matrix tags AC-89 and AC-153..155, but one TC-228 clause-owner branch lacks an assertion.

## Verdict

**CONDITIONAL** — The computed Quoin matrix tags AC-89 and AC-153..155, but one TC-228 clause-owner branch lacks an assertion.

## Coverage

Plan completion: not assessed. Quoin matrix: 310 criteria; 259 tagged, 49 untagged, 2 method-without-symbol. AC-89 and AC-153..155 are tagged. This review evaluates the PR diff; repository-wide existing gaps are outside this verdict.

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-89 | examined | Every identity digest of an unchanged wire shape recomputes to the value an in-repo fixture recorded before the move to `quire-canonical`: the owner-free nominal fixture keeps exact recorded node keys, every lowered `ir_id`, lowered package `package_id` and canonical bytes. Nodes newly bearing a mandatory owner under AC-153 are checked for owner/projection equality and key recomputation in TC-228; their pre-owner golden is inapplicable. QSL-638's emitter output supplies the authoritative owner-bearing golden in a subsequent conformance row, which remains explicitly planned until that output |
| FR-038-AC-153 | examined | A declared record, tuple and function each carries its source owner in the node and equal identity projection; QSpec's two-owner packages for `Point` and recursive `List` admit with distinct `Point` ids, distinct `List` group labels and member ids, and equal builtin `Integer` and application-keyed `three` ids under the two owners. An undeclared model declaration and clause function each carries its model owner, while an anonymous type, `source_locus` node and application-keyed node carry no owner. |
| FR-038-AC-154 | examined | Omitting a required node or projection owner, inserting `null`, using the wrong owner kind, adding `version` to `ModelOwner`, or placing an owner on an owner-free node refuses `malformed_wire` at the node or projection object that lacks a required owner and at the present `owner` otherwise, before identity validation. A well-shaped projection owner differing from its node's owner refuses `stale_dependency` at the first differing projection value. The reader does not reconstruct an omitted owner from an occurrence, source map or lock. |
| FR-038-AC-155 | examined | Before structural key re-derivation, a `SourceOwner` absent from `lock.sources` refuses `missing_declaration`/`missing-selection` at its node's `node_id`; a source-map region of its `declaration` occurrence naming another source pair refuses `invalid_package`/`invalid-value` at that source-map entry even when both pairs are lock-selected. A `ModelOwner` whose `identity` is unselected, whose `node` names no declaration, or whose `node` names the wrong kind refuses `missing_declaration`/`missing-selection` at its node's `node_id`, even if nothing reaches that node: a relationship cannot back |
| TC-048 | examined | For the owner-free nominal fixture, recompute every node key and package_id and compare each with the digest the fixture recorded before the move; lower every node and compare each ir_id, lowered package_id and canonical bytes. |
| TC-228 | examined | Give an unreachable clause function an absent operation member, then a field as its owner node; as positive controls, use an operation member's clause and an object type's invariant. |
| owner.rs | examined | validate_owner_schema and validate_owner_joins enforce owner shape, selected source/model joins, subtype and clause matching, digest ordering, and visit charge. |
| v2-mod.rs | examined | CheckedNodeOwner is a closed tagged enum; owner checks run ahead of identity and stale structural-key validation. |
| TC-228-tests | examined | Nine owner admission and refusal tests exercise schema, source and model joins, clause kinds, order, and work charge. |
| TC-048-tests | examined | Owner-free nominal fixture retains exact pre-owner golden; mixed fixtures compare unchanged owner-free nodes and recompute package digest from canonical bytes. |
| encode.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_abstraction_relation.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_adr002_members.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_dependency_reference.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_frame_entries.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_model_fields.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_model_members.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_parameters.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_reader.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_temporal.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package_v2_union.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| main.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |
| checked_package.rs | examined | PR diff adapts owner-bearing fixtures, assertions, or wire encoding. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-228 requires an unreachable clause function owned by an absent operation member to refuse missing_declaration/missing-selection. The clause test covers a selected operation, a selected object type, and wrong-kind field/object/operation owners; it never names a missing operation member, leaving that lookup branch unverified. | tests/it/checked_package_v2_owners.rs:467 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed db4487f514562739017f8914b03c14d76d0d698c | The TC-228 clause table now includes an absent operation member, keeps the clause node unreachable, and asserts missing_declaration/missing-selection at its node_id. |
