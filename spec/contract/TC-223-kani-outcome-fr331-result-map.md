---
id: TC-223
title: "Every Kani outcome kind maps to its one QSL terminal value"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-069
    type: references
---
# TC-223: Every Kani outcome kind maps to its one QSL terminal value

## Description

Verify FR-031-AC-5: the map from each Kani outcome kind to the one
`qsl_replay::TerminalValue` that QSL ADR-013's O-16 proof column gives it.
The terminal value and record are QSL's types; the map is Contract IR's.

## Test Procedure

For every `KaniOutcomeKind` with a decided target, compare the mapped
`TerminalValue` with its row of the O-16 proof column, written in the test as
an exhaustive `match` with no wildcard. Build one outcome of each refusal kind
and each limit kind and read each mapped value's cause. Build a proof from a
check count of zero and of three and read each mapped value, then read its
`category()` through QSL.

## Expected Results

`Proved` maps to `Proved { success_checks }` with the run's count,
`Counterexample` to `Refuted`, `Refused`, `InvalidInput` and
`IncompleteInput` to `Declined` with three distinct `ProofRefusalCause`s, and
`TimedOut`, `ResourceExhausted` and `Cancelled` to `Incomplete` with three
distinct `IncompleteCause`s. The zero-check proof maps to
`Proved { success_checks: 0 }`, whose QSL category is `Inconclusive` with cause
`KaniVacuousProof`; the three-check proof's category is `Success`. No kind maps
to `Tested` or `Failed`. Changing any arm of the map fails the test, and a new
outcome kind fails to compile in the test's exhaustive `match`.

## Status

Planned. `tests/it/kani_shared.rs` verifies today's map onto this crate's own
`KaniProviderResult`, which the QSL target replaces.
