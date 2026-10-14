---
id: TC-453
title: "Verify selected finite population binding projection"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-053
    type: verifies
---
# TC-453: Verify selected finite population binding projection

## Description

Verify that the public model API projects an exact source-backed QSL
population binding from an admitted V2 package, retains its occurrence
provenance, and reports typed non-finite or invalid selection outcomes.

## Test Procedure

1. Obtain the QSL checked-source fixture with two distinct bindings of
   `Population<M::Order>[N]` and one different object type. Extend the producer
   fixture, in the authoritative QSL checkout, to emit an actual checked V2
   package; consume that fixture here rather than copying its schema or
   synthesizing V2 bytes in the IR test.
2. Read the package through `CheckedPackageV2::read`, then select the exact
   binding identity and `Expression` occurrence for each population binding
   through a compiled default-feature consumer of
   `quire_contract_model::checked_package`.
3. Assert each selected result retains its package, binding, occurrence,
   population type node, selected object-type identity, source regions, and
   exact inclusive `[0,N]`. Include one binding with maximum zero. Change only
   a source maximum and regenerate its admitted fixture; confirm the selected
   extent changes. Confirm the `N=0` result is `[0,0]`.
4. Select the other `Population<M::Order>[N]` occurrence and verify that it
   returns its own bound and provenance. Change only reader/Codegen resource
   ceilings and fixture object membership; verify neither changes either
   selected result.
5. Verify a field's edge multiplicity only through
   `CheckedPackageV2::model_object_fields`; no population result or selected
   extent supplies or replaces that field multiplicity.
6. Query without a selector and assert `Unavailable` with no extent. Query
   selectors for an absent binding, stale binding, foreign package, a missing
   occurrence, and a valid non-population binding; assert the distinct typed
   outcomes and absence of extent for each.
7. Select a domain-package PopulationRecord whose closure is `Closed`, then
   one whose closure is `Open`; both have no maximum. Assert the typed
   `Unbounded` result has no finite extent. Where QSL refuses an unbounded or
   unknown-closure source before V2 emission, assert that no admitted-package
   projection is produced and retain QSL's refusal as the upstream result.
8. Inspect the query path and compile the public consumer. Verify the query
   obtains a finite extent only from the selected admitted population binding
   and occurrence; it does not use domain closure, member or fixture counts,
   Codegen/IR limits, or a caller-supplied maximum. Verify the consumer needs
   no source-text parsing or QSL/Codegen dependency and the IR root crate
   exposes none of the model items.

## Expected Results

Every finite result matches the exact selected checked binding and source
occurrence, including `N=0`, and preserves that occurrence's authentic
provenance. Same-type bindings remain distinct. Missing, stale, foreign,
non-population, unavailable, and unbounded selections are typed and carry no
finite extent. Closure, resource ceilings and fixture membership never
produce or alter a finite maximum. The model consumer uses the selected
population projection only for the population extent; `model_object_fields`
remains authoritative for edge multiplicity.
