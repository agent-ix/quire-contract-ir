---
id: SR-10780
title: scope-boundary review of IR-778 cancellation origin
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-contract-ir; spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md,
  spec/kani/functional/FR-039-root-crate-public-interface.md, spec/kani/matrix/TC-449-kani-cancellation-origin.md,
  spec/kani/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-778. Exact reviewed head:. The new type belongs in the cycle-free root Kani module and Codegen owns its QSL conversion. The event-to-constructor producer responsibility is unresolved.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The contract says “the producer” supplies a recorded event but does not identify the owner or the concrete event source at the Kani outcome construction seam. Current Contract IR has no Kani cancellation raise site and Codegen has no cancelled-outcome producer. Specify which component reads the caller handle or timer-owner event and passes it to the IR constructor, so an implementation cannot merely accept a freely chosen Requested or Deadline label. | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:50-60 |

## Verdict

Needs a focused correction before handoff. The new criterion and TC-449 remain planned; no implementation test is claimed.

## Reviewed units

- `FR-030-AC-7` (examined): A producer given a caller-request cancellation event builds `Cancelled { source: Requested }`, and one given a timer-owner deadline cancellation event builds `Cancelled { source: Deadline }`; each serializes and deserializes with that exact `kind` source and no Boolean claim. Construction of a cancelled outcome without a source fails to compile. Deserialization refuses a unit `"cancelled"` kind, a missing or unknown source, and an origin on a non-cancelled kind. Every pre-existing non-cancelled kind round trips with its former kind spelling and without a cancellation source. A timeout event wi
- `FR-030` (examined): The `cancelled` kind carries one required `KaniCancellationSource` value: `Requested` when the caller-owned cancellation handle records a caller request, or `Deadline` when the frontend that owns the timer cancels that handle at its deadline. The producer supplies the recorded event at outcome construction; neither Contract IR nor a consumer derives it from `code`, `source_id`, `context`, elapsed time, a generic timeout, or a failed charge. The source is part of the `Cancelled` variant of `KaniOutcomeKind`, so every cancelled outcome has exactly one source and no other kind has one. `TimedOut`
- `FR-039` (examined): | `kani` | types `FiniteInput`, `FiniteObject`, `FiniteReference`, `PopulationCompleteness`, `ResourceBounds`, `ValidatedFiniteInput`, `KaniOutcome`, `KaniOutcomeKind`, `KaniCancellationSource`, `KaniOutcomeError` | FR-030, STD-001 |
- `TC-449` (examined): Supply the Contract IR boundary with each producer-reported source event: `Requested` from the caller's cancellation request and `Deadline` from the timer owner's cancellation. Serialize and deserialize each outcome and read its kind, source and Boolean claim. Compile a probe that requests `Cancelled` without a source. Deserialize outcomes with a unit `"cancelled"` kind, a `cancelled` kind with no `source`, an unknown source, and a non-cancelled kind with a cancellation source member. Build a `TimedOut` outcome from an execution timeout for which no cancellation event was recorded. Round-trip
- `TC-449-matrix` (examined): | TC-449 | Kani cancellation carries the producer's requested or deadline origin | Integration | P0 | FR-030-AC-7 | 🚧 planned; no test tagged yet |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: the revised contract names Codegen as event observer and TC-449 requires live first-event reversal under IR-701. |
