---
id: SR-12120
title: "IR-840 base spec review of IR PR #440"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir (PR #440, base origin/main ); git diff origin/main...HEAD: spec/checked_package/matrix/tests.md:32 (TC-048 Traces To cell, 148 -> 188 ids); checked against spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Acceptance Criteria table), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/TC-445-direct-postcondition-integer-comparison.md, spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md, the per-criterion rows of spec/checked_package/matrix/tests.md and the trace tags under tests/ and crates/"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

# IR-840 base spec review of IR PR #440

## Summary

Ticket: IR-840. The PR edits one cell: the Traces To cell of the TC-048 summary
row in `spec/checked_package/matrix/tests.md:32`. It goes from 148 to 188
FR-038 criterion ids. The added ids are AC-176..196, AC-210..213, AC-222..224,
AC-247..251 and AC-262..268.

What TC-048 verifies was measured independently from four sources:

- (A) FR-038 ids named in the TC-048 document, with "through" ranges expanded.
  Incidental mentions were checked by hand: AC-6 belongs to FR-046's own AC-6,
  and AC-153/AC-155 are TC-228 context.
- (B) the Verification column of FR-038's Acceptance Criteria table.
- (C) the per-criterion `| FR-038 |... | TC-048 |` rows at tests.md:64-92.
- (D) `#[trace(...)]` and `/// Trace:` tags that name TC-048.

Results:

- All 188 ids in the new cell exist in FR-038. None is retired or missing: there
  is no AC-16, AC-34, AC-66 (retired), AC-156..158 or AC-219..221.
- Of the 40 added ids, only AC-183 is named by another TC: the TC-226
  description. FR-038 itself verifies AC-183 as `Test (TC-048, TC-226)`, so that
  shared claim is consistent, not a contradiction.
- AC-202..209 are correctly left out. FR-038 verifies each one as
  `Test (TC-445)` (FR-038:4568-4575), the TC-445 summary row claims them
  (tests.md:43), and so does the TC-445 Description (TC-445:21). The TC-048
  document never names them, and the tests that tag them (`checked_package_v2_comparison_operands.rs`,
  `checked_package_v2_frame_entries.rs`, `comparison_operands.rs`) carry no TC
  id; TC-445:112-113 names that test file.
  The ticket's statement that TC-048 text and the coverage row bind AC-202..213
  is wrong for AC-202..209. The FR-038 coverage row (tests.md:21) lists TC-445
  among its test cases, and that is why those ids appear there. The ticket's
  list should read 210..213, not 202..213.
- AC-219..221 are correctly left out because they do not exist. AC-236..238 are
  correctly left out because no TC claims them (see SR-12121).
- AC-153/154 are correctly left out. Both are `Test (TC-228)` in FR-038, are
  claimed by the TC-228 Description, and are tagged only to TC-228.

The cell still leaves out 21 criteria that TC-048 does verify (FND-001..003).
The PR body gives "TC-048 text only, no coverage or per-criterion row" as the
reason for leaving out AC-225..235 and AC-242..245. That reason is false: FR-038's
own per-criterion Verification cell for each of them reads `Test (TC-048)`. The
rule was also applied unevenly. AC-224 is PLANNED/UNRUN and was added, while
AC-228..235, PLANNED under the same kind of TC-048 section, were rejected.

A scratch copy (outside the repo) holds the corrected 206-id cell. On it,
`quire coverage --strict --scope.` still reports 43 unbacked and 0
contradicted, which matches both origin/main and the PR head. `quire validate`
of tests.md is clean. Adding AC-202..209 to that copy also leaves coverage at
43/0. So coverage does not check this index, and "0 contradicted" is no evidence
in either direction about the cell.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| TC-048 Traces To cell | examined | spec/checked_package/matrix/tests.md:32 | FR-038-AC-1... FR-038-AC-175, FR-038-AC-176... FR-038-AC-268 (188 ids) |
| FR-038-AC-176..196 (added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4542-4562 | AC-176/183/184/185 Test (TC-048); AC-177 "verified by the TC-048 composite operand tests"; AC-186..196 IR-663 intake retention, TC-048 "Private intake refusal-origin checks" |
| FR-038-AC-210..213 (added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Test (TC-048) |
| FR-038-AC-222..224 (added) | examined | spec/checked_package/matrix/tests.md:64,92 | FR-038-AC-222, FR-038-AC-223 / FR-038-AC-224 under TC-048 |
| FR-038-AC-247..251 (added) | examined | spec/checked_package/matrix/tests.md:65-68,86 | FR-038-AC-247.. FR-038-AC-251 under TC-048 |
| FR-038-AC-262..268 (added) | examined | spec/checked_package/matrix/tests.md:79-85 | Implemented (IR-851); tagged tests, TC-048 |
| FR-038-AC-202..209 (not added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4568-4575 | Implemented (IR-703); tagged local controls exist... Test (TC-445) |
| TC-445 summary row | context_only | spec/checked_package/matrix/tests.md:43 | TC-445 Typed clause-context integer comparison operands... FR-038-AC-202 through FR-038-AC-209 |
| FR-038-AC-225..235 (not added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4593-4603 | IMPLEMENTED (IR-739) / PLANNED/UNRUN (IR-491, IR-691)... Test (TC-048) |
| FR-038-AC-242..245 (not added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4610-4613 | IMPLEMENTED (IR-800; tagged reader tests passed)... Test (TC-048) |
| FR-038-AC-214 (not added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4587 | CODE-GATED (IR-575...). The production CheckedPackageV2::read path directly decodes the closed wire... Test, Inspection |
| FR-038-AC-9 (not added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4379 | The shipped default read-limit policy is exactly those six finite values... Test (TC-048) |
| FR-038-AC-21 (not added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4392 | A node whose declared qualified_name differs from its nominal preimage's qualified_declaration refuses... Test (TC-048) |
| FR-038-AC-153, AC-154 (not added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Test (TC-228) |
| FR-038-AC-236..238 (not added) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4604-4606 | IMPLEMENTED (IR-777)... Test |
| FR-038 coverage row | context_only | spec/checked_package/matrix/tests.md:21 | FR-038-AC-1 through... FR-038-AC-262 through FR-038-AC-268 / TC-048, TC-050, TC-052, TC-053, TC-226, TC-227, TC-228, TC-445 |
| TC-048 section headings | context_only | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md | Native String, UUID and Timestamp admission (FR-038-AC-242 through FR-038-AC-245); Reader peak memory (FR-038-AC-214); Aggregate-bodied union value keys (FR-038-AC-225 through FR-038-AC-227); Residual operation leaf sources (FR-038-AC-228 through FR-038-AC-231); Bounded integer record literal inside a closed composite (FR-038-AC-232 through FR-038-AC-235) |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The cell leaves out FR-038-AC-225..AC-235 and AC-242..AC-245 (15 ids), which TC-048 verifies. Three sources say so. FR-038's own Verification cell for each is `Test (TC-048)` (AC-235: `Test (TC-048), Inspection`). TC-048 has a section for each group. TC-048-tagged tests exist for AC-225..227 and AC-242..245. No other TC claims any of them. The PR's reason for leaving them out ("no coverage or per-criterion row") is false, so the PR's stated deliverable, that the cell lists every FR-038 criterion TC-048 verifies, is not met. Fix: add all 15. | spec/checked_package/matrix/tests.md:32, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4593-4603, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4610-4613, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:256, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:967, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1117, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2031, tests/conformance_qspec/main.rs:505, tests/it/checked_package_v2_operand_families.rs:230 |
| FND-002 | medium | The cell leaves out FR-038-AC-214. TC-048 has the procedure "Reader peak memory (FR-038-AC-214)". The AC-214 tests are tagged TC-048 (`tests/ir575_reader_memory.rs` "Tracing: TC-048, FR-038-AC-214", `checked_package_v2_canonical_encoding.rs:864`). No other TC claims it. FR-038's Verification cell ("Test, Inspection") names no TC, and so do the cells of AC-177..182 and AC-186..196, which the PR did add. Fix: add AC-214. | spec/checked_package/matrix/tests.md:32, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1539, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4587, tests/ir575_reader_memory.rs:36 |
| FND-003 | medium | The cell leaves out FR-038-AC-9 and AC-21. FR-038 verifies both as `Test (TC-048)`. Real tests are tagged to TC-048: `#[trace("TC-048", "FR-038-AC-9", "FR-038-AC-26")]` at checked_package_v2_reader.rs:2120, and `#[trace("TC-048", "FR-038-AC-21")]` at:2615,:2668 and three more. No other TC claims either one. The TC-048 procedure text does not mention them, which is a separate gap in the TC-048 text, not a reason to leave them out of the index. Fix: add AC-9 and AC-21. | spec/checked_package/matrix/tests.md:32, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4379, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4392, tests/it/checked_package_v2_reader.rs:2120, tests/it/checked_package_v2_reader.rs:2615 |

## Verdict

FAIL: not mergeable as is. The 40 added ids are all correct. Each one exists,
each is backed by TC-048 text and FR-038 (by its Verification cell, its
statement or a per-criterion row), and none conflicts with another TC's claim.
Leaving out AC-202..209, AC-219..221, AC-236..238 and AC-153/154 is also
correct, and the ticket text is wrong to include 202..209. The cell is still
short of "every FR-038 criterion TC-048 verifies" by 21 ids. The corrected cell
holds 206 ids, ascending, in the existing `FR-038-AC-n` comma-separated form:
FR-038-AC-1..5, 9..11, 17..33, 35..38, 42..65, 67..72, 74..122, 151..152,
159..201, 210..218, 222..235, 239..268. That is the PR head's cell plus AC-9,
AC-21, AC-214, AC-225..235 and AC-242..245.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 : tests.md:32 now lists FR-038-AC-225 through AC-235 and AC-242 through AC-245. The cell holds exactly the 206 ids recommended in this review: ascending, no duplicates, none nonexistent, and AC-202..209 and AC-236..238 still absent. |
| FND-002 | fixed | Round 1 : AC-214 is in the cell. Keeping it there is correct even though its FR-038 Verification cell reads "Test, Inspection" with no TC. TC-048 has the "Reader peak memory (FR-038-AC-214)" procedure, the tests carry TC-048 tags (tests/ir575_reader_memory.rs:36, checked_package_v2_canonical_encoding.rs:864), and no other TC claims it. The missing TC in the Verification cell is the defect IR-861 tracks (SR-12121 FND-001), and it is no reason to leave AC-214 out of the index. |
| FND-003 | fixed | Round 1 : AC-9 and AC-21 are in the cell. |
