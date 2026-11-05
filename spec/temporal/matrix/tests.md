---
id: TM-010
title: "quire-contract-ir temporal test matrix"
type: TestMatrix
---
# quire-contract-ir temporal test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-350 | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8 | TC-461 | 🚧 Planned in `agent-ix/quire-integration` (IR-1005). AC-1 through AC-6 and AC-8 remain unbacked until the QSL-typed bridge and executable correspondence tests carry their criterion trace tags there; AC-7 is withdrawn and has no TC-461 coverage. FR-028-AC-3/TC-041 continue to prohibit QSL dependencies in both Contract IR packages. |

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-461 | Project native temporal subjects and join owner results through correspondence | Integration | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8 | 🚧 Planned in `agent-ix/quire-integration` (IR-1005); covers QSL classification, admitted checked-package/v2 inputs, every bounded past operator, TL owner readers, native/TL joins, mutations and limits. |

## Input-binding evidence

The instance-binding control in `agent-ix/quire-integration`
`tests/temporal_mapping_binding.rs` exercises the checked-view input boundary:
foreign but equal admitted instances, altered disposition, retained definition
authority and surrounding closure. It is partial TC-461 evidence only. The
projection, TL reader, native-result join and limit controls above remain Planned;
this input check does not confer whole-criterion coverage.

## Criterion dispositions

Every live FR-350 criterion remains **Planned and unbacked**. The implementation
and executable TC-461 tests belong to `agent-ix/quire-integration`, under
IR-1005. The following are the missing bridge controls, not coverage supplied by
the input-binding test or by owner-contract documentation.

| Criterion | Status | Remaining implementation and executable evidence (IR-1005) |
|---|---|---|
| FR-350-AC-1 | Planned; unbacked | Implement `project_tl`, formula/history/trace/request construction and owner-reader admission for every supported row, including each bounded past operator under both false-extension profiles. Test origin-complete history and the bridge's `incomplete`/`missing-history` cutoff decision. The existing instance-binding controls cover only the pre-projection portion of TC-461 step 1. |
| FR-350-AC-2 | Planned; unbacked | Implement unsupported projection decisions preserving every classified dimension and emitting no partial artifacts; test timestamped, mixed, infinite-trace and finite-window equivalence refusals. |
| FR-350-AC-3 | Planned; unbacked | Implement the correspondence value and projection reader; test every preserved typed-member mutation and non-Boolean leaf refusal. Classification binding alone does not construct or verify these members. |
| FR-350-AC-4 | Planned; unbacked | Implement fixed-sample projection; test retained epoch/period/unit, changed-period and event-count refusals, and missing-sample incompleteness. |
| FR-350-AC-5 | Planned; unbacked | Implement `join_tl` and `read_tl_join`; test every decision row, ordered causes, all result axes, equal-final Boolean restriction and exact direct correction predecessors. |
| FR-350-AC-6 | Planned; unbacked | Implement valuation admission, projection/join readers and deterministic refusal/incomplete paths; test missing, duplicate, swapped, replayed and foreign cells, owner-reader refusals, malformed/noncanonical data, exact and exceeded limits, no partial artifacts and no evaluator invocation. |
| FR-350-AC-8 | Planned; unbacked | Implement all seven `TlLimits` counters, defaults and pre-work charging; test each exact/one-over boundary and simultaneous-crossing priority with the required named cause and no partial artifact. |

FR-350-AC-7 remains withdrawn and has no coverage obligation. These dispositions
leave the live criteria and their Test (TC-461) verification method unchanged;
none is qualified as Inspection evidence.

## Owner contracts and remaining work

QSL FR-515/FR-527 supply the checked subject and authenticated classification;
FR-045 defines support and FR-043 supplies native evaluation inputs/results.
QSL's checked-view mapping supports bounded-past classification. This does not
discharge the bridge's outstanding history premise or construct TL artifacts.
QSpec FR-092/FR-095 supply bounded-past semantics and correspondence meaning;
the selected TL owners supply their formula, history/trace, request and result
readers as listed in FR-350. These available contracts are dependencies to
consume, not evidence that the bridge is implemented or an open upstream
blocker for the missing controls above.

IR-1271 completed the FR-350/TC-461 owner and consumed-interface correction; it
did not implement the bridge. P-TL-9's formula/evaluation work does not own the
cross-owner bridge. IR-1005 retains the remaining implementation and test work
in `agent-ix/quire-integration`; Contract IR retains this normative contract and
matrix, with no QSL dependency added to either package.
