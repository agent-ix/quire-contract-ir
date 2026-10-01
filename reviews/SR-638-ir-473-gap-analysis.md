---
id: SR-638
title: "gap analysis of PR 234 (range-set merge and cap, FR-014-AC-7)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@d8b1f16c424733c9e4d807e11ecc02ff52f6c6e0; spec/contract/FR-014-expression-semantics.md, crates/quire-contract-model/src/expression.rs, tests/it/expression.rs"
review_set: base
---
# SR-638: gap analysis of PR 234

## Summary

Ticket: IR-473. Planless gap analysis of FR-014-AC-7 against the tagged tests and the changed code (Plan completion: not assessed). Five tests carry `Tracing: TC-016` / `FR-014-AC-7`: three unit tests in `range_set_tests` (expression.rs:4015, 4045, 4058) and two integration tests (tests/it/expression.rs:1638, 1651). The bindings are correct. All new production code is owned by FR-014. `make spec` reports FR-014-AC-7 as backed, and the unbacked count is 17, the same as origin/main. Mutation probes were run in a throwaway worktree and reverted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The rational half of FR-014-AC-7 is unbacked: no test drives a rational range set past the limit or through repeated merging. The one rational assertion feeds an operand that is already oversized, which the operand pre-check catches. Replacing `bounded_ranges(output)?` with `output` in `rational_range_results` leaves every test green | crates/quire-contract-model/src/expression.rs:2724 |
| FND-002 | low | Pre-existing, not introduced here: the rational `!= 0` guard split is unguarded by any test. Mutating `minimum <= -1` to `minimum < -1` in `refine_rational_ranges`, which drops -1 from `p` in [-1, 1] and so narrows the set unsoundly, leaves every test green | crates/quire-contract-model/src/expression.rs:3371 |

## Finding Detail

- FND-001: add an integration test that mirrors `guarded_fold` for a rational input. Use `p >= -1 && p <= 1 && p != 0` with denominator maximum 1, and assert that a 120-leaf product checks. Without merging, it grows as 2^n and hits the cap at 7 leaves. Also add a rational sum that goes past 64 points and is refused with `rational range set exceeds the checked range-set limit`. Both would kill the surviving mutation.
- FND-002: a rational guarded division such as `p != 0 && 1/p ...` over `p` in [-1, 1] with an assertion on the accepted result would kill it. Ticket it separately if it is not fixed here.

## Verdict

Approve with one medium test gap. Integer coverage is strong. The exhaustive small-domain test checks covered-set equality and canonical form for Add, Subtract and Multiply. The repeated-product test pins 2 intervals over 40 products. The cap boundary is tested at 64 and 65 for unit and integration. The 120-leaf product test is a true regression test: on origin/main it would need 2^120 intervals.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | FR-014 specifies a limit of 64, but no test pins it: the range-set tests compute expected sizes from `MAX_RANGE_SET_SIZE` itself, so changing the constant to 65 leaves every test green. Assert the literal 64 in at least one test | crates/quire-contract-model/src/expression.rs:4070 |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4b79f5f |
| FND-002 | fixed | 4b79f5f |
| FND-003 | fixed | 53c8d06 |
