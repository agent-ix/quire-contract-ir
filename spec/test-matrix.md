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
| FR-001 | FR-001-AC-1, FR-001-AC-2 | TC-008, TC-001 | ✅ covered |
| FR-002 | FR-002-AC-1 | TC-001 | ✅ covered |
| FR-003 | FR-003-AC-1 | TC-002 | ✅ covered |
| FR-004 | FR-004-AC-1 | TC-003 | ✅ covered |
| FR-005 | FR-005-AC-1 | TC-003 | ✅ covered |
| FR-006 | FR-006-AC-1 | TC-004 | ✅ covered |
| FR-007 | FR-007-AC-1 | TC-002 | ✅ covered |
| FR-008 | FR-008-AC-1 through FR-008-AC-4 | TC-005 through TC-012, TC-023 | ✅ covered |
| FR-009 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-4 through FR-009-AC-6 | TC-004, TC-006, TC-022, TC-024, TC-025, TC-032 | ✅ covered |
| FR-010 | FR-010-AC-1 | TC-003 | ✅ covered |
| FR-021 | FR-021-AC-1 through FR-021-AC-5 | TC-023, TC-025 through TC-028 | ✅ covered |
| FR-022 | FR-022-AC-1 through FR-022-AC-6 | TC-029 through TC-034 | ✅ covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| Issue #3 deliverables | FR-001 through FR-009 | TC-001 through TC-012 | ✅ covered |
| Issue #3 acceptance | FR-008, FR-010 | TC-003, TC-005 through TC-012 | ✅ covered |
| Issue #1 human ownership | FR-006, FR-009 | TC-004 and protected-branch API | ✅ covered |
| Issue #39 shared assurance migration | FR-009, FR-022 | TC-029 through TC-034 | ✅ covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| Deterministic schema validation | Draft 7 corpus plus mutation probes | TC-005 through TC-012 | ✅ covered |
| Reviewable provenance | schema inspection and valid fixtures | TC-005, TC-006 | ✅ covered |
| No silent identity omission | targeted negative fixtures | TC-007 through TC-012 | ✅ covered |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Compatibility and pinning language exists | Inspection | P0 | FR-001, FR-002 | ✅ implemented |
| TC-002 | Eight repositories and release classes are complete | Inspection | P0 | FR-003, FR-007 | ✅ implemented |
| TC-003 | License, clean-room, agent, and boundary rules exist | Inspection | P0 | FR-004, FR-005, FR-010 | ✅ implemented |
| TC-004 | CODEOWNER and human-only decision gate agree | Inspection | P0 | FR-006, FR-009 | ✅ implemented |
| TC-005 | Generated-artifact envelope is accepted | Integration | P0 | FR-008 | ✅ implemented |
| TC-006 | External-engine envelope retains inconclusive status | Integration | P0 | FR-008, FR-009 | ✅ implemented |
| TC-007 | Missing backend identity is rejected | Integration | P0 | FR-008 | ✅ implemented |
| TC-008 | Invalid digest and unknown schema are rejected | Integration | P0 | FR-001, FR-008 | ✅ implemented |
| TC-009 | Missing producer/tool/provenance identity is rejected | Integration | P0 | FR-008 | ✅ implemented |
| TC-010 | Missing input identities are rejected | Integration | P0 | FR-008 | ✅ implemented |
| TC-011 | Missing nested schema identity is rejected | Integration | P0 | FR-008 | ✅ implemented |
| TC-012 | Missing output identities are rejected | Integration | P0 | FR-008 | ✅ implemented |
| TC-023 | Shared responsibility registry has exactly one owner per responsibility | Inspection | P0 | FR-008, FR-021 | ✅ implemented |
| TC-024 | Historical PGM-01 records remain byte-identical and explicitly lossy | Integration | P0 | FR-009 | ✅ implemented |
| TC-025 | Domain/result ownership, non-execution, and runtime independence agree | Inspection | P0 | FR-009, FR-021 | ✅ implemented |
| TC-026 | Retained #1/#7/#20 disposition bytes match the reviewed receipt and its markers | Inspection | P0 | FR-021 | ✅ implemented |
| TC-027 | Prototype inventory preserves threat/domain cases and rejects its architecture | Inspection | P0 | FR-021 | ✅ implemented |
| TC-028 | Campaign documents state no conflicting executor/envelope/retention prescription, quoted review citations excepted | Inspection | P0 | FR-021 | ✅ implemented |
| TC-029 | Shared components classify against the accepted matrix and a drifted consumed artifact fails closed | Integration | P0 | FR-022 | ✅ implemented |
| TC-030 | The native conformance result reaches Quoin through the declared adapter and an empty run is refused | Integration | P0 | FR-022 | ✅ implemented |
| TC-031 | Quire's static export is retained by digest and no shared component executes a producer | Integration | P0 | FR-022 | ✅ implemented |
| TC-032 | Immutable legacy records map read-only with their digests preserved and non-readable inputs keep their own outcome | Integration | P0 | FR-009, FR-022 | ✅ implemented |
| TC-033 | Every required result state has a demonstrated case and none collapses into another | Integration | P0 | FR-022 | ✅ implemented |
| TC-034 | Weakening a load-bearing shared-assurance check turns its gate red | Integration | P0 | FR-022 | ✅ implemented |
