---
id: TC-049
title: "CheckedPackage V1 to V2 migration returns the exact MigrationOutcome"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-217
    type: references
---
# TC-049: CheckedPackage V1 to V2 migration returns the exact MigrationOutcome

## Description

Verify FR-038-AC-6 (QSpec FR-322-AC-12) against the vendored
`migration-correspondence-vectors.json`.

## Test Procedure

Admit each vector's V1 source and V2 target package. Migrate with the vector's
reconstruction inputs and target key and compare the serialized outcome with
the recorded outcome. Apply each refusal vector's RFC 6902 patch to its base
record and migrate with the patched inputs and target key. Separately migrate a
V2 target whose node families or lock differ from the V1 source, and a V2
target whose lock selects a `sha256-jcs` domain package.

## Expected Results

Positive vectors return the exact relinked correspondence, including every
ordered row and basis. Every refusal vector returns its exact code and subject
as the first failing check. A mismatched target, and a target selecting a
domain package that no V1 compiled-model input can reconstruct, refuses as
`migration_target_incompatible`; no refusal exposes a V2 package.
