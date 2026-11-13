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
exists. This requirement adds that witness kind (QSL ADR-013 QC-30 item 4).

## Inputs

A `SelectedSourceOperationRow`, a stage row beside the operation-member row:
the full anchor `CheckedNodeId`, the declaring source node, its `SourceOwner`
authority and identity, the operation name, and the stage's selected
source-operation identity (opaque to IR, as the model operation identity is).

## Outputs

A joined source-operation row for the anchor, exposed to the callable read of
[FR-354](./FR-354-project-admitted-runtime-context.md), or a typed refusal.

## Behavior

IR SHALL join the anchor to an admitted source-declared operation anchor and
its declaring source type, and compare authority, identity and operation name
with the anchor and the FR-365 effects row. It SHALL refuse a foreign or
dangling anchor, a row of another source type, an absent operation and
competing rows for one anchor. A source operation row SHALL NOT be accepted
for a model-owned anchor, and a model operation-member row SHALL NOT be
accepted for a source anchor. Absent the row, the callable read SHALL keep the
unavailable-witness outcome. IR SHALL NOT mint the operation identity.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-368-AC-1 | With a valid row, the callable read of a source anchor returns the joined row and not unavailable-witness. | Test |
| FR-368-AC-2 | Without a row, the source anchor returns unavailable-witness. | Test |
| FR-368-AC-3 | A foreign anchor, a dangling anchor, a row of another type, an absent operation and two rows for one anchor each refuse. | Test |
| FR-368-AC-4 | A source row for a model anchor and a model row for a source anchor each refuse. | Test |
| FR-368-AC-5 | Model-owned operation-member rows read unchanged. | Test |

## Dependencies

- FR-352, FR-354 own the witness convention and callable read.
