---
id: TC-229
title: "CheckedPackage V2 reads declared population references"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---
# TC-229: CheckedPackage V2 reads declared population references

## Description

Verify FR-038-AC-303 through FR-038-AC-307 at the public V2 reader boundary
against QSpec FR-322-AC-82/83. Read QSpec's selected checkout at test time;
copy no upstream fixture, schema or binary into this repository.

## Test Procedure

1. Read QSpec's `model-owner-form-vectors.json` for the `counters` and
   `archive` P7/P8 preimages and compare them with the production declaration
   key function. Build selected documents with population entries and admit
   self-typed `relation`/`population` nodes through the public reader. Omit the
   node owner and projection owner separately; change fixed members and owner
   strings separately.
2. Join each valid owner to its exact selected `identity` and `node`. Change
   the owner's node to an object type and an unknown node, then change an
   object-type or relationship node's owner to a declared population. Compare
   code, cause and node-id locus, with the two UTF-8 owner strings as the
   declaration key.
3. Apply the `population_reference_cases` recipes from QSpec's `adverse.json`
   to the selected positive operation fixture. Admit `all_instances` and
   `lookup` over each declared population and the published parameter operand.
   Check dependencies, identities, the shared reference node, and a maximum
   carried in the result type only. Exercise a population reference in a
   collection operand, reaches population link and each forbidden declaring
   member position.
4. Admit QSpec's amended published operation fixture so the reader
   authenticates its published package and application identities. Check the
   population binding's operand identity and both query dependencies. Apply
   QSpec's ownerless relation/population case and check the schema refusal.

## Expected Results

The two declared populations have distinct authenticated keys and admit only
with matching selected declarations. An absent owner refuses `malformed_wire`
before the key and owner joins; a missing or wrong-kind declaration refuses
`missing_declaration`/`missing-selection`. Both model operations admit a
declared reference and the existing typed parameter binding, while forbidden
uses refuse `ill_typed`/`operator-ineligible` at the offending position. The
amended QSpec fixture's identities match exactly, and the retired ownerless
node refuses. Every test binds its requirement with `#[trace]`.

## Status

Implemented by IR-937's traced production reader and QSpec conformance tests.
