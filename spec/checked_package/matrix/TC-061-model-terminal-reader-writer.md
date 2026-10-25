---
id: TC-061
title: "Read and write checked-package model terminal declarations"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-060
    type: verifies
---

# TC-061: Read and write checked-package model terminal declarations

## Description

Verify [FR-060](../functional/FR-060-admit-model-terminal-declarations.md):
the public V2 reader and writer carry each QSpec terminal mode intact and
refuse malformed declarations in QSpec's order.

## Test Procedure

Use QSpec TC-961 definitions MT-01 through MT-16 as the published oracle.
Build IR-owned V2 wire vectors for `when` with
and without `self`, and `any`, under one selected model. Read each through
the public reader, write the admitted package, then read its output. For
each adverse vector, edit a valid package, rebuild dependencies and
identities except where the vector tests their mismatch, and read it again.
Include a second node for one model and a second claim occurrence on one
node. Add two-defect vectors at each reader-order boundary. Do not place any
QSpec file, schema or copied fixture in this repository.

## Expected Results

The positive modes preserve the four-binding body, node identities,
dependencies and claim occurrences through read/write/read. The adverse
forms refuse with FR-476's exact code, cause and pointer, with earlier graph
and body-step defects winning over terminal defects.
