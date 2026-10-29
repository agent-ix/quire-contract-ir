---
id: SR-16617
title: "IR-9 PR 542 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #542 retained-kind comparison operand slice"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---
# SR-16617: IR-9 PR 542 incremental gap analysis

## Summary

Diff-scoped check against FR-038-AC-202 through AC-209, TC-445 and FR-056-AC-8. Repository-wide plan completion was not assessed. FR-056-AC-8 requires a clean authoritative scan across both production roots; this PR is one remediation slice and does not claim that result.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings in this PR slice (placeholder) | - |

## Coverage

- Inspection confirms three production predicates stopped comparing raw `node_tag` and `semantic_form`, removing six paired post-intake wire-string decisions. No scanner rule, exemption, root, or local marker changed. The author's reported scanner count is 245 to 239 unresolved with exit 1; this review did not independently run the external scanner, so the count remains author evidence, not a reviewer measurement.
- New `#[trace("TC-445")]` controls exercise each changed predicate using opposite raw and retained kinds; all three passed in the 7/7 focused reviewer run. The tests preserve the exact error payload and first-defect path required by FR-038-AC-207. Existing public admitted clause tests remain outside this private mismatch claim.
- FR-056-AC-8 remains open at repository scope. No new requirement gap or unowned behavior is introduced by this slice.

## Verdict

PASS for the incremental slice. The repository-wide clean-scan criterion remains open.
