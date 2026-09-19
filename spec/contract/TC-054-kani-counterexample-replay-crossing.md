---
id: TC-054
title: "Kani counterexample replay through the complete-V1 executor conforms"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: verifies
  - target: ix://agent-ix/quire-specification/AD-016
    type: references
---
# TC-054: Kani counterexample replay through the complete-V1 executor conforms

## Description

Verify FR-031-AC-3: every serialized Kani counterexample either reproduces
through the QSL complete-V1 executor entry
`value::expression::CheckedPackage::call` with the same outcome and witness,
or returns a typed non-success disagreement. Per AD-016 (AD-016 arrows 6 and
7), a criterion whose test lives in another repo gets its own row, discharged
by the test that repo owns: this crossing behaviour's test is
quire-spec-language's, not Contract IR's, and does not share a row with
FR-031-AC-4 (witness vocabulary), which TC-221 keeps.

## Test Procedure

Discharged by the QSL crossing test at `agent-ix/quire-spec-language#243`
(the layer-6 `replay` facade), which has not started.

## Expected Results

Every serialized counterexample either reproduces through the executor with
the same outcome and witness, or returns a typed non-success disagreement.
Proof, refusal, and inconclusive results never masquerade as replayed
counterexamples; replay mismatch, unavailable runtime, and a malformed packet
remain typed non-success.

## Status

Planned. `agent-ix/quire-spec-language#243`, the QSL layer-6 `replay` facade
this test needs, has not started.
