---
id: SR-16622
title: "IR-275 PR 552 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@558592b7cadd8ea9ed4fa8546240fb33465b29a5; PR #552 checked_package production and focused tests; IR-275 AC-2; FR-349-AC-4/5; FR-038-AC-2/97/117"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-349", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---
# SR-16622: IR-275 PR 552 incremental gap analysis

## Summary

Planless incremental analysis of the changed code and tests against typed term selection, exact refusal precedence and the deep-chain boundary. The code defect blocking literal IR-275 AC-2 is reported once in SR-16621.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- IR-275 AC-2: source audit found the three remaining live term-member reads named in SR-16621. This gap artifact does not duplicate that finding.
- FR-349-AC-4 and FR-038-AC-2: added reader mutations assert exact first code and pointer for missing, nonstring and unknown body tags. Existing QSpec adverse corpus was reported passing by the author; final gate remains pending.
- FR-038-AC-97: the new mixed-defect test asserts invalid interval before a missing application result type, preserving the strict-wire priority.
- FR-349-AC-5 and FR-038-AC-117: existing 100000-node chain control and short-stack tests remain; the PR did not weaken their assertions or change the graph-depth bound. No full matrix or aggregate test was run in this review.

## Verdict

PASS for incremental test and trace coverage, subject to SR-16621 FND-001. This does not claim AC-2 acceptance or final gate success.
