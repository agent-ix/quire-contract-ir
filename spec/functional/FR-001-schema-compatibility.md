---
id: FR-001
title: "Enforce schema compatibility"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-001: Enforce schema compatibility

## Description

The governance contract shall define explicit wire-schema identity, reject unknown major versions, and prohibit implicit migration exactly as specified by PGM-01-R01.

## Inputs

A serialized document and its declared schema identity, major version, and schema digest.

## Outputs

An accepted v1 document or an explicit unsupported-schema result.

## Behavior

- The published Draft 7 schema is the normative acceptance boundary.
- An unknown major returns `UNSUPPORTED_SCHEMA`; no validator guesses or silently migrates it.
- A migration is a separately identified derivation with explicit source and output identities.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-001-AC-1 | A v2 envelope is rejected with `UNSUPPORTED_SCHEMA` while conforming v1 envelopes are accepted. | Test (TC-008) |
| FR-001-AC-2 | The canonical policy requires schema identity and digest pins and forbids silent migration. | Inspection (TC-001) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).

