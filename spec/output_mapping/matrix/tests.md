---
id: TM-006
title: "quire-contract-ir output mapping test matrix"
type: TestMatrix
---
# quire-contract-ir output mapping test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-032 | FR-032-AC-1 through FR-032-AC-5 | TC-043, TC-051 | ✅ implemented target-neutral admission and the closed STD-003 refusal catalog with a total unresolved-obligation precedence; no target mapper credited |
| FR-033 | FR-033-AC-1 through FR-033-AC-5 | TC-043 | ✅ implemented bounded mapper/record accounting; no preservation default or partial record claim |
| FR-034 | FR-034-AC-1 through FR-034-AC-5 | TC-043 | ✅ implemented atomic generated package and downstream observer evidence |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-003 | exact registered code set both ways, unresolved-obligation precedence, required field paths, and no spelling shared with STD-001 | TC-051, TC-043 | ✅ implemented: the 37 `MappingRequestErrorCode` spellings and the registry are one closed set |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-043 | Output-mapping request, mapper seam, per-obligation records, limits, atomic package, and observer separation conform | Integration | P0 | FR-032, FR-033, FR-034 | ✅ implemented in `tests/it/output_mapping.rs` and model overflow tests; target-specific OCL/SysML/FRETish semantics excluded |
| TC-051 | The output-mapping refusal catalog is closed and registered | Inspection | P0 | STD-003, FR-032-AC-5 | ✅ implemented in `tests/it/output_mapping.rs` against the compiled-in registry |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-051 cases | Coverage, closure | every emitted spelling has one registry row; every registry row is emittable; each spelling round-trips its wire form; neither refusal catalog shares a spelling with the other |
| TC-043 cases | Coverage, permutation, boundary, error, transition, identity, atomicity | each exact target profile independently without target semantics; nonempty ordered unique obligation selection; every source-fact state and disposition invariant; separate observation/protocol adequacy; missing/duplicate/foreign/stale/cross-profile/cross-wired inputs; zero/exact/just-over/overflow request, obligation, node, depth, work, record and emitted-byte limits; malformed and UTF-8-unsafe regions; cancellation/allocation/mapper failure; deterministic replay and mutation of every record/package identity member; path/time/locale/display/observer independence; no partial package or preservation fallback |
