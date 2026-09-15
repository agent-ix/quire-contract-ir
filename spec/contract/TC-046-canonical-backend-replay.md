---
id: TC-046
title: "Canonical backend counterexample replay conforms"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-219
    type: references
---
# TC-046: Canonical backend counterexample replay conforms

## Description

Verify typed counterexample round-trip, native replay, verdict parity, and
minimization lineage for bounded backend results.

## Test Procedure

Serialize and replay a known bounded counterexample, then independently mutate
values, IEEE bits, identities, state anchors, bounds, source maps, package/run
digests, domain membership, decode form, native availability, and verdict.
Minimize an accepted failure and a candidate that no longer fails.

## Expected Results

The accepted case preserves both verdicts and all canonical identities. Every
mutation remains a typed non-success or parity failure; minimization creates a
linked revision only for a domain-valid candidate preserving both failures.
