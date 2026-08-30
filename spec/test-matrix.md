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

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| PGM-01-R01 | PGM-01-R01-AC-1 | TC-PGM-01, TC-PGM-07 | covered |
| PGM-01-R02 | PGM-01-R02-AC-1 | TC-PGM-01 | covered |
| PGM-01-R03 | PGM-01-R03-AC-1 | TC-PGM-02 | covered |
| PGM-01-R04 | PGM-01-R04-AC-1 | TC-PGM-03 | covered |
| PGM-01-R05 | PGM-01-R05-AC-1 | TC-PGM-03 | covered |
| PGM-01-R06 | PGM-01-R06-AC-1 | TC-PGM-04 | covered |
| PGM-01-R07 | PGM-01-R07-AC-1 | TC-PGM-02 | covered |
| PGM-01-R08 | PGM-01-R08-AC-1 | TC-PGM-05, TC-PGM-06, TC-PGM-07, TC-PGM-08 | covered |
| PGM-01-R09 | PGM-01-R09-AC-1 | TC-PGM-04, TC-PGM-06 | covered |
| PGM-01-R10 | PGM-01-R10-AC-1 | TC-PGM-03 | covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| Issue #3 deliverables | PGM-01-R01 through R09 | TC-PGM-01 through TC-PGM-08 | covered |
| Issue #3 acceptance | PGM-01-R08, R10 | TC-PGM-03, TC-PGM-05 through TC-PGM-08 | covered |
| Issue #1 human ownership | PGM-01-R06, R09 | TC-PGM-04 and protected-branch API | covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| Deterministic corpus validation | repeatable local/CI command | TC-PGM-05 through TC-PGM-08 | covered |
| Reviewable provenance | schema inspection and valid fixtures | TC-PGM-05, TC-PGM-06 | covered |
| No silent identity omission | negative fixtures | TC-PGM-07, TC-PGM-08 | covered |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-PGM-01 | Compatibility and pinning language exists | Inspection | P0 | PGM-01 | ✅ implemented |
| TC-PGM-02 | Eight repositories and release classes are complete | Inspection | P0 | PGM-01 | ✅ implemented |
| TC-PGM-03 | License, clean-room, agent, and boundary rules exist | Inspection | P0 | PGM-01 | ✅ implemented |
| TC-PGM-04 | CODEOWNER and human-only decision gate agree | Inspection | P0 | PGM-01 | ✅ implemented |
| TC-PGM-05 | Generated-artifact envelope is accepted | Integration | P0 | PGM-01 | ✅ implemented |
| TC-PGM-06 | External-engine envelope is accepted | Integration | P0 | PGM-01 | ✅ implemented |
| TC-PGM-07 | Missing backend identity is rejected | Integration | P0 | PGM-01 | ✅ implemented |
| TC-PGM-08 | Missing/invalid output or schema identity is rejected | Integration | P0 | PGM-01 | ✅ implemented |
