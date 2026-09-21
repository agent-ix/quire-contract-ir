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

The governance contract shall define the source-tag gating rule exactly as specified by PGM-01-R03.

## Inputs

The exact dependency manifests and retained evaluator evidence for each candidate.

## Outputs

A topologically valid set of immutable v0.1.0 source tags and checksums.

## Behavior

- A repository may tag only once the exact dependency tags and checksums its own manifest names are available.
- Added manifest dependencies add corresponding topological gates.
- Rebuilds of an existing tag are forbidden.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-003-AC-1 | The policy gates each source tag on the exact dependency tags and checksums its manifest names. | Inspection (TC-002) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
