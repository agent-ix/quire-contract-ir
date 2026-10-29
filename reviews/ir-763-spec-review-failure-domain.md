---
id: SR-8344
title: "spec-review/failure-domain review of IR-763 structural candidates"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-048, FR-019; spec/checked_package/matrix/TC-048; spec/model/matrix/TC-058, tests.md; spec/spec.md"
review_set: subset
---

## Summary

PR #362; Identity, malformed-shape, resource and deep-stack failure domains have explicit candidate boundaries.

## Verdict

**PASS** — Identity, malformed-shape, resource and deep-stack failure domains have explicit candidate boundaries.

## Examined scope

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-048 | examined | When a caller assembles a CheckedPackage V2 graph before reader admission, `quire-contract-model` SHALL derive typed candidate structural node identities and recursion-group labels through the reader's canonical rules. The result is assembly data, not an admission verdict. |
| FR-048-AC-3 | examined | A reader-admissible group with structural and application members, including an off-cycle member, produces member candidates and one label agreeing with reader admission in graph order. Permuting members requires new ordinals, keys and label. A body/member change changes its local label candidate; retaining an old member key or a corrupt retained label yields the reader's typed stale-key refusal at the existing pass. |
| FR-048-AC-4 | examined | Repeated selectors, an unknown tag/form, malformed closed shapes (including bounded-domain type, literal type and decimal grammar), an invalid model-owned fixed shape, and a group member outside this candidate API's local-digest scope each return a distinct typed error and no candidate result. The helper does not normalize the invalid draft or widen reader admission. |
| FR-048-AC-5 | examined | Exact canonical byte ceiling succeeds; one less and zero retain the typed canonical error. The first unpaid work charge returns typed exhaustion with position and count. A deeply nested in-memory draft reaches a typed result or refusal on a 256 KiB stack without recursive traversal or cleanup. Inspection finds one structural preimage/encoder and unchanged reader charge and refusal precedence. |
| FR-019 | context_only | \| `checked_package` (V2 candidate structural identity) \| function `derive_structural_candidates`; types `StructuralCandidates`, `CandidateStructuralNodeId`, `CandidateRecursionGroupLabel`, `StructuralCandidateError` \| FR-048 \| \| `checked_package` (V2 scalar operands) \| types `CheckedScalarOperand`, `CheckedScalarOperandChild`, `CheckedScalarOperandError`, `CheckedScalarOperandRange` \| FR-038 \| \| `checked_package` (V2 clause comparisons) \| types `CheckedClauseComparison`, `CheckedComparisonClause |
| TC-058 | context_only | Compile and run an external default-feature `quire_contract_model` consumer that builds ordered `CheckedSemanticNodeV2` drafts, calls `derive_structural_candidates`, obtains each typed candidate node ID and group label through accessors, and exhaustively matches `StructuralCandidateError`. Compile-fail probes shall reject direct construction of candidate results and imports through `quire_contract_ir`. Compare the default-feature root export set against the updated FR-019 inventory, retaining th |
| TestMatrix | context_only | \| FR-048 \| FR-048-AC-1 through FR-048-AC-5 \| TC-048, TC-058 \| 🚧 planned for IR-763: TC-048 compares structural/group candidates with reader admission and typed stale-key, shape, byte and work refusals; TC-058 compiles the public candidate API and checks FR-019's inventory. No tagged implementation tests have run. \| |
| SpecIndex | context_only | [FR-048](model/functional/FR-048-derive-structural-and-group-candidates.md) defines a typed pre-read candidate boundary for the reader's structural keys and recursion-group labels. Final checked-package admission remains with [FR-038](checked_package/functional/FR-038-consume-checked-package-v2.md). STD-001 is the stable diagnostic code registry, FR-044 exports the form of its codes as the typed `Std001Code` of the model crate, which `KaniOutcome.code` carries, and STD-003 is the closed output-m |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
