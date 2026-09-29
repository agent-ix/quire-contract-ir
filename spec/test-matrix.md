---
id: TM-001
title: PGM-01 governance test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: covers
---
# PGM-01 Governance Test Matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-001 | FR-001-AC-2 | TC-001 | ✅ covered |
| FR-003 | FR-003-AC-1 | TC-002 | ✅ covered |
| FR-004 | FR-004-AC-1 | TC-003 | ✅ covered |
| FR-005 | FR-005-AC-1 | TC-003 | ✅ covered |
| FR-006 | FR-006-AC-1 | TC-004 | ✅ covered |
| FR-007 | FR-007-AC-1 | TC-002 | ✅ covered |
| FR-009 | FR-009-AC-2 | TC-004 | ✅ covered |
| FR-010 | FR-010-AC-1 | TC-003 | ✅ covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Status |
|---|---|---|---|
| Issue #3 deliverables | FR-001, FR-003 through FR-007, FR-009 | TC-001 through TC-004 | ✅ covered |
| Issue #3 acceptance | FR-010 | TC-003 | ✅ covered |
| Issue #1 human ownership | FR-006, FR-009 | TC-004 and protected-branch API | ✅ covered |

## Non-Functional Requirement Coverage

All three rows in this section — deterministic schema validation, reviewable
provenance, and no silent identity omission — were verified solely by the
PGM-01 Draft 7 corpus and its mutation probes over
`schemas/derivation-evidence-envelope-v1.schema.json`. That schema, its
validator and its fixtures are deleted with the withdrawal of PGM-01-R08, so the
rows are removed rather than left asserting over an empty population. The
substrate's own non-functional requirements are covered by TM-002; see
[`contract-test-matrix.md`](contract-test-matrix.md).

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Schema compatibility language exists | Inspection | P0 | FR-001 | ✅ implemented |
| TC-002 | Repository release classes and the source-tag gating rule are complete | Inspection | P0 | FR-003, FR-007 | ✅ implemented |
| TC-003 | License, clean-room, agent, and boundary rules exist | Inspection | P0 | FR-004, FR-005, FR-010 | ✅ implemented |
| TC-004 | CODEOWNER and human-only decision gate agree | Inspection | P0 | FR-006, FR-009 | ✅ implemented |
