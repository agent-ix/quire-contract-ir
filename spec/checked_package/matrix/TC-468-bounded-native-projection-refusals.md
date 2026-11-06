---
id: TC-468
title: "Bounded native clock and type-projection refusals"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-359
    type: verifies
---
# TC-468: Bounded native clock and Exact-projection refusals

## Description

Verify shared work and storage accounting for selected monitor clock reads and
Exact type projection, including owned diagnostics and nested Exact projection.
Inspect the selected runtime projection public signature; its full allocation
qualification is tracked separately under IR-1366.

## Test Procedure

For each qualified public read, run an admitted success and a semantic refusal
with an owned dynamic node path or form. Measure charges under an ample
caller-owned budget, then repeat at exact work and storage limits, one unit
below each limit, and zero storage. Compose a read after an earlier stage or
source join under the same budget. Exercise a nested Exact refusal. Repeat
against an admitted package clone. Inspect allocation sites and public
signatures, including the staged runtime signature.

## Expected Results

Exact limits preserve complete success and payable semantic errors. One-below
limits refuse at the first unpaid charge, with fixed-size storage exhaustion
when an owned diagnostic cannot be paid. Zero storage returns the compact form
without allocating a diagnostic. Cumulative counters remain bounded and
monotone; refusal leaves no successful delta or partial value; repeated inputs
agree. The public clock and Exact reads use the shared budget without numeric
entry points. Runtime full-precharge remains Planned under IR-1366.
