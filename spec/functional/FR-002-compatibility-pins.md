---
id: FR-002
title: "Require compatibility and dependency pins"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-002: Require compatibility and dependency pins

## Description

The governance contract shall define crate compatibility and exact qualification pins exactly as specified by PGM-01-R02.

## Inputs

A candidate's crate, schema, toolchain, engine, feature, target, input, output, and configuration identities.

## Outputs

A reviewable set of exact source and artifact pins.

## Behavior

- Pre-1.0 minor releases may break; patch releases preserve the documented contract.
- Release and qualification evidence records exact revisions, tags, lockfiles, versions, feature sets, and digests.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-002-AC-1 | The canonical policy distinguishes semver convenience ranges from exact release and qualification pins. | Inspection (TC-001) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
