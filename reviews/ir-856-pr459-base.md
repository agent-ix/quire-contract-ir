---
id: SR-12541
title: "IR-856 PR 459 spec base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-016-canonicalization-digests.md; spec/model/matrix/tests.md; crates/quire-contract-model/src/canonical.rs; crates/quire-contract-model/src/identity.rs; crates/quire-contract-model/src/binding.rs; crates/quire-contract-model/src/wire.rs; crates/quire-contract-model/src/conformance.rs"
review_set: subset
---

## Summary

Ticket: IR-856. Diff-scoped review of PR 459 at b644efed1d4b648a74e4897fbbc7fa78be4c6480. The changed FR-016 paragraph makes a universal public-input claim that the public generic canonical API does not support.

## Examined units

- **FR-016 statement** (`examined`): "The public refusal contract covers byte-limit and allocation failures of validated model values. An encoder protocol or internal error would indicate a bug in a model-owned `Encode` implementation, not an outcome of a supported input."
- **FR-016-AC-3** (`context_only`): "A deterministic reservation-failure harness forces canonical byte allocation failure and verifies `canonicalization_resource_exhausted`, no partial public bytes, and no digest."
- **FR-016-AC-4** (`context_only`): "Each of the five object kinds exposes a budgeted and a budget-free canonical operation; the budget-free result equals the budgeted result at `u64::MAX`; and a zero budget returns `canonicalization_resource_exhausted` with no bytes and no digest for every kind."
- **TC-017 row** (`examined`): "The `canonical.rs` unit test for an internal encoder fault remains tagged TC-017 only: FR-016's public resource-refusal criteria do not own the internal fault-message wording."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The public CanonicalBody extension permits caller-owned Encode failures, contradicting the new claim that only model-owned Encode bugs reach Protocol/Internal. | spec/model/functional/FR-016-canonicalization-digests.md:136 |

## Verdict

FAIL. The public trait is re-exported at crates/quire-contract-model/src/lib.rs:100; external body types may implement it, enter the public ContractPackage<B>::new at identity.rs:1139 and canonical_package at canonical.rs:180. A caller-owned Encode can return Protocol/Internal after package validation. Narrow the statement to the built-in body types or state that the diagnostic wording is noncontractual for every Encode failure. Sealing CanonicalBody would be a separate public API change. No new AC was added; existing FR-016-AC-1..3 remain untagged and AC-4..9 are tagged, unchanged by this PR.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The revised FR-016 statement limits the guaranteed resource refusal to built-in bodies, acknowledges caller-owned Encode errors through the public generic API, and leaves non-resource fault-message wording noncontractual. TC-017 now names both built-in and caller-owned bodies; CanonicalBody remains public and unsealed. |
