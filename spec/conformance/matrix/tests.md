---
id: TM-005
title: "quire-contract-ir conformance test matrix"
type: TestMatrix
---
# quire-contract-ir conformance test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-018 | FR-018-AC-1 through FR-018-AC-3 | TC-018 | ✅ implemented |
| FR-020 | FR-020-AC-1 through FR-020-AC-3 | TC-018 | 🚧 AC-1 and AC-2 implemented; AC-3, the decimal-string integer members in the published fixture schema and corpus, the 2^53 bound on a revision and a byte offset, and the corpus's canonical files holding `quire-canonical`'s bytes (IR-274 code change B), is planned: the schemas, corpus files and runner are amended together by that code change, and none of them is amended by the spec change |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-018 | Schema, corpus, diagnostics, dependencies, and interfaces conform | Integration | P0 | FR-018..FR-020 | ✅ implemented |

