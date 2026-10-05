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

Verify FR-038-AC-136 through FR-038-AC-143 (IR-628): the admitted
`CheckedPackageV2` exports `model_object_fields`, which returns a model
declaration object type's effective fields in ascending name order, each with
the member type FR-322 step 4 derives (`Boolean`, `Integer`, `IntRange` with
`i128` bounds, `Reference`, `Option`, `Collection`) or `None` where the tables
give none, from the model the reader retained at admission, by the functions
admission uses, and independent of every node body.

## Test Procedure

Build domain package documents in this repository and a lock selecting each:
one with two `Int[0, 1000]` fields and a read of one; one with an inheritance
chain, an inherited field, a redefinition with a narrower range and a
most-derived redefiner; one declaring every kind of AC-138 (the `i128` extremes
among them); one with an ambiguous name; one with a 10000-type chain and 1000
redefinitions. Build the checked packages in the test, keying every read's
type node by an independent recomputation through `quire-canonical`, never
through the reader. Admit each, call `model_object_fields` on the model
declaration node, and compare the result with the declared values. Call it with
an unknown id and with each non-model-object node of AC-139. Admit the AC-123
tampered packages with a reader that has no IR-627 stage (the test's package
builder before that stage lands, or the unmutated package after) and compare the
result with the unmutated package's. For each returned type, admit a read keyed
from it and a read keyed from it perturbed by one unit. Compile the API
fixtures of AC-142 (exhaustive matches, a struct literal that must fail to
compile) and scan the new items for `#[non_exhaustive]`.

## Expected Results

The fields, order, types and `None` entries equal the declaration; each error is
the variant AC-139 names; a tampered node body changes nothing; a read keyed from
the returned type admits and its perturbation refuses `ill_typed`; the API
fixtures compile or fail as stated; the 10000-type document returns without panic
or limit result.

## Status

PLANNED: no test exists. The code change adds the retained model, the accessor and
the five public types, and the FR-019 table row.
