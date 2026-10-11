---
id: SR-8342
title: "spec-review/ears-conformance review of IR-763 structural candidates"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@47be2719a17f4f6ceaf2a59c959464593395d08d; spec/model/functional/FR-048, FR-019; spec/checked_package/matrix/TC-048; spec/model/matrix/TC-058, tests.md; spec/spec.md"
review_set: subset
---

## Summary

PR #362 at 47be2719a17f4f6ceaf2a59c959464593395d08d; The new FR statement uses a named actor, event trigger and concrete response; the scoped grammar check reported no finding.

## Verdict

**PASS** — The new FR statement uses a named actor, event trigger and concrete response; the scoped grammar check reported no finding.

## Examined scope

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-048 | examined | When a caller assembles a CheckedPackage V2 graph before reader admission, `quire-contract-model` SHALL derive typed candidate structural node identities and recursion-group labels through the reader's canonical rules. The result is assembly data, not an admission verdict. |
| FR-019 | context_only | \| `checked_package` (V2 candidate structural identity) \| function `derive_structural_candidates`; types `StructuralCandidates`, `CandidateStructuralNodeId`, `CandidateRecursionGroupLabel`, `StructuralCandidateError` \| FR-048 \| \| `checked_package` (V2 scalar operands) \| types `CheckedScalarOperand`, `CheckedScalarOperandChild`, `CheckedScalarOperandError`, `CheckedScalarOperandRange` \| FR-038 \| \| `checked_package` (V2 clause comparisons) \| types `CheckedClauseComparison`, `CheckedComparisonClause |
| TC-058 | context_only | Compile and run an external default-feature `quire_contract_model` consumer that builds ordered `CheckedSemanticNodeV2` drafts, calls `derive_structural_candidates`, obtains each typed candidate node ID and group label through accessors, and exhaustively matches `StructuralCandidateError`. Compile-fail probes shall reject direct construction of candidate results and imports through `quire_contract_ir`. Compare the default-feature root export set against the updated FR-019 inventory, retaining th |
| TestMatrix | context_only | \| FR-048 \| FR-048-AC-1 through FR-048-AC-5 \| TC-048, TC-058 \| 🚧 planned for IR-763: TC-048 compares structural/group candidates with reader admission and typed stale-key, shape, byte and work refusals; TC-058 compiles the public candidate API and checks FR-019's inventory. No tagged implementation tests have run. \| |
| SpecIndex | context_only | [FR-048](model/functional/FR-048-derive-structural-and-group-candidates.md) defines a typed pre-read candidate boundary for the reader's structural keys and recursion-group labels. Final checked-package admission remains with [FR-038](checked_package/functional/FR-038-consume-checked-package-v2.md). STD-001 is the stable diagnostic code registry, FR-044 exports the form of its codes as the typed `Std001Code` of the model crate, which `KaniOutcome.code` carries, and STD-003 is the closed output-m |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
