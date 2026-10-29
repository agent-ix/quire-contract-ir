---
id: SR-15434
title: "IR-275 PR 509 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-275; PR #509; crates/quire-contract-model/src/checked_package/v2/typed_terms.rs; crates/quire-contract-model/src/checked_package/v2/mod.rs; crates/quire-contract-model/src/checked_package/v2/relationship_end.rs; tests/it/checked_package_v2_typed_terms.rs; spec/checked_package/functional/FR-349-retain-typed-checked-v2-terms.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

FR-349 has three criteria explicitly planned for later slices. The AC-1 trace currently overstates admitted-package coverage: one test exercises fabricated terms outside the reader, and the admitted fixture does not include every specified production.

## Verdict

**CONDITIONAL** — FR-349 has three criteria explicitly planned for later slices. The AC-1 trace currently overstates admitted-package coverage: one test exercises fabricated terms outside the reader, and the admitted fixture does not include every specified production.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-1 is tagged by a direct from_validated round trip over fabricated JSON; the admitted-package test asserts only one aggregate and never admits a dependency_reference term. Add reader-admitted controls for the absent production before claiming every admitted root and nested production. | tests/it/checked_package_v2_typed_terms.rs:11 |
| FND-002 | medium | AC-2 remains untagged and the repository still has admitted-body string-key consumers. The draft identifies IR-969 through IR-973 as later migrations; keep the criterion explicitly partial until those slices land. | FR-349-AC-2 |
| FND-003 | medium | AC-4 remains untagged; no changed test compares the first code, cause and pointer for malformed or unknown terms through the new retained-term path. | FR-349-AC-4 |
| FND-004 | medium | AC-5 remains untagged; no changed test exercises the typed decoder at the configured input boundary or checks an over-limit refusal. | FR-349-AC-5 |

## Coverage

Reviewed the PR. Reviewed only PR #509's changed code and FR-349/FR-019 spec delta. Plan completion: not assessed. Computed strict matrix: FR-349-AC-1 and AC-3 tagged; AC-2, AC-4 and AC-5 untagged. The three untagged criteria were declared planned/partial in the draft PR. No semantic review opt-in was requested; only mechanical trace and diff-scoped code/test alignment were checked.


## Reviewed units

- FR-349-AC-1 (examined): Reading a conforming V2 package returns a typed body variant for every admitted root production; aggregate members and application arguments expose typed nested productions, including bindings and dependency references.
- FR-349-AC-2 (examined): Frame, abstraction-relation and extension-form nodes expose distinct typed bodies with their admitted members, and no admitted body is selected by a caller's string-key lookup.
- FR-349-AC-3 (examined): Each admitted package serializes and canonically encodes to the same bytes and identities as before the representation change, including the two currently required copies of every body.
- FR-349-AC-4 (examined): A malformed or unknown term still refuses at the same first code, cause and JSON pointer as the existing reader, before an admitted typed package is returned.
- FR-349-AC-5 (examined): A nested term at the configured input limit is processed without call-stack overflow, and an over-limit term refuses under the existing read limit.
- FR-019 public-items row (examined): The checked_package V2 typed terms are listed in the Rust library interface.
- FR-038 (context_only): The reader owns wire order, limits, and refusal envelope.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, admitted positive fixtures now assert aggregate, reference, application and literal roots, nested binding/literal/reference, and the existing dependency-call fixture asserts the admitted application's typed DependencyReference callee with exact package and node digests. |
| FND-002 | deferred | FR-349-AC-2 remains planned outside this typed first slice; IR-969 through IR-973 own the remaining admitted-reader, operation, operand, lowering and CG consumer migrations. |
| FND-003 | deferred | FR-349-AC-4 first-refusal regression controls are outside this draft first slice and remain planned for the subsequent reader-control work stated in PR #509. |
| FND-004 | deferred | FR-349-AC-5 typed-decoder input-boundary controls are outside this draft first slice and remain planned for the subsequent reader-control work stated in PR #509. |
