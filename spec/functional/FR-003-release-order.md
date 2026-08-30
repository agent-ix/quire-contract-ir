---
id: FR-003
title: "Define source release order"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-003: Define source release order

## Description

The governance contract shall define the eight-repository source-tag dependency order exactly as specified by PGM-01-R03.

## Inputs

The exact dependency manifests and retained evaluator evidence for each candidate.

## Outputs

A topologically valid set of immutable v0.1.0 source tags and checksums.

## Behavior

- `quire-contract-ir`, `quire-contract-runtime`, and `tl-syntax` are independent roots.
- Codegen follows IR plus runtime; analyze follows IR; parse and MLTL follow syntax; rewrite follows syntax plus evaluator evidence.
- Added manifest dependencies add corresponding topological gates.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-003-AC-1 | The policy names all eight repositories and the complete root/dependent ordering. | Inspection (TC-002) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).

