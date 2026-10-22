---
id: SR-15435
title: "IR-275 PR 509 base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-275; PR #509; crates/quire-contract-model/src/checked_package/v2/typed_terms.rs; crates/quire-contract-model/src/checked_package/v2/mod.rs; crates/quire-contract-model/src/checked_package/v2/relationship_end.rs; tests/it/checked_package_v2_typed_terms.rs; spec/checked_package/functional/FR-349-retain-typed-checked-v2-terms.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

FR-349 states the wire-preserving ownership boundary and names its five verification obligations. A stray word makes the Behavior paragraph ungrammatical.

## Verdict

**CONDITIONAL** — FR-349 states the wire-preserving ownership boundary and names its five verification obligations. A stray word makes the Behavior paragraph ungrammatical.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The Behavior paragraph contains the stray standalone word “It” before “The reader SHALL”; remove it so the normative requirement reads unambiguously. | spec/checked_package/functional/FR-349-retain-typed-checked-v2-terms.md:29 |

## Coverage

Exact reviewed head 00b1f915d2611f9fc1953fb2ed47e1e6b86e0de4. Reviewed only PR #509's changed code and FR-349/FR-019 spec delta. Plan completion: not assessed. Quire validation of the changed FR-349 and FR-019 artifacts passed. Reviewed FR-349 wording and its FR-019 public-inventory entry against the unchanged QSpec ownership statement in the draft.


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
| FND-001 | fixed | At b8f3edbb39c99e6681ac777143f7b083c09e420c, the stray word is removed; the Behavior paragraph now reads “dependency-reference terms, and binding members nested under aggregates. The reader SHALL distinguish…” |
