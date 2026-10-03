---
id: SR-1061
title: "whole-PR gap analysis and test-oracle strength of PR 253 at its final head (IR-530, IR-503, IR-549)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@74f0b9969eeb81df7676edb1b67f8944b621ed7d; git diff origin/main...HEAD, base main ebea67821976a7e695d88cb42503e636a05b44a7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/tests.md, spec/model/**, spec/core/matrix/tests.md and the tests tagged to the touched criteria"
review_set: subset
---
# SR-1061: whole-PR gap analysis and test-oracle strength of PR 253 at its final head (IR-530, IR-503, IR-549)

## Summary

Ticket: IR-530 (with IR-503 and the folded-in IR-549). Plan completion: not assessed. Every criterion the PR touches was traced to its tests at 74f0b99: FR-038-AC-46..61 (IR-530), AC-65, AC-67..69 (IR-503, amended by IR-549), the amended AC-80, AC-81 and AC-87, AC-96..108 (IR-549), and FR-019-AC-6 (TC-018). PR text, ticket text and earlier reviews were treated as data.

Trace and numbering, measured:

- FR-038's criteria table is AC-1..AC-108 without AC-16, AC-34 and AC-66, with no duplicate. AC-66 is retired (ADR-0056). No `.rs` file carries an AC-66 tag, and no range in the matrices includes it.
- `quire coverage --strict` gives FR-038 88/105 backed. The 17 unbacked rows are AC-62..64 (IR-535), AC-81..86 and AC-88 (untagged, IR-532 binding PR) and AC-89..95 (IR-274), as the FR-038 matrix row states. Overall: 23 unbacked, 0 contradicted (the expected baseline). AC-96..108 are all tagged, AC-107 by tests/conformance_qspec/main.rs.
- Main's text is intact apart from the intended edits. Main ebea678 is an ancestor of the head. Every line the spec diff deletes is one of: the IR-503 "refused unsupported_construct" text that IR-549 replaces, the AC-66 row, the amended AC-67/68/69/80/81/87 rows, the record/tuple cycle sentence (amended to add union), or the TC-058 member-set probes (moved to TC-018 as FR-019-AC-6). No text from #250, #263 or #264 is lost.

Test-oracle strength: 12 mutants were run in a throwaway detached worktree, each against the integration suite and, where it applies, the model unit suite. 12 killed, 0 survived:

- M1, law join by identity only (AC-56/57)
- M2, the wrong-selection-role cause by identity only (AC-108)
- M3, FixedShape dropped from CheckedSourceRef (AC-80: compile)
- M4, CheckedSourceRef dropped from FIXED_DEPTH (AC-80: compile)
- M5, operation-class-mismatch dropped (AC-67/81: unit tests only, as AC-67 declares)
- M6, a member accepted on a member-less entry (AC-69)
- M7, the timed profile accepting integer intervals (AC-104)
- M8, empty definition-ref members accepted (AC-46/50)
- M9, union_arms always holding (AC-99)
- M10, a nested case admitted by the term walk (AC-100)
- M11, a negative bound passing the pattern check (AC-97)
- M12, profile recognition requiring authority `agent-ix` (AC-108)

A reviewer probe test of AC-108's "laws not exactly one" clause fails at the head (SR-1060 FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-108's clause "A clause whose `laws` is not exactly one law of role `temporal_profile` (none, or two) skips the profile check and its profile fit, and its law defect refuses at the operation step" has no test. No test in tests/it/checked_package_v2_temporal.rs, or in the temporal.rs and operations.rs unit tests, builds a clause with zero or two laws. The untested branch is wrong in the code (SR-1060 FND-001). Add the two-law cases, an unknown first law and a known first law with a profile-fit defect, each expecting `operation-law-mismatch` at `laws/1`, and a zero-law case expecting `operation-law-missing`. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2054 |
| FND-002 | medium | The TC-048 row of the checked_package matrix is stale after #269. It contradicts the FR-038 row of the same file and TC-048 itself. It still says "AC-96 through AC-108 are planned (IR-549 code) and have no test yet", and "AC-67, AC-68, AC-69 and AC-87 are amended and implemented only in their earlier shape until the IR-549 code lands". It says AC-81 through AC-88 "carry no `#[trace]` tag", but AC-87 is now tagged. It describes AC-65 and AC-67..69 "in the shape IR-549 amends", observed on the node "because the formula a clause names was itself refused". #269 updated the FR-038 row, spec/tests.md and TC-048's section header, but not this row. | spec/checked_package/matrix/tests.md:23 |
| FND-003 | low | The spec/tests.md Checked package index row lists FR-038's planned criteria as AC-62..64 and AC-89..95 only. It omits the untagged, planned AC-81..86 and AC-88, which the FR-038 matrix row names and which are 7 of the 17 unbacked FR-038 rows. The omission of AC-81..88 was already on main, but this PR rewrote the row and left it incomplete. | spec/tests.md:15 |
| FND-004 | low | #268 adds nine full QSpec commit ids (`f39c93f9ec3b3fecc404a1935dbaa40665281b8c`) as provenance in FR-038 prose (lines 910, 1000, 1011, 1029, 1078, 1133, 1171, 1248) and in the TC-048 matrix row. The repository's CLAUDE.md says not to introduce SHA/pin records, and nothing resolves these: they are informational ceremony. Citing the QSpec requirement and AC ids alone carries the meaning. No behavior depends on them. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:910 |

