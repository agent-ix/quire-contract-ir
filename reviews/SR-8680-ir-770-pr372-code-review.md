---
id: SR-8680
title: IR-770 PR 372 code-review review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@ebab4128c2d93f79c61d3af28d0e37521afa4208; crates/quire-contract-model/src/checked_package/v2/derived_keys.rs, crates/quire-contract-model/src/checked_package/v2/frame.rs, crates/quire-contract-model/src/checked_package/v2/model_fields.rs, crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs, crates/quire-contract-model/src/checked_package/v2/model_members.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, crates/quire-contract-model/src/checked_package/v2/owner.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_model_fields.rs, spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md, spec/checked_package/matrix/tests.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket IR-770. Examined crates/quire-contract-model/src/checked_package/v2/derived_keys.rs, crates/quire-contract-model/src/checked_package/v2/frame.rs, crates/quire-contract-model/src/checked_package/v2/model_fields.rs, crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs, crates/quire-contract-model/src/checked_package/v2/model_members.rs, crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs, crates/quire-contract-model/src/checked_package/v2/owner.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_model_fields.rs, FR-040-AC-14, FR-040-AC-15, FR-040-AC-16. Rust behavior and tests for selected record frame fields, with no code defect found.

## Verdict

**PASS** — Rust behavior and tests for selected record frame fields, with no code defect found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Checks

Reviewed the Rust idiom, panic, ownership, boundary, test, and CI-diff checks. The PR changes no CI workflow. The focused author gates were reported separately; this review did not rerun full Cargo gates.
