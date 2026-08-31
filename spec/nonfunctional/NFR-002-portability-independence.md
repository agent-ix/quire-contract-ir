---
id: NFR-002
title: "Remain portable and implementation-language independent"
type: NFR
quality_attribute: portability
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: traces_to
---
# NFR-002: Remain portable and implementation-language independent

## Statement

The v0.1 semantic and wire contracts shall avoid Rust layout, target pointer
width, operating-system paths, solver APIs, runtime APIs, and architecture-model
vocabulary.

## Scope

Public types, JSON schemas, canonical encoding, fixtures, diagnostics, and
documentation.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Forbidden public vocabulary | 0 occurrences | Any occurrence fails | Scan schema and public model names |
| Target-dependent canonical fields | 0 fields | Any field fails | Schema and golden-fixture review |
| Minimum Rust version | 1.75 | Newer requirement fails | Build with declared MSRV before release |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | Public schema and model names contain no Rust-layout, architecture-language, solver, runtime, pointer-width, or operating-system-path vocabulary. | Test (TC-019) |
| NFR-002-AC-2 | The library and conformance corpus pass with Rust 1.75 before a v0.1 source decision. | Test (TC-019) |

## Verification

Schema/API inspection, vocabulary scan, and declared MSRV check (TC-016,
TC-019).