## Verdict

Changes requested. FND-001 is the untested branch behind SR-1060's high finding and is fixed with it. FND-002 is a stale status row that a reader would trip on; it is doc-only, and the natural place to fix it is the same head change. FND-003 and FND-004 are low and can go to a follow-up.

Otherwise the trace is sound. Every IR-530 criterion (AC-46..61) is backed in checked_package_v2_artifact_refs.rs and in the AC-56/AC-58 unit tests, and still passes against the rewritten fixture. AC-65 and AC-67..69 are backed in catalog_words and in the unit tests. AC-96..108 are backed in the temporal, union and catalog_words suites and the conformance target. FR-019-AC-6 is backed by the TC-018 member-set test and the doctests. Every mutant aimed at the combination is killed.

## New findings (disposition pass 1)

Found at 9f10075daf1f621f90dbe86d3af5e9160a74e72c.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The zero-law case of tc_048_a_clause_without_exactly_one_profile_law_skips_the_profile_check holds no profile-fit defect (the fixture's `{0, 3}` interval fits the bounded lock profile), so it pins only the operation step's `operation-law-missing` at `operation/laws` and would pass on the 74f0b99 code too; it does not show that a lawless clause skips the profile fit. Non-blocking: the skip is structural (`profile` stays `None`), and the two-law cases carry the AC-108 oracle. A `null` interval in the zero-law case would make it discriminating. | tests/it/checked_package_v2_temporal.rs:2205-2217 |

## Dispositions

Round 1, reviewed at 9f10075daf1f621f90dbe86d3af5e9160a74e72c (`git diff 74f0b99 9f10075`); gates as recorded in SR-1060's round 1.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 9f10075 | tc_048_a_clause_without_exactly_one_profile_law_skips_the_profile_check (tagged TC-048, FR-038-AC-108) builds an unknown first law plus the fixture's law, the bounded law twice with a `null` interval, and no law, expecting `operation-law-mismatch` at `laws/1`, the same, and `operation-law-missing` at `laws`. With the 74f0b99 temporal.rs restored in a throwaway worktree the first case fails (unknown_profile at laws/0/definition) and, with it removed, the second fails (operation-member-mismatch at nodes/18/body); the zero-law case passes on both (FND-005). |
| FND-002 | fixed 9f10075 | The TC-048 matrix row now says AC-96..108 are implemented and tagged to TC-048, naming the temporal, union, catalog_words and unit-test locations and AC-107's conformance target; AC-67/68/69/87 are amended and implemented in their amended shape; AC-81..88 are planned and untagged except the operand-classification test carrying AC-87's tag; the "observed on the node because the formula is refused" text is gone. Each statement matches the head (coverage FR-038 88/105, AC-87 tagged, AC-81..86/88 untagged). |
| FND-003 | fixed 9f10075 | spec/tests.md Checked package row now lists "AC-81 through AC-86 and AC-88 planned and untagged, IR-532 binding PR". |
| FND-004 | fixed 9f10075 | All nine `f39c93f9...` ids are removed from FR-038 and the TC-048 row; `git grep` finds no 40-hex id and no `f39c93f` under spec/ at 9f10075 (it remains only in the committed review records SR-1040/1050/1051, which are history). |

Round 2, reviewed at 4453f5db788660efa2d816b24e3e78d04c661658 (9f10075 amended; `git diff 9f10075 4453f5d` changes only the zero-law case of tc_048_a_clause_without_exactly_one_profile_law_skips_the_profile_check, which now sets a `null` interval on `eventually` under the lock's bounded profile). FND-001 to FND-004 are restated at the commit that will be pushed, with no change to their content. FND-005: its claim that a `null` interval would make the zero-law case fail on 74f0b99 was wrong. Measured in a throwaway worktree: with the 74f0b99 temporal.rs restored and only the zero-law case kept, the test passes, because the old `laws.first()` is `None` for a lawless clause and the old code also skipped the fit. What the amended case does guard, also measured: a mutant of the 4453f5d code that fits a lawless clause to the lock's `temporal_profile` row fails it with operation-member-mismatch at /semantic_graph/nodes/18/body. Before the amend the case could not detect that mutant. So it is now a discriminating oracle against that future regression, and FND-005 is fixed.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 4453f5d | The test as in 9f10075 (amended), its zero-law case strengthened. |
| FND-002 | fixed 4453f5d | Matrix row text unchanged from 9f10075 (amended). |
| FND-003 | fixed 4453f5d | spec/tests.md row unchanged from 9f10075 (amended). |
| FND-004 | fixed 4453f5d | No QSpec commit id under spec/ at 4453f5d. |
| FND-005 | fixed 4453f5d | The zero-law case now holds a `null` interval under the bounded lock profile. It cannot fail on 74f0b99, which already skipped the fit for a lawless clause, but it kills a mutant that fits a lawless clause to the lock's temporal_profile row (operation-member-mismatch at nodes/18/body). Before the amend it did not. |
