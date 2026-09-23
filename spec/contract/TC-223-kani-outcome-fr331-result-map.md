---
id: TC-223
title: "Every Kani outcome kind maps to its one FR-331 terminal result"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: verifies
---
# TC-223: Every Kani outcome kind maps to its one FR-331 terminal result

## Description

Verify FR-031-AC-5: the Kani outcome kind to QSpec FR-331 terminal result map
that QSL ADR-013's O-16 proof column fixes and C-09 assigns to Contract IR.

## Test Procedure

Walk `KaniOutcomeKind::ALL` against a table written out from the O-16 proof
column, one row per kind. For each kind compare `provider_result()` with its
row, and the result's serialized form with the FR-331 wire value. Build a
vacuous proof through `KaniOutcome::proved_from_checks(0, ..)` and map it.

## Expected Results

`ALL` names each of the ten kinds exactly once. `Proved` maps to `proved`,
`Counterexample` to `refuted`, `Refused`, `InvalidInput` and `IncompleteInput`
to `declined`, `Unavailable` to `unsupported`, `TimedOut`, `ResourceExhausted`
and `Cancelled` to `incomplete`, and `Inconclusive` to `inconclusive`. A
vacuous proof carries the cause `kani_vacuous_proof` and maps to
`inconclusive`. Changing any arm of the map fails the test.

## Status

Implemented in `tests/it/kani_shared.rs`.
