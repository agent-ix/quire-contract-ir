---
id: TM-010
title: "quire-contract-ir temporal test matrix"
type: TestMatrix
---
# quire-contract-ir temporal test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-350 | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-7, FR-350-AC-8 | TC-461 | 🚧 Planned; the temporal bridge module and executable correspondence tests are not implemented. The current computed matrix reports FR-350 and AC-1 through AC-8 untagged and TC-461 without a source symbol: 10 additional unbacked findings in the strict gate (73 total on this branch; origin/main has 63). Executable tests must carry TC-461 and criterion trace tags before these rows can be backed. |

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-461 | Project native temporal subjects and join owner results through correspondence | Integration | FR-350-AC-1, FR-350-AC-2, FR-350-AC-3, FR-350-AC-4, FR-350-AC-5, FR-350-AC-6, FR-350-AC-7, FR-350-AC-8 | 🚧 Planned; covers QSL classification, strict `/2` inputs, every bounded past operator, the canonical record vector, TL owner readers, native/TL joins, mutations and limits. |
