---
id: TC-468
title: "Bounded native clock and type-projection refusals"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-359
    type: verifies
---
# TC-468: Bounded native clock and type-projection refusals

## Description

Verify shared work and storage accounting for selected monitor clock reads,
Exact type projection and selected runtime type projection, including their
owned diagnostic payloads.

## Test Procedure

For each public read, run one admitted success and one semantic refusal with
an owned dynamic node path or form. Measure each operation's admitted charges
under an ample caller-owned budget, repeat at exact work and storage limits,
one unit below each limit, and zero storage. Compose a read after an earlier
stage or source join under the same budget. For runtime type projection,
include a nested Exact refusal converted to its public error. Repeat against
an admitted package clone and inspect allocation sites and public signatures.

## Expected Results

Exact limits preserve complete success and payable semantic errors. One-below
limits refuse at the first unpaid charge, with fixed-size storage exhaustion
when an owned diagnostic cannot be paid. Zero storage returns that compact
form without allocating a diagnostic. Cumulative counters remain bounded and
monotone, refusal leaves no successful delta or partial value, and repeated
inputs agree. The public reads use the shared budget without a separate
numeric-limit entry point.
