---
id: TC-223
title: "Kani outcomes map to their QSL terminal value, and a non-vacuous inconclusive outcome to a typed absence"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-069
    type: references
---
# TC-223: Kani outcomes map to their QSL terminal value, and a non-vacuous inconclusive outcome to a typed absence

## Description

Verify FR-031-AC-5, FR-030-AC-4 and FR-030-AC-5: the map from each Kani
outcome to the one `qsl_replay::TerminalValue` that QSL ADR-013's O-16 proof
column and QSpec FR-331 give it, the typed absence for a non-vacuous
`Inconclusive` against a `TerminalValue` with no inconclusive arm, the two
`Unavailable` cause codes, and the SUCCESS check count a `proved` outcome
carries. The terminal value is QSL's type; the map is
Contract IR's.

## Test Procedure

Build, through FR-030's classification, a proof from a check count of three
and from a count of zero, and read each outcome's kind, cause, count and
Boolean claim. Map every outcome in FR-031's table and compare each value with
its row, written in the test as an exhaustive `match` over `KaniOutcomeKind`
with no wildcard. Build one outcome of each refusal kind and each limit kind
and read each mapped value's cause. Build an `Unavailable` outcome for an
absent solver and one for an absent backend, read each cause code, and map
both; attempt to build an `Unavailable` outcome with another cause code. Map
an `Inconclusive` outcome with a cause other than `kani_vacuous_proof`. Read each
mapped value's `category()` through QSL.

## Expected Results

The three-check proof is `proved` with count three and maps to
`Proved { success_checks: 3 }`, category `Success`. The zero-check run is
`inconclusive` with cause `kani_vacuous_proof`, no count and no Boolean
claim, and maps to `Proved { success_checks: 0 }`, category `Inconclusive`
with cause `KaniVacuousProof`. `Counterexample` maps to `Refuted`; `Refused`,
`InvalidInput` and `IncompleteInput` to `Declined` with three distinct
`ProofRefusalCause`s; `TimedOut`, `ResourceExhausted` and `Cancelled` to
`Incomplete` with three distinct `IncompleteCause`s. The two `Unavailable`
outcomes carry `kani_solver_absent` and `kani_backend_absent` and map to
`Unsupported(SolverAbsent)` and `Unsupported(BackendAbsent)`; the other cause
code yields no `Unavailable` outcome. The non-vacuous `Inconclusive` returns
the typed absence and no value. No outcome
maps to `Tested` or `Failed`. Changing any arm of the map fails the test, and
a new outcome kind fails to compile in the test's exhaustive `match`.

## Status

Planned. `tests/it/kani_shared.rs` verifies today's map onto this crate's own
`KaniProviderResult`, which the QSL target replaces.
