---
id: TC-046
title: "Contract IR defines no replay envelope and calls no replay executor"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-219
    type: references
---
# TC-046: Contract IR defines no replay envelope and calls no replay executor

## Description

Verify FR-037-AC-6: counterexample replay is QSL's and codegen's, and Contract
IR carries none of it. quire-specification:TC-219 and QSL's replay executor
tests verify the replay properties themselves.

## Test Procedure

Inventory the public items of both workspace crates and search `src/` and
`crates/quire-contract-model/src/` for a replay envelope, request, result,
parity or minimization type, and for a call to `qsl_replay::replay`,
`quire_spec_language::runtime` or any other executor entry.

## Expected Results

No such type is public and no such call exists.

## Status

Planned.
