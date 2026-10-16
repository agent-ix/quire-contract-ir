---
id: SR-11852
title: "IR-822 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@edd0830f45634d804bc6adb93654bc874b98fdc9 (base 4895a92daf8a16e292a17ca528cb7711c07ac3c3); tests/it/conformance.rs"
review_set: subset
---

## Summary

Ticket: IR-822. PR quire-contract-ir#424. Code review with the rust-review lane folded in, scoped to the one Rust hunk: a doc line and a trace attribute on `tc_018_a_successful_expression_document_is_checked_against_the_schema` (tests/it/conformance.rs:1467-1469). No behaviour change.

Checked: tag spelling `#[ix_trace_rs::trace("TC-018", "FR-020-AC-15")]` with the `/// Tracing: ...` line plus one `/// FR-020-AC-N.` line per criterion matches the file's convention (lines 714-718, 795-798, 832-836, 1159-1162); `quire matrix` binds FR-020-AC-15 to tests/it/conformance.rs:1471:1 as `tagged`. The test asserts exit 2 (via `error_code`, which asserts `status.code() == Some(2)`, empty stdout and the protocol identity) and the code `invalid_corpus`, and its control fixture shows the same document without the unknown member runs as `match` with `valid: true`.

Mutation, in a detached throwaway worktree at edd0830 (removed afterwards): making the `document_json` branch in `validate_successful_package_schema` never fire (`.filter(|_| false)` at crates/quire-contract-model/src/conformance.rs:800) turns the test red (`left: Some(0) right: Some(2)` at conformance.rs:123); reverted, it passes (`cargo test --test it tc_018_a_successful_expression_document`, 1 passed).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | PR #424 description is stale: it says no trace tag changed, the test carries TC-018 only, and coverage goes 47 to 48 unbacked, while head adds the tag and the count is 48 at base and head | quire-contract-ir#424 body |

FND-001 failure scenario: a merger or later miner reading the PR body records that AC-15 is unbacked and that a follow-up tag change is owed, though edd0830 already lands it.

## Verdict

PASS with one low finding on PR metadata. The code hunk is correct, convention-conforming and binds; the test kills the skip-the-schema-check mutant.
