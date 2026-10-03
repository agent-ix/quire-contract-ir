---
id: SR-781
title: "integrity review of PR 255 (IR-486 equality over a recursive record is admitted)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@154fc38ef518b036ef68ca1bb25fd6911846750d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md Operation leaves prose, new subsection, AC-43, AC-69..71; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md description, AC-43 procedure, Recursive compared types; spec/checked_package/matrix/tests.md FR-038 and TC-048 rows; overlap with open PRs 254 (7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee), 250 (81cd8036de8a1c3819811747cc0fc2ccbc64a2b7), 253 (fb2345a6ac6782c89896e8230c680cd7633f3a9d), 243 (5ca1531a60ce7d2e785a6d3d4e1d2cb73017a0b4) via git merge-tree"
review_set: subset
---
# SR-781: integrity review of PR 255 (IR-486 equality over a recursive record is admitted)

## Summary

Ticket: IR-486. Base: origin/main is af733f23f42788ea2c8dfa1eb31adaff39e85960, which is the PR's
merge base, so the base is current and GitHub reports MERGEABLE. AC-43 is amended in place and
keeps its trace tags. The matrix states that its current test asserts the earlier clause,
following the same pattern as IR-535. No criterion is removed. The FR-038 and TC-048 matrix ranges
add AC-69..71 as 🚧. TC-048's description and procedures cover each new clause. No stale cyclic
refusal is left in TC-048. The PR introduces no pins, SHAs, vendored files or compatibility layer.
The title carries no bare ticket id, and the body says "Part of IR-486".

Overlap with open PRs, predicted with `git merge-tree` against the reviewed head:

- #254 (IR-503, head 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee) conflicts in all three files: the
  FR-038 AC table, TC-048, and the `tests.md` FR-038 and TC-048 cells. It also collides on AC
  numbering (FND-001).
- #250 (IR-505, head 81cd8036de8a1c3819811747cc0fc2ccbc64a2b7) conflicts only in the `tests.md`
  FR-038 and TC-048 cells. The texts are compatible.
- #253 (IR-530, head fb2345a6ac6782c89896e8230c680cd7633f3a9d) conflicts only in the `tests.md`
  FR-038 and TC-048 cells. The texts are compatible.
- #243 (IR-274, head 5ca1531a60ce7d2e785a6d3d4e1d2cb73017a0b4) auto-merges with this PR in FR-038.
  Its one conflict, in AD-005, is with main and does not come from this PR.

Cross-repo note: QSL FR-093-AC-19 names IR's refusal of recursive equality as its one admission
exception (STD-129). QSL will need to drop it once the IR code lands. CG's tc_029 refusal pin flips
then too.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The AC ids collide with open PR #254. After its fix round, #254 (head 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee) defines FR-038-AC-65 to AC-69, with its own AC-69 (a `temporal_interval` or `fairness` member). This PR defines AC-69 to AC-71. The PR body's "#254 (IR-503) takes AC-65..68" is stale. Both PRs add rows after AC-64 and edit the same matrix cells, so the second to merge conflicts. A careless resolution leaves two different FR-038-AC-69 rows, and trace tags against AC-69 become ambiguous. Recommendation: whichever merges second renumbers. If #254 lands first, this PR becomes AC-70..72, and every reference to AC-69..71 moves with it: AC-43's "(FR-038-AC-69)", the TC-048 section and description, and both `tests.md` cells. If this PR lands first, #254's AC-69 becomes AC-72. The leader decides the order. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:989-991 |
| FND-002 | low | The base paragraph still says, unqualified, "the number of text leaves is memoised per type node" and "deriving them costs the supplied leaves times the nesting depth times the width". It also says "each type node outside a cycle is counted once". The subsection restricts the memo to nodes whose subtree reaches no open composite. `Node` under `Two` is in a cycle and is still memoisable. A composite that is not memoisable must be re-counted to skip a sibling with no text, so the cost bound does not hold for cyclic types. Fix: qualify the base sentences by reference to the subsection (memo where it applies, and the cost bound for types whose nodes are all memoisable), and leave the budget as the bound otherwise. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:673-683 |

## Verdict

Request changes, for the AC-69 collision with #254 (medium). One low internal inconsistency: the
base prose's memo and cost claims.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | TC-048's AC-43 procedure still says "a type whose only cycle runs through an option". The amended AC-43 and the new prose refuse any cycle with no record or tuple on it, and define a mixed type: an independent record-free cycle is refused even beside a record cycle. Fix: say "a type that reaches an option again with no record or tuple between the two visits", matching AC-43. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:294-296 |

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@2f54f96507fd81b6d1b428cbf1113ad4cd50f035. The PR was rebased onto origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39, after #254 merged, and is one commit on it. Its diff against that base is the delta I reviewed; the rebase is excluded. GitHub reports MERGEABLE. `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/212 rows backed, FR-038 42/70 (main: 163/209, 42/67). The new ids are AC-70 to AC-72, and the only FR-038-AC-69 row is #254's. No AC id is duplicated. The diff and the PR body contain no SHA.

Notes on the round-1 checks: the ids are renumbered to AC-70..72 after #254 merged. Every cross-reference moved: AC-43, AC-44, the TC-048 description and section, and both `tests.md` cells. The `tests.md` FR-038 cell keeps #254's AC-65..69 note. The base prose now qualifies the memo and the cost bound (lines 792-808).

Overlap re-check: #254 is merged. #250 and #253 still edit the same `tests.md` cells, which is a mechanical conflict only. One new low finding is below.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |
| FND-002 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |

Round 2, reviewed at agent-ix/quire-contract-ir@7775e142e702e2a3e21a8426eca9356638838889: the delta from 2f54f96507fd81b6d1b428cbf1113ad4cd50f035, on base origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39, which is unchanged. GitHub reports MERGEABLE. `make spec` at round 2: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/212 rows backed, FR-038 42/70. The delta adds no SHA. The leader relayed a planner decision, recorded on IR-486, that the recursion leaf is now required wherever text is reachable. I have not confirmed it independently.

Notes on the round-2 checks: TC-048's AC-43 procedure now matches the amended AC-43 ("reaches an option again with no record or tuple between the two visits"), and adds the sibling-shared option admission case.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-003 | fixed | 7775e142e702e2a3e21a8426eca9356638838889 |
