---
id: TM-006
title: "quire-contract-ir output mapping test matrix"
type: TestMatrix
---
# quire-contract-ir output mapping test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-032 | FR-032-AC-1 through FR-032-AC-6 | TC-043, TC-051 | 🚧 PLANNED/UNRUN (IR-1105): the current tests still exercise request and source content digests. The amended name/version selections, typed source content and bounded request material need code and new tests before credit; the earlier nondigest admission and refusal controls retain their recorded evidence only. |
| FR-033 | FR-033-AC-1 through FR-033-AC-6 | TC-043 | 🚧 PLANNED/UNRUN (IR-1105): current records carry a SHA identity. The amended typed content, digest-free record and `record` byte-limit refusal need code and tests; disposition and completeness evidence from the earlier contract remains historical. |
| FR-034 | FR-034-AC-1 through FR-034-AC-9 | TC-043 | 🚧 PLANNED/UNRUN (IR-1105): current packages carry target, record and package digests. The amended digest-free package, exact target bytes, bounded request/record/package material and typed refusals need code and tests; earlier atomicity and region controls are pre-change evidence. |

## Diagnostic Registry Coverage

| Registry | Verification | Test Cases | Status |
|---|---|---|---|
| STD-003 | exact registered code set both ways, unresolved-obligation precedence, required field paths, and no spelling shared with STD-001 | TC-051, TC-043 | 🚧 PLANNED/UNRUN (IR-1105): remove `invalid_digest`, change the bounded-material refusal paths, and reverify closure and path exactness before credit. |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-043 | Output-mapping request, mapper seam, per-obligation records, limits, atomic package, and observer separation conform | Integration | P0 | FR-032, FR-033, FR-034 | 🚧 PLANNED/UNRUN (IR-1105 digest-free contract); existing tests assert the prior content-digest behavior and cannot credit the amended ACs. |
| TC-051 | The output-mapping refusal catalog is closed and registered | Inspection | P0 | STD-003, FR-032-AC-5 | 🚧 PLANNED/UNRUN until the compiled registry and emitted cases match the amended STD-003 (IR-1105). |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-051 cases | Coverage, closure | every emitted spelling has one registry row; every registry row is emittable; each spelling round-trips its wire form; neither refusal catalog shares a spelling with the other |
| TC-043 cases | Coverage, permutation, boundary, error, transition, typed content, atomicity | each exact target profile independently without target semantics; nonempty ordered unique obligation selection; every source-fact state and disposition invariant; separate observation/protocol adequacy; missing/duplicate/foreign/stale/cross-profile/cross-wired inputs; zero/exact/just-over/overflow request, obligation, node, depth, work, record and emitted-byte limits; request, record and package canonical material at exactly `maximum_request_bytes` and one byte over, a `u64::MAX` limit and region bound as decimal strings, compared with independently authored expected bytes; malformed and UTF-8-unsafe regions; cancellation/allocation/mapper failure; deterministic replay and mutation of every typed record/package member; path/time/locale/display/observer independence; no partial package or preservation fallback |
