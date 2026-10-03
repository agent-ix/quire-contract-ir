---
id: TM-003
title: "quire-contract-ir core test matrix"
type: TestMatrix
---
# quire-contract-ir core test matrix

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Status |
|---|---|---|---|
| StR-001 | FR-011 through FR-015, FR-019, FR-023, FR-028 through FR-040, FR-344 | TC-015 through TC-018, TC-035, TC-041 through TC-045, TC-047, TC-048, TC-050 through TC-053, TC-055, TC-056, TC-058, TC-222, TC-223 | 🚧 bounded-Kani profile, firewall and dispatch, the target-neutral output-mapping foundation, complete-V1 ContractPackage lowering, the V2 reader and its frame entries, operation anchors and state clauses are implemented; the proof check count, the `Unavailable` cause split, provider negotiation, the root and model crate interfaces with no root re-export of the model and the FR-344 refusals are planned |
| StR-002 | FR-016 through FR-018, FR-020 | TC-017, TC-018 | 🚧 implemented except FR-016-AC-5 through FR-016-AC-8 and FR-020-AC-3 (IR-274, planned) |
| StR-003 | FR-012, FR-014, FR-015, FR-017 through FR-020 | TC-015, TC-016, TC-018, TC-058 | 🚧 implemented except FR-019-AC-5, the model crate's explicit public item list (TC-058, planned; FR-019-AC-6, the artifact-reference member sets, is implemented) |

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-011 | FR-011-AC-1 through FR-011-AC-3 | TC-015 | 🚧 implemented except the clause of AC-3 that refuses a source or requirement revision above 9007199254740992 (2^53; IR-274, planned): the revision types still accept any positive `u64` |
| FR-012 | FR-012-AC-1 through FR-012-AC-6 | TC-015, TC-016 | 🚧 implemented except the clause of AC-6 that refuses a byte offset above 9007199254740992 (2^53; IR-274, planned): `SourceLocation` still accepts any `u64` |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | repeated golden corpus and cross-platform comparison | TC-017, TC-019 | same-process goldens implemented (TC-017); cross-platform comparison (TC-019) planned |
| NFR-002 | vocabulary, schema and API inspection | TC-015, TC-016, TC-019 | AC-3/4 implemented; cross-platform AC-1 planned |
| NFR-003 | negative corpus, mutation, panic-free, and orphan checks | TC-017 through TC-019 | version and orphan fail-closed checks implemented (TC-017, TC-018); threshold analysis (TC-019) planned |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-001 | exact registered code sets, precedence, structured fields, and no message parsing | TC-015 through TC-018, TC-223 | 🚧 closed code catalogs implemented and verified by TC-015 through TC-018; `kani_outcome_invalid`, raised by the validated `KaniOutcome` constructors, is planned (TC-223) |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-015 | Package, revision, anchor, clause, dependency, and diagnostic identities conform | Integration | P0 | FR-011, FR-012, NFR-002, STD-001 | ✅ implemented |
| TC-019 | Determinism, portability, and fail-closed metrics meet thresholds | Analysis | P0 | NFR-001..NFR-003 | 🚧 planned: cross-platform determinism, portability and fail-closed threshold analysis has no executable test |

