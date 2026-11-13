---
id: FR-368
title: "Project the source operation anchor witness"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-362
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-365
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-569
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---

# FR-368: Project the source operation anchor witness

## Description

[FR-352](./FR-352-project-admitted-exact-value-types.md) returns a typed
unavailable-witness outcome for a state `operation_anchor` whose context is a
source-declared node, until a distinct source-operation identity witness
exists. This requirement adds that witness kind for the operation anchor
(QSL ADR-013 QC-30 item 4). Exact type projection of a source-declared type
(a reference to it, a record view) is not part of this change and keeps FR-352's
unavailable-witness outcome.

## Inputs

A `SelectedSourceOperationRow` supplied beside the package, with the fields of
QSL FR-390-AC-70: `source_authority`, `source_identity` (`<type>::<operation>`),
`anchor_id` and `structural_node`. IR mints no operation identity and adds no
opaque field. The IR public inventory (FR-019) gains the row type and its read.

## Outputs

A joined source-operation row for the anchor, exposed to the callable read of
[FR-354](./FR-354-project-admitted-runtime-context.md), or the typed outcome.

## Behavior

The reader SHALL join `anchor_id` and `structural_node`, which are equal,
to an admitted `state`/`operation_anchor` whose `context` is a source-declared
object type, and SHALL require `source_authority` to equal that type's
`SourceOwner` authority and `source_identity` to equal `<type>::<operation>`
for the final segment of the context's `declaration` and the anchor's
`operation` text. Where the optional `operation_effects` carrier is present,
the reader SHALL also require a row for the context and that operation
([FR-365](./FR-365-read-source-declared-operation-effects.md)); where it is
absent, the reader SHALL skip that comparison and the effects stay unavailable
under FR-354. For a foreign or dangling anchor, a row of another type or
operation, and a source row for a model anchor or a model operation-member row
for a source anchor, the reader SHALL return the FR-352 outcome the model
operation-member row gets for the same defect. Two rows for one anchor SHALL
return `ConflictingIdentityWitness`. Without a row, a source anchor SHALL keep
`UnavailableIdentityWitness`. The reader SHALL NOT match a source anchor to a
model operation member by name.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-368-AC-1 | With a valid row, the callable read of a source anchor returns the joined row and not unavailable-witness. | Test |
| FR-368-AC-2 | Without a row, the source anchor returns `UnavailableIdentityWitness`. | Test |
| FR-368-AC-3 | A foreign anchor, a dangling anchor, a row of another type or operation and a wrong authority each return the FR-352 outcome of the model operation-member row for that defect; two rows for one anchor return `ConflictingIdentityWitness`. | Test |
| FR-368-AC-4 | A source row for a model anchor and a model row for a source anchor each return that outcome. | Test |
| FR-368-AC-5 | Model-owned operation-member rows read unchanged in a package that also holds a source type. | Test |
| FR-368-AC-6 | With `operation_effects` present, a valid row for an operation with no effects row refuses; with `operation_effects` absent, the same row joins and effects stay unavailable. | Test |

## Dependencies

- FR-352 and FR-354 own the witness convention and the callable read.
- [FR-365](./FR-365-read-source-declared-operation-effects.md) supplies the effects row compared when the carrier is present.
