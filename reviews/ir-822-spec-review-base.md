---
id: SR-11850
title: "IR-822 base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@edd0830f45634d804bc6adb93654bc874b98fdc9 (base 4895a92daf8a16e292a17ca528cb7711c07ac3c3); spec/conformance/functional/FR-020-json-conformance-interface.md, spec/conformance/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-020
    type: references
---

## Summary

Ticket: IR-822. PR quire-contract-ir#424, branch spec/ir-822-document-json-expression. Base checklist over `git diff origin/main...edd0830`: new id FR-020-AC-15 is appended after AC-14 and no existing criterion is renumbered or reworded; the Behavior paragraph's `invalid_corpus` list now names the `document_json` expression document beside the package; the FR-020 matrix row range is extended to AC-15 and its status text is true at head (all fifteen FR-020 criteria are `tagged` in `quire coverage --json`, AC-15 bound to `tests/it/conformance.rs:1471` `tc_018_a_successful_expression_document_is_checked_against_the_schema`). The intermediate commit 79123dd that reverted a `spec/tests.md` summary edit nets to no change in that file. `quire validate` on both edited documents is clean.

Factual check of the criterion against code: `validate_successful_package_schema` (crates/quire-contract-model/src/conformance.rs:764-818) checks a `document_json` expression document against `expressionInput` only when `operation_succeeded` is true, i.e. when `execute_expression` (crates/quire-contract-model/src/wire.rs:659-702) returned `valid: true`, which requires decode, `check_expression_input` and both canonical encodings to succeed.

Coverage arithmetic, measured with `quire coverage --json`: origin/main 495/559 rows backed, 48 `unbacked_rows`; pre-tag commit 79123dd 495/560 backed, 48 `unbacked_rows`, AC-15 `backed: false` / `untagged`; head 496/560 backed, 48 `unbacked_rows`. The `unbacked_rows` list counts reference-table rows none of whose `target_ids` is bound; the AC-15 row's targets are `FR-020-AC-15` and `TC-018` (from its `Test (TC-018)` cell) and TC-018 is bound, so the row never entered that list. The tag's effect shows in `backed` (495 to 496) and the per-criterion status (untagged to tagged), not in the 48.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-020-AC-15 | examined | A `document_json` expression document that decodes successfully and that the published schema rejects makes the runner exit 2 with the code `invalid_corpus`. |
| FR-020 Behavior `invalid_corpus` list | examined | a successful package or `document_json` expression document the published schema rejects |
| spec/conformance/matrix/tests.md FR-020 row | examined | FR-020-AC-1 through FR-020-AC-15 ... AC-15 is the expression-document schema check, and every FR-020 criterion has a test tag |
| FR-020-AC-1 through FR-020-AC-14 | context_only | unchanged; confirmed not renumbered |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-020-AC-15's precondition "decodes successfully" is broader than the runner's gate and the Behavior paragraph's "successful": a document that decodes but fails the expression check (`valid: false`) is never schema-checked, so the criterion as written is false for it | spec/conformance/functional/FR-020-json-conformance-interface.md:149 |

FND-001 failure scenario: a `document_json` expression fixture whose document decodes, carries an unknown member the schema rejects, and fails type checking yields `valid: false`; `operation_succeeded` is false, the schema check is skipped, and the runner compares against the expectation (exit 0 or 1), not exit 2 `invalid_corpus`. Aligning the wording with the Behavior paragraph ("A successful `document_json` expression document", or "... that the runner evaluates as valid ...") makes the criterion true for what the code and test do.

## Verdict

PASS with one medium wording finding. The change is additive, ids are stable, matrix and status text are true at head, and the bound test genuinely exercises the criterion (see SR-11852 for the mutation). The coverage count of 48 unbacked rows is correct at base and head and is explained above.
