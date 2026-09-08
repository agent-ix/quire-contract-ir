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
| Semantic portability | independent of compiler version | any compiler-specific wire or model meaning fails | schema, API and golden-fixture review |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | Public schema and model names contain no Rust-layout, architecture-language, solver, runtime, pointer-width, or operating-system-path vocabulary. | Test (TC-019) |
| NFR-002-AC-2 | The library and conformance corpus preserve identical semantic behavior on the exact supported and qualification toolchains declared by NFR-005; compiler selection does not change wire or canonical meaning. | Test (TC-019, TC-036) |
| NFR-002-AC-3 | Issue #6 public JSON field names, identity kinds, anchors, clause kinds, dependency kinds, and diagnostic codes contain no Rust, GUMBO, AADL, HAMR, solver, runtime, pointer-width, or operating-system-path vocabulary. | Test (TC-015) |
| NFR-002-AC-4 | Issue #8 public type and expression schema/API vocabulary contains no Rust, GUMBO, AADL, HAMR, solver, runtime, pointer-width, or operating-system-path vocabulary. | Test (TC-016) |

## Verification

Identity vocabulary inspection (TC-015), typed schema/API inspection (TC-016),
and complete vocabulary/toolchain checks (TC-019, TC-036). NFR-005 owns the
compiler-version policy so portability is not confused with an inherited MSRV.
