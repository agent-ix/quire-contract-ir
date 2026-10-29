---
id: SR-10781
title: integrity review of IR-778 cancellation origin
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir; spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md,
  spec/kani/functional/FR-039-root-crate-public-interface.md, spec/kani/matrix/TC-449-kani-cancellation-origin.md,
  spec/kani/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-778. Reviewed the PR head. The changed FR, interface inventory, test case, and matrix agree on the two-value source and planned status; no additional integrity defect was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean for this method. The new criterion and TC-449 remain planned; no implementation test is claimed.

## Reviewed units

- `FR-030-AC-7` (examined): A producer given a caller-request cancellation event builds `Cancelled { source: Requested }`, and one given a timer-owner deadline cancellation event builds `Cancelled { source: Deadline }`; each serializes and deserializes with that exact `kind` source and no Boolean claim. Construction of a cancelled outcome without a source fails to compile. Deserialization refuses a unit `"cancelled"` kind, a missing or unknown source, and an origin on a non-cancelled kind. Every pre-existing non-cancelled kind round trips with its former kind spelling and without a cancellation source. A timeout event wi
- `FR-030` (examined): The `cancelled` kind carries one required `KaniCancellationSource` value: `Requested` when the caller-owned cancellation handle records a caller request, or `Deadline` when the frontend that owns the timer cancels that handle at its deadline. The producer supplies the recorded event at outcome construction; neither Contract IR nor a consumer derives it from `code`, `source_id`, `context`, elapsed time, a generic timeout, or a failed charge. The source is part of the `Cancelled` variant of `KaniOutcomeKind`, so every cancelled outcome has exactly one source and no other kind has one. `TimedOut`
- `FR-039` (examined): | `kani` | types `FiniteInput`, `FiniteObject`, `FiniteReference`, `PopulationCompleteness`, `ResourceBounds`, `ValidatedFiniteInput`, `KaniOutcome`, `KaniOutcomeKind`, `KaniCancellationSource`, `KaniOutcomeError` | FR-030, STD-001 |
- `TC-449` (examined): Supply the Contract IR boundary with each producer-reported source event: `Requested` from the caller's cancellation request and `Deadline` from the timer owner's cancellation. Serialize and deserialize each outcome and read its kind, source and Boolean claim. Compile a probe that requests `Cancelled` without a source. Deserialize outcomes with a unit `"cancelled"` kind, a `cancelled` kind with no `source`, an unknown source, and a non-cancelled kind with a cancellation source member. Build a `TimedOut` outcome from an execution timeout for which no cancellation event was recorded. Round-trip
- `TC-449-matrix` (examined): | TC-449 | Kani cancellation carries the producer's requested or deadline origin | Integration | P0 | FR-030-AC-7 | 🚧 planned; no test tagged yet |
