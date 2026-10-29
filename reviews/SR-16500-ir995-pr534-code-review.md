---
id: SR-16500
title: "IR-995 PR 534 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/selected_population.rs; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---
# SR-16500: IR-995 PR 534 code and Rust review

## Summary

Ticket: IR-995. Independent review of the two-file production and test diff on main. The Rust review lane is included here. The relationship-member check now resolves the linked node's index once and reads both node and retained kind at that position; the population discriminator checks the closed `CheckedNodeKind` variant. No public API, dependency, wire form, or gate configuration changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- Examined FR-038-AC-269 and FR-038-AC-273 population acceptance and refusal, FR-053-AC-2's distinct binding/occurrence context, and FR-056-AC-7's post-intake branch ownership.
- Checked node/kind/index alignment, malformed linked ids, absent index entries, bounds-safe access, result ordering, work charging, and the existing public population tests. The newly introduced `get` lookups fail closed at the existing `population` refusal point.
- `git diff --check` passed. Existing focused tests and Clippy results were read from the author evidence; the final aggregate gate belongs to the final reviewed head and was not run in this review.

## Verdict

PASS for code and Rust at this head. The test-evidence gap for the new caller path is recorded separately in SR-16501. The whole-repository string-edge scan remains red.
