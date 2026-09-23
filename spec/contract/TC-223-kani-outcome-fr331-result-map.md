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

For every `KaniOutcomeKind`, compare `provider_result()` with its row of the
O-16 proof column, written in the test as an exhaustive `match` with no
wildcard, and compare the result's serialized form with the FR-331 wire value.
Build one outcome of each refusal kind with a distinct cause and read each
`provider_record()`. Build a zero-check proof through
`KaniOutcome::proved_from_checks(0, ..)` and read its record.

## Expected Results

`Proved` maps to `proved`, `Counterexample` to `refuted`, `Refused`,
`InvalidInput` and `IncompleteInput` to `declined`, `Unavailable` to
`unsupported`, `TimedOut`, `ResourceExhausted` and `Cancelled` to
`incomplete`, and `Inconclusive` to `inconclusive`. The three refusal records
are all `declined` and keep three distinct causes. The zero-check proof
records `inconclusive` with cause `kani_vacuous_proof`. Changing any arm of the
map fails the test, and a new outcome kind fails to compile in the test's
exhaustive `match`.

## Status

Implemented in `tests/it/kani_shared.rs`.
