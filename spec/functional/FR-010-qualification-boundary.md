---
id: FR-010
title: "Preserve the qualification boundary"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-010: Preserve the qualification boundary

## Description

The governance contract shall prevent program releases from conferring consuming-project validation, accreditation, or certification exactly as specified by PGM-01-R10.

## Inputs

A program release, consuming project, intended use, tool configuration, environment, hazards, evidence, and authority.

## Outputs

A bounded project-specific claim or an explicit absence of such a claim.

## Behavior

- Reusable crates and evidence are qualification support only.
- A consuming-project claim identifies its exact context, independent review, and authority.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-010-AC-1 | The policy explicitly states that a crate release does not validate, accredit, or certify a consuming project. | Inspection (TC-003) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
