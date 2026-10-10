---
id: TC-446
title: "Proof-coverage baseline retains the population and qualifies evidence"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-045
    type: verifies
---
# TC-446: Proof-coverage baseline retains the population and qualifies evidence

## Description

Verify FR-045's candidate census, evidence qualification and four projections
using a small source-derived inventory whose expected claims are independently
enumerated.

## Test Procedure

Build a fixture with two requirement criteria, three obligations, two production
functions in one module, explicit many-to-many mappings and one eligible entity
with zero eligible mapped obligations.
Run the baseline with one matching production Kani proof and one missing result.
Then vary one input at a time: remove or duplicate a result; add or change a
method, proof subject or backend/profile attempt; alter source or bounds; report
zero checks or only a tag/LLVM probe; supply an FR-029 `unsupported` negotiation
disposition without an artifact, run or Kani outcome; supply a producer-stage
refusal or Kani `refused` outcome; supply counterexample, invalid/incomplete, unavailable, timeout,
exhaustion, cancellation, inconclusive or unknown state; provide a shadow proof
with and without separately executed sampled refinement; alter the authoritative
inventory without altering any auxiliary catalog.

## Expected Results

Every source candidate stays visible, with an eligibility decision and a gap or
qualified evidence. Only the exactly joined, nonvacuous, load-bearing production
proof receives unqualified production-proof credit in its declared domain.
Narrowed and sampled/shadow evidence retain their limited strength. The four
projections have independent totals and reasons; missing results do not shrink
denominators, and an eligible zero-mapping entity has no full credit. Changes to
method, subject and profile attempts leave candidate keys and denominators stable.
The unsupported negotiation decision is recorded without an invented Kani outcome;
refusals retain their producer and stage. Repeating identical inputs yields identical output, while the
inventory change changes the candidate set without a hand-maintained ledger.
