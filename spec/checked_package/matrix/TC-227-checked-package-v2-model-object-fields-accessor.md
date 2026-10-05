---
id: TC-227
title: "CheckedPackage V2 returns a model object type's fields and derived member types as values"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-227: CheckedPackage V2 returns a model object type's fields and derived member types as values

## Description

Verify FR-038-AC-136 through FR-038-AC-144 (IR-628): the reader computes, at
admission and under the `work` limit, each selected object type's effective field
table and retains it; the admitted `CheckedPackageV2` exports
`model_object_fields`, which returns a model declaration object type's fields in
ascending name order, each with the member type FR-322 step 4 derives (`Boolean`,
`Integer`, `IntRange` with `i128` bounds, `Reference`, `Option`, `Collection`) or
`None` where the tables give none, by the functions admission uses, independent
of every node body.

## Test Procedure

Build domain package documents in this repository and a lock selecting each:
one with two `Int[0, 1000]` fields and a read of one; one with an inheritance
chain, an inherited field, a redefinition with a narrower range and a
most-derived redefiner; one declaring every kind of AC-138 (the `i128` extremes
and one past each among them); one with an ambiguous name, in a package with no
read of it and a sibling with a read; one with a 10000-type chain and 1000
redefinitions. Build the checked packages in the test, keying every read's type
node by an independent recomputation through `quire-canonical`, never through
the reader. Admit each, call `model_object_fields` on the model declaration
node, and compare the result with the declared values. Call it with an unknown id
and with each non-model-object node of AC-139, including two admitted packages
that differ only in an unread model node's body. In a crate-internal test,
replace a read node's body bounds after admission and compare the result. For
each returned type, admit a read keyed from it and a read keyed from its
perturbation (AC-141). Compile the API fixtures of AC-142 (exhaustive matches, a
struct literal that must fail to compile). Find the smallest admitting `work`
limit for AC-144 and test one below it.

## Expected Results

The fields, order, types and `None` entries equal the declaration; each error is
the variant AC-139 names; a replaced node body changes nothing; a read keyed from
the returned type admits and its perturbation refuses `ill_typed`; the API
fixtures compile or fail as stated; the 10000-type document admits under a
sufficient limit and is `incomplete` below the charged sum, and the call does no
resolution.

## Status

PLANNED: no test exists. The code change adds the admission-time field tables,
the accessor and the five public types.
