---
id: SR-10870
title: "IR-238 PR 389 code-review replacement"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/intake.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_envelope.rs, tests/it/checked_package_v2_canonical_encoding.rs, tests/it/main.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-051", type: reviews }
  - { target: "ix://agent-ix/quire-contract-ir/TC-451", type: reviews }
---

## Summary

Replacement review of IR-238 PR #389 at frozen, scoped to `git diff origin/main...HEAD`. The changed Rust implementation retains the one generic envelope shape and drives owned read, raw-term borrowed intake, and typed borrowed canonical encoding. The public encoder has seven borrowed members and explicit canonical limits. Fixed-depth fields use serde serialization; arbitrary-depth JSON terms reuse iterative encoders. No clones, identity derivation, admission, producer limits, or QSL adoption were introduced. The previous clean code review still holds after rebase.

## Verdict

**PASS** — No code-review findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Inspected the exact current diff, API exports, intake and owned wire paths, error propagation, and all changed/new tests for oracle strength. FR-051-AC-1..3 and TC-451 were inspected. The three focused TC-451 integration tests passed. The stale-base FR-019 inventory concern was checked against current main and is recorded separately as a specification finding. Plan completion: not assessed.
