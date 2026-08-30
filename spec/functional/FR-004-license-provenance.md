---
id: FR-004
title: "Preserve licensing and third-party provenance"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-004: Preserve licensing and third-party provenance

## Description

The governance contract shall preserve generated-code licensing and third-party provenance exactly as specified by PGM-01-R04.

## Inputs

Program source, reusable templates, generated artifacts, dependencies, and copied or adapted third-party elements.

## Outputs

Dual-licensed program material and a reviewable provenance inventory.

## Behavior

- Program repositories and templates use `MIT OR Apache-2.0`.
- Generated source has an explicit compatible SPDX expression.
- Unknown, incompatible, or absent third-party licenses block incorporation.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-004-AC-1 | The policy requires explicit generated-code licensing and immutable third-party origin, license, digest, transformation, and reviewer identity. | Inspection (TC-003) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
