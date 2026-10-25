---
id: TC-060
title: "Read and write checked-package state-graph claims"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-059
    type: verifies
---

# TC-060: Read and write checked-package state-graph claims

## Description

Verify [FR-059](../functional/FR-059-admit-state-graph-claim-nodes.md): the
public V2 reader and writer carry each QSpec state-graph form intact and
refuse malformed forms in QSpec's order.

## Test Procedure

Use QSpec TC-960 definitions SN-01 through SN-17 as the published oracle.
Build IR-owned V2 wire vectors for `possible`,
`always-possible` and `unique-path` with real Boolean predicate and parameter
nodes. Read each through the public reader, write the admitted package, then
read its output. For each adverse vector, edit a valid package, rebuild
dependencies and identities except where the vector tests their mismatch,
and read it again. Add a two-defect vector at each reader-order boundary.
Do not place any QSpec file, schema or copied fixture in this repository.

## Expected Results

The positive forms preserve the four-binding body, node identities,
dependencies and claim occurrences through read/write/read. The adverse
forms refuse with FR-475's exact code, cause and pointer, with earlier graph
and state-graph defects winning.
