---
id: SR-16511
title: "IR-996 PR 535 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-160/162; FR-054-AC-1/5; FR-056-AC-8; TC-048; TC-458; scalar operand source and tests"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-054", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---
# SR-16511: IR-996 PR 535 incremental gap analysis

## Summary

Planless, diff-scoped trace and behavior check for retained scalar child kinds. The real external two-root scanner loses the two target reports; the repository remains red.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-160/162 and TC-048: the new reader-admitted graph-literal test checks the singleton `(13, 13)` independently in the checked and lowered views. Private damaged-state tests show a retained parameter cannot be reclassified by raw literal spelling and the reverse; they explicitly make no new admission claim.
- FR-054-AC-1/5 and TC-458: the new integration test verifies positional graph-child identity, admitted source correspondence and unchanged wire. Existing TC-458 tests cover inline identity, repeated child, occurrence selection, missing range, and refusal.
- FR-056-AC-8: the installed external scanner, run with both production roots and empty rules on the exact base and head, reports 268 to 266. The two `ScalarArguments::project` reports disappear. Two unchanged `type_range` reports shift line numbers; no new item appears. This is not a clean-source result or AC-8 acceptance.
- No spec or matrix artifact changed. The computed matrix and focused test evidence are checked separately from the final aggregate gate.
- Plan completion: not assessed.

## Verdict

PASS for the incremental slice. FR-056-AC-8 and unrelated reported string edges remain open.
