---
id: SR-7881
title: IR-739 base review of union value key requirements
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; IR-739
review_set: subset
---

## Summary

The three new AC IDs are unique and mapped to TC-048 as planned evidence. The positive, adverse, group, byte-limit and refusal-order oracles are specified.

## Scope

- `FR-038-AC-225` (examined): PLANNED/UNRUN (IR-739). `CheckedPackageV2::read` admits QSpec's current `positive-union-nodes.json`, read directly from an authoritative checkout by `make conformance-qspec`, with both its payload-bearing `Shape::Rect` and nullary `Shape::Empty` aggregate-bodied `value`/`union_value` nodes keyed under `quire.application-node/v1`. The test compares each retained key with an independently fixed QSpec preimage/key vector, its exact aggregate body and the fixture's recorded `package_id`; an ordinary application still admits, while application-free `record_value` and `tuple_value` controls retain s
- `FR-038-AC-226` (examined): PLANNED/UNRUN (IR-739). For each of the two authentic QSpec union values of AC-225, changing its retained key to the prior structural key, rewriting every reference and source-map occurrence of that key, mirroring the identity projection and recomputing only the package identity needed to reach node-key validation, yields `invalid_package`/`stale-node-key` at that node's `node_id`; `locus` is the supplied typed key and `expected_node_id` is the distinct current application-derived typed key. The untampered package admits, so an earlier graph or package refusal cannot pass this control. Chang
- `FR-038-AC-227` (examined): PLANNED/UNRUN (IR-739). In a reader-admissible recursion group containing a `union_value` and a nonapplication node, the union value's preimage uses the complete graph-order `{size, ordinal}` and rewrites each reference to an in-group member in its nested payload aggregate to `group_reference`, while an outside reference remains unchanged; the current keyed package admits and a retained stale key refuses with the distinct production-derived expected key. At the private key-stage seam, for the same preimage, a canonical byte ceiling equal to its independently measured byte length accepts its ke
- `TC-048` (examined): PLANNED/UNRUN (IR-739). Read the authoritative QSpec checkout's `proposals/checked-package-v2/fixtures/positive-union-nodes.json` through `make conformance-qspec` and the production `CheckedPackageV2::read` path. Require the complete package to admit with its recorded `package_id`. Select
- `FR-038` (examined): [FR-046](../../model/functional/FR-046-derive-candidate-application-node-identity.md) owns the implemented public pre-read candidate-key helper. The reader and helper shall share the authoritative application preimage construction and canonical encoder. This sharing retains the reader's existing authenticated all-node graph-order context, refusal translation, and one work-unit charge before preimage construction and hashing; it does not run public draft-context

## Verdict

**PASS** — No defect found in this lens.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
