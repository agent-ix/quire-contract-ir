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
| FR-020 | FR-020-AC-1 through FR-020-AC-3 | TC-018 | 🚧 AC-1 and AC-2 implemented; AC-3 (IR-274 code change B) is implemented except one clause: the schemas, corpus files and runner are amended together, the fixture schema takes the eight integer members as `IntegerString` strings and refuses each of the listed spellings, both schemas bound a revision and a byte offset at 2^53, a `document_json` decoder fixture of the expression operation supplies a number straight to the decoder and its expectation carries `invalid_wire_format` while the same number in an ordinary input is `invalid_corpus`, the 47 canonical files holding one of the members are re-recorded from `quire-canonical`'s bytes and the corpus runs exit 0 with every fixture a `match`. The clause not met (stays partial, IR-568) is that the 47 files are all scanned for strings-only spelling but only four equal bytes written out in the test (`tc_018_the_recorded_canonical_files_spell_the_eight_members_as_strings`) |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-018 | Schema, corpus, diagnostics, dependencies, and interfaces conform | Integration | P0 | FR-018..FR-020 | ✅ implemented |

