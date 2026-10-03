---
id: SR-771
title: "integrity review of PR 254 (IR-503 new catalog words)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@731e307dc132975d3f3f6fac172c8c81bfe83b18; spec/checked_package/functional/FR-038-consume-checked-package-v2.md new prose and AC-65..68; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md description and Catalog words; spec/checked_package/matrix/tests.md FR-038, TC-048, FR-035 rows; tests/it/support/checked_package.rs v2_all_families temporal clause; overlap with open PRs 243, 250, 253 (git merge-tree)"
review_set: subset
---
# SR-771: integrity review of PR 254 (IR-503 new catalog words)

## Summary

Ticket: IR-503. Base: origin/main is af733f23f42788ea2c8dfa1eb31adaff39e85960 (#252), and that is
the PR's merge base, so the base is current and GitHub reports MERGEABLE. AC-65..68 are the next
free ids after AC-64. No existing criterion is removed or edited. The FR-038 and TC-048 matrix
rows extend their ranges to AC-68 and keep every earlier planned note. No stale reference to the
old clause shape remains in `spec/`.

Overlap with open PRs, predicted with `git merge-tree` against the reviewed head:

- #250 (IR-505, head 81cd8036de8a1c3819811747cc0fc2ccbc64a2b7): conflicts in `tests.md`, in the
  FR-038 and TC-048 cells. The texts are compatible. The merged cell keeps #254's AC-65..68
  planned note and #250's AC-45/AC-5 owner-clause change.
- #253 (IR-530 draft, head fb2345a6ac6782c89896e8230c680cd7633f3a9d, based on eedc378): conflicts
  in `tests.md`, in the FR-038 cell, where it rewrites AC-46..61 as implemented. The texts are
  compatible. The resolver keeps #254's AC-65..68 sentence.
- #243 (IR-274, head 5ca1531a60ce7d2e785a6d3d4e1d2cb73017a0b4): merges cleanly with #254 in
  FR-038 (its hunks are at lines 146-195). Its one conflict, in AD-005, is with main, not with
  this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-67 contradicts AC-66. AC-67, and the prose at lines 646-648, say a `temporal_interval` or `fairness` member "on an entry whose catalogued member is another kind or none" refuses `operation-member-mismatch`. That covers the seven member-less `temporal_formula` entries, and, for `fairness`, the eight `temporal_interval` entries. All of them are `temporal_formula` entries, which AC-66 refuses `unsupported_construct` at `operator` before the member is compared. TC-048 builds exactly this conflict: "a `fairness` member on `quire.op.temporal.holds`" is expected to give `operation-member-mismatch` (TC-048 lines 150-151 and 165-166), while AC-66 requires `unsupported_construct`. A correct implementation fails the TC as written. Fix: limit the member clause to entries whose operator class is not `case`, `temporal_formula` or `temporal_fairness`, and pick such an entry in TC-048 (for example `quire.op.boolean.not`, or `quire.op.temporal.clause`). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1005 |
| FND-002 | medium | The new clause shape breaks the shared fixture, and nothing records it. The `v2_all_families()` temporal clause (tests/it/support/checked_package.rs:643-655) has member `profile_operator` and zero arguments. Under the new shape it refuses `operation-member-mismatch`. About 60 call sites in 9 test files build on that fixture, behind ✅ rows for FR-035 (TC-044), TC-052, TC-053, TC-056, TC-222 and TC-048. The new prose (line 668) also says every formula node a clause can name is refused. So no conformant temporal clause can admit, and an all-families package can keep one only through a `temporal`/`formula` node whose body is not a temporal application, which IR admits only because it does not check FR-370 placement. Fix: state in FR-038 and the matrix what an admitted all-families package holds for the temporal family after this change, and mark the affected ✅ rows as needing re-verification under IR-503 code. | spec/checked_package/matrix/tests.md:12,21,23-26,28 |

## Verdict

Request changes. One high internal contradiction: AC-67 and TC-048 against AC-66. One medium
unrecorded consequence: the shared all-families fixture's temporal clause.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The re-verification note is on the FR-035, FR-038, TC-044, TC-052, TC-053 and TC-222 rows. It is missing from three rows whose tests also build `v2_all_families()`: TC-056 (and its FR-040 row; `tests/it/checked_package_v2_frame_entries.rs`), TC-050 (`tests/it/checked_package_v2_lowering.rs`, which it shares with TC-052) and TC-048's own row. The FR-038 prose names TC-056 and TC-048 but not TC-050 or FR-040. Add the note to those rows, and add TC-050 and FR-040 to the prose list. | spec/checked_package/matrix/tests.md:23,27,28 |

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee (delta from 731e307dc132975d3f3f6fac172c8c81bfe83b18; base origin/main af733f23f42788ea2c8dfa1eb31adaff39e85960 unchanged; GitHub MERGEABLE). `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/209 rows backed, FR-038 42/67. AC ids now run to AC-69, so the next free id is AC-70.

Notes on the round-1 checks: AC-67 now holds only the precedence and least-digest clauses, both for body-root nodes. New AC-69 limits the stray-member mismatch to entries outside the three refused classes, and pins `fairness` on `quire.op.temporal.holds` as `unsupported_construct`. TC-048 now uses `quire.op.boolean.not` and `quire.op.temporal.clause`. FR-038 and the matrix record the fixture consequence. One new low finding is below.

Overlap re-check at the new head: #250 and #253 still conflict only in the `tests.md` FR-038 and TC-048 cells, and the texts are compatible. #243 still merges cleanly with this PR.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
| FND-002 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |

Round 2, reviewed at agent-ix/quire-contract-ir@b1679e36a5acefb327f623be7c997f305509fb0f (delta from 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee; base origin/main af733f23f42788ea2c8dfa1eb31adaff39e85960 unchanged; GitHub MERGEABLE). `make spec` at round 2: validate passes (304 docs, including the four committed reviews/ SR files), 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/209 rows backed, FR-038 42/67. The committed reviews/SR-770, SR-772, SR-773 and SR-774 files are byte-identical to the reviewer's finals. The new expression-form gap sentence is accurate: QSpec's schema `cause_tag` enum lacks the tag, and STD-153 carries it as item (3) in a 2026-10-03 comment.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-003 | still-open | The fix adds the note to the FR-040, TC-048, TC-050 and TC-056 rows, and the FR-038 prose list now reads FR-035, FR-040, TC-044, TC-048, TC-050, TC-052, TC-053, TC-056 and TC-222. One fixture user is still missing. Grepping tests/it for v2_all_families finds tests/it/complete_v1_contract_package.rs, whose helper at line 42 builds on v2_all_families(); its eight tc_047_* tests are TC-047's evidence. The FR-035 row, which covers TC-044 and TC-047, carries the note, but the TC-047 row (tests.md line 22) does not, and the prose list omits TC-047. Fix: add the note to the TC-047 row and add TC-047 to the prose list. Every other file that uses the fixture maps to a noted row: complete_v1_checked_package.rs to TC-044, the reader, dependency_reference and dependency_selections tests to TC-048, lowering to TC-050 and TC-052, frame_bodies to TC-053, frame_entries to TC-056, adr002_members to TC-222. |

Round 3, reviewed at agent-ix/quire-contract-ir@7aaed4df179f57d2e756df7072400ff9eed6befd (delta from b1679e36a5acefb327f623be7c997f305509fb0f; base origin/main af733f23f42788ea2c8dfa1eb31adaff39e85960 unchanged; GitHub MERGEABLE). `make spec` at round 3: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/209 rows backed, FR-038 42/67. I grepped the whole tree independently for `v2_all_families`: nine test files under tests/it use it, besides tests/it/support/checked_package.rs, which defines it. No crate test module or src test uses it. By their `tc_` tags the nine map to TC-044, TC-047, TC-048 (reader, dependency_reference, dependency_selections), TC-050 and TC-052 (lowering), TC-050 and TC-056 (frame_entries; FR-040), TC-053 and TC-222 (FR-344). Every one of those rows now carries the re-verification note, and the FR-038 prose list names exactly that set.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-003 | fixed | 7aaed4df179f57d2e756df7072400ff9eed6befd |
