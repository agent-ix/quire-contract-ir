---
id: TC-044
title: "Complete-V1 target-neutral ContractPackage lowering conforms"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-217
    type: references
---
# TC-044: Complete-V1 target-neutral ContractPackage lowering conforms

## Description

Verify exact lowering and per-item refusal across scalar/composite, domain,
expression/function, model/state, temporal, protocol, claim, and source-map
families.

## Test Procedure

Lower an independently constructed complete-V1 vector and mutation variants for
each source/type/anchor/identity/bound/dependency/version axis. Submit a mixed
supported, unbounded, unknown-tag, malformed, and unavailable item request;
then canonicalize, re-read, and resolve every emitted identity reference.

## Expected Results

Every represented item round-trips with identical meaning and source
correspondence. Each mutation returns only its exact item disposition, no
substitute node or backend artifact, and does not change independent siblings.
