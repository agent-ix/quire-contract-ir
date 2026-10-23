---
id: FR-349
title: "Retain typed checked-package V2 terms"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---
# FR-349: Retain typed checked-package V2 terms

## Description

When the V2 reader admits a checked package, Contract IR SHALL retain each
semantic node body as a closed, typed term whose variant and members correspond
to the QSpec body production selected by the node's tag and form. The retained
term SHALL let IR consumers distinguish the admitted productions without
looking up the `term` string in an untyped JSON object.

The V2 wire and its identity preimage remain those of QSpec FR-322. The reader and writer consume the single-body wire: only `semantic_graph.nodes`
ships bodies; the package-id projection is derived as FR-038 specifies.

## Behavior

The reader SHALL distinguish literal, reference, application, aggregate and
dependency-reference terms, and binding members nested under aggregates.
The reader SHALL distinguish the frame, abstraction-relation and extension-form bodies
selected by their node kinds. Each variant SHALL retain its production's
required members with typed node identities and closed member kinds where
those types exist in the IR model. An application-node preimage's
`group_reference` is not an admitted wire term and SHALL remain confined to
preimage construction.

The reader SHALL validate the wire grammar and report the existing first
refusal and path before constructing an admitted typed body. Serialization and
canonical encoding of a typed body SHALL yield the same JSON and JCS bytes as
the corresponding QSpec wire body. The reader SHALL preserve the existing
bounded-input and nonrecursive processing of deeply nested untrusted terms.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-349-AC-1 | Reading a conforming V2 package returns a typed body variant for every admitted root production; aggregate members and application arguments expose typed nested productions, including bindings and dependency references. | Test |
| FR-349-AC-2 | Frame, abstraction-relation and extension-form nodes expose distinct typed bodies with their admitted members, and no admitted body is selected by a caller's string-key lookup. | Test |
| FR-349-AC-3 | Each admitted single-body package serializes and canonically encodes to the published QSpec bytes and identity, with each body shipped once in semantic_graph.nodes. Typed-term retention does not change those body bytes or the graph-derived identity. | Test |
| FR-349-AC-4 | A malformed or unknown term still refuses at the same first code, cause and JSON pointer as the existing reader, before an admitted typed package is returned. | Test |
| FR-349-AC-5 | A nested term at the configured input limit is processed without call-stack overflow, and an over-limit term refuses under the existing read limit. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the reader order,
  limits, refusal envelope and current identity projection.
- QSpec FR-322 owns the admitted V2 body grammar, wire members and identity
  preimage.
