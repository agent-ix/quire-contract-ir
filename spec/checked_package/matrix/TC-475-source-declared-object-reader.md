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

Verify FR-362-AC-1 through FR-368-AC-5 at the public V2 reader boundary over
in-repo packages that declare a source object type with a field, an operation,
a query, a population and a subtype. Build every package in the test; copy no
fixture from QSL or QSpec.

## Test Procedure

1. Build the base source-object package. Admit it; resolve a field read; join
   the source-object rows; read the protocol owner, population reference,
   operation effects, dispatch table and operation anchor witness (FR-362
   through FR-368 happy paths).
2. Edit one member per case: lock source entry, `SourceOwner`, population
   owner, effect row `document`, `provenance.member`, origin, effect members,
   dispatch candidate and clause identities, row key, `members`, `supertypes`,
   and anchor rows. Compare code, cause and locus with the requirement.
3. Re-read the existing TC-048, TC-228, TC-229 and TC-463 packages and compare
   typed reads, re-encoded bytes and refusals with their prior results.

## Expected Results

Each valid package admits with the stated typed reads. Each edit refuses at its
first failing check. Every package without a source object reads unchanged.
Every test binds its criterion with `#[trace]`.

## Status

Planned (IR-1529).
