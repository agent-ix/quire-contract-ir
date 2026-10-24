---
id: SR-16523
title: "IR-9 PR 538 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@3dee1546ad6cf77e68c60613b18f89c4df09ede7; FR-018-AC-1/2/3; FR-056-AC-8; TC-018; two corpus-generator producer files"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-018", type: references }
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---
# SR-16523: IR-9 PR 538 incremental gap analysis

## Summary

Planless, diff-scoped trace and preservation review of the corpus generator's typed selectors. This PR removes the two producer files' closed-string decisions without claiming a clean repository scan.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-018-AC-1/2/3 and TC-018: existing native tests cover the real runner, full corpus regeneration, byte identity, and failure preservation. The new enums alter selection within existing fixture builders; they do not mint new cases or coverage tokens. Source comparison found the same nine package mutations, ten expression mutations, declaration labels, clause-root flag and ordered case additions.
- FR-056-AC-8: the author's inline evidence on IR-9 reports the official two-root scanner at 245 unresolved rows before and 219 after, with all 26 selected producer reports removed and 219 other reports retained. The scanner still exits 1, so AC-8 is open. Its raw attached logs are private to the author's session and were not independently read here; source inspection confirms the targeted string branches disappeared, but does not independently establish the global row count.
- No spec, test, corpus fixture, scanner rule, local exemption, or matrix artifact changed. A final gate is not part of this review.
- Plan completion: not assessed.

## Verdict

PASS for this incremental source and trace slice; FR-056-AC-8 remains open. The reported global scanner counts are author evidence, qualified above.
