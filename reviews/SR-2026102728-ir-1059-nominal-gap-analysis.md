---
id: SR-2026102728
title: "Gap analysis of IR-1059 nominal rekey evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #669 d937faad; FR-038-AC-155/227/343; QSpec FR-322-AC-74/169/170; tests/conformance_qspec/nominal_rekey.rs"
review_set: subset
---

## Summary

Ticket IR-1059, PR #669 at d937faad. The published NK-00..14 replay is traced and checks exact code, cause and locus, while the broader stage-6 shape obligation has no discriminating public-reader evidence in this change.

## Verdict

**FAIL** — stage-6 semantic-shape cases outside NK-00..14 are untested and source inspection finds incorrect handling. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new FR-038-AC-343 tagged tests cover all NK rows but omit QSpec RM-CU-ORDER and FR-322-AC-74 malformed population `max`, which are stage-6 controls. The changed implementation gives the wrong refusal for the former and has no stage-6 branch for the latter; add public-reader assertions that fail on these cases without narrowing the criterion. | tests/conformance_qspec/nominal_rekey.rs:15 |

## Coverage

Plan completion: not assessed. Examined FR-038-AC-155/227/343 trace tags and the PR's changed tests and source. The full repository matrix was attempted but its output did not isolate these criteria in TSV form; this is a diff-scoped manual trace and behavioral gap check, not a repository-wide PASS. Optional semantic review was not requested.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | Fix commit e1326778 replaces `reaches_population.rs` with one blank line while `main.rs` still declares the module. This deletes the seven published reaches-population controls and five unbounded-population shape controls, including the new test for the stage-6 population fix. Restore those tests and their exact public-reader assertions in this PR. | tests/conformance_qspec/reaches_population.rs:1 |
| FND-003 | high | IR FR-038-AC-22 still requires `invalid_semantic_graph` for a zero exponent and repeated compound unit, but the changed code and its AC-22-tagged test assert `invalid_package`/`invalid-derived-shape` under merged QSpec FR-322 stage 6. Reconcile the local criterion with QSpec before treating that test as passing AC-22 evidence, and trace its stage-order assertion to AC-343. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5159 |
| FND-004 | medium | Fix commit 73193649 hand-edits `spec/checked_package/matrix/tests.md` with an AC-343 status row. The binding is computed from trace tags by Quire; the session's AGENTS.md explicitly forbids hand-editing a Test Matrix. Remove the row and rely on the tagged tests and computed matrix. | spec/checked_package/matrix/tests.md:30 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head e80f3453 has a current-key versus retained-key compound-unit public-reader control tagged AC-22 and AC-343, five QSpec unbounded-population rows with exact code/cause/locus, and an arbitrary Set element control. Focused tests remain queued. |
| FND-002 | fixed | Head 3d50b60a restored all 332 lines of `reaches_population.rs`, including the seven reaches-population rows and five unbounded-population shape rows; head e80f3453 retains it. |
| FND-003 | fixed | Head 73193649 reconciles AC-22 and TC-048 to QSpec's stage-6 code/cause/locus, preserves the malformed parameter, non-unit, dependency, self-type, empty-unit, zero/repeated/order controls, and adds AC-343 trace on the changed test. |
| FND-004 | fixed | Head e80f3453 removes the hand-edited AC-343 TestMatrix row; the new test retains its trace tag for Quire's computed matrix. |
