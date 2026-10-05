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

Verify FR-038-AC-123 through FR-038-AC-135 (IR-627): the reader re-derives the
node key of the ten anonymous node shapes `MemberType::node_key` already
derives (`scalar_type` `boolean` and `integer`, `composite_type` `reference`,
`option`, `set`, `bag`, `sequence` and `ordered_set`, `bounded_domain`
`integer_range` and `collection_bounds`) from each node's own body, and refuses
a node whose key differs at its `node_id`, wherever the node is reached from.
AC-131 and AC-132 (the forms with no derived preimage, and declared nodes) are
gated on IR-627-Q1 to Q4 and are planned.

## Test Procedure

Over a lock selecting a domain package document built here (an object type
with fields declared `Int[0, 1000]`, `Int[0, 0]`, the `i128` extremes,
`Option<Int[0, 1000]>`, `Set<Int[0, 1000]>`, `Sequence<Int[0, 1000]>`, a
bounded collection and a `Reference`, and an operation with a parameter typed
`Int[0, 1000]`), build a checked package whose reads name each type node, a
scalar operand that reaches an `Int[0, 1000]` node without a member read, and a
state field whose body names one. Admit the unmutated package. Then apply each
single mutation the criteria name (a body value, a body member, a
`semantic_type`, a literal `type`, a body `reference`), keep the `node_id`,
patch `identity_projection`, recompute `package_id` through `quire-canonical`
in the test and not through the reader, and read each mutated package. Also
read the packages of AC-127 that pair a tampered node with another defect, and
the AC-129 package whose `model_selections` row is re-pointed, with and
without the other document in the evidence. Read the three QSpec positive
fixtures of FR-038-AC-107 and the in-repo fixture packages with their derived
keys (admit) and with one placeholder key restored on an undeclared
derived-shape node (`stale-node-key`); read the in-repo `aaaa` and `bbbb`
nodes with keys regenerated but bodies and `semantic_type` unmigrated
(`stale-node-key`); and apply `adverse.json` (eleven entries) and
`dependency-selection-vectors.json` over `positive-all-families.json` carrying
derived keys (AC-133 through AC-135). The QSpec fixtures are read
from the checkout `QUIRE_SPECIFICATION_DIR` names, never copied.

The required regression test is the IR-627 tamper probe: the `Int[0, 1000]`
node with `max` changed to `10`, and separately to `5000`, each refused.

## Expected Results

The unmutated packages admit. Each mutation refuses
`invalid_package`/`stale-node-key` at the tampered node's `node_id` with no
package; the stage order and ascending node-id order decide which node is
reported (AC-127). The re-pointed selection refuses `missing_import`/
`missing-selection` without the other document and admits with it. The AC-131
and AC-132 cases do not exist until the gate lifts.

## Status

Planned. No test is written. AC-123 through AC-130 await the code change that
adds the re-derivation stage and regenerates the in-repo fixtures (AC-134);
AC-131 and AC-132 await the answers to IR-627-Q1 to Q4; AC-133 and AC-135
await QSpec's three fixtures carrying derived keys (IR-627-Q5), and the code
change is ordered after them.
