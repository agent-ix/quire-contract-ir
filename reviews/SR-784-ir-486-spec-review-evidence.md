---
id: SR-784
title: "evidence review of PR 255 (IR-486)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@154fc38ef518b036ef68ca1bb25fd6911846750d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md AC-43 and AC-69..71 verification column; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; spec/checked_package/matrix/tests.md; make spec coverage before/after"
review_set: subset
---
# SR-784: evidence review of PR 255 (IR-486)

## Summary

Ticket: IR-486. All three new criteria and AC-43 are `Test (TC-048)`. That is the right method:
each is a reader outcome over a self-built package, or a work and stack measurement of one read.
TC-048's description and its new "Recursive compared types" procedure cover every clause of
AC-69..71, and the AC-43 procedure is updated for the amended clause. The 🚧 markers are honest.
AC-69..71 have no test, and AC-43's tagged test
(`tc_048_leaf_walk_refuses_a_cycle_and_an_unresolved_node`, operations.rs:4318) still asserts the
refusal, which both matrix cells state.

`make spec` before and after:

| | origin/main af733f2 | head |
| --- | --- | --- |
| validate | passes | passes |
| grammar findings | 1 | 1 |
| strict unbacked rows | 23 | 23 |
| rows backed | 163/204 | 163/207 |
| FR-038 backed | 42/62 | 42/65 |

`make spec` exits 2 at both, on the 23 baseline unbacked rows, none of which is in
`spec/checked_package`. This matches the PR body's counts.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Adequate. No evidence-method finding of its own. The evidence-related defects are filed under base and criterion-strength.
