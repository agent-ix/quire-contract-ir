---
id: SR-611018
title: IR-1038 PR 620 scoped gap analysis
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-contract-ir; IR-1038 PR #620; FR-066-AC-3/4/5, FR-019-AC-5 and changed tests'
review_set: subset
---

## Summary

Ticket: IR-1038. This scoped review checks the changed implementation against its requirement and tagged integration tests. Plan completion: not assessed.

## Verdict

CONDITIONAL — FR-066-AC-3 has an executable external integration test, including typed constructor/accessor and writer round trip. FR-066-AC-4 has no tagged test in the computed matrix. The implementation still has the raw point path named in the code-review finding; FR-066-AC-5 remains unfulfilled at this head. No new test-only stub or unowned public behavior was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-066-AC-5's no-fallback-inference obligation is not met: `typed_clause_body` still reads the JSON point before the typed carrier. | crates/quire-contract-model/src/checked_package/v2/structural.rs:868-870 |
| FND-002 | medium | FR-066-AC-4 has no criterion-tagged test in the computed matrix. | spec/checked_package/functional/FR-066-retain-identity-bound-execution-points.md:94 |

## Scope

- FR-066-AC-3 (examined): external typed point construction, inspection and read/write/read test binding.
- FR-066-AC-4 (examined): malformed point and refusal tests already present, but none tags this exact criterion.
- FR-066-AC-5 (examined): single body carrier and no fallback inference; residual path recorded.
- FR-019-AC-5 (examined): public API inventory updated for the new types.
- Plan completion: not assessed. Computed matrix at this head: 754 criteria, 642 tagged, 92 untagged, 19 method-without-symbol, one tagged-by-ignored-test. These repository-wide gaps are inherited, not introduced by this PR; this review reports only the FR-066-AC-4 binding defect in scope.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed |, `typed_clause_body` uses `operation.execution_point.cloned` as its admitted point carrier; the raw point check is confined to structural malformed-input refusal. |
| FND-002 | fixed |, TC-056 integration tests tag FR-066-AC-4 at the duplicate-member precedence, malformed-point, point-agreement, distinct-anchor, and duplicate-anchor assertions. |

## Disposition verdict

PASS for the recorded findings; final gate is owned by the code lead.
