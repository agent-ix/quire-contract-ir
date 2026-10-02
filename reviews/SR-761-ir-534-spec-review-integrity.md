---
id: SR-761
title: "integrity review of PR 252 (IR-534 selections bind by identity)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@31d2f9654bd4136ae64c1500074ce01793382af8; whole spec/ tree grep for version wording on selections (FR-038, FR-040, FR-019, FR-035, FR-344, AD-004, AD-001, TC-044, TC-048, TC-056, tests.md); stale class-number and AC references; overlap with PR 250 (IR-505) and merged PR 251"
review_set: subset
---
# SR-761: integrity review of PR 252 (IR-534 selections bind by identity)

## Summary

Ticket: IR-534. I grepped the whole `spec/` tree for `version` wording tied to selections, locks,
dependencies or domain packages, and for stale class numbers. These are clean:

- FR-040, FR-344, FR-019, FR-035 and AD-004: their `version` hits are other concepts (contract
  and schema tags, profile versions, the dependency mutation axis).
- Class numbers: no reference to "class 5", "five classes" or "class 2" remains anywhere.
- AC ids: no reference to AC-16, AC-34 or AC-65+.
- #251: AC-62..64 are the next free ids after #251's AC-46..61, and #251's rows and planned
  markers are kept.

Overlap with PR 250 (IR-505, head f7ca22c193e71cf2e6a0e2fceaf60ef7c03b32ac, based on e80ea70):
#250 does not touch FR-038 or TC-048. It edits only `tests.md` (the FR-040, FR-038, TC-048 and
TC-056 status cells). `git merge-tree` shows #250 already conflicts with main (#251) in `tests.md`,
and it conflicts with #252's head the same way. Whichever merges second hand-resolves the FR-038
and TC-048 status cells, in either order. The two texts are compatible in substance:

- #250 makes AC-45 and AC-5's owner clause implemented (content-only `ModelOwner`).
- #252 amends AC-45's invariance from "version changes" to "row digest changes" and marks the
  amended selection clauses planned until IR-535.
- The merged cell must keep #251's AC-46..61 planned text, #252's AC-62..64 and amended-clause
  planned text, and #250's AC-45/AC-5 implemented claim, limited to the owner clause. AC-45's
  "lock row carries no version" clause stays planned (IR-535).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The PR amends AC-45 so the node keys are unchanged "when only the selected row's `digest` changes" (the row has no version). TC-048's AC-45 procedure still says "Change only the selected domain package's version and re-read" and "change only the selected version and re-read". The expected results still say "unchanged by a version-only change of the selection". Under the amended shape that step cannot be performed: a row carrying `version` refuses `unknown_member`. Separately, the expected results at lines 80-81 list only `authority`, `revision` or `export` as `unknown_member`, while the procedure at line 53 now adds `version`. Rewrite the AC-45 steps as a digest-only change (another document of the same identity), and add `version` to the expected `unknown_member` list. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:58-59,64-65,80-81,84-86 |
| FND-002 | low | There are two matrix inconsistencies. (1) The FR-038 row lists AC-45 among the amended selection clauses planned until IR-535, but the TC-048 row's list ("AC-2, AC-10, AC-19, AC-20, AC-27, AC-31 and AC-32") omits AC-45. (2) Both cells keep the unchanged sentence that AC-1 to AC-4, AC-10 to AC-44 (TC-048), or AC-17 to AC-33 (FR-038), are "implemented", which in the same cell contradicts the new statement that the amended clauses are planned and their tests assert the earlier shape. Exclude the amended clauses from the implemented ranges, or qualify them, and add AC-45 to the TC-048 list. | spec/checked_package/matrix/tests.md:15,23 |
| FND-003 | low | With version gone, AC-10's last clause ("two entries sharing an identity but differing in digest refuse as `stale_dependency` at `lock.model_selections`, never as `malformed_wire`") and AC-20's main clause assert the same thing. One assertion owned by two criteria drifts. Drop the clause from AC-10 and keep it in AC-20, or have AC-10 refer to AC-20. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:865,874 |

## Verdict

Mostly consistent. The prose, class order and criteria agree with each other, with one stale
TC-048 procedure (FND-001) and two matrix wording problems.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@a1f44f02c78fa786dc3e5582a45f46a069d74bd5 (delta from 31d2f9654bd4136ae64c1500074ce01793382af8; base origin/main eedc378d7fe879e0d107e04318773528f9595422 unchanged; merge clean). `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/204 rows backed, FR-038 42/62. No new finding.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
| FND-002 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
| FND-003 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
