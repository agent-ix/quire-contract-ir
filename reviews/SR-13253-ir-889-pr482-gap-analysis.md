---
id: SR-13253
title: "IR-889 IR PR 482 gap analysis: FR-035-AC-11, AC-15, AC-16 against spec, matrix and code"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@4d01ba18aaf6c70a4d9f39e05202cf88b557ffa8 vs origin/main; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md, crates/quire-contract-model/src/checked_package/v2/contract_package.rs, tests/it/complete_v1_contract_package.rs"
review_set: subset
---

## Summary

Ticket: IR-889. Plan completion: not assessed. The checks ran at the PR head with quoin 0.28.3.

**`quire matrix --scope .`** tags each criterion to one test:

| Criterion | Test | Line |
| --- | --- | --- |
| FR-035-AC-11 | `reader_incomplete_reports_each_exhausted_meter_and_pointer` | :217 |
| FR-035-AC-15 | `reader_uses_one_work_ceiling_across_value_and_reference_validation` | :307 |
| FR-035-AC-16 | `reader_charges_each_occurrence_and_region_at_its_own_pointer` | :340 |

**`quire coverage --scope . --strict --json`:**

- 545 of 614 targets are backed.
- There are 43 unbacked rows and 0 status lies.
- FR-035-AC-11, AC-15 and AC-16 are `backed: true`.
- The only FR-035 rows still unbacked are AC-9 and AC-12 through AC-14. AC-9 predates this PR, and IR-893 owns AC-12 through AC-14.
- This matches the PR's own report: 43 unbacked and 0 contradicted.

**Status text.** The TC-047 status paragraph and the FR-035 and TC-047 rows in `tests.md` drop the stale-assertion caveat. They now say AC-11 is backed by the corrected assertions and AC-15 and AC-16 by the boundary tests. AC-12 through AC-14 stay PLANNED/UNRUN under IR-893, and both rows stay 🚧. This agrees with the matrix and with the code.

**Code without an owning requirement.** None. Every changed production line in `contract_package.rs` implements FR-035-AC-15 (the shared meter) or AC-16 (the per-occurrence and per-region charge and pointer).

**Stubs and coverage inflation.** None. Each tagged test drives the real reader over real lowered bytes. Each asserts consumed and path, and run against the prior reader at bd44821 the tests fail.

## Examined criteria

- FR-035-AC-11 (examined): "... `consumed` amount at the failed charge ... the failing source-map value for occurrences. For work it is `/semantic_graph/nodes/<i>/body` when value validation stops, or the failed-charge pointer reported by reference validation ... The work count and occurrence pointer follow AC-15 and AC-16."
- FR-035-AC-15 (examined): "... A limit that fits each pass separately but is below their combined count also returns `Incomplete`, never `Admitted`; no second meter restarts at zero."
- FR-035-AC-16 (examined): "... `path` naming the particular occurrence value ... or its `/regions/<k>` member, whose charge crossed the limit. A limit equal to that total admits; a stop does not report only `/source_map`."
- TC-047 (examined): the pinned mixed_fixture work (7/20/27) and occurrence (18) boundaries, and the status paragraph.
- FR-035-AC-12 through FR-035-AC-14 (context_only): out of scope; IR-893.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS.** AC-11, AC-15 and AC-16 are backed by tagged tests that exercise them. The matrix, `tests.md` and the TC-047 status agree with each other and with the code. The PR claims nothing more than it delivers.
