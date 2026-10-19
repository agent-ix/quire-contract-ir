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
definition, title, enum entry and string `const` value. A forbidden name used
as a schema `const` must fail the check. Enumerate every exported item of the
model crate, including feature-gated exports, re-exports and public items
reachable through public modules. For each exported type, also enumerate its
public fields, enum variants and public associated functions. The current
public modules are `selected_population` and `source_obligation`; a newly
declared public module, feature-gated export or unrecognized public declaration
must fail the inventory until its names are checked. Fail if any enumerated
name contains a forbidden implementation-specific term. Include a positive
control showing that a benign `jsonpointer` name is admitted and adverse
controls for a runtime title, pointer-width property, architecture-language
enum name, forbidden schema `const`, feature-gated export, public field, enum
variant and public associated function.

## Expected Results

The complete public schema and model-name inventory, including string `const`
values and all exported item kinds under their applicable features, has zero
forbidden names. Every adverse control is detected and the benign control is not.

## Status

Planned until the NFR-002-AC-1 inventory test covers the amended scope. Its
existing trace tag alone does not establish that coverage. TC-019 remains
planned for NFR-001's separate determinism measurements.
