---
id: TC-456
title: "Public schema and model names exclude implementation-specific vocabulary"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-002
    type: verifies
---
# TC-456: Public schema and model names exclude implementation-specific vocabulary

## Description

Verify NFR-002-AC-1 independently of TC-019's cross-run and cross-platform
determinism analysis. Inspect the public schema and model-name inventory for
Rust-layout, architecture-language, solver, runtime, pointer-width and
operating-system-path vocabulary.

## Test Procedure

Enumerate every repository-owned public JSON schema's name-bearing property,
definition, title and enum entries. Enumerate the model crate's public root
exports, including their defining module names. Fail if any name contains a
forbidden implementation-specific term. Include a positive control showing
that a benign `jsonpointer` name is admitted and adverse controls for a
runtime title, pointer-width property and architecture-language enum name.

## Expected Results

The complete public schema and model-name inventory has zero forbidden names.
The adverse controls are detected and the benign control is not.

## Status

Planned until the inventory test is tagged NFR-002-AC-1. TC-019 remains
planned for NFR-001's separate determinism measurements.
