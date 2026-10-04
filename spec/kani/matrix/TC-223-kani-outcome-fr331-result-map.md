---
id: TC-223
title: "Kani outcomes are built only by validated constructors, with their check count and cause codes"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: verifies
---
# TC-223: Kani outcomes are built only by validated constructors, with their check count and cause codes

## Description

Verify FR-030-AC-4 through FR-030-AC-6: the typed `Std001Code` every outcome
and error carries, the two `Unavailable` cause codes, the one
`Inconclusive` cause, the SUCCESS check count a `proved` outcome carries, and
the private fields and typed `KaniOutcomeError` that keep those rules from
being bypassed. The map from a Kani outcome to a QSL `TerminalValue` is not
verified here: it is owned by `agent-ix/quire-contract-codegen`, tracked there
under Linear IR-358. The case previously named FR-031-AC-5, which is retired.

## Test Procedure

Build, through FR-030's classification, a proof from a check count of three
and from a count of zero, and read each outcome's kind, cause, count and
Boolean claim. Build one outcome of each refusal kind and each limit kind and
read each kind and cause. Build an `Unavailable` outcome for an absent solver
and one for an absent backend, and read each cause code. Request an
`Unavailable` outcome with another cause code, an `Inconclusive` outcome with a
cause other than `kani_vacuous_proof`, a `proved` outcome with a count of zero,
and a `proved` and a `counterexample` outcome through the non-success
constructor. Compile a probe that builds a `KaniOutcome` with a struct literal
from outside the `kani` module, and one that passes a string literal as the
code of the non-success constructor. Serialize the zero-check outcome, and
deserialize an outcome whose `code` is `"Bad-Code"`. Enumerate `KaniOutcomeKind` in an exhaustive
`match` with no wildcard.

## Expected Results

The three-check proof is `proved` with count three. The zero-check run is
`inconclusive` with cause `kani_vacuous_proof`, no count and no Boolean claim.
The two `Unavailable` outcomes carry `kani_solver_absent` and
`kani_backend_absent`. The five invalid requests each return `KaniOutcomeError`
with code `kani_outcome_invalid` and no outcome, and the struct-literal probe
fails to compile. The zero-check outcome's code equals
`Std001Code::KANI_VACUOUS_PROOF` and serializes as `"kani_vacuous_proof"`, the
error's code equals `Std001Code::KANI_OUTCOME_INVALID`, the string-literal probe
fails to compile, and the `"Bad-Code"` outcome fails to deserialize. A new outcome kind fails to compile in the test's exhaustive
`match`.

## Status

Partly implemented. FR-030-AC-6 is verified by
`tc_223_the_outcome_and_its_error_carry_a_typed_std001_code` in
`tests/it/kani_shared.rs`: the zero-check outcome's code and serialization, the
`"Bad-Code"` refusal on read, and the error's code, read through the non-success
request for a `proved` outcome. The string-argument probes are `compile_fail`
doctests on `KaniOutcome::non_success`. The count-bearing `proved` request, the
`Unavailable` and `Inconclusive` cause rules and the struct-literal probe
(FR-030-AC-4, AC-5) are planned and no test is tagged for them. The test
`kani_outcome_kinds_map_to_their_one_fr331_result` verifies the retired
`KaniProviderResult` map and does not back this case.
