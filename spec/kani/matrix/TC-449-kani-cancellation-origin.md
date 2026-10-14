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

Verify FR-030-AC-7's IR carrier and wire refusal at Contract IR's public Kani
outcome boundary. A second, planned CG integration control checks that the
Kani run's observed cancellation event, rather than a test-supplied label,
determines the origin. CG's event producer and conversion to QSL's terminal
value belong to IR-701.

## Test Procedure

At the IR model boundary, construct one outcome for each typed source, then
serialize and deserialize it and read its kind, source and Boolean claim.
Compile a probe that requests `Cancelled` without a source. Deserialize
outcomes with a unit `"cancelled"` kind, a `cancelled` kind with no `source`, an unknown
source, and a non-cancelled kind with a cancellation source member. Build a
`TimedOut` outcome from an execution timeout for which no cancellation event
was recorded. Round-trip every existing non-cancelled kind as a control.

At CG's bounded launcher/execution boundary, start a controllable run and
trigger a caller cancellation through the same control the launcher reads;
repeat with the timer owner triggering deadline cancellation. Observe the
event that actually stopped each run and the IR outcome constructed from it.
Reverse which event wins, including a later competing cancellation, and
assert that the constructed source follows the first observed event. A
producer that hard-codes or guesses either source must fail this control.
Check that the launcher's ordinary elapsed execution ceiling, with no
recorded cancellation event, remains `TimedOut`. IR-701 owns this integration
control after its source-bearing producer exists.

## Expected Results

The IR outcomes retain their distinct `Requested` and `Deadline` sources after
the round trip and have no Boolean claim. The no-source compile probe fails
and each malformed or mismatched wire input is refused; none is
given a default source. The execution timeout stays `TimedOut`. Every
non-cancelled kind retains its previous serialized spelling, carries no
cancellation source and retains its prior Boolean claim. Once CG's producer is
implemented, each live-run outcome equals the source of the event that stopped
that run, and the reversed event reverses the outcome source.

## Status

IR carrier, validation and local serialization controls are tagged in
`tests/it/kani_shared.rs`. CG currently has no Kani cancellation event reader
or IR `Cancelled` producer; IR-701 implements and tests that downstream source
observation, event reversal and QSL mapping.
