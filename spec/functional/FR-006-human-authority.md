---
id: FR-006
title: "Enforce contribution provenance and human authority"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-006: Enforce contribution provenance and human authority

## Description

The governance contract shall apply neutral contribution gates and reserve release authority to the named human exactly as specified by PGM-01-R06.

## Inputs

A contribution method, candidate pull request, CODEOWNERS file, protected-branch configuration, and review record.

## Outputs

Truthful contribution provenance and an open, approved, deferred, or rejected human decision.

## Behavior

- Human and agent-assisted work meets identical technical and provenance gates.
- Agents may prepare but do not self-approve or decide evidence sufficiency.
- `@kreneskyp` is the v0.1 decision authority enforced through CODEOWNERS and protected main.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-006-AC-1 | Repository policy and CODEOWNERS agree on the named human authority and forbid agent self-approval. | Inspection (TC-004) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).

