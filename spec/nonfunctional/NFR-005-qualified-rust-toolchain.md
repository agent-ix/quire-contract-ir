---
id: NFR-005
title: "Build with the declared Rust compiler"
type: NFR
quality_attribute: portability
relationships:
  - target: ix://agent-ix/quire-contract-ir/ADR-0055
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: constrains
---
# NFR-005: Build with the declared Rust compiler

## Statement

Every library, binary and test target shall compile and pass its tests with the
compiler named in `rust-toolchain.toml`.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Rust targets | all declared targets compile and test | any compiler/target incompatibility fails | Test |

## Verification

The `supported-rust` and `qualification-rust` targets of `make ci` compile and
test the workspace with that compiler.
