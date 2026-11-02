---
id: SR-2026110101305
title: "IR-1272 PR 839 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1272 PR 839; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs; tests/conformance_qspec/protocol_mutation_manifest.rs; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Reviewed frozen PR #839 head 59acb0cf5b9a74f6fd209538181864a9ae2d06fd against merged QSpec 6c45e0bc and FR-038-AC-435/436.

## Verdict

CONDITIONAL. The implementation matches the QSpec V13/V14 refusal shapes, scopes the state-clause placement check, and preserves preceding graph and state validation. The tagged tests omit one required structural-locus assertion.

## Reviewed units

- FR-038-AC-435: examined. Every admitted `quire.op.protocol.control` body on a protocol member has `result_type` and node `semantic_type` equal to the package's Boolean scalar-type node; equality of those two references to another type is insufficient. A resolved non-Boolean type refuses `ill_typed`/`operator-ineligible` at the containing protocol node with no admitted package. TC-048 reads the merged QSpec STD-609 protocol-node mutation manifest and base fixture by reference, applies V13 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/64`, with that node as structural locus.
- FR-038-AC-436: examined. A state-clause body reference to a protocol member node other than a `protocol_clause` or `protocol_reference` carrier refuses `ill_typed`/`operator-ineligible` at the containing state-clause node with no admitted package, even when the referenced member is Boolean typed. The protocol step applies this placement rule after earlier graph and state defects. TC-048 reads the same merged manifest and base fixture by reference, applies V14 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/54`, with that node as structural locus.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The V13/V14 tests assert refusal code, cause and JSON path, but do not assert the structural node locus required by FR-038-AC-435/436; a reader returning the wrong node id would pass. | tests/conformance_qspec/protocol_mutation_manifest.rs:70-90 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The V13 and V14 cases now compare refusal.locus with the mutated package node_id at graph indices 64 and 54, respectively. |
