---
id: TM-010
title: "quire-contract-ir temporal test matrix"
type: TestMatrix
---
# quire-contract-ir temporal test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-350 | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8 | TC-461 | 🚧 Planned in `agent-ix/quire-integration` (IR-358). AC-1 through AC-6 and AC-8 remain unbacked until the QSL-typed bridge and executable correspondence tests carry their criterion trace tags there; AC-7 is withdrawn and has no TC-461 coverage. FR-028-AC-3/TC-041 continue to prohibit QSL dependencies in both Contract IR packages. |

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-461 | Project native temporal subjects and join owner results through correspondence | Integration | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-8 | 🚧 Planned in `agent-ix/quire-integration` (IR-358); covers QSL classification, strict `/2` inputs, every bounded past operator, TL owner readers, native/TL joins, mutations and limits. |
