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

Verify FR-030-AC-4 and FR-030-AC-5: the two `Unavailable` cause codes, the one
`Inconclusive` cause, the SUCCESS check count a `proved` outcome carries, and
the private fields and typed `KaniOutcomeError` that keep those rules from
being bypassed. The typed `Std001Code` every outcome and error carries is
FR-030-AC-6, verified under TC-443. The map from a Kani outcome to a QSL
`TerminalValue` is not verified here: it is owned by
`agent-ix/quire-contract-codegen`, tracked there under Linear IR-358. The case
previously named FR-031-AC-5, which is retired.

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
from outside the `kani` module. Enumerate `KaniOutcomeKind` in an exhaustive
`match` with no wildcard.

## Expected Results

The three-check proof is `proved` with count three. The zero-check run is
`inconclusive` with cause `kani_vacuous_proof`, no count and no Boolean claim.
The two `Unavailable` outcomes carry `kani_solver_absent` and
`kani_backend_absent`. The five invalid requests each return `KaniOutcomeError`
with code `kani_outcome_invalid` and no outcome, and the struct-literal probe
fails to compile. A new outcome kind fails to compile in the test's exhaustive
`match`.

## Status

Planned. No test is tagged for this case. The count-bearing `proved` request, the
`Unavailable` and `Inconclusive` cause rules and the struct-literal probe
(FR-030-AC-4, AC-5) are not built. The `non_success` refusal of a `proved` or
`counterexample` request is built and is asserted by the TC-443 test, but this
case does not claim it until the rest of AC-5 lands. The obsolete
`kani_outcome_kinds_map_to_their_one_fr331_result` test of the IR-owned
`KaniProviderResult` map was removed under IR-347; it did not back this case.
