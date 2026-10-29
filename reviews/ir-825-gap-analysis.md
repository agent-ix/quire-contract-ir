---
id: SR-11846
title: "IR-825 PR 423 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; git diff origin/main...HEAD: FR-052-AC-1, AC-2, AC-3, AC-5, AC-6 and AC-24, FR-019 source_obligation row, TC-452, spec/model/matrix/tests.md TC-058 and TC-452 rows, and the tests that carry the new tags"
review_set: subset
---

## Summary

Ticket: IR-825, PR quire-contract-ir#423. This gap analysis is planless and
scoped to the PR diff. Plan completion: not assessed.

The computed matrix (`quire matrix --format tsv`, quire 0.36.2) shows these
criteria as `tagged`:

| Criterion | Tagged tests |
| --- | --- |
| FR-052-AC-1 | `default_feature_consumer_borrows_public_census_and_rows` and `tc_058_model_root_exports_exactly_the_documented_public_items` |
| FR-052-AC-2 | `synthetic_many_to_many_and_unmapped_inventory_survive_permutation_and_row_removal` |
| FR-052-AC-3 | seven tests |
| FR-052-AC-5 | the consumer test |
| FR-052-AC-6 | `every_owner_position_refuses_missing_and_typed_incomplete_outcomes` |

FR-052-AC-24 is `untagged` and stays Planned until IR-820 is decided, as
FR-052, TC-452 and tests.md all state. `quire coverage --scope . --strict`
reports 43 unbacked rows and 0 contradicted statuses at HEAD, against 48 on
origin/main. The drop of five matches the five newly backed criteria. The
command still exits non-zero, because the other unbacked rows already exist
on main and are out of scope. `quire validate` over spec/** reports 96/96 docs
grammar-clean.

FR-019's `source_obligation` row now credits FR-052-AC-1 through AC-6. That is
accurate: AC-4 was already backed, and AC-1, AC-2, AC-3, AC-5 and AC-6 are
newly tagged. The TC-058 row adds FR-052-AC-1, which matches the new tag on
`tc_058`. The PR adds no production code, so there is no reverse gap. There
are no stubs.

The dispatching brief asked for test-oracle strength, so the optional
semantic step ran as a mutation pass. The results are recorded in SR-11845.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-052 status prose now claims "AC-3 tests assert every named rejection and absence from accepted rows", and TC-452 marks AC-3 Implemented. That overstates the evidence for conflicting assertion provenance. No test covers a same-ID provenance conflict that has a member with a foreign provenance source, and the precedence branch that gives that case its `ConflictingProvenance` disposition survives mutation (see SR-11845 FND-001). The Implemented claim for AC-3 is true only after that case is added. The TC-452 "Synthetic backing" section also still reads as future work, now contradicting its own Execution Status. Examples are "add a `#[test]` external consumer", "add that assertion", and "Three clauses need new assertions before AC-3 is traced". It also omits `tc_058` from the list of tests that back AC-1. | spec/model/functional/FR-052-source-obligation-correspondence.md:487-495; spec/model/matrix/TC-452-source-obligation-correspondence.md:173-262; spec/model/matrix/TC-452-source-obligation-correspondence.md:13-17 |

## Verdict

CONDITIONAL. Every in-scope criterion is tagged, AC-24 is correctly left
Planned and untagged, and coverage moves from 48 to 43 unbacked rows with 0
contradicted. The AC-3 status claim depends on the test added for SR-11845
FND-001. The TC-452 synthetic-backing prose should be reworded from
instructions to a record of what exists.

## Coverage

- Plan completion: not assessed
- Criteria examined: FR-052-AC-1, AC-2, AC-3, AC-5, AC-6 (tagged), AC-24 (Planned, untagged by design)
- Matrix: 43 unbacked rows and 0 contradicted (HEAD); 48 and 0 on origin/main
- Reverse gap: none (no production code changed)
- Stubs: none
- Semantic review: run as mutation testing (16 mutants, 15 killed); see SR-11845

## Dispositions

Round 1. Checked against fix
with `git diff origin/main...HEAD`. `quire coverage --strict` still reports 43
unbacked rows and 0 contradicted. In `quire matrix`, FR-052-AC-1, 2, 3, 5 and
6 are tagged and AC-24 is untagged. `quire validate` over spec/** and
reviews/** reports 899/899 docs clean.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Both parts are resolved. First, the AC-3 claim in FR-052 is now true without a prose change: the new same-ID mixed-provenance case backs the conflicting-provenance disposition, and it kills the early-check mutant (SR-11845 FND-001). The coder's statement that FR-052 needed no edit is correct. Second, the TC-452 "Synthetic backing" section is now a record of what exists rather than a list of instructions. It names each backing test, lists `tc_058` for the inventory half of AC-1, adds the mixed-provenance case under AC-3, and drops "Three clauses need new assertions before AC-3 is traced". |
