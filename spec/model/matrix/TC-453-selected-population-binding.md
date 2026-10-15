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

Verify that the public model API projects a QSL-produced population node only
when the selector names that exact node and source occurrence, preserves the
typed pre-admission unknown-closure outcome, and reports typed non-finite or
invalid selection outcomes.

## Test Procedure

1. Obtain the QSL checked-source fixture with two distinct bindings of
   `Population<M::Order>[N]` and one different object type. Extend the producer
   fixture, in the authoritative QSL checkout, to emit an actual checked V2
   package; consume that fixture here rather than copying its schema or
   synthesizing V2 bytes in the IR test.
2. Read the package through `CheckedPackageV2::read`, then select the exact
   `bounded_domain` / `model_population` node identity and its `Type`
   occurrence through a compiled default-feature consumer of
   `quire_contract_model::selected_population`.
3. Assert each selected result retains its package, population-node identity,
   occurrence, selected object-type identity, source regions, and exact
   inclusive `[0,N]`. Include one node with maximum zero. Change only a source
   maximum and regenerate its admitted fixture; confirm the selected node and
   extent change. Confirm the `N=0` result is `[0,0]`.
4. Select the other `Population<M::Order>[N]` occurrence and verify that it
   returns its own bound and provenance. Change only reader/Codegen resource
   ceilings and fixture object membership; verify neither changes either
   selected result.
5. Rebuild an otherwise valid fixture after changing one parameter node's
   `semantic_type` reference from the first population node to the second,
   re-deriving affected identities and `package_id`, while retaining the
   original source-map rows and original selector. The reader admits this
   retargeted package under FR-038. Query the original selector, then pair the
   second node with the first occurrence.
6. Verify a field's edge multiplicity only through
   `CheckedPackageV2::model_object_fields`; no population result or selected
   extent supplies or replaces that field multiplicity.
7. Query without a selector and assert `Unavailable` with no extent. Query
   selectors for an absent binding, stale binding, foreign package, a missing
   occurrence, and a valid non-population binding; assert the distinct typed
   outcomes and absence of extent for each.
8. Select a domain-package PopulationRecord with `Closed` extent and no
   maximum; assert `Unbounded` and no extent. Separately feed QSL's typed
   `AdmissionOutcome::UnknownClosure` through the neutral source adapter and
   assert the distinct `UnknownClosure` result retains selected model/source
   provenance, carries no V2 package or binding, and has no extent. Assert it
   differs from `Unavailable`. QSL's TC-226 verifies the owner-side typed
   admission result for open extent and unresolved object/subtype closure.
9. Inspect the query path and compile the public consumer. Verify the query
   reads finite `N` only from the named admitted population node body after
   requiring the matching node/occurrence source-map row; it never follows a
   parameter's `semantic_type`, and does not use domain closure, member or
   fixture counts, Codegen/IR limits, or a caller-supplied maximum. Verify the
   consumer needs no source-text parsing or QSL/Codegen dependency and the IR
   root crate exposes none of the model items.

## Expected Results

Every finite result matches the exact selected checked population node and
source occurrence, including `N=0`, and preserves that occurrence's
provenance. Re-pointing a parameter's `semantic_type` does not change the
selected node's result; pairing another node with a source occurrence not
mapped to that node returns `MissingOccurrence`. Same-type bindings remain
distinct. Missing, stale, foreign, non-population, unavailable, unbounded,
and unknown-closure outcomes are typed and carry no finite extent. Unknown
closure remains QSL's pre-admission outcome and is preserved through the
adapter without fabricating an admitted package. Closure, resource ceilings
and fixture membership never produce or alter a finite maximum. The model
consumer uses the selected population projection only for the population
extent; `model_object_fields` remains authoritative for edge multiplicity.

## Current Verification Status

The default-feature public consumer currently exercises QSL-emitted maxima
`0`, `1`, `3`, and `[2,5]` for two distinct same-type bindings. It verifies
exact node/occurrence identity, provenance, selected object type, reader-limit
invariance, typed non-finite outcomes, and rejection of malformed maximum
body shapes. These checks credit the available source and projection paths.

The QSL-799 producer extension is required to credit only the remaining
source-dependent controls: a regenerated package whose checked maximum alone
changes, a second object type, and a valid `semantic_type` retargeted and
re-keyed package. No copied V2 bytes or IR-authored substitute fixture will be
used for those controls.
