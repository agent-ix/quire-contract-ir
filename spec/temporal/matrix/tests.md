---
id: TM-010
title: "quire-contract-ir temporal test matrix"
type: TestMatrix
---
# quire-contract-ir temporal test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-350 | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8, FR-350-AC-9, FR-350-AC-10, FR-350-AC-11 | TC-461 | 🚧 Planned in `agent-ix/quire-integration`. Each criterion is unbacked until a bridge test there carries its criterion trace tag. FR-350-AC-7 is withdrawn and has no coverage obligation. FR-028-AC-3/TC-041 keep both Contract IR packages free of QSL dependencies. |

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-461 | Project native temporal subjects and join owner results through correspondence | Integration | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8, FR-350-AC-9, FR-350-AC-10, FR-350-AC-11 | 🚧 Planned in `agent-ix/quire-integration`. |

## Criterion to procedure

| Criterion | TC-461 steps | Bridge component (AD-009) |
|---|---|---|
| FR-350-AC-1 | 2 | projection construction |
| FR-350-AC-2 | 1, 3 | admission and support |
| FR-350-AC-3 | 7 | correspondence record |
| FR-350-AC-4 | 6 | native input binding, construction |
| FR-350-AC-5 | 9, 10 | join decision, corrections |
| FR-350-AC-6 | 5, 8, 12 | valuation binding, strict readers |
| FR-350-AC-8 | 11 | limits |
| FR-350-AC-9 | 4 | native input binding |
| FR-350-AC-10 | 7, 10 | correspondence and join records |
| FR-350-AC-11 | 7, 9 | record encoding, construction order |

The existing `tests/temporal_mapping_binding.rs` in `agent-ix/quire-integration`
exercises part of step 1 only; it confers no whole-criterion coverage.
