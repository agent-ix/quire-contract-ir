---
id: SR-1591
title: "integrity review of PR 299's spec edits (IR-628 AC-136 to AC-144 statuses)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@656fd549e65d0736993f87021b549dc4c2272b19; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-227-checked-package-v2-model-object-fields-accessor.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md, spec/tests.md; compared with open PR #297 (81fda6a6c6f2367dc6f2eabb9b3f17e4a5982549)"
review_set: subset
---
# SR-1591: integrity review of PR 299's spec edits

## Summary

Ticket: IR-628. The PR's spec edits are of four kinds:

- AC-136 to AC-144 move from "planned" to "implemented, ungated".
- TC-227's status becomes "Implemented", with a list of what was not exercised.
- The checked_package matrix row and `spec/tests.md` row are updated to match.
- The five new public types are added to FR-019's V2 reader row. That row is correct, and
  matches the `pub use` in `v2/mod.rs`.

This review checks whether those statuses are truthful against the code and the
reviewer's mutation runs (SR-1590). It also checks whether the spec's own mutation rows
can be caught at all.

Overlap with #297 (IR-627 code). `git merge-tree` of the two heads conflicts in three
files: `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`, the
adjacent AC-123 to AC-135 and AC-136 to AC-144 status rows; `spec/checked_package/matrix/tests.md`,
the FR-038 row and the TC-226, TC-227 and TC-228 rows; and `spec/tests.md`. The coder
listed only the last two, and FR-038 conflicts as well. The code (`v2/mod.rs`,
`model_members.rs`, `tests/it/main.rs`) auto-merges. The reviewer built the merged tree,
taking this PR's side of the spec conflicts, and the full workspace suite passed: 394
integration tests and 163 unit tests. Whichever PR lands second must merge in three ways:

- both status sets in FR-038's AC table;
- the union of the TC lists (TC-226, TC-227, TC-228) in the FR-038 matrix row, with both
  TC rows' new statuses;
- both sentences in `spec/tests.md`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-138 is marked "implemented, ungated", but one of its rows cannot be satisfied. The reader refuses a field typed as a relationship (`check_type_ref` pushes `malformed` for `relationships.contains(type_ref)`), so no admitted package holds the "relationship type ... present with `member_type` `None`" field the AC requires. Item 23 lists it too. TC-227's status notes the gap, but the FR-038 row says implemented without qualification. The AC and item 23 need amending, or the row needs a stated exception | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2933, crates/quire-contract-model/src/checked_package/v2/model_members.rs:1874 |
| FND-002 | low | AC-138's mutation row "bounds narrowed through `i64` or `f64` (the `i128` extremes differ)" is false for `f64`. `i128::MAX as f64 as i128` saturates back to `i128::MAX`, and `i128::MIN` is exact in `f64`. Every other bound in the fixture (0, 1000, -5, 5) is exact too. The reviewer's f64 round-trip mutant survives. Catching it needs a bound that `f64` cannot represent, such as 2^53 + 1 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2933 |
| FND-003 | low | Two AC-143 mutation rows cannot be caught as written. (a) "equality of packages gains the retained tables" is an equivalent mutant for every admitted package, because the tables are a pure function of documents that the package's digests fix. The reviewer's mutant survives, and only a crate-internal package with differing tables could detect it. (b) "it recurses over the chain": a recursive component search survives both the 200-type chain and the 1500-type chain, because neither depth can overflow a test thread's stack. Both rows claim a kill no test can give | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2938 |
| FND-004 | low | AC-144 is marked implemented, and it states that a frame entry, an abstraction relation's field entry and a relationship edge read each charge the table's entries end to end. The tests check those charges only at `ModelOwners::resolve_member` and `DomainModel::resolve`. TC-227's status discloses this, but the FR-038 row does not | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2939, crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs:314 |
| FND-005 | low | The coder settled three questions the spec text leaves open, and none of them is written back into the spec. (1) Systems interfaces get tables and are charged. Item 1 says "object type declaration", and the cost paragraph sums "over object types". (2) A diamond's or a repeated edge's copied entries are charged once per edge. (3) `AmbiguousField` carries the smallest ambiguous name; item 19 says only "the name". All three are consistent with the numbers the spec states: no stated case has an interface, a repeated edge or two ambiguous names. They are unstated behaviour, and (3) is untested (SR-1590 FND-003) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2021, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2087 |

## Verdict

The statuses are mostly truthful. AC-136, AC-137, AC-139, AC-140, AC-141, AC-142 and
AC-144 have direct, passing tests. Most of their mutation rows are killed, and one is not
(SR-1590 FND-001). AC-138 is overclaimed: one of its rows is impossible (FND-001). The
other findings are low. Three mutation rows of the spec cannot be caught at all
(FND-002, FND-003). AC-144's end-to-end wording goes beyond its tests (FND-004). Three
decisions are left unstated (FND-005). `quire coverage --strict` does not overstate
coverage: each AC has its own direct claims. The strict failure (37 unbacked rows) is
main's baseline.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 65622b8230443d661fa00aa55c2f02169b3fba3e |
| FND-002 | fixed | 65622b8230443d661fa00aa55c2f02169b3fba3e |
| FND-003 | fixed | 65622b8230443d661fa00aa55c2f02169b3fba3e |
| FND-004 | still-open | AC-144 still calls for frame, abstraction and relationship-edge field charge observations. The only checks remain at ModelOwners::resolve_member and DomainModel::resolve; TC-227 still discloses that end-to-end cases are unexercised. |
| FND-005 | fixed | 65622b8230443d661fa00aa55c2f02169b3fba3e |
| FND-004 | fixed | e48401f |
