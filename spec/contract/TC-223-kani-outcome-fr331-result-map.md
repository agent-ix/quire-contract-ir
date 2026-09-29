---
id: TC-223
title: "Every Kani outcome maps to its one QSL terminal value, and outcomes are built only by validated constructors"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: verifies
  - target: ix://agent-ix/quire-spec-language/FR-069
    type: references
---
# TC-223: Every Kani outcome maps to its one QSL terminal value, and outcomes are built only by validated constructors

## Description

Verify FR-031-AC-5, FR-030-AC-4 and FR-030-AC-5: the map from each Kani
outcome to the one `qsl_replay::TerminalValue` that QSL ADR-013's O-16 proof
column and QSpec FR-331 give it, the two `Unavailable` cause codes, the one
`Inconclusive` cause, the SUCCESS check count a `proved` outcome carries, and
the private fields and typed `KaniOutcomeError` that keep those rules from
being bypassed. The terminal value is QSL's type; the map is
Contract IR's.

## Test Procedure

Build, through FR-030's classification, a proof from a check count of three
and from a count of zero, and read each outcome's kind, cause, count and
Boolean claim. Map every outcome in FR-031's table and compare each value with
its row, written in the test as an exhaustive `match` over `KaniOutcomeKind`
with no wildcard. Build one outcome of each refusal kind and each limit kind
and read each mapped value's cause. Build an `Unavailable` outcome for an
absent solver and one for an absent backend, read each cause code, and map
both. Request an `Unavailable` outcome with another cause code, an
`Inconclusive` outcome with a cause other than `kani_vacuous_proof`, a
`proved` outcome with a count of zero, and a `proved` and a `counterexample`
outcome through the non-success constructor. Compile a probe that builds a
`KaniOutcome` with a struct literal from outside the `kani` module. Read each
mapped value's `category()` through QSL.

## Expected Results

The three-check proof is `proved` with count three and maps to
`Proved { success_checks: 3 }`, category `Success`. The zero-check run is
`inconclusive` with cause `kani_vacuous_proof`, no count and no Boolean
claim, and maps to `Proved { success_checks: 0 }`, category `Inconclusive`
with `vacuous_proof_cause()` `KaniVacuousProof` (QSpec FR-331-AC-8). `Counterexample` maps to `Refuted`; `Refused`,
`InvalidInput` and `IncompleteInput` to `Declined` with three distinct
`ProofRefusalCause`s; `TimedOut`, `ResourceExhausted` and `Cancelled` to
`Incomplete` with three distinct `IncompleteCause`s. The two `Unavailable`
outcomes carry `kani_solver_absent` and `kani_backend_absent` and map to
`Unsupported(SolverAbsent)` and `Unsupported(BackendAbsent)`. The five invalid
requests each return `KaniOutcomeError` with code `kani_outcome_invalid` and
no outcome, and the struct-literal probe fails to compile. No outcome
maps to `Tested` or `Failed`. Changing any arm of the map fails the test, and
a new outcome kind fails to compile in the test's exhaustive `match`.

## Status

Planned. The test at `tests/it/kani_shared.rs:250` carries the TC-223 and
FR-031-AC-5 tags but verifies the retired `KaniProviderResult` map, so it does
not back this case.
