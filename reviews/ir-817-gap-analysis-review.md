---
id: SR-11824
title: "gap-analysis of IR-817 PR #418"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #418; spec/conformance/functional/FR-020-json-conformance-interface.md; spec/conformance/matrix/tests.md; spec/core/functional/FR-012-anchors-clauses-dependencies.md; spec/core/matrix/tests.md; spec/model/functional/FR-023-executable-projection-binding.md; spec/tests.md; tests/it/conformance.rs"
review_set: subset
---

## Summary

Ticket: IR-817. Plan completion: not assessed. Checked each retagged test body against every criterion it now carries. Bindings judged correct: AC-3 fixture-schema test (all eight members, accepted and refused spellings including 0, 1.0, "+1", "01", "-0", ""); AC-4 package-schema test (revision and byte-offset pointers at 2^53 valid, 2^53+1 invalid, both schemas); AC-5/AC-6 probe-coverage test (2^53+1 and u64::MAX give invalid_corpus at fixtures.<id>.input with detail "instance does not match schema"; at 2^53 the run either matches or fails later at fixtures.covers, proving the input schema passed); AC-8 (expectation code asserted); AC-10/AC-11 (47 files byte-equal to literals, scan refuses digit or '-' after any of the eight names); AC-12 (exit 0, row count equals inputs, every row status match). Dropping AC-3 from tc_018_a_successful_expression_document_is_checked_against_the_schema is correct: its assertions match no FR-020 criterion. Coverage measured: origin/main 438 tagged / 56 untagged, strict 48 unbacked / 0 contradicted; head spec with origin/main tests 65 untagged (strict unchanged 48/0); head 447 tagged / 56 untagged, FR-020 12/12 tagged, strict 48/0. The PR body's figures hold.

## Verdict

**PASS with one medium oracle gap** on AC-9 and two low gaps.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-9 says refused "at the fixture schema check", but the decoder-fixture test asserts only error code invalid_corpus, not path fixtures.<id>.input or detail "instance does not match schema"; the runner has other invalid_corpus paths (malformed fixture input), so the stage clause is not discriminated | tests/it/conformance.rs:884-921 |
| FND-002 | low | AC-7 oracle checks the decoder fixture input is a document_json string but not that the text carries a JSON number in one of the eight members; relies on fixture naming (verified by hand: minimum -10 and value 1) | tests/it/conformance.rs:875-879 |
| FND-003 | low | After losing its AC-3 tag, tc_018_a_successful_expression_document_is_checked_against_the_schema backs no criterion: FR-020 describes schema-checking a successful package, not a successful document_json expression input; behaviour tested but unspecified | tests/it/conformance.rs:1390-1452 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Test now asserts code, path fixtures.<id>.input and detail 'instance does not match schema'. Mutant with the fixture-schema check bypassed (it fails at tests/it/conformance.rs:968 (exit 1 instead of 2)) makes it fail. |
| FND-002 | fixed | Test parses each decoder document and asserts the member at the pointer is the JSON number -10 or 1. Mutant turning -10 into "-10" in the bound fixture (it fails (at the corpus-success assert, line 841, before the number check; the number check itself was verified by reading)) makes it fail. |
| FND-003 | deferred | Follow-up IR-822 (Backlog) adds an FR-020 criterion for schema-checking a successful document_json expression document; the test correctly stays TC-018-only meanwhile. |
