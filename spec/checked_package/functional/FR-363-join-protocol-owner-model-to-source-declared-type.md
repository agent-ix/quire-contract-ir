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

[FR-064](./FR-064-bind-protocol-clause-to-owner-model.md) requires a protocol
clause's `owner_model` to name a selected model declaration, so a protocol
`over` a source-declared object type is refused today. This requirement
amends FR-064: the reader SHALL also accept an `owner_model`, and a role
`model`, that names an admitted source-declared object type node (QSL ADR-013
QC-30 item 3, FR-464, FR-465). FR-064-AC-4 is amended to match.

## Inputs

A protocol clause node whose `owner_model`, or a protocol role whose `model`,
is a local node reference to a node of the package.

## Outputs

The admitted clause-to-owner edge, readable by the refinement class catalogs
as a model owner edge is, or one typed located refusal.

## Behavior

Every `SourceOwner` of the package is already joined to exactly one
`lock.sources` entry in the owner-join step of FR-038 (FR-038-AC-61,
FR-038-AC-155), before any protocol check, and a missing entry refuses
`missing_declaration`/`missing-selection` at the source type's own `node_id`.
The protocol step therefore SHALL NOT repeat that join. For an `owner_model` or
role `model` whose target is a source-declared object type, the reader SHALL
admit the edge without requiring a selected model row or a result of
`owners.recover`. For a target that is source-local but not a source-declared
object type, the reader SHALL refuse `invalid_model_binding`/`malformed-declaration`
at `owner_model` (or the role's `model`) as FR-064-AC-4 states. A local edge
naming no node, and every model-owned target, keep their FR-064 refusals and
loci unchanged.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-363-AC-1 | A protocol clause over a source-declared type, with the type's lock entry present, admits and the typed clause exposes the source type as its owner edge. | Test |
| FR-363-AC-2 | A role `model` naming a source-declared type admits. | Test |
| FR-363-AC-3 | With the lock entry of the source type removed, the same package refuses `missing_declaration`/`missing-selection` at the source type's own `node_id`, by the FR-038 owner join, not at the clause. | Test |
| FR-363-AC-4 | An `owner_model` and a role `model` naming a source-local node that is not an object type (a source record, a population reference) refuse `invalid_model_binding`/`malformed-declaration` at the edge. | Test |
| FR-363-AC-5 | A model-owned clause over a selected model reads unchanged, and the FR-064 refusals for an unselected target, a missing node, a non-model node and the `model_root` keep their codes and loci in a package that also holds a source type. | Test |

## Dependencies

- [FR-064](./FR-064-bind-protocol-clause-to-owner-model.md) owns the model join this amends.
- [FR-362](./FR-362-read-source-declared-object-types.md) defines a source-declared type.
