---
id: SR-202610310111
title: "IR-1005 PR 749 integrity analysis"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR #749; FR-350, TC-461, TM-010 placement correction"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: references
---

## Summary

Ticket IR-1005. Checked the changed requirement, TC and matrix for consistency of owner, evidence and withdrawn criteria. The new bridge owner aligns with FR-028 and retains substantive AC1-6/8 obligations. Base review FND-001 records the sole observed matrix conflict.

## Verdict

CONDITIONAL on base review FND-001; no separate integrity defect.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| TM-010 | examined | \| FR-350 \| FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-7, FR-350-AC-8 \| TC-461 \| 🚧 Planned in `agent-ix/quire-integration` (IR-358). AC-1 through AC-6 and AC-8 remain unbacked until the QSL-typed bridge and executable correspondence tests carry their criterion trace tags there; AC-7 is withdrawn. FR-028-AC-3/TC-041 continue to prohibit QSL dependencies in both Contract IR packages. \| |
| TC-461 | examined | \| TC-461 \| Project native temporal subjects and join owner results through correspondence \| Integration \| FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8 \| 🚧 Planned in `agent-ix/quire-integration` (IR-358); covers QSL classification, strict `/2` inputs, every bounded past operator, TL owner readers, native/TL joins, mutations and limits. \| |
| FR-350-AC-7 | examined | Withdrawn by the IR-1141 no-hash decision under QSpec #517/#518: the fixed 1429-byte vector, SHA-256 known answers and derived correspondence reference are removed. This id is retained and never reused. |
