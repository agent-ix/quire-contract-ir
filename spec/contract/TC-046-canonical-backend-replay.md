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
digests, immutable backend-result digest, selected occurrence, reported backend
verdict, domain membership, decode form, and native availability.
Minimize an accepted failure and a candidate that no longer fails.

## Expected Results

The accepted case preserves both verdicts and all canonical identities. Every
payload or verdict substitution fails immutable-result verification before native
execution; every other mutation remains a typed non-success or parity failure.
Minimization creates a linked revision only for a domain-valid candidate
preserving both failures.
