---
id: TM-002
title: "quire-contract-ir v0.1 semantic contract test matrix"
type: TestMatrix
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/11
    type: covers
---
# quire-contract-ir v0.1 semantic contract test matrix

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | FR-011 through FR-015, FR-019 | TC-015 through TC-018 | ✅ implemented |
| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018 | ✅ implemented |
| StR-003 | FR-012, FR-014, FR-015, FR-017 through FR-020 | TC-015, TC-016, TC-018, TC-020 | ✅ implemented |

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-011 | FR-011-AC-1 through FR-011-AC-3 | TC-015 | implemented |
| FR-012 | FR-012-AC-1 through FR-012-AC-6 | TC-015, TC-016 | implemented |
| FR-013 | FR-013-AC-1 through FR-013-AC-4 | TC-016 | implemented |
| FR-014 | FR-014-AC-1 through FR-014-AC-6 | TC-016 | implemented |
| FR-015 | FR-015-AC-1 through FR-015-AC-7 | TC-016 | implemented |
| FR-016 | FR-016-AC-1 through FR-016-AC-3 | TC-017 | implemented |
| FR-017 | FR-017-AC-1, FR-017-AC-2 | TC-017 | implemented |
| FR-018 | FR-018-AC-1 through FR-018-AC-3 | TC-018 | ✅ implemented |
| FR-019 | FR-019-AC-1, FR-019-AC-2 | TC-018 | ✅ implemented |
| FR-020 | FR-020-AC-1, FR-020-AC-2 | TC-018 | ✅ implemented |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | repeated golden corpus and cross-platform comparison | TC-017, TC-019 | issue #9 same-process goldens implemented; cross-platform TC-019 planned |
| NFR-002 | vocabulary, schema, API, and MSRV inspection | TC-015, TC-016, TC-019 | AC-3/4 implemented; AC-1/2 planned |
| NFR-003 | negative corpus, mutation, panic-free, and orphan checks | TC-017 through TC-019 | issue #9 version/orphan fail-closed checks implemented; full TC-019 planned |
| NFR-004 | repository/assurance/plan inspection | TC-014, TC-020, TC-021 | foundation covered |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-001 | exact registered code sets, precedence, structured fields, and no message parsing | TC-015 through TC-018 | ✅ implemented |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-014 | Baseline, licenses, protected workflow, and publication lock agree | Inspection | P0 | NFR-004 | ✅ implemented |
| TC-015 | Package, revision, anchor, clause, dependency, and diagnostic identities conform | Integration | P0 | FR-011, FR-012, NFR-002, STD-001 | ✅ implemented |
| TC-016 | Types, expressions, short-circuiting, and definedness conform | Integration | P0 | FR-012..FR-015, NFR-002, STD-001 | ✅ implemented |
| TC-017 | Canonical bytes, digests, migrations, and orphan classes conform | Property | P0 | FR-016, FR-017, NFR-001, NFR-003 | ✅ implemented |
| TC-018 | Schema, corpus, diagnostics, dependencies, and interfaces conform | Integration | P0 | FR-018..FR-020 | ✅ implemented |
| TC-019 | Determinism, portability, and fail-closed metrics meet thresholds | Analysis | P0 | NFR-001..NFR-003 | 🚧 planned across issues #8–#10 |
| TC-020 | Five assurance artifacts declare boundaries, evidence, failures, and owner | Inspection | P0 | StR-003, NFR-004 | ✅ implemented |
| TC-021 | Composite review and dependency DAG preserve spec-first child gates | Inspection | P0 | NFR-004 | ✅ implemented |
