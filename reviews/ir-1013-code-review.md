---
id: SR-612150
title: "IR-1013 PR 657 code review of clause point applicability tests"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_frame_entries.rs; FR-058-AC-8; TC-462"
review_set: subset
---

## Summary

Reviewed only the 140 added lines in PR #657 against FR-058-AC-8 and TC-462. The Rust review checked the real reader path and the shared fixture helpers. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**PASS** — the two tests exercise all eight admitted and eight inapplicable clause-kind/point combinations. Positive assertions compare the public accessor with each exact typed variant and name. Negative assertions require a refusal, its exact code and cause, the clause-node pointer, and that node's locus digest. The fixture's operation anchor is valid in each adverse operation-clause case. The new tests have real `#[trace]` bindings and introduce no copied fixture, policy change, or production behavior.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- **FR-058-AC-8 (examined):** For an invariant or constraint, a well-formed `Pre` or `Post` point refuses reader admission as `invalid_model_binding`/`malformed-declaration` at the clause node under FR-040. Valid `Initialization` and `Handler` points for both invariant and constraint admit. A precondition or postcondition admits either `Pre` or `Post` if the operation name and referenced anchor point agree, as QSpec FR-341-AC-14 requires; an `Initialization` or `Handler` point refuses `invalid_model_binding`/`malformed-declaration` at the clause node. An accessor returns the exact admitted variant without imposing a narrower consumer policy.
- **TC-462 (examined):** Test Procedure step 4 calls for invariant/constraint initialization and handler admission, pre/post refusal, precondition/postcondition pre and post admission, initialization/handler refusal, and operation-name and clause-anchor agreement. This diff supplies the sixteen applicability cases. Existing tests outside this diff address other TC-462 steps and were read only as context.
- **Changed file (examined):** `tests/it/checked_package_v2_frame_entries.rs:3824-3963`.
- **Reader context:** `crates/quire-contract-model/src/checked_package/v2/state.rs:523-551` is read-only context for the oracle. No production code or CI workflow changed.

## Checks

`git diff --check...HEAD` and `cargo fmt --all -- --check` passed. Focused `cargo test --locked --test it tc_462_clause_point_applicability -- --nocapture` passed both tests, with 1008 others filtered out. Aggregate gates are reserved for the code lead's final reviewed head.
