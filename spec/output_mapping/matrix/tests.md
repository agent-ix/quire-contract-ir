---
id: TM-006
title: "quire-contract-ir output mapping test matrix"
type: TestMatrix
---
# quire-contract-ir output mapping test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-032 | FR-032-AC-1 through FR-032-AC-6 | TC-043, TC-051 | 🚧 AC-1 through AC-5 implemented: target-neutral admission and the closed STD-003 refusal catalog with a total unresolved-obligation precedence; no target mapper credited. AC-6 (the request identity material is encoded under `maximum_request_bytes`, IR-274 code change C) is implemented and verified: exact-length and one-byte-over admission through the public call in `tests/it/output_mapping.rs`, and the unit tests of `output_mapping.rs`, where each identity step takes the encoder as an argument and a spy records the ceiling it is handed, which is the limits' `maximum_request_bytes` and varies with it, never `u64::MAX`; a source scan in TC-048 allows one `Limits::new`, spelled with the request byte limit |
| FR-033 | FR-033-AC-1 through FR-033-AC-6 | TC-043 | 🚧 AC-1 through AC-5 implemented: bounded mapper/record accounting; no preservation default or partial record claim. AC-6 (the record identity material is encoded under `maximum_request_bytes` and refuses `request_limit_exceeded` at `record.identity`; IR-274 code change C, IR-74) is implemented and verified in `tests/it/output_mapping.rs`: material of exactly the limit yields a record, one byte over refuses at `record.identity` while the request still admits, and the same candidate maps under a larger limit |
| FR-034 | FR-034-AC-1 through FR-034-AC-7 | TC-043 | ✅ AC-1 through AC-5 implemented: atomic generated package and downstream observer evidence. AC-6 (the three identity steps run under the request byte limit; IR-274 code change C, IR-74) is implemented: the request, record and package steps encode through `quire-canonical` under `maximum_request_bytes`, verified at the exact length and one byte over through the public calls and the unit seams and by the ceiling spy. AC-7 (the identities are `quire-canonical`'s bytes, and `MappingLimits` and `OutputByteRegion` enter the material as decimal strings; IR-567 amended it to what admission does) is implemented: the record and package identities equal the SHA-256 of expected text written out in `tests/it/output_mapping.rs`, the three materials' expected bytes are written out in the unit tests of `output_mapping.rs` (the request material is the identity admission keeps by length as `request_bytes`; admission computes no request digest), the limits `18446744073709551615` and `9007199254740993`, and the region bounds `0`, `1`, `9007199254740993` and `18446744073709551615`, appear as strings in that written-out text, and a request whose `maximum_emitted_bytes` is `18446744073709551615` and one whose is `18446744073709551614` both admit with materials that differ. A zero `MappingLimits` member is not an AC-7 case: `MappingLimits::new` refuses it (`zero_limit`, FR-032-AC-2). A source byte offset or a requirement revision above 2^53 in a record's source can no longer reach the material: FR-012-AC-6 and FR-011-AC-3 bound those types at construction (IR-274 code change B), so the `allocation_failed` at `record.identity` that case once refused with is unreachable, and the unit test of `output_mapping.rs` now shows the span refused at construction and an offset of 2^53 encoded as a number |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-003 | exact registered code set both ways, unresolved-obligation precedence, required field paths, and no spelling shared with STD-001 | TC-051, TC-043 | ✅ implemented: the 37 `MappingRequestErrorCode` spellings and the registry are one closed set |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-043 | Output-mapping request, mapper seam, per-obligation records, limits, atomic package, and observer separation conform | Integration | P0 | FR-032, FR-033, FR-034 | 🚧 implemented in `tests/it/output_mapping.rs` and model overflow tests for FR-032-AC-1 through AC-5, FR-033-AC-1 through AC-5 and FR-034-AC-1 through AC-5, and for FR-032-AC-6, FR-033-AC-6, FR-034-AC-6 and FR-034-AC-7 (IR-274 code change C; IR-567 amended AC-7; the unit tests of `crates/quire-contract-model/src/output_mapping.rs` carry the hand-written identity material and the ceiling seams); target-specific OCL/SysML/FRETish semantics excluded |
| TC-051 | The output-mapping refusal catalog is closed and registered | Inspection | P0 | STD-003, FR-032-AC-5 | ✅ implemented in `tests/it/output_mapping.rs` against the compiled-in registry |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-051 cases | Coverage, closure | every emitted spelling has one registry row; every registry row is emittable; each spelling round-trips its wire form; neither refusal catalog shares a spelling with the other |
| TC-043 cases | Coverage, permutation, boundary, error, transition, identity, atomicity | each exact target profile independently without target semantics; nonempty ordered unique obligation selection; every source-fact state and disposition invariant; separate observation/protocol adequacy; missing/duplicate/foreign/stale/cross-profile/cross-wired inputs; zero/exact/just-over/overflow request, obligation, node, depth, work, record and emitted-byte limits; request, record and package identity material at exactly `maximum_request_bytes` and one byte over, a `u64::MAX` limit and region bound as decimal strings, and the identities compared with `quire_canonical::to_vec` output; malformed and UTF-8-unsafe regions; cancellation/allocation/mapper failure; deterministic replay and mutation of every record/package identity member; path/time/locale/display/observer independence; no partial package or preservation fallback |
