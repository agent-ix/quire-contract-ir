---
id: SR-15436
title: "IR-275 PR 509 ears-conformance review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; IR-275; PR #509; crates/quire-contract-model/src/checked_package/v2/typed_terms.rs; crates/quire-contract-model/src/checked_package/v2/mod.rs; crates/quire-contract-model/src/checked_package/v2/relationship_end.rs; tests/it/checked_package_v2_typed_terms.rs; spec/checked_package/functional/FR-349-retain-typed-checked-v2-terms.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

The new requirement statements use named actors, SHALL responses and observable conditions; no EARS defect was found.

## Verdict

**PASS** — The new requirement statements use named actors, SHALL responses and observable conditions; no EARS defect was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Exact reviewed. Reviewed only PR #509's changed code and FR-349/FR-019 spec delta. Plan completion: not assessed. Quire validation of the changed spec artifacts passed without EARS warnings. The standalone stray word is recorded in the base review only.


## Reviewed units

- FR-349-AC-1 (examined): Reading a conforming V2 package returns a typed body variant for every admitted root production; aggregate members and application arguments expose typed nested productions, including bindings and dependency references.
- FR-349-AC-2 (examined): Frame, abstraction-relation and extension-form nodes expose distinct typed bodies with their admitted members, and no admitted body is selected by a caller's string-key lookup.
- FR-349-AC-3 (examined): Each admitted package serializes and canonically encodes to the same bytes and identities as before the representation change, including the two currently required copies of every body.
- FR-349-AC-4 (examined): A malformed or unknown term still refuses at the same first code, cause and JSON pointer as the existing reader, before an admitted typed package is returned.
- FR-349-AC-5 (examined): A nested term at the configured input limit is processed without call-stack overflow, and an over-limit term refuses under the existing read limit.
- FR-019 public-items row (examined): The checked_package V2 typed terms are listed in the Rust library interface.
- FR-038 (context_only): The reader owns wire order, limits, and refusal envelope.
