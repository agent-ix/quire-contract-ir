---
id: TC-047
title: "Complete-V1 lowering emits one canonical ContractPackage"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/issues/110
    type: references
---
# TC-047: Complete-V1 lowering emits one canonical ContractPackage

## Description

Verify FR-035-AC-5: the aggregate `ContractPackage` half of FR-035's Outputs,
separately from the per-item records TC-044 covers.

## Test Procedure

Lower one mixed request holding supported, unsupported, unbounded and absent
items. Read the emitted `ContractPackage`, canonicalize it, and take its digest.
Repeat the identical call and compare bytes. Mutate one represented node and
repeat.

## Expected Results

One canonical cycle-free versioned `ContractPackage` carries every `lowered`
node of the call once, plus the admitted nodes those reach, and no other node:
a refused request's node appears only as meaning a lowered node reaches. The
canonical bytes are RFC 8785 and carry every member of every represented node. Repeated identical calls
produce identical canonical bytes and digest. Mutating any represented node
changes them.

## Status

Implemented in `tests/it/complete_v1_contract_package.rs`.
