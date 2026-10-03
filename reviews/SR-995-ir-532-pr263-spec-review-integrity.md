---
id: SR-995
title: "integrity review of PR 263 (FR-038 operation-law ACs AC-81 to AC-88)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@376269e0d67ee1451ee9629dbe897c06a8a730de; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md; compared with merged FR-038-AC-17, AC-36, AC-43, AC-44, AC-56, AC-57, AC-66 to AC-69 and open PRs #250 (17c38988295df042c8f478cdab3a5b689c15c2cb), #253 (d049ceaafe63d208a8ac3a1a4a8facddf17fe06a), #259 (00d83e4a30020ca2901890a649d5b7164f5e8251)"
review_set: subset
---
# SR-995: integrity review of PR 263

## Summary

Ticket: IR-532. Checks consistency, atomicity and cross-reference integrity of the
new ACs against merged FR-038 criteria and the held PRs.

- No merged AC is deleted or reworded. New AC ids AC-81 to AC-88 are free at
  origin/main and on #250, #253 and #259.
- Cross-references hold. AC-82's law-definition defer goes to AC-56/AC-57, the
  Mode bullet's leaf defer to "Operation leaves" (AC-44), and AC-81's order defer
  to AC-67.
- AC-81 vs AC-67 (open question c). AC-67 (planned, IR-503) already states
  "an unknown identity refuses `unknown-operation` and an `operator` that differs
  ... refuses `operation-class-mismatch`", with no pointers, before its ordering
  clause. AC-81 gives the pointers and the admission control, and the existing
  tests back it. Recommendation: keep AC-81 as the owner of the base refusals.
  Do not amend AC-67 to absorb them, because tagging these tests to AC-67 would
  claim its ordering clause, which they do not exercise. Trim AC-67's restatement
  in #253, which owns AC-67's code and rewrites the neighbouring AC-66 anyway.
- Conflicts (open question 5), predicted with `git merge-tree` against a merge of
  this PR onto main. Today #250 merges clean against main. With this PR merged
  first, #250 conflicts in `spec/checked_package/matrix/tests.md` (FR-038 and
  TC-048 rows). #259 and #253 already conflict with main in tests.md and in code,
  and this PR adds one more conflict on the same two rows. The FR-038 prose and
  the TC-048 file auto-merge with #253. Merging this PR forces #250 to rebase,
  which the planner asked to avoid. Timing is the leader's call.
- Sequencing for the binding PR (open question b). #253 rewrites the shared
  fixtures `dummy_law_definition`, `real_integer_division_truncating_definition`
  and `empty_lock` to `{authority, identity}`. That touches the AC-82 tests too
  (`too_many_laws` 3288, `wrong_law_role` 3328), not only the AC-56 tests at 3363
  and 3398. #253 also adds its own AC-56 test. The binding PR should land after
  #253 and tag the post-#253 bodies. All 24 test names survive in #253.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-81's first two clauses restate AC-67's opening clause (unknown-operation, operation-class-mismatch). There is no contradiction, but the same refusal is now stated twice. AC-67's restatement should be trimmed to its ordering and lowest-digest clauses | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1389, 1403 |
| FND-002 | low | AC-83 and AC-85 both state the same case: a `literal` typed at a `decimal_range` that pins `nearest-even`, as the first operand of `quire.op.decimal.add` under `toward-zero`, refuses `operation-mode-type-mismatch` at `operation.mode/value`. One test (operations.rs:5755) would then back the same clause under two ACs. Keep it in AC-83 only | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1405, 1407 |

## Verdict

Consistent apart from two low duplications. The base review (SR-994) holds the
contradiction with AC-68. Merge timing against #250 is open for the leader.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | deferred | AC-81 now owns the base refusals with pointers and the admission control; trimming AC-67's restatement belongs to #253 (IR-503), which owns AC-67 and its test, so no change is needed in this PR. |
| FND-002 | fixed | 9d4379b |
