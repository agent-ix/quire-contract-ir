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
| FR-006 | FR-006-AC-1 | TC-004 | ✅ covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Status |
|---|---|---|---|
| Issue #3 deliverables | FR-001, FR-006 | TC-001, TC-004 | ✅ covered |
| Issue #1 human ownership | FR-006 | TC-004 and protected-branch API | ✅ covered |

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
| TC-004 | CODEOWNER and human-only decision gate agree | Inspection | P0 | FR-006 | ✅ implemented |
