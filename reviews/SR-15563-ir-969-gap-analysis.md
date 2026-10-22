---
id: SR-15563
title: "Gap analysis — IR-969 typed V2 graph joins"
type: SpecReview
analysis: gap-analysis
scope: "quire-contract-ir PR #515 at 84be2d2; FR-349-AC-1/2/3/4/5 and changed tests"
review_set: subset
---

## Summary

Compared the changed graph-reader behavior with FR-349 and the tests tagged for typed terms, special bodies, first refusals and input limits. No acceptance-evidence gap was found in this PR diff. Plan completion: not assessed.

## Verdict

**PASS** — changed behavior has criterion-tagged assertions that exercise the production reader, including admitted typed special bodies and refusal precedence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. Reviewed `tests/it/checked_package_v2_typed_terms.rs` for FR-349-AC-1/3; `tests/it/checked_package_v2_frame_bodies.rs`, `tests/it/checked_package_v2_abstraction_relation.rs`, and `tests/conformance_qspec/extension.rs` for FR-349-AC-2/4; and the exact-byte-limit small-stack test in `tests/it/checked_package_v2_flat_wire.rs` for FR-349-AC-5. The full repository matrix was not run in this diff-scoped review; its attempted computation was stopped after it exceeded the focused review window. Existing `#[trace]` bindings and assertions were inspected directly. No plan was selected and no optional semantic audit was run.

## Head confirmation

The 4b0ea163..84be2d2 delta only changes the structural unit test to decode three JSON fixtures into `CheckedTerm` before calling `application_dependencies`. It leaves the production paths and the code finding unchanged. `git diff --check` on the delta passed.
