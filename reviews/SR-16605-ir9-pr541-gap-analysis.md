---
id: SR-16605
title: "IR-9 PR 541 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #541 @; FR-038-AC-2/114/115/116; FR-056-AC-8; TC-048; flat-wire headers"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---
# SR-16605: IR-9 PR 541 incremental gap analysis

## Summary

Planless incremental check of the changed flat-wire header decision, refusal order and test bindings. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-2 and TC-048: private actual-check controls assert typed refusal code and pointer for invalid special forms, valid forms with invalid schema, unknown tag and ordinary invalid form. The public strict reader fixtures remain separate admission evidence.
- FR-038-AC-114/115/116: the changed header path still checks the current node body before a later node header; new controls assert that ordering. The existing body grammar and stale-identity controls are unchanged and were inspected as context.
- FR-056-AC-8: the author's two-root empty-rules scanner receipt reports 245 to 241, exactly four targeted flat-wire rows removed, with remaining source still red. Source diff inspection supports the local removal; this review does not certify a clean repository scan.
- No specification or matrix file changed. No coverage is credited to absent QSpec #425 fixture content; the separate landed timed fixture controls provide only their stated scope.

## Verdict

PASS for this incremental flat-wire slice. FR-056-AC-8 remains open.
