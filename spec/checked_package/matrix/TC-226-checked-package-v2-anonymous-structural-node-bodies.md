---
id: TC-226
title: "CheckedPackage V2 refuses a tampered anonymous structural node body"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-226: CheckedPackage V2 refuses a tampered anonymous structural node body

## Description

Verify FR-038-AC-123 through FR-038-AC-130 (IR-627): a model-owned member read
names an anonymous `bounded_domain` node by the digest of its key, and the
reader compares that node's body values with the member type's declared bounds
by exact integer comparison, refusing a node whose body does not hold the
range its key names. AC-128 through AC-130 (re-derivation of every anonymous
structural node's key) are gated on the preimage being published by QSpec
(IR-627-Q1 to Q4) and are planned.

## Test Procedure

Over a lock selecting a domain package document built here (an object type
with a field declared `Int[0, 1000]`, one declared `Int[0, 0]`, one declared at
the `i128` extremes, and one bounded collection field), build a checked package
whose `query` read of each field names the `bounded_domain` node keyed by the
member type's key. Admit the unmutated package. Then apply each single mutation
the criteria name to the node body (or `semantic_form`), keep its `node_id`,
patch `identity_projection`, recompute `package_id` through `quire-canonical`
in the test and not through the reader, and read each mutated package.

The required regression test is the IR-627 tamper probe: the `Int[0, 1000]`
node with `max` changed to `10`, and separately to `5000`, each refused.

## Expected Results

The unmutated packages admit. Each mutation refuses
`invalid_package`/`stale-node-key` at the tampered node's `node_id` with no
package; graph-shape defects and ascending node-id order decide which node is
reported (AC-127). The AC-128 to AC-130 cases do not exist until the gate lifts.

## Status

Planned. No test is written. AC-123 through AC-127 await the code change that
adds the body comparison to the model-owned member step; AC-128 through AC-130
await the QSpec owner's answers to IR-627-Q1 to Q4.
