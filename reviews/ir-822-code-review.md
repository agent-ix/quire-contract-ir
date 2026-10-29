---
id: SR-11852
title: "IR-822 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/conformance.rs"
review_set: subset
---

## Summary

Ticket: IR-822. PR quire-contract-ir#424. Code review with the rust-review lane folded in, scoped to the one Rust hunk: a doc line and a trace attribute on `tc_018_a_successful_expression_document_is_checked_against_the_schema` (tests/it/conformance.rs:1467-1469). No behaviour change.

Checked: tag spelling `#[ix_trace_rs::trace("TC-018", "FR-020-AC-15")]` with the `/// Tracing: ...` line plus one `/// FR-020-AC-N.` line per criterion matches the file's convention (lines 714-718, 795-798, 832-836, 1159-1162); `quire matrix` binds FR-020-AC-15 to tests/it/conformance.rs:1471:1 as `tagged`. The test asserts exit 2 (via `error_code`, which asserts `status.code() == Some(2)`, empty stdout and the protocol identity) and the code `invalid_corpus`, and its control fixture shows the same document without the unknown member runs as `match` with `valid: true`.

Mutation, in a detached throwaway worktree (removed afterwards): making the `document_json` branch in `validate_successful_package_schema` never fire (`.filter(|_| false)` at crates/quire-contract-model/src/conformance.rs:800) turns the test red (`left: Some(0) right: Some(2)` at conformance.rs:123); reverted, it passes (`cargo test --test it tc_018_a_successful_expression_document`, 1 passed).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | PR #424 description is stale: it says no trace tag changed, the test carries TC-018 only, and coverage goes 47 to 48 unbacked, while head adds the tag and the count is 48 at base and head | quire-contract-ir#424 body |

FND-001 failure scenario: a merger or later miner reading the PR body records that AC-15 is unbacked and that a follow-up tag change is owed, though already lands it.

## Verdict

PASS with one low finding on PR metadata. The code hunk is correct, convention-conforming and binds; the test kills the skip-the-schema-check mutant.

## Dispositions

Round 1, verified against the PR #424 body as returned by `gh pr view 424`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The PR body now states the test carries `FR-020-AC-15` beside `TC-018` (tag and doc line only), and that `quire coverage --strict` stays at 48 unbacked rows before and after because the AC-15 row targets TC-018; the "spec-only, no trace tag" and "47 to 48" claims are gone. Its "495/560 -> 496/560" is the pre-tag to head (origin/main is 495/559), which is accurate for "before and after the tag". |
