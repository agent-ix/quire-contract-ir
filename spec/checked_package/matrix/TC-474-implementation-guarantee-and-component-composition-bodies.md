---
id: TC-474
title: "CheckedPackage V2 implementation guarantee and component composition bodies admit or refuse exactly"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-362
    type: verifies
  - target: ix://agent-ix/quire-specification/FR-490
    type: references
  - target: ix://agent-ix/quire-specification/FR-491
    type: references
---
# TC-474: CheckedPackage V2 implementation guarantee and component composition bodies admit or refuse exactly

## Description

Verify FR-362-AC-1 through FR-362-AC-6: the V2 reader admits QSpec FR-490's
`correspondence`/`implementation_guarantee` body and FR-491's
`correspondence`/`component_composition` body, retains them as typed terms, and
refuses every other shape with the code, cause and path FR-362 names.

## Test Procedure

Over the in-repo all-families fixture with the infinite-trace profile selected,
the orders domain package (`Order` with `scaled` and `reset`, and a subtype
`Sub` that only inherits them) and an empty abstraction relation node, add one
guarantee or composition node and read it. Admit the bodies of FR-362-AC-1 and
AC-2 and compare the typed body with the wire body. Then apply each single
mutation the criteria name and read each mutated package.

## Expected Results

The unmutated packages admit. Each mutation returns exactly the code, cause and
path its criterion names and no package.

## Status

Implemented in `tests/it/checked_package_v2_guarantee_composition.rs`.
Positive coverage of an `assumption` environment entry over a `predicate` node
is not yet exercised.
