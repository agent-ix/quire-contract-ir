---
id: NFR-005
title: "Qualify the exact supported Rust toolchain"
type: NFR
quality_attribute: portability
relationships:
  - target: ix://agent-ix/quire-contract-ir/ADR-0055
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: constrains
---
# NFR-005: Qualify the exact supported Rust toolchain

## Statement

The crate shall declare and qualify exact Rust 1.98.1 as its initial supported
minimum and candidate qualification compiler. An older compiler shall not be
retained merely because a scaffold or another repository declared it.

## Scope

Cargo metadata, toolchain and Clippy configuration, local orchestration, hosted
checks, specifications, assurance inputs, all library/binary/test targets, and
the exact required compiler-adjacent tools and embedded targets.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Rust targets | all declared targets compile/test as applicable | any compiler/target incompatibility fails | Test |
| Required tools | each required operation executes | launch/version-only evidence or genuine incompatibility fails | Test |
| Formatting migration | formatted by 1.98.1 | unchecked output fails; changed formatting is permitted | Test |

## Known non-compatibility findings

The `idna 0.4.0` and `time 0.3.36` vulnerabilities observed during
specification review were supply-chain findings, not Rust 1.98.1
incompatibilities. The implementation candidate updates their locked
resolutions within the existing dependency constraints. Downstream
shared-assurance matrix disagreement for `ix-flow` remains separate and shall
not cause a compiler reversion.

## Verification

The `supported-rust` and `qualification-rust` targets of `make ci` run the
compiler, targets and required tools under 1.98.1.
