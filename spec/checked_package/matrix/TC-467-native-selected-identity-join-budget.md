---
id: TC-467
title: "Shared native selected-identity join budget"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: verifies
---
# TC-467: Shared native selected-identity join budget

## Description

Exercise the mandatory caller-owned work and logical-storage budget through
admitted selected-identity stage rows and all five source-row join families.

## Test Procedure

Run the FR-358 tagged tests in the `it` integration binary with
`conformance-cli` enabled. For each family, obtain its successful work and
storage charges from an ample budget, then repeat the same admitted value at
each exact limit and one unit below each limit. Join two collection rows under
one budget, including a nonempty prior map copied by `Arc::make_mut`, and
repeat the ordered joins against a cloned admitted package. Run the existing
FR-352 semantic-refusal tests at ample limits and inspect the public API and
call sites for the mandatory signature, cycle boundary and absence of a
parallel budget path.

## Expected Results

Every exact-limit call yields the same selected identity value. Every
one-below call returns the exhausted resource kind, first native locus and
first unpaid charge, with no selected value. Cumulative counters equal the
sum of the two admitted operation deltas; the second operation refuses when
their combined allowance is one below exact. The first selected value remains
unchanged after refusal, and a cloned package gives the same result and
charges. Typed non-budget identity refusals retain their existing causes and
first loci. The crate exports one mandatory `NativeJoinBudget` acquisition
path and its charge metadata without a QSL or runtime dependency.
