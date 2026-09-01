---
id: FR-008
title: "Validate domain derivation provenance"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---
# FR-008: Validate domain derivation provenance

## Description

The repository shall validate its historical v0.1 domain-derivation record
without claiming a common evidence envelope, execution authority, or retention
authority, exactly as specified by PGM-01-R08.

## Inputs

The published schema, corpus manifest, and candidate domain-derivation records.

## Outputs

A deterministic validation report with a Boolean validity result and structured errors.

## Behavior

- `jsonschema` Draft7Validator plus FormatChecker is the sole conformance engine; no handwritten validator reimplements schema constraints.
- The gate validates every manifest fixture and fails if schema acceptance differs from the declared valid/invalid result or expected code.
- Stable targeted codes are `UNSUPPORTED_SCHEMA`, `MISSING_PRODUCER`, `MISSING_INPUTS`, `MISSING_SCHEMA_IDENTITY`, `MISSING_BACKEND`, `MISSING_OUTPUTS`, and `INVALID_DIGEST`; every other Draft 7 finding is `SCHEMA_VIOLATION`.
- Mutation probes weaken producer, backend, output, and provenance requirements and must be detected by the corpus gate.
- Format probes reject malformed RFC 3339 timestamps, repository URIs, and artifact URI references.
- The Python runtime, jsonschema package, and its RFC 3339/RFC 3986 format validators are exact declared dependencies; the gate fails closed if a required checker is unavailable.
- The record remains producer-owned structured output. Quire supplies static
  definition references only, and Quoin may retain or audit the explicit record
  without invoking its producer.
- The schema and validator are historical compatibility surfaces, not a generic
  stdout scraper, universal runner, evidence store, or parallel result family.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-008-AC-1 | Both valid fixtures pass the published schema and every invalid fixture fails with its declared code. | Test (TC-005) |
| FR-008-AC-2 | Mutating any probed nested required-field set or identity format causes the conformance gate to fail. | Test (TC-005) |
| FR-008-AC-3 | Tool, input, schema, backend, output, and provenance identity omissions have targeted negative evidence. | Test (TC-007) |
| FR-008-AC-4 | Governance classifies the schema as a producer-owned domain result and assigns execution, static definitions, retention/audit, and human decisions to distinct owners. | Test (TC-023) |

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
