---
id: SR-1583
title: "integrity review of quire-contract-ir PR #298 (IR-627 recursion-group amendment)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@a5e2e32ea87a3c649a827caf296ef294d243a583; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md, spec/checked_package/matrix/tests.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: IR-627. Integrity lens: IDs, numbering, matrix consistency and the strict
delta. The PR's base is bf36cda. `origin/main` is now a71ae81, which is IR-628
spec #296. That merge already holds FR-038-AC-136 through FR-038-AC-144 and TC-227,
so all three new IDs collide, and `git merge-tree` reports content conflicts in
all three spec files. The author's delta "37 -> 40 unbacked rows" is correct
against the stale base only: `make spec` printed 37 at bf36cda and 40 at
a5e2e32, with validation clean. Covered (✅) rows are unchanged. The FR-038 and
TC-226 rows that change are 🚧 planned rows. The mutation-row correction is
accurate: `recursion` and `group_reference` are preimage-only, and each is
reached through a wire member. The PR edits no CI files. QSL and QSpec are cited
by section and paraphrased, never copied. Open PR #297 (IR-627 code) adds no new
AC IDs. It does implement the superseded refusal (`derived_key` returns `None`
for a node with `recursion_group`), and it edits FR-038, so it must be reconciled
with whatever this PR becomes.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | high | The new AC IDs collide with merged main. `origin/main` (a71ae81, IR-628 #296) defines FR-038-AC-136 through FR-038-AC-144 (TC-227, model object fields). This PR mints FR-038-AC-136, AC-137 and AC-138 with different meanings. Its matrix ranges ("AC-109 through AC-138") and its TC-226 row overlap TC-227's criteria. The PR does not merge cleanly: FR-038, `checked_package/matrix/tests.md` and `spec/tests.md` conflict. Rebase onto main, renumber to the next free IDs (FR-038-AC-145 through AC-147 at the time of review), update every cross-reference in the prose, TC-226 and both matrices, and re-measure the strict delta against main. | FR-038-AC-136, FR-038-AC-137, FR-038-AC-138, spec/checked_package/matrix/tests.md, spec/tests.md | wrong-requirement |

## Dispositions

Round 1, reviewed at 49b85f5d96d54b655660117c5fff54ba4973eade.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7f70b5c: Renumbered to FR-038-AC-145..150. The merge with a71ae81 is clean, #296's AC-136..144 rows are byte-identical, TC-227 is untouched, and the strict delta against main is 47 -> 53, exactly AC-145..150. |
