---
id: TC-475
title: "Checked-package reader changes for source-declared objects"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-362
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-363
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-364
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-365
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-366
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-367
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-368
    type: verifies
---
# TC-475: Checked-package reader changes for source-declared objects

## Description

Verify every AC of FR-362 through FR-368, FR-365-AC-6, FR-367-AC-6 and FR-368-AC-6 included at the public V2 reader boundary over
in-repo packages that declare a source object type with a field, an operation,
a query, a population and a subtype. Build every package in the test; copy no
fixture from QSL or QSpec.

## Test Procedure

1. Build the base source-object package: a source type with a field, a
   modifying operation with an anchor, a query, a population and a subtype that
   redefines the query. Admit it; resolve a field read; join the source-object
   rows; read the protocol owner, population reference, operation effects,
   dispatch table and operation anchor witness (happy paths).
2. Build a second package that mixes a selected domain model with source types
   (including a same-name pair), a model protocol clause, model effect rows, a
   model dispatch table and model operation-member rows. Check each
   model-unchanged AC (FR-362-AC-3, FR-363-AC-5, FR-365-AC-5, FR-365-AC-6,
   FR-366-AC-5, FR-367-AC-6, FR-368-AC-5).
3. Edit one member per case and compare the code, cause and locus each FR
   names: the lock entry; a protocol edge to a non-object-type source node;
   population positions, a population with a `ModelOwner` and a declaration, a
   `SourceOwner` with no declaration, and a repeated population name; effect
   row `document`, `provenance.member`, origin in both directions, repeated key
   and `effect` against the anchor frame; the three clause-function forms,
   dispatch candidate and clause identities, a mixed source and model row, and
   the FR-367 family check; row key, `members`, `supertypes`, supply order of
   foreign, unknown, duplicate and missing rows; anchor rows and the
   `operation_effects` present and absent cases.
4. Re-run the existing TC-048, TC-228, TC-229, TC-463 and TC-472 suites and
   the FR-064 protocol-owner tests with no expectation edit.

## Expected Results

Each valid package admits with the stated typed reads. Each edit refuses at its
first failing check. Every package without a source object reads unchanged.
Every test binds its criterion with `#[trace]`.

## Status

Planned (IR-1529).
