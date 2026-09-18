---
id: TC-051
title: "The output-mapping refusal catalog is closed and registered"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/STD-003
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: verifies
---
# TC-051: The output-mapping refusal catalog is closed and registered

## Description

Verify that the stable output-mapping refusal spellings and the STD-003 registry
are the same closed set, and that they share no spelling with the STD-001
semantic diagnostic catalog.

## Test Procedure

Read `spec/contract/STD-003-output-mapping-refusal-registry.md` at compile time.
For every member of the complete `MappingRequestErrorCode` catalog, locate its
registry row, serialize it, and resolve the spelling back to the same member.
Count the registry's code rows. Compare both catalogs against each other in both
directions.

## Expected Results

Each emitted spelling matches exactly one registry row, the registry holds no
row outside the emitted catalog, every spelling round-trips through its stable
wire form, and no spelling appears in both the output-mapping and semantic
diagnostic registries.
