---
id: SR-1130
title: "gap analysis of PR 274 against FR-038-AC-112, FR-038-AC-113 and TC-048 (IR-552 code)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@cb0a8bf46b879a39de3c27682c208cb879299a7d; git diff origin/main...cb0a8bf (base 90ad41d); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-112, AC-113; AC-70, AC-97, AC-106 for the retagged and added tests), spec/checked_package/matrix/tests.md (FR-038 and TC-048 rows), spec/tests.md"
review_set: subset
---
# SR-1130: gap analysis of PR 274 against FR-038-AC-112, FR-038-AC-113 and TC-048 (IR-552 code)

## Summary

Ticket: IR-552. In scope: FR-038-AC-112 and FR-038-AC-113 as merged in #272, traced to the tests in `tests/conformance_qspec/main.rs`, the matrix flips, and the bindings of the added recursion tests. Plan completion: not assessed.

What I examined, with `make conformance-qspec` run against a throwaway worktree of quire-specification origin/main (396493c):

- **FR-038-AC-112.** Every clause has code. The harness reads `adverse.json` from `QUIRE_SPECIFICATION_DIR` and applies both lists to a fresh copy of `positive-all-families.json`, refreshing no identity. It runs the `flattened` control and keeps the expected-failure list with id, today's refusal and owning ticket. It fails on an absent id, a stale entry, a 'neither' refusal and an unlisted mismatch. The five listed `body_grammar_mutations` give exactly the listed refusals on origin/main, and IR-495 owns them. The result test fails on origin/main because of the cause comparison (SR-1129 FND-001), so the AC is not verified today. See FND-001.
- **FR-038-AC-113.** It replaces `dependency_selections` in both the `lock` and the `identity_preimage`. It derives through `CheckedPackageIdentityPreimageV2` and `quire_canonical::sha256`, as the reader does (mod.rs:772), and the result equals the recorded `package_id`. An entry with `version` fails to decode as a V2 lock. The unchanged base derives a different id. The fail-closed cases are tested: variable unset or empty, file missing, recorded id differing. The result test passes on origin/main. See FND-002 for the 'own package_id' and `base` clauses.
- **Bindings.** The four new harness tests are tagged TC-048 with AC-112 or AC-113, which is correct. `tc_048_the_reader_admits_equality_over_a_record_cycling_through_a_tuple` is tagged AC-70. AC-70 covers recursive compared types through a tuple (its `Pair` case), and the Cell case is QSpec FR-322's own tuple example, so the tag is correct. `tc_048_a_union_cycle_under_an_option_enters_at_recursion_one` is tagged AC-106, recursive unions, also correct. The temporal test keeps `FR-038-AC-97`, and the edit there is comment text only, so the FR-370 'orphan' claim holds.
- **Matrix.** The FR-038 row, the TC-048 row and the spec/tests.md cell move AC-112 and AC-113 from planned to implemented. `make spec` keeps the baseline: validate passes, grammar 1, 23 unbacked under strict, 0 contradicted. The trace tags back the flip, but the claimed behaviour does not hold for AC-112 (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FR-038 matrix row says AC-112 is 'implemented ... and verified by TC-048' and that 'every structural_mutations entry refuses as recorded'. TC-048 says the same. On quire-specification origin/main, `tc_048_qspec_adverse_mutations_refuse_as_recorded` fails on `negative-temporal-interval-bound` (recorded `refused:invalid_package`, reader `refused:invalid_package/invalid-value`). The reader does refuse it with the recorded code. The harness's whole-string comparison is stricter than AC-112's 'cause ... where it gives one' (SR-1129 FND-001). The flip is backed by tags but not by a passing run against current QSpec. Fix the comparison; the matrix text can stay as written once the run passes. | spec/checked_package/matrix/tests.md:15, 23; tests/conformance_qspec/main.rs:289-307 |
| FND-002 | low | AC-113 says 'the base fixture unchanged derives its own different package_id' and names the `base` fixture through the vectors file. The harness checks only that the unchanged base does not derive the recorded id. It never checks that the result equals the base's own `package_id.digest`. It also ignores the file's `base` member (`fixtures/positive-all-families.json`) and hardcodes `MUTATED_BASE`. If the base changes, the run fails closed on the id, but the failure message points at the derivation, not at the base. Assert `derived_with(base, None) == base.package_id.digest`, and read the base path from `vectors.base`. | tests/conformance_qspec/main.rs:51, 343-376, 544-551 |

## Verdict

AC-113 is implemented and verified, with one low gap against its 'own package_id' wording. AC-112 is fully implemented in code, and its fail-closed semantics match the AC. But its result test is red on current QSpec because of a comparison stricter than the AC, so its 'verified' matrix flip does not hold today (FND-001, high). Seven bindings were checked, and all are correct. No production code without an owning requirement was found: the production edits are comments only. No QSpec file is copied.

## Dispositions

Round 1 was reviewed at 05465f3f419fe4d49aa6c8a598bbac730cd40fc8. `make conformance-qspec` against quire-specification origin/main (396493c) passes all 6 tests. The AC-112 matrix flip is now backed by a passing run. Guard-deletion mutants for the own-package_id check and the selections non-empty guard each fail `tc_048_qspec_selection_run_fails_closed` (see SR-1129 Dispositions). `make spec` is at the baseline.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 05465f3 | With `refuses_as`, `negative-temporal-interval-bound` meets its cause-less recorded outcome. The AC-112 result test passes on QSpec main, so the 'verified' and 'every structural_mutations entry refuses as recorded' claims now hold. |
| FND-002 | fixed 05465f3 | `selection_problems` checks that the unchanged base derives its own recorded `package_id.digest`, and a changed base digest fails that check. `selection_inputs` reads the base path from the vectors' `base` member through `base_path_of`. |
