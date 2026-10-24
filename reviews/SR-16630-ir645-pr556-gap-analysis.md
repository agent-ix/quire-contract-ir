---
id: SR-16630
title: "Incremental gap analysis — IR-645 FR-014-AC-9"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-645; PR #556 FR-014-AC-9, TC-444, expression.rs and tests/it/expression.rs"
review_set: subset
---

## Summary

Incremental trace review of the FR-014-AC-9, TC-444, test and changed source chain. The implementation can be inspected against TC-444, and the tagged source guard detects the former independent-field representation. Qualified Inspection backing is still external to this PR.

## Verdict

**PASS** for the PR diff — no inflated qualified evidence claim. IR-645 remains open for the required Inspection mechanism and other strict coverage rows.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed.

- FR-014-AC-9 (examined): paired integer and rational variants plus typed binary operator inputs; unary negation inspects and reconstructs paired variants.
- TC-444 (examined): the tagged AST guard checks the top-level paired representation and two binary signatures; it is an aid to review, not the qualified Inspection required by this TC.
- TC-016 (context only): existing behavior tests cover public invalid-operand diagnostics. The changed code retains the same diagnostic branches and source-span arguments.
- Reverse mapping: the changed production behavior has FR-014-AC-9 as owner. No new unowned production behavior was identified in the diff.
