---
id: SR-12171
title: "IR-861 spec integrity review of IR PR #441"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@243009d0d25f271521b9a372fe30bfc4f8bde109 (PR #441, base origin/main 555639d54ff55f30d5d401b7748342d717fb5007); git diff origin/main...HEAD: FR-038 Verification cells for 38 criteria and the TC-048 'Selected variant types' section; consistency against spec/checked_package/matrix/tests.md:21 (FR-038 coverage row) and :32 (TC-048 Traces To), and quire coverage --scope . --strict on origin/main and on the head"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-861 spec integrity review of IR PR #441

## Summary

Ticket: IR-861. `quire validate` of the FR-038, TC-048 and tests.md files is
clean on the head (quire 0.36.2, engine 0.50.2).

I ran `quire coverage --scope . --strict` myself, on an export of origin/main
555639d and on the PR head:

- origin/main: 43 unbacked rows, 0 contradicted statuses.
- head: 39 unbacked rows, 0 contradicted statuses.

The four rows that changed are FR-038-AC-193, AC-194, AC-195 and AC-201. On
origin/main each one was an `unbacked-row` with no TC target. AC-193, AC-194 and
AC-201 were also `no-symbol-row` (Inspection). On the head, each targets TC-048
and is no longer listed as unbacked. No row went from backed to unbacked, and no
contradicted status appeared. The other 34 edited rows were already backed on
main through their trace tags, so their only change is the new TC-048 target.

Why the four changed: coverage counts a Verification row as backed once it
names a TC. It does not require a symbol tagged to that criterion. The computed
matrix on the head still shows AC-193, AC-194 and AC-201 as
`method-without-symbol` and AC-195 as `untagged`. So the drop from 43 to 39
comes from naming the TC, and it adds no new evidence.

- For AC-193, AC-194 and AC-201, that is legitimate. TC-048 holds the source
  Inspection procedures (private-intake steps 6 and 7, structural-inequality
  step 5), and the criteria record their inspections as completed or
  implemented.
- AC-195 is different (FND-002).

The FR-038 coverage row (tests.md:21) already lists "FR-038-AC-222 through
FR-038-AC-238", with TC-048 among its TCs. Contrary to the PR body, it needs
no change. The TC-048 Traces To cell (tests.md:32) lists 165..175, 177..182,
186..201, 214 and 246, but not 236..238 (FND-001).

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| TC-048 Traces To cell | examined | spec/checked_package/matrix/tests.md:32 | ... FR-038-AC-232, FR-038-AC-233, FR-038-AC-234, FR-038-AC-235, FR-038-AC-239, FR-038-AC-240 ... |
| FR-038 coverage row | examined | spec/checked_package/matrix/tests.md:21 | ... FR-038-AC-222 through FR-038-AC-238 ... TC-048, TC-050, TC-052, TC-053, TC-226, TC-227, TC-228, TC-445 |
| FR-038-AC-193 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4748 | IMPLEMENTED; source Inspection completed (IR-663 public boundary exclusion). ... Inspection (TC-048) |
| FR-038-AC-194 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4749 | IMPLEMENTED; source Inspection completed (IR-663 production ownership). ... Inspection (TC-048) |
| FR-038-AC-195 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4750 | IMPLEMENTED; final-head runtime/Clippy checks UNRUN, full gates pending (IR-663 consumer lint gate). The CODE slice passes the repository's model-consumer Clippy lane ... Test (TC-048) |
| FR-038-AC-201 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4756 | Implemented (IR-690); source inspection finds one shared retained-index/family/catalog/reference-resolution and domain-projection owner ... Inspection (TC-048) |
| FR-038-AC-236..238 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4793-4795 | IMPLEMENTED (IR-777) ... Test (TC-048) |
| Other 31 edited Verification cells | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4720-4832 | AC-165..175, 177..182, 186..192, 196..200, 214, 246: Test (TC-048) / Test, Inspection (TC-048) |
| SR-12121 FND-002 disposition | context_only | reviews/SR-12121-ir-840-pr440-spec-review-integrity.md:63 | deferred: IR-861: give AC-236..238 an owning TC procedure, then index them. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | After this PR, FR-038 says AC-236..238 are `Test (TC-048)`, and TC-048 has a section titled "(FR-038-AC-236 through FR-038-AC-238)". The TC-048 Traces To cell in tests.md still leaves out all three, so the index now contradicts both documents. Before this PR, no document claimed them, and the three documents agreed. SR-12121 FND-002 deferred this exact step to IR-861 ("give AC-236..238 an owning TC procedure, then index them"). The PR body gives two reasons for deferring, and both are wrong. First, it says the FR-038 coverage row needs the ids, but tests.md:21 already lists "FR-038-AC-222 through FR-038-AC-238". Second, it says the cell must wait for IR-852 edits, but no IR-852 code PR is open, and adding three ids to one cell costs a later PR at most a one-line rebase. Coverage does not check this cell (0 contradicted on both sides), so no gate will catch it. Fix in this PR: insert FR-038-AC-236, FR-038-AC-237, FR-038-AC-238 after FR-038-AC-235 in tests.md:32, and correct the PR body. | spec/checked_package/matrix/tests.md:32, spec/checked_package/matrix/tests.md:21, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4793-4795, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1459 |
| FND-002 | low | The 43 -> 39 drop in strict coverage includes AC-195, but nothing new backs it. Its Verification cell is now `Test (TC-048)`. Its own status says "final-head runtime/Clippy checks UNRUN". The computed matrix still shows it as `untagged`. No TC-048 test runs a Clippy lane: step 7 of the procedure only says "the CODE slice shall run both repository Clippy lanes ... and record their actual exits". Coverage now reports as backed a gate that has not run, because unrelated tagged TC-048 tests exist. Naming TC-048 is honest at the procedure level, so the cell is acceptable. The PR body, though, presents the drop as a coverage gain. Fix: say in the PR body that AC-193, AC-194 and AC-201 are backed by TC-048 Inspection procedures, and that AC-195 is backed by naming the TC only, with its Clippy evidence still UNRUN. Changing AC-195's verification method from Test is outside this PR. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4750, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1986-1991 |

## Verdict

FAIL: not mergeable until FND-001 is fixed. The fix is a three-id insertion in
tests.md:32. The PR does not need to wait for IR-852, and the FR-038 coverage
row needs no change. All 38 Verification edits are honest, and the coverage
change is fully explained: four rows, AC-193, 194, 195 and 201, with no
contradicted status. FND-002 is low and only needs the PR body corrected.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (e1a572d): the TC-048 Traces To cell in tests.md:32 now lists FR-038-AC-236, FR-038-AC-237 and FR-038-AC-238 between AC-235 and AC-239. The cell holds 221 ids, ascending, with no duplicates. Only that cell changed, and the FR-038 coverage row (tests.md:21) is unchanged, which is correct. The PR body no longer defers this step to IR-852. |
| FND-002 | fixed | Round 1 (e1a572d): the PR body now says that AC-193, 194, 195 and 201 count as backed once they name a TC, without a tagged symbol, and that "this is not new executed evidence". It also says that AC-195 is a Clippy-lane Test that stays untagged and UNRUN. |
