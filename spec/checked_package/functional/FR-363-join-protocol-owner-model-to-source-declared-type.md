---
id: FR-363
title: "Join a protocol owner model and role model to a source-declared type"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-064
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-362
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-569
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---

# FR-363: Join a protocol owner model and role model to a source-declared type

## Description

Today a protocol clause's `owner_model` and a role's `model` must resolve
through the owner join of [FR-064](./FR-064-bind-protocol-clause-to-owner-model.md)
to a selected model row, so a protocol `over` a source-declared object type is
refused. The reader SHALL also accept an `owner_model` or role `model` that is
a source-declared object type node, joined through that node's `SourceOwner`
(QSL ADR-013 QC-30 item 3, FR-464, FR-465) to a `lock.sources` entry.

## Inputs

A protocol clause node whose `owner_model`, or a protocol role whose `model`,
names a local `model`/`object_type` node with a `SourceOwner`; the package
`lock.sources` entries.

## Outputs

The admitted clause-to-owner edge for the source type, readable by the
refinement class catalogs exactly as a model owner edge is, or one typed
located refusal.

## Behavior

For an `owner_model` or role `model` that names a node whose owner is a
`SourceOwner`, the reader SHALL join the `SourceOwner` authority and identity
to exactly one `lock.sources` entry and SHALL NOT require a selected model row
or a result of `owners.recover` for it. An absent `lock.sources` entry, a
`SourceOwner` that matches two entries, or a node that is not a
`model`/`object_type` SHALL refuse with the code and cause the model-owned join
gives for a missing or wrong-kind owner declaration, at the same locus (the
clause or role). A model-owned `owner_model` keeps today's join and refusals
unchanged.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-363-AC-1 | A protocol clause over a source-declared type with a matching `lock.sources` entry admits and the typed clause exposes the source type as its owner edge. | Test |
| FR-363-AC-2 | A role `model` naming a source-declared type with a matching entry admits. | Test |
| FR-363-AC-3 | The same clause with no matching `lock.sources` entry, and with a `SourceOwner` whose authority or identity differs from the entry, each refuse at the clause node with the model-owned join's code and cause. | Test |
| FR-363-AC-4 | An `owner_model` naming a non-object-type node, and one naming a source type of a different `SourceOwner` than the lock entry, refuse. | Test |
| FR-363-AC-5 | A model-owned clause over a selected model reads unchanged, and a source type in the same package does not satisfy a model-owned clause's join. | Test |

## Dependencies

- [FR-064](./FR-064-bind-protocol-clause-to-owner-model.md) owns the model join this extends.
- [FR-362](./FR-362-read-source-declared-object-types.md) defines a source-declared type.
