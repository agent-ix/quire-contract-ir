---
id: SR-811
title: "integrity review of PR 257 quantity bound class (IR-450)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@10a17ec6b705ec88b5442a3367b826043d76ed2b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md; spec/checked_package/matrix/tests.md"
review_set: subset
---
# SR-811: integrity review of PR 257 quantity bound class

## Summary

Ticket: IR-450. This review covers consistency and completeness of the changed FR-038 prose,
AC-8, AC-73, the TC-050 procedure and the matrix rows. It also checks those against the rest
of the spec tree and the open PRs. A grep of `spec/` finds no other text naming the eight
forms or the removed "need no bound here" sentence. FR-019 does not mention them. AC-22
lowers its parameter and compound-unit package with `require_bounds: false`
(`tests/it/checked_package_v2_parameters.rs:363`), so AC-73 does not flip it. The amended
AC-8 keeps exactly eight unbounded forms. The author did not grow the list to ten. Units are
a separate positional class instead, and AC-8 and TC-052's eight-form test are consistent
with that.

Open PRs, from `git merge-tree` against each head. #243 (IR-274) merges with this PR's
FR-038 text without conflict. Its only conflict, in AD-005, comes from #243 being 12 commits
behind main. #250 (IR-505) and #253 (IR-530 and IR-503) conflict on the FR-038 row of
`tests.md`, which every FR-038 PR edits. #258 (IR-533) conflicts on both FR-038 and
`tests.md`, and also allocates FR-038-AC-73 (FND-002). The IR-450 code follow-up will edit
`lower_one`, near #243's `lower.rs` hunk at line 506.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-038 prose contradicts itself on what "unbounded by form" means. It keeps "A reachable type is unbounded by form exactly when [the eight forms]. No other family and no other form of those two families is unbounded by form". Two sentences later it says "The two quantity forms `unit` and `compound_unit` are unbounded by form at a position only". The next paragraph then says that a type "unbounded by form" that is typed at, or requested, but named at no position, is bounded only when a reachable `bounded_domain` covers it. A reader who takes units as unbounded by form adds them to `requires_bound()`. That makes a requested `unit` and a `compound_unit`'s dependency units raise, which is the opposite of the quantity paragraph and of AC-73's lowering cases. Keep "unbounded by form" for the eight forms only. Call a quantity its own class, for example "a quantity is unbounded at a position and nowhere else". Have the position paragraph say "a type that is unbounded by form, or a quantity". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1040-1047, 1068-1083 |
| FND-002 | low | FR-038-AC-73 collides with open PR #258 (IR-533), which adds FR-038-AC-73 through AC-78 to the same table position and to the same `tests.md` FR-038 row ranges. Both edit the same lines, so the second to merge gets a textual conflict and must renumber its ACs and their trace rows and TC text. Do not hand-merge the two AC-73 rows. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1179; spec/checked_package/matrix/tests.md:15 |
| FND-003 | low | The `tests.md` FR-038 row says AC-73 "amends AC-8's closed list of forms". AC-8's list of eight is unchanged. The amendment adds a clause that a requested `unit` or `compound_unit` does not raise, and that clause is already true of today's reader. Say what actually changed: AC-8 now names the requested quantity cases, and AC-73 adds the positional rule. | spec/checked_package/matrix/tests.md:15 |

## Verdict

Request changes. FND-001 is the defect to fix before the code PR, because it decides which
implementation a coder writes. FND-002 and FND-003 are bookkeeping.

## Dispositions

Round 1 at 4b19dbe6da4c34665076d325b4d7d385d265fa50. "Unbounded by form" now names only the eight forms. A quantity is its own class, checked by a separate position predicate and "never through the typed path". The leader's decision stands that #257 merges first. Open PR #258 (IR-533) still adds FR-038-AC-73 through AC-78 at its head, so #258 must renumber to AC-74 onward when it rebases. The tests.md FR-038 row now says AC-8's list of eight is unchanged. A grep of `spec/` finds no stale wording: "unbounded by form at a position", "covered or not", "amends AC-8". No new findings.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed 4b19dbe6da4c34665076d325b4d7d385d265fa50 |
| FND-002 | accepted-no-change | Leader decision: #257 merges first, and #258 (IR-533), which still claims AC-73 to AC-78, renumbers to AC-74 onward on rebase. The textual conflict forces this, so a duplicate id cannot land silently |
| FND-003 | fixed | fixed 4b19dbe6da4c34665076d325b4d7d385d265fa50 |
