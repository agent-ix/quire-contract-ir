---
id: TC-453
title: "Verify selected population binding projection"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-053
    type: verifies
---
# TC-453: Verify selected population binding projection

## Description

Verify that the public model API projects a QSL-produced population node when the selector names that exact node and source occurrence, preserves the
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
   inclusive `[0,N]`, with `N` as validated canonical nonnegative decimal
   text. Include one node with maximum zero and one with a canonical decimal
   maximum above `u64::MAX`; assert both admit and project unchanged. Read the authoritative
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
7. Read an admitted reaches application linked to a `model_population` with
   no `max` and select its exact binding and `Type` occurrence. Assert
   `AdmittedUnbounded` retains package name, binding, occurrence, object type
   and authentic source regions, carries no finite maximum, and is distinct
   from a finite `max` of zero and from declaration-only `Unbounded`.
   Supply negative and non-integer maxima and a second body member in
   separately read packages; assert each reader refusal is
   `invalid_package`/`invalid-derived-shape` at that population node, rather
   than a projection result.
8. Query without a selector and assert `Unavailable` with no extent. Query
   selectors for a structural node ID absent from the named package, a foreign package name, a missing
   occurrence and a valid non-population binding; assert the distinct typed
   outcomes and absence of extent for each. A duplicated `max` fails reader
   admission under step 7.
9. Select a domain-package PopulationRecord with `Closed` extent and no
   maximum; assert `Unbounded` and no extent. Separately feed QSL's typed
   `AdmissionOutcome::UnknownClosure` through the neutral source adapter and
   assert the distinct `UnknownClosure` result retains selected model/source
   provenance, carries no V2 package or binding, and has no extent. Assert it
   differs from `Unavailable`. QSL's TC-226 verifies the owner-side typed
   admission result for open extent and unresolved object/subtype closure.
10. Inspect the query path and compile the public consumer. Verify the query
   reads finite `N` as exact canonical decimal text only from the named admitted population node body after
   requiring the matching node/occurrence source-map row; it never follows a
   parameter's `semantic_type`, and does not use domain closure, member or
   fixture counts, Codegen/IR limits, or a caller-supplied maximum. Verify the
   consumer needs no source-text parsing or QSL/Codegen dependency and the IR
   root crate exposes none of the model items.

## Expected Results

Every finite result, including one above `u64::MAX`, matches the exact selected checked population node and
source occurrence, including `N=0`, and preserves that occurrence's
provenance. Re-pointing a parameter's `semantic_type` does not change the
selected node's result; pairing another node with a source occurrence not
mapped to that node returns `MissingOccurrence`. Same-type bindings remain
distinct. Missing, absent-node `StaleBinding`, foreign-name `ForeignPackage`, non-population, unavailable, admitted unbounded, declaration-only unbounded,
and unknown-closure outcomes are typed and carry no finite extent. Unknown
closure remains QSL's pre-admission outcome and is preserved through the
adapter without fabricating an admitted package. Closure, resource ceilings
and fixture membership never produce or alter a finite maximum. The model
consumer uses the selected population projection only for the population
extent; `model_object_fields` remains authoritative for edge multiplicity.

## Current Verification Status

The existing default-feature public consumer exercises QSL-emitted maxima
`0`, `1`, `3`, and `[2,5]`, plus same-type/second-type, changed-maximum, and
retargeted variants through the pre-cutover `package_id: CheckedSemanticId`
selector and result. It verifies node/occurrence selection, provenance,
selected object type, reader-limit invariance, typed non-finite outcomes,
and malformed maximum body handling under that interface. Its changed-maximum
control currently expects a changed binding ID. No copied V2 bytes or
IR-authored substitute fixture is used.

PLANNED/UNRUN (IR-1141 #709): replace that consumer evidence with exact V2
package `name` and structural node-ID selection and result checks, a body-only
maximum change that retains the structural ID while changing extent, a
different admitted node and occurrence that change the selected ID and extent,
and exact foreign-package-name refusal. These structural controls are not
claimed as verified by the existing consumer.

PLANNED/UNRUN (IR-1141 derived-shape and population correction): the admitted
no-`max` outcome with exact binding and source provenance, canonical finite
maximum above `u64::MAX`, and node-located derived-shape refusals in steps 3
and 7 require new reader/projection evidence. The existing bounded-path
evidence remains historical; it does not back these corrected obligations.
