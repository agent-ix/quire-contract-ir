---
id: StR-002
title: "Exchange contracts reproducibly across implementations"
type: StR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: depends_on
---
# StR-002: Exchange contracts reproducibly across implementations

## Stakeholder Need

Tool authors need an implementation-language-independent serialized contract,
canonical byte representation, stable digests, and explicit version behavior.

## Rationale

Rust memory layout and map iteration order are unsuitable interchange
contracts. Reproducible bytes and fail-closed version handling permit independent
tools to compare identities across operating systems and languages.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-002-VC-1 | Semantically identical supported packages produce identical canonical bytes and digests on every supported platform. | Golden corpus and cross-platform comparison (TC-017) |
| StR-002-VC-2 | Unknown schema majors and unregistered migrations produce structured rejection. | Negative schema fixtures (TC-018) |

## Dependencies

StR-001 defines the authoritative semantic subject.
