---
id: SR-8681
title: IR-770 PR 372 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@ebab4128c2d93f79c61d3af28d0e37521afa4208; crates/quire-contract-model/src/checked_package/v2/derived_keys.rs, crates/quire-contract-model/src/checked_package/v2/frame.rs, crates/quire-contract-model/src/checked_package/v2/model_fields.rs, crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs, crates/quire-contract-model/src/checked_package/v2/model_members.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, crates/quire-contract-model/src/checked_package/v2/owner.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_model_fields.rs"
review_set: subset
---

## Summary

Ticket IR-770. Examined FR-040-AC-14, FR-040-AC-15, FR-040-AC-16, crates/quire-contract-model/src/checked_package/v2/derived_keys.rs, crates/quire-contract-model/src/checked_package/v2/frame.rs, crates/quire-contract-model/src/checked_package/v2/model_fields.rs, crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs, crates/quire-contract-model/src/checked_package/v2/model_members.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, crates/quire-contract-model/src/checked_package/v2/owner.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_model_fields.rs. All three changed acceptance criteria are tagged to behavioral tests; no changed-code reverse gap or stub found.

## Verdict

**PASS** — All three changed acceptance criteria are tagged to behavioral tests; no changed-code reverse gap or stub found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. FR-040-AC-14, FR-040-AC-15, and FR-040-AC-16 are tagged in the computed matrix. The repository matrix has other untagged criteria outside this PR; this is a scoped verdict, not an aggregate release gate.
