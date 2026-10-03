---
id: TM-006
title: "quire-contract-ir output mapping test matrix"
type: TestMatrix
---
# quire-contract-ir output mapping test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-032 | FR-032-AC-1 through FR-032-AC-6 | TC-043, TC-051 | 🚧 AC-1 through AC-5 implemented: target-neutral admission and the closed STD-003 refusal catalog with a total unresolved-obligation precedence; no target mapper credited. AC-6 (the request identity material is encoded under `maximum_request_bytes`, IR-274) is planned: the request is encoded with a `u64::MAX` ceiling and its length compared afterwards |
| FR-033 | FR-033-AC-1 through FR-033-AC-6 | TC-043 | 🚧 AC-1 through AC-5 implemented: bounded mapper/record accounting; no preservation default or partial record claim. AC-6 (the record identity material is encoded under `maximum_request_bytes` and refuses `request_limit_exceeded` at `record.identity`; IR-274, IR-74) is planned: the record identity is encoded with a `u64::MAX` ceiling |
| FR-034 | FR-034-AC-1 through FR-034-AC-7 | TC-043 | 🚧 AC-1 through AC-5 implemented: atomic generated package and downstream observer evidence. AC-6 and AC-7 (the three identity steps run under the request byte limit, the identities are `quire-canonical`'s bytes, and `MappingLimits` and `OutputByteRegion` enter the material as decimal strings; IR-274, IR-74) are planned: the steps run under `u64::MAX` and the material holds those `u64` values as numbers |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-003 | exact registered code set both ways, unresolved-obligation precedence, required field paths, and no spelling shared with STD-001 | TC-051, TC-043 | ✅ implemented: the 37 `MappingRequestErrorCode` spellings and the registry are one closed set |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-043 | Output-mapping request, mapper seam, per-obligation records, limits, atomic package, and observer separation conform | Integration | P0 | FR-032, FR-033, FR-034 | 🚧 implemented in `tests/it/output_mapping.rs` and model overflow tests for FR-032-AC-1 through AC-5, FR-033-AC-1 through AC-5 and FR-034-AC-1 through AC-5; FR-032-AC-6, FR-033-AC-6, FR-034-AC-6 and FR-034-AC-7 are planned (IR-274); target-specific OCL/SysML/FRETish semantics excluded |
| TC-051 | The output-mapping refusal catalog is closed and registered | Inspection | P0 | STD-003, FR-032-AC-5 | ✅ implemented in `tests/it/output_mapping.rs` against the compiled-in registry |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-051 cases | Coverage, closure | every emitted spelling has one registry row; every registry row is emittable; each spelling round-trips its wire form; neither refusal catalog shares a spelling with the other |
| TC-043 cases | Coverage, permutation, boundary, error, transition, identity, atomicity | each exact target profile independently without target semantics; nonempty ordered unique obligation selection; every source-fact state and disposition invariant; separate observation/protocol adequacy; missing/duplicate/foreign/stale/cross-profile/cross-wired inputs; zero/exact/just-over/overflow request, obligation, node, depth, work, record and emitted-byte limits; request, record and package identity material at exactly `maximum_request_bytes` and one byte over, a `u64::MAX` limit and region bound as decimal strings, and the identities compared with `quire_canonical::to_vec` output; malformed and UTF-8-unsafe regions; cancellation/allocation/mapper failure; deterministic replay and mutation of every record/package identity member; path/time/locale/display/observer independence; no partial package or preservation fallback |
