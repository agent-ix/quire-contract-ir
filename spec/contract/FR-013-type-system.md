---
id: FR-013
title: "Represent the closed v0.1 contract type system"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
---
# FR-013: Represent the closed v0.1 contract type system

## Description

The v0.1 model shall define Boolean, signed and unsigned bounded integer,
rational, string, enum, record, option, bounded collection, input, and state
reference types without embedding Rust or architecture-language vocabulary.

## Inputs

Named type declarations, field declarations, numeric bounds, collection bounds,
enum variants, and value-reference declarations.

## Outputs

Acyclic validated type declarations and typed value-reference identities.

## Behavior

Validation rejects duplicate names, empty enums, recursive records, invalid
numeric bounds, unbounded collections, and references to absent declarations.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-013-AC-1 | Every v0.1 type construct has positive and negative fixtures with deterministic diagnostics. | Test (TC-016) |
| FR-013-AC-2 | Public serialized types contain no Rust, GUMBO, AADL, HAMR, solver, or runtime-specific vocabulary. | Inspection (TC-016) |

## Dependencies

FR-011 supplies package-scoped declaration identity.
