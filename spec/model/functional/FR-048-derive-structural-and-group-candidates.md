---
id: FR-048
title: "Assemble structural node and recursion-group identities"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-046
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-058
    type: references
---
# FR-048: Assemble structural node and recursion-group identities

## Description

When a producer assembles a CheckedPackage V2 graph, it SHALL use QSpec FR-322's structural node ids with producer-assigned ordinals and shared string recursion-group labels. The reader SHALL validate the complete graph. The former derived-key and group-label candidate helper is removed without a shim or alternate hash path.

## Inputs

A complete ordered graph of typed V2 node drafts, with each node's explicit `{package, path, ordinal}`, body, owner, declaration and references. The package name and selected dependency names are explicit; no byte or work ceiling is accepted by an identity-derivation operation.

## Outputs

The typed graph may be encoded and submitted to `CheckedPackageV2::read`. Only the reader returns an admitted package or typed refusal/incomplete result. No candidate key or group-label object is returned.

## Behavior

A declared node's path equals its declaration path, including when several declared nodes with distinct paths share one recursion-group label; an ungrouped derived node's path is `[node_tag, semantic_form, ...operands]`, each operand a string or nested node id. An anonymous derived group member uses the sole declared member's path when exactly one exists, or a producer-chosen path when zero or multiple exist. No grouped path embeds operands. The structural ordinal is producer assigned and unique only among nodes sharing package and path, independent of graph order. The reader checks all graph ids for shape and uniqueness, joins owners, resolves references and dependencies, checks group membership and body semantics, and retains caller limits on the encoder and reader. It performs no structural preimage construction, local-member hash, label hash, key recomputation or stale-key refusal.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-048-AC-1 | An external model-crate consumer constructs ordered structural ids, graph nodes and a complete envelope, then reads them through `CheckedPackageV2::read`; the raw graph is not an admitted package. The model crate's public inventory contains no candidate-key or group-label helper, and the root crate exports none. | Test |
| FR-048-AC-2 | PLANNED/UNRUN (IR-1141). Owner-free reference, option, collection and bounded-domain nodes and source-owned or model-owned declarations admit with their QSpec structural ids and exact bodies. Changing an owner, declaration, semantic type, body or outside reference is checked by the applicable owner, path, body or graph rule; a changed path component changes the structural id. | Test |
| FR-048-AC-3 | PLANNED/UNRUN (IR-1141). A group with declared structural and application members, including TC-946 K14's three-declared-member cycle and an off-cycle member, retains each declared member's own path under one shared label; an anonymous derived member has the sole declared member's path or a producer-chosen path for zero or multiple declared members. Permuting the group retains every structural id and exact reference while changing graph-array order; a body or reference change is checked by its own graph/body rule. A repeated id, invalid label, mismatched cyclic-component label or unresolved exact reference refuses at its independent locus. | Test |
| FR-048-AC-4 | Repeated ids, unknown tag/form, malformed closed body, invalid owner shape, invalid or mismatched group label or out-of-range structural ordinal return distinct typed reader refusals and no admitted package; no invalid draft is silently normalized. | Test |
| FR-048-AC-5 | Canonical envelope encoding succeeds at its exact byte ceiling and fails with a typed no-bytes result one byte below it. Reader work exhaustion names the limit and failing position. A 100000-node in-memory graph and deeply nested body are traversed iteratively without stack overflow; no second graph copy, identity hash or reader hash work charge exists. | Test |

## Dependencies

FR-038 and QSpec FR-322 own the reader, structural identity and group rules. FR-019 owns the public inventory, and FR-051 owns canonical envelope encoding. The QSpec #517 structural-id ruling treats derived paths as identity labels and requires no reader recomputation. Body and graph checks apply independently.
