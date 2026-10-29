---
id: SR-7920
title: IR-739 PR 355 code and Rust review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/derived_keys.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/conformance_qspec/main.rs, tests/it/checked_package_v2_composite_operands.rs, tests/it/checked_package_v2_structural_keys.rs, tests/it/checked_package_v2_union.rs, tests/it/support/checked_package.rs
review_set: subset
---

## Summary

Ticket: IR-739. Reviewed the eight-file Rust and test diff against QSpec FR-322 and IR FR-038-AC-225 through AC-227. The reader selects aggregate-bodied union values for the shared application preimage and excludes them from the structural key pass. No copied fixture or encoder was introduced.

## Verdict

PASS. No production-code or Rust defect found. Test evidence gaps are recorded separately in SR-7921.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reviewed all eight changed files and the reader stage order. The supplied pre-PR receipts show workspace tests and QSpec conformance passed; make ci stops at the unchanged 42-row strict-coverage baseline. No full reviewer gate was run.

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-225 | examined | PLANNED/UNRUN (IR-739). `CheckedPackageV2::read` admits QSpec's current `positive-union-nodes.json`, read directly from an authoritative checkout by `make conformance-qspec`, with both its payload-bearing `Shape::Rect` and nullary `Shape::Empty` aggregate-bodied `value`/`union_value` nodes keyed under `quire.application-node/v1`. The test compares each retained key with an independently fixed QSpec preimage/key vector, its exact aggregate body and the fixture's recorded `package_id`; an ordinary application still admits, while application-free `record_value` and `tuple_value` controls retain s |
| FR-038-AC-226 | examined | PLANNED/UNRUN (IR-739). For each of the two authentic QSpec union values of AC-225, changing its retained key to the prior structural key, rewriting every reference and source-map occurrence of that key, mirroring the identity projection and recomputing only the package identity needed to reach node-key validation, yields `invalid_package`/`stale-node-key` at that node's `node_id`; `locus` is the supplied typed key and `expected_node_id` is the distinct current application-derived typed key. The untampered package admits, so an earlier graph or package refusal cannot pass this control. Chang |
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
