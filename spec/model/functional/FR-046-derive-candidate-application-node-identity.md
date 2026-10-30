---
id: FR-046
title: "Construct a structural application node identity before reader admission"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: traces_to
---
# FR-046: Construct a structural application node identity before reader admission

## Description

When a producer assembles a V2 application before reader admission, the model crate SHALL expose the typed structural node-id value of QSpec FR-322 and require the complete package to pass the strict reader before it is treated as admitted. The former content-derived candidate helper is removed; no compatibility helper or hash is retained.

## Inputs

A package name, application node tag and semantic form, ordered path operands, and graph-order ordinal when a recursion group requires it. An operand is a string or nested structural node id. The complete draft graph and body are supplied to the package encoder and reader, not to an identity hash function.

## Outputs

A typed `CheckedNodeId { package, path, ordinal }` may be constructed before admission. It is a value, not an admission verdict. The reader returns an admitted package or its existing typed refused/incomplete outcome.

## Behavior

The model crate SHALL export the structural `CheckedNodeId` by name. Its package and path are explicit and its ordinal is bounded by QSpec FR-322. Ordinary ungrouped applications use ordinal zero and derived path `[node_tag, semantic_form, ...operands]`. A recursion-group member uses its declared path and graph-order ordinal. The producer keeps references and dependencies consistent with the final ids. The reader validates closed shape, uniqueness, reference resolution, owner joins, body grammar and operation semantics; it does not hash the draft or authenticate content-derived keys. No candidate helper constructs an admitted package.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-046-AC-1 | An external model-crate consumer constructs a typed structural application id before any package read, encodes a complete package and obtains the same id only after strict reader admission. A raw id alone cannot construct `CheckedPackageV2`; the root bridge exports none of the model crate's V2 identity surface. | Test, Inspection |
| FR-046-AC-2 | Withdrawn by the IR-1141 structural-identity decision: an application path is an identity label; the reader does not recompute it from body, semantic type or other node members. The former every-member-changes-digest proof has no replacement criterion and this id is not reused. | Withdrawn |
| FR-046-AC-3 | A group containing application and nonapplication nodes uses every member in graph order. Self and internal references resolve by the exact group ordinal; outside and dependency references retain their complete structural ids. Permuting members changes group ordinals while preserving the specified graph order. | Test |
| FR-046-AC-4 | A missing group member, repeated structural id, wrong group size or ordinal, unresolved target, nonapplication body in an application form, or mismatched body target returns its exact typed reader refusal without an admitted package. A 100000-node chain and deep in-memory body do not overflow a 256 KiB stack during the applicable iterative reader and encoder walks. | Test, Inspection |
| FR-046-AC-5 | Canonical envelope encoding at its exact byte limit succeeds, one byte lower refuses with the typed encoder limit and no bytes; an unencodable value returns the typed encoder error. The byte ceiling makes no claim to bound caller-owned graph memory. | Test, Inspection |
| FR-046-AC-6 | Withdrawn by the IR-1141 structural-identity decision: the former candidate-key, retained-body stale-key and expected-key proof does not apply to structural node ids. The reader still validates bodies, references and operations under their own criteria; this id is not reused. | Withdrawn |

## Dependencies

FR-038 and QSpec FR-322 own structural ids, graph validation and the reader. FR-019 owns the public inventory; FR-051 owns typed envelope construction. The QSpec #517 structural-id ruling treats derived paths as identity labels and requires no reader recomputation; the historical AC-2 and AC-6 are retained as tombstones.
