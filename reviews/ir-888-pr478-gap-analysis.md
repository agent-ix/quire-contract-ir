---
id: SR-13161
title: "IR-888 quire-contract-ir PR 478 gap analysis: FR-035-AC-10/AC-11 backing vs matrix"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #478 vs origin/main; tests/it/complete_v1_contract_package.rs; read-only: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-047
    type: reviews
---

## Summary

Ticket: IR-888 (under IR-878). Plan completion: not assessed. This is a planless gap analysis. `quire coverage --scope . --strict` (quire 0.36.2) was measured the same way on an origin/main worktree and on the PR head. On main: 541/609 rows backed, 42 unbacked, 0 contradicted, FR-035 8/11. On the PR head: 543/609 backed, 40 unbacked, 0 contradicted, FR-035 10/11. The only rows that changed are FR-035-AC-10 and FR-035-AC-11, which are now backed. Rust symbols went from 948/949/952 to 950/951/955 bound/tagged/candidates. This matches the author's claim. No production code changed, and TC-456, `public_interface.rs` and `identity.rs` are untouched. The reorder (noncanonical) case is not part of AC-10, and no cumulative work bound is asserted (IR-889 open), as decided.

## Reviewed units

- FR-035-AC-10 (examined; backed by `reader_refuses_wrong_string_version_before_shape_identity_and_nodes`, binding correct)
- FR-035-AC-11 (examined; backed by `reader_incomplete_reports_each_exhausted_meter_and_pointer`, binding correct; clause gaps in SR-13160 FND-001/FND-002)
- TC-047 Status section and spec/checked_package/matrix/tests.md rows for FR-035 and TC-047 (examined; FND-001)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The status wording is stale once this PR lands. TC-047 `## Status` still says "FR-035-AC-10 and FR-035-AC-11 are planned and unrun; their tagged TC-047 reader tests follow spec review". The tests.md FR-035 row says "Only AC-10 and AC-11 remain PLANNED/UNRUN", and the TC-047 row says "only reader-outcome AC-10/AC-11 controls are PLANNED/UNRUN". This PR adds the tagged, passing tests and moves both rows to backed, so the matrix prose now contradicts the computed matrix. Update the three statements in this PR, and name any clause the tests still leave open (SR-13160 FND-001) | spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md:97-98, spec/checked_package/matrix/tests.md:12, spec/checked_package/matrix/tests.md:30 |

## Verdict

**CHANGES REQUESTED (medium).** The coverage delta is exact and honest: +2 backed, -2 unbacked, 0 contradicted, and nothing else moved. The test-to-AC bindings are correct. The hand-written status prose in TC-047 and tests.md was not updated and now understates the PR's own evidence (FND-001).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-047 Status, the tests.md FR-035 row and the TC-047 row now state that AC-10 and AC-11 are backed by the two named tagged tests, with the AC-11 work-limit scope open under IR-889; strict coverage is 543/609 backed, 40 unbacked, 0 contradicted |
