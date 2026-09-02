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

- Each serialized boundary declares its own schema identity and major wire version.
- An unknown major is rejected; no consumer guesses or silently migrates it.
- A migration is a separately identified derivation with explicit source and output identities.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-001-AC-2 | The canonical policy requires schema identity and digest pins and forbids silent migration. | Inspection (TC-001) |

### Retired criteria

`FR-001-AC-1` required a v2 derivation-evidence envelope to be rejected with
`UNSUPPORTED_SCHEMA` while conforming v1 envelopes were accepted. Its subject
was the `quire.derivation-evidence/v1` envelope, whose schema, validator and
fixture corpus are deleted with the withdrawal of PGM-01-R08 under
[engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7).
The criterion is retired rather than repointed at the contract package wire
form. That boundary does reject an unknown major — `unsupported_schema_version`,
via the `migration-unsupported` corpus fixture that `make corpus` runs on every
invocation — but it is a different boundary with its own diagnostic, already
specified by
[FR-017](../contract/FR-017-version-orphan-coverage.md) and registered in
[STD-001](../contract/STD-001-diagnostic-registry.md). Restating it here would
duplicate a claim rather than preserve one. The identifier is not reused.

## Dependencies

- **Governed by**: [PGM-01](../program/PGM-01-governance.md).
