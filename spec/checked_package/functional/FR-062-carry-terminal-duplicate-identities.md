---
id: FR-062
title: "Carry both terminal duplicate identities in a typed reader refusal"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-060
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-476
    type: references
---

# FR-062: Carry both terminal duplicate identities in a typed reader refusal

## Description

When the V2 reader finds two admitted `temporal`/`terminal` nodes for one
model, or two `claim` occurrences on one terminal node, its refusal SHALL
identify both conflicting parties through a typed payload. This completes
[FR-060](./FR-060-admit-model-terminal-declarations.md)'s reader output for
QSpec FR-476's one-per-model rule. The existing refusal code, cause, primary
path and locus SHALL retain their specified values.

## Inputs

An otherwise valid V2 package with terminal nodes that pass their individual
body and target checks, and either two nodes referencing one model or one
node carrying two or more `claim` occurrences. The reader compares nodes in
ascending node-id digest order as QSpec FR-476 requires. A `CheckedNodeId`
and `CheckedOccurrence` are the reader's existing public typed identities.

## Outputs

A `CheckedPackageRefusal` with code `ambiguous_declaration`, cause
`ambiguous-name`, a primary RFC 6901 `path`, one primary `locus`, and a
terminal-specific optional typed related-identity payload. The payload is a
closed sum of exactly these cases:

- `Nodes { first: CheckedNodeId, second: CheckedNodeId }`: two distinct
  terminal nodes of one model, first and second in ascending node-id digest
  order;
- `Occurrences { node: CheckedNodeId, first: CheckedOccurrence, second:
  CheckedOccurrence }`: two `claim` occurrences of one terminal node, the
  first two in that node's admitted occurrence order.

The payload is present exactly for the two terminal-duplicate refusals and
absent on every other refusal, including another cause of
`ambiguous_declaration`/`ambiguous-name`. The two cases cannot be confused
by a caller or collapsed to one node or ordinal. The payload is read-only
diagnostic data: it changes neither the package wire nor identity.

## Behavior

The V2 reader SHALL construct the `Nodes` case when QSpec FR-476's final
one-per-model check finds a second distinct terminal node. `first` SHALL be
the earlier node and `second` the node whose check fails. The refusal's
primary `path` SHALL point at that second node. Its `locus` SHALL be
`second`. The payload SHALL also retain `first`. For a single terminal node
with two or more `claim` occurrences, the reader SHALL construct the
`Occurrences` case with the node key and the first two distinct occurrences.
Its `path` SHALL point at that node. Its `locus` SHALL be that node key. A third
or later occurrence cannot replace either selected occurrence. These
refusals retain `ambiguous_declaration`/`ambiguous-name` and FR-476's
per-node-before-uniqueness ordering.

The related identities SHALL not be encoded as message prose, another JSON
pointer, a second `locus`, `related_path`, `document_pointer`,
`expected`/`actual`, or an extension detail. `document_pointer` remains the
FR-038 field for a selected document's number or a held extension
definition; it SHALL be absent for both terminal-duplicate cases. No
compatibility reader or fallback interpretation is introduced.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-062-AC-1 | With two individually valid terminal nodes on one model and recomputed identities, the reader refuses `ambiguous_declaration`/`ambiguous-name` at the second node in ascending node-id digest order, with `locus` equal to that node and typed `Nodes { first, second }` containing exactly both distinct node ids in that order. The same model with either node alone admits. | Test |
| FR-062-AC-2 | One terminal node with two `claim` occurrences and matching source-map entries refuses `ambiguous_declaration`/`ambiguous-name` at the node with typed `Occurrences { node, first, second }`; the two occurrence values equal the first two checked occurrences, and a third occurrence does not replace them. One occurrence admits. | Test |
| FR-062-AC-3 | The related payload is absent on a nonterminal `ambiguous_declaration`/`ambiguous-name` refusal and on each earlier strict-wire, graph-shape, key, source-map, body or target refusal; a terminal node with a wrong target and a duplicate still reports the target defect first with no related payload. | Test |
| FR-062-AC-4 | A public consumer using only `CheckedPackageRefusal`, `CheckedNodeId` and `CheckedOccurrence` can exhaustively distinguish the two related-identity cases without parsing prose, node-body JSON or paths. Each terminal-duplicate refusal retains its primary code, cause, path and locus and has no `document_pointer`, `related_path`, `expected`/`actual` or extension detail. | Test |

## Dependencies

- [FR-060](./FR-060-admit-model-terminal-declarations.md) owns terminal
  admission, the uniqueness check and its primary refusal code and locus.
- [FR-038](./FR-038-consume-checked-package-v2.md) owns
  `CheckedPackageRefusal` and its public typed fields.
- `ix://agent-ix/quire-specification/FR-476` requires both node identities
  or both occurrences to be named on a duplicate terminal refusal.

## Status

Planned. The current refusal type has one `locus` and no typed terminal
duplicate pair; this requirement claims no implementation evidence.
