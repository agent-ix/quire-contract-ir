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
under QSL ADR-013 QC-30 item 4. The reader and encoder SHALL read, encode and refuse a package that declares no source object type exactly as it did before this change set.

## Inputs

A checked-package V2 byte string or typed graph, with or without nodes whose
owner is a `SourceOwner`, supplied to the reader of
[FR-038](./FR-038-consume-checked-package-v2.md) or the encoder of
[FR-051](./FR-051-encode-checked-package-v2-envelope.md).

## Outputs

The admitted package, or the typed located refusal the reader gives today.

## Behavior

The reader SHALL treat a node as a source-declared object type exactly when it
is a `model`/`object_type` node whose `owner` is a `SourceOwner` and whose
`declaration` is present. The reader SHALL apply every source-object rule of
FR-363 through FR-368 only to such a node, or to a row, reference or anchor
that names one. Where a package has no such node, the reader SHALL NOT
evaluate any rule of FR-363 through FR-368 and SHALL NOT introduce a new code,
cause, locus, work charge or ordering.

The reader SHALL NOT join a source-declared node, directly or by name, to a
selected domain document or a selected model row, and SHALL NOT match a source
operation to a model operation because their names agree.

The shapes of the nodes, rows and members above are owned by QSL ADR-013 QC-30
item 2 and QSpec FR-322, FR-464, FR-494 and FR-496. This requirement adds no
wire member and no wire shape.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-362-AC-1 | The existing suites of TC-048, TC-228, TC-229, TC-463 and TC-472, FR-064's protocol-owner tests and the published QSpec checked-package fixtures pass with no edit to any expectation, including every exact work-charge assertion, and none of those packages declares a source object type. | Test |
| FR-362-AC-2 | A checked package that declares one source object type with one field and one operation, with every other source-object construct absent, admits through the public reader and exposes the type as a source-declared node with its `SourceOwner`. | Test |
| FR-362-AC-3 | A source-declared type is never resolved against a selected domain document: a package whose lock selects no domain document admits the type, and a package with a source type and a selected model of the same name keeps two distinct nodes with distinct owners. | Test |

## Dependencies

- FR-038 and FR-051 own strict admission and the encoder.
- QSL ADR-013 QC-30 item 4 is the complete list of reader changes.
