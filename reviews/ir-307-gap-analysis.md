---
id: SR-985
title: "gap analysis of PR 230 (same_type compares resolved operand types)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@c24c6a4f2fba713b35ea577969fa0a7c5f3c0082; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---
# SR-985: gap analysis of PR 230

Former id: SR-625 (cited by the marker on IR-307).

## Summary

Ticket: IR-307. This is a planless gap analysis, proportional to a one-line fix, so plan completion was not assessed. The governing rule is QSpec FR-322's `same_type` constraint row, which lives in quire-specification, not in this repo's `spec/`. The ticket's deliverables are two cases in one new test, and both were re-measured: distinct parameters of one record type are admitted, and parameters of two record types are refused at argument 1. The new test is untagged, like every sibling unit test in `operations.rs`, so the matrix has no row to change. `make spec` fails with the same 22 unbacked rows on this head and on clean origin/main 3e7935f.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- FR-322 `same_type` row ("The two named operands resolve to the same type node"), examined: the implementation matches it for `reference` operands. The non-reference gap is recorded as SR-982 FND-001.
- The ticket deliverable "two distinct parameters of the same record type admitted", examined: covered.
- The ticket deliverable "parameters of different record types refused", examined: covered.
- The catalog `same_type` users (`quantity.*`, `enum.*`, `structural.eq/ne`), context_only.
- The `make spec` baseline on origin/main, examined: identical 22 unbacked rows.

## Verdict

No gap is introduced by this PR.
