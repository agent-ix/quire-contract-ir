---
id: FR-005
title: "Enforce clean-room grammar provenance"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-005: Enforce clean-room grammar provenance

## Description

The governance contract shall prohibit unlicensed grammar reuse and require clean-room evidence exactly as specified by PGM-01-R05.

## Inputs

Behavioral sources, origin/license inventory, authored grammar, fixtures, attestations, and differential results.

## Outputs

A reviewable clean-room provenance record or a blocking finding.

## Behavior

- Unlicensed internals are not copied, translated, transformed, or used as a line-by-line guide.
- Differential evidence retains inputs and outcomes rather than copied implementation internals.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | The policy enumerates both prohibited reuse and the five retained clean-room evidence elements. | Inspection (TC-003) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).

