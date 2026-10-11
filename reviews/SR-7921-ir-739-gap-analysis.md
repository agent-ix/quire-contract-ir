---
id: SR-7921
title: IR-739 PR 355 gap analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir@0bd9041956f772ccb5c1b22b7ec969c40859b787; crates/quire-contract-model/src/checked_package/v2/derived_keys.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/conformance_qspec/main.rs, tests/it/checked_package_v2_composite_operands.rs, tests/it/checked_package_v2_structural_keys.rs, tests/it/checked_package_v2_union.rs, tests/it/support/checked_package.rs
review_set: subset
---

## Summary

Ticket: IR-739. Quoin matrix binds all three changed criteria to real tests. No untagged criterion, unowned new source branch, or stub was found in the changed slice. Two required adverse controls are absent despite tags, and the existing spec status text falsely calls the now-run tests UNRUN.

## Verdict

CONDITIONAL. Three medium evidence findings must be fixed before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The authentic body-member mutation is exercised only on Rect (node 10). AC-226 says to change one key-covered body member for each of Rect and nullary Empty. A mutant that skips stale-key validation for an Empty body change passes the tagged test. Add an Empty mutation that reaches the key stage, with package identity refreshed, and assert the typed stale-key result and expected digest. | tests/conformance_qspec/main.rs:599-626 |
| FND-002 | medium | The grouped union-value tests prove admission/stale key and private byte/work order, but none pairs its stale key with the specified earlier wire/package/owner/graph defects or later derived/declaration/frame/operation/reference defects. Reordering the new key stage after one of those later stages, or before an earlier stage, can pass all new AC-227 tests. Add the paired public-reader precedence controls named by TC-048. | tests/it/checked_package_v2_union.rs:563-619 |
| FND-003 | medium | TC-048 still labels the three new public-reader controls planned and UNRUN, and each FR-038 AC row also says PLANNED/UNRUN. The exact pre-PR receipts show their tagged integration, model-unit and QSpec tests ran and passed on this head. Once this code PR merges, those status claims become false. In this same PR, change only the evidence/status prose in TC-048 and AC-225 through AC-227 to name the implemented and run controls, preserving the requirements; do not claim broader full-gate or conformance results than the receipts prove. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:72,1683-1685 |

## Coverage

Plan completion: not assessed. Quoin matrix 0.28.3 reports AC-225 bound to two tests, AC-226 to one, and AC-227 to two. QSpec conformance passed 13 tests and workspace suites passed 467 integration and 205 model unit tests. Strict coverage remains at the main baseline of 42 unbacked rows and zero contradictions. Semantic intent/test/code comparison was requested for this PR and performed.

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-225 | examined | PLANNED/UNRUN (IR-739). `CheckedPackageV2::read` admits QSpec's current `positive-union-nodes.json`, read directly from an authoritative checkout by `make conformance-qspec`, with both its payload-bearing `Shape::Rect` and nullary `Shape::Empty` aggregate-bodied `value`/`union_value` nodes keyed under `quire.application-node/v1`. The test compares each retained key with an independently fixed QSpec preimage/key vector, its exact aggregate body and the fixture's recorded `package_id`; an ordinary application still admits, while application-free `record_value` and `tuple_value` controls retain s |
| FR-038-AC-226 | examined | PLANNED/UNRUN (IR-739). For each of the two authentic QSpec union values of AC-225, changing its retained key to the prior structural key, rewriting every reference and source-map occurrence of that key, mirroring the identity projection and recomputing only the package identity needed to reach node-key validation, yields `invalid_package`/`stale-node-key` at that node's `node_id`; `locus` is the supplied typed key and `expected_node_id()` is the distinct current application-derived typed key. The untampered package admits, so an earlier graph or package refusal cannot pass this control. Chang |
| FR-038-AC-227 | examined | PLANNED/UNRUN (IR-739). In a reader-admissible recursion group containing a `union_value` and a nonapplication node, the union value's preimage uses the complete graph-order `{size, ordinal}` and rewrites each reference to an in-group member in its nested payload aggregate to `group_reference`, while an outside reference remains unchanged; the current keyed package admits and a retained stale key refuses with the distinct production-derived expected key. At the private key-stage seam, for the same preimage, a canonical byte ceiling equal to its independently measured byte length accepts its ke |
| TC-048 | examined | ## Aggregate-bodied union value keys (FR-038-AC-225 through FR-038-AC-227)

PLANNED/UNRUN (IR-739). Read the authoritative QSpec checkout's
`proposals/checked-package-v2/fixtures/positive-union-nodes.json` through
`make conformance-qspec` and the production `CheckedPackageV2::read` path.
Require the complete package to admit with its recorded `package_id`. Select
the payload-bearing `Rect` and nullary `Empty` `value`/`union_value` nodes by
tag and form, assert their exact aggregate bodies, and compare each retained
key to QSpec's independently fixed `quire.application-node/v1` preimage bytes
a |
