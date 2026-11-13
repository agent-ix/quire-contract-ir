---
id: FR-362
title: "Read source-declared object types and leave packages without them unchanged"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-363
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-364
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-365
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-366
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-367
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-368
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-569
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---

# FR-362: Read source-declared object types and leave packages without them unchanged

## Description

A source-declared object type is a `model`/`object_type` node that carries a
`declaration` and a `SourceOwner` (QSL ADR-013 QC-30 item 2a) instead of an
owner that joins a selected domain document. The strict V2 reader SHALL admit
such a type, and the six reader changes of
[FR-363](./FR-363-join-protocol-owner-model-to-source-declared-type.md) through
[FR-368](./FR-368-project-source-operation-anchor-witness.md), as one change set
under QSL ADR-013 QC-30 item 4. A package that declares no source object type
SHALL read, encode and refuse exactly as it did before this change set.

## Inputs

A checked-package V2 byte string or typed graph, with or without nodes whose
owner is a `SourceOwner`, supplied to the reader of
[FR-038](./FR-038-consume-checked-package-v2.md) or the encoder of
[FR-051](./FR-051-encode-checked-package-v2-envelope.md).

## Outputs

The admitted package, or the typed located refusal the reader gives today.

## Behavior

A node is a source-declared object type exactly when it is a
`model`/`object_type` node whose `owner` is a `SourceOwner` and whose
`declaration` is present. Every source-object rule of FR-363 through FR-368
SHALL apply only to such a node or to a row, reference or anchor that names
one. Where a package has no such node, no rule of FR-363 through FR-368 is
evaluated and no new code, cause, locus, charge or ordering is introduced.

The reader SHALL NOT join a source-declared node, directly or by name, to a
selected domain document or a selected model row, and SHALL NOT match a source
operation to a model operation because their names agree.

The shapes of the nodes, rows and members above are owned by QSL ADR-013 QC-30
item 2 and QSpec FR-322, FR-464, FR-494 and FR-496. This requirement adds no
wire member and no wire shape.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-362-AC-1 | Every package of the existing TC-048, TC-228, TC-229 and TC-463 fixtures and every published QSpec checked-package fixture, none of which declares a source object type, reads to the same admitted package, the same typed reads, the same canonical bytes on re-encode, and the same refusal code, cause and locus for each refused variant, as on the commit before this change set. | Test |
| FR-362-AC-2 | A checked package that declares one source object type with one field and one operation, with every other source-object construct absent, admits through the public reader and exposes the type as a source-declared node with its `SourceOwner`. | Test |
| FR-362-AC-3 | A source-declared type is never resolved against a selected domain document: a package whose lock selects no domain document admits the type, and a package with a source type and a selected model of the same name keeps two distinct nodes with distinct owners. | Test |

## Dependencies

- FR-038 and FR-051 own strict admission and the encoder.
- QSL ADR-013 QC-30 item 4 is the complete list of reader changes.
