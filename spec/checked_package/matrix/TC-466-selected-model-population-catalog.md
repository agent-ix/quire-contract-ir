---
id: TC-466
title: "Selected-model population catalog includes unused declarations under finite limits"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-466: Selected-model population catalog includes unused declarations under finite limits

## Description

Verify FR-038-AC-474 through FR-038-AC-477 through the admitted
`CheckedPackageV2` public API.

## Test Procedure

Admit a checked package selecting two real model documents in reversed graph
order. One model declares two populations with distinct original identities
and the same member type, one never used by any application; the other model
declares an empty population, or no population in a separate control. Read the
catalog and compare every selected model/root, population and member identity
and normalized structural `EffectiveId`, including its complete type owner
component. In a crate-local defensive control, perturb retained member data
after admission to an unresolved, cyclic-alias or wrong-kind type and require
the accessor's typed whole-call refusal. Clone the
package and compare the result. Discover the smallest successful work and
entry allowances and call again one below each; compare typed refusal kind,
consumed amount, first unpaid locus and absence of a partial result.

## Expected Results

Every selected root and source-unused population is present, in identity
order. Distinct population originals remain distinct even with the same
normalized member type. The complete original member set is returned in
identity order; an unresolved member is explicit. The exact allowance returns
the same borrowed catalog as a larger allowance, one below refuses without
partial output, and a cloned package produces the same values and charges.

## Status

IMPLEMENTED (IR-1358): `selected_population_catalog_keeps_unused_origins_and_full_effective_type_ids` and `selected_population_catalog_refuses_wrong_kind_without_partial_result` pass against admitted packages; `population_catalog_defensively_refuses_a_retained_unresolved_member` passes for the retained-state invariant control. Public inventory tests and scoped Clippy pass on PR #923.
