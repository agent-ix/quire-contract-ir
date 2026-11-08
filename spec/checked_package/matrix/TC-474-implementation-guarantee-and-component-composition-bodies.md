---
id: TC-474
title: "Implementation guarantee and component composition bodies"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-474: Implementation guarantee and component composition bodies

## Description

Verify FR-038-AC-531 through FR-038-AC-538: the strict V2 reader admits the
`correspondence`/`implementation_guarantee` and `correspondence`/
`component_composition` bodies of QSpec FR-490 and FR-491 as typed terms and
refuses every other shape, identity, order, target and member in reader order.

## Test Procedure

Build each package in-repo from the temporal fixture (a clause under the
infinite-trace profile, the `Order` and `Sub` object types), an empty
abstraction relation node, a `predicate` node and the node under test. Admit
the valid guarantee and composition and compare the typed term with the body.
Then edit one member per case: wrong shapes, `semantic_type` and
`dependencies`, array order, target kinds, unresolved or inherited
operations, non-Rust paths, `bound` 0, duplicate progress and environment
members, duplicate components and undeclared or self-referring premises.

## Expected Results

The valid nodes admit with their typed terms. Each edited case refuses with
the code, cause and pointer FR-038 "Implementation guarantee and component
composition bodies" names.

## Status

Implemented in `tests/it/checked_package_v2_guarantee_composition.rs`.
QSpec FR-491 static checks 4 and 5 are not made: they read the component
packages.
