---
id: TC-449
title: "Kani cancellation carries the producer's requested or deadline origin"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: verifies
---
# TC-449: Kani cancellation carries the producer's requested or deadline origin

## Description

Verify FR-030-AC-7 at Contract IR's public Kani outcome boundary. The case
checks origin provenance, the serialized contract and invalid kind/source
pairs. CG's conversion to QSL's terminal value belongs to IR-701 and is not
asserted here.

## Test Procedure

Supply the Contract IR boundary with each producer-reported source event:
`Requested` from the caller's cancellation request and `Deadline` from the
timer owner's cancellation. Serialize and deserialize each outcome and read
its kind, source and Boolean claim. Compile a probe that requests `Cancelled`
without a source. Deserialize outcomes with
a unit `"cancelled"` kind, a `cancelled` kind with no `source`, an unknown
source, and a non-cancelled kind with a cancellation source member. Build a
`TimedOut` outcome from an execution timeout for which no cancellation event
was recorded. Round-trip every existing non-cancelled kind as a control.

## Expected Results

The two cancelled outcomes retain their distinct `Requested` and `Deadline`
sources after the round trip and have no Boolean claim. The no-source compile
probe fails and each malformed or mismatched wire input is refused; none is
given a default source. The execution timeout stays `TimedOut`. Every
non-cancelled kind retains its previous serialized spelling, carries no
cancellation source and retains its prior Boolean claim.

## Status

Planned. IR-778 specifies the producer-boundary contract; its implementation
and this focused test follow in the code stage.
