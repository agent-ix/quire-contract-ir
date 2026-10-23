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
| FR-020 | FR-020-AC-1 through FR-020-AC-15 | TC-018 | ✅ implemented (AC-3 through AC-14 are the IR-274 code change B obligations split under IR-817, and AC-15 is the expression-document schema check, and every FR-020 criterion has a test tag): the schemas, corpus files and runner are amended together, the fixture schema takes the eight integer members as `IntegerString` strings and refuses each of the listed spellings, both schemas bound a revision and a byte offset at 2^53, a `document_json` decoder fixture of the expression operation supplies a number straight to the decoder and its expectation carries `invalid_wire_format` while the same number in an ordinary input is `invalid_corpus`, the 47 canonical files holding one of the members are re-recorded from `quire-canonical`'s bytes and the corpus runs exit 0 with every fixture a `match`. Each of the 47 files equals bytes written out in the test, not produced by the encoder, and all 47 are scanned for strings-only spelling (`tc_018_the_recorded_canonical_files_spell_the_eight_members_as_strings`). `tc_018_probe_and_coverage_bounds_fail_at_the_fixture_schema` verifies each probe clause-reference revision and coverage trace revision/offset member at 2^53+1 and `u64::MAX` fails at its fixture input with `invalid_corpus`, while 2^53 passes that schema. |

| FR-020 | FR-020-AC-16 through FR-020-AC-19 | TC-018 | 🚧 planned (IR-941): authoritative root-only shared CLI adoption, caller-owned exits, unchanged bytes/argv and actual output-failure controls; existing AC-1 through AC-15 remain implemented. |

| FR-057 | FR-057-AC-1 through FR-057-AC-10 | Pending shared-owner verification allocation | 🚧 proposed/planned (IR-123): source/consumer/result association and adverse controls; shared owner concurrence and selected review set pending. Existing TC-018 producer checks are prerequisites only, not cross-repository acceptance evidence. |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-018 | Schema, corpus, diagnostics, dependencies, and interfaces conform | Integration | P0 | FR-018..FR-020 | 🚧 corpus and interface checks implemented; FR-019-AC-8 through AC-13 now have independent boundary and signature checks |

