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

1. Run QSL's authoritative `population_v2_fixtures` producer and consume its
   `SameTypeAndOtherType`, `ChangedMax`, and `RetargetedParameter` emitted V2
   packages here. Do not copy schema or synthesize V2 bytes in the IR test.
2. Read the package through `CheckedPackageV2::read`, then select the exact
   `bounded_domain` / `model_population` node identity and its `Type`
   occurrence through a compiled default-feature consumer of
   `quire_contract_model::selected_population`.
3. Assert each selected result retains the exact package `name`, population-node structural ID,
   occurrence, selected object-type identity, source regions, and exact
   inclusive `[0,N]`. Include one node with maximum zero. Read the authoritative
   body-only changed-maximum variant and confirm its selected node ID and
   package name remain stable while its maximum and extent change. Select a
   different admitted population node with its own occurrence and confirm
   that its returned node ID and extent differ. Confirm the `N=0` result is `[0,0]`.
4. Select the second same-type `Population<M::Gadget>[N]` occurrence and the
   `Population<M::Widget>[N]` occurrence; verify each returns its own bound,
   object type, and provenance. Change only reader/Codegen resource ceilings
   and fixture object membership; verify neither changes either selected
   result.
5. Read the producer's structurally valid retargeted package with updated
   references and retained structural IDs, where one parameter's
   `semantic_type` points from the first same-type population node to the
   second. Confirm the complete package value changes while its exact `name`
   remains the same; direct selection still returns the original population
   nodes, extents, object types, and source-map provenance.
6. Verify a field's edge multiplicity only through
   `CheckedPackageV2::model_object_fields`; no population result or selected
   extent supplies or replaces that field multiplicity.
7. Query without a selector and assert `Unavailable` with no extent. Query
   selectors for a structural node ID absent from the named package, a foreign package name, a missing
   occurrence, a valid non-population binding, and a reader-admitted package
   with a duplicated `max` binding; assert the
   distinct typed outcomes and absence of extent for each.
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
distinct. Missing, absent-node `StaleBinding`, foreign-name `ForeignPackage`, non-population, unavailable, unbounded,
and unknown-closure outcomes are typed and carry no finite extent. Unknown
closure remains QSL's pre-admission outcome and is preserved through the
adapter without fabricating an admitted package. Closure, resource ceilings
and fixture membership never produce or alter a finite maximum. The model
consumer uses the selected population projection only for the population
extent; `model_object_fields` remains authoritative for edge multiplicity.

## Current Verification Status

The default-feature public consumer exercises QSL-emitted maxima `0`, `1`,
`3`, and `[2,5]`, plus the authoritative same-type/second-type, changed-maximum,
and retargeted variants. It verifies exact package name and node/occurrence identity,
provenance, selected object type, reader-limit invariance, typed non-finite
outcomes, and rejection of malformed maximum body shapes, including a
reader-admitted duplicate-`max` package. No copied V2 bytes or IR-authored
substitute fixture is used.
