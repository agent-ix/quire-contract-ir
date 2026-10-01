---
id: SR-660
title: "code review of PR 240 (IR-486 exact operation leaf count, paths and laws)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@5dddf2eaea8b4115ca611c915318bcd48d394d4e; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: base
---
# SR-660: code review of PR 240

## Summary

Ticket: IR-486. The PR makes `check_leaf_count` compare the supplied `operation.leaves` with the derived ones. The count is compared with `!=`: too few refuses `operation-law-missing`, too many refuses `operation-law-mismatch` at the first extra leaf. When the count matches, the new `LeafWalk::first_leaf_fault` derives the expected paths one at a time and checks each leaf's `laws` for exactly one catalogued `text_profile` law. Then comes a lock-selection check (`operation-law-unselected`). A supplied leaf on an entry with no leaf source now refuses `operation-law-mismatch` ahead of the per-law loop. The Rust-review lane is folded into this file.

Compared with the reference reader (quire-specification `tests/checked_package_v2.rs`, origin/main, `collect_leaves` 1467-1516 and `classify` 1681-1885; untrusted external code, read as data), these all match the reference: path segments (`field:<name>`, `position:<n>`, `inner`), declaration pre-order, the law test (`laws.len()==1`, role `text_profile`, a catalogued definition), refusal order (missing, then shape mismatch over every leaf, then unselected) and the placement of the no-source check (ahead of unselected). `text_profile` is a value role, not a profile role, so checking `definition_selections` is correct.

Probes. These ran as temporary unit tests in the review worktree and have been reverted. A duplicate leaf at the right count refuses at `/1/path`. An extra trailing segment refuses at `/0/path`. Leaf 0 unselected plus leaf 2 at a wrong path refuses mismatch at `/2/path`, as the reference does. A cycle with too few leaves refuses `operator-ineligible`. 70 levels of 2 shared fields (2^70 leaves, the count saturates) refuses `operation-law-missing` straight away.

The memo is per walk and keyed on the resolved structural position, and `kids` is filled for every composite that `count` pushed. So `first_leaf_fault` cannot read a stale or missing entry after a successful `count`. Both walks are iterative and charge every visit. The `tc_048_leaf*` tests pass under `ulimit -v 1000000` with a 1 MiB stack in 0.17 s.

Mutations re-run by the reviewer: `!=` to `<`, the path comparison disabled, the law check disabled, the unselected check disabled and the no-source check disabled. Each one turns at least one test red. The three adjusted older tests are not weakened. The two same-type tests used to supply `[{path:["field:name"],laws:[]}]` over an all-integer or empty record, which the reference refuses as a mismatch (expected `[]`). Their subject (same_type) is still asserted. The leaf mode-type test used to supply a rounding leaf with no law over a decimal field, which the reference also refuses as a mismatch. It now exercises the leaf mode-type check over a text field.

Gates at head, run by the reviewer: `make fmt-check lint test corpus` exit 0 (179 it + 82 unit + 2 doc tests). `make deny` ok. No conflict with open PRs #239 or #241 (`git merge-tree` clean; disjoint files).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A `result_inner` entry whose result is not a set, bag or ordered_set returns before any count, so supplied leaves are never compared. A probe admitted `collection.flatten` from `sequence<sequence<text>>` to `sequence<text>` with one leaf. The reference expects `[]` and refuses `operation-law-mismatch`, and FR-038 says such a result expects none and that extra leaves refuse | crates/quire-contract-model/src/checked_package/v2/operations.rs:2031-2053 |
| FND-002 | medium | If the compared type of an `operand:0`/`inner:0` entry does not resolve (for example an untyped literal first operand), the leaf checks are skipped and any leaf list is admitted. A probe admitted `structural.eq` over two untyped literals with one junk leaf. The reference refuses `operator-ineligible` (`expected_leaves` None). This was already the case before the PR, but FR-038 does not state it | crates/quire-contract-model/src/checked_package/v2/operations.rs:2027-2030, 2051-2053 |
| FND-003 | low | No test pins the no-source leaf check ahead of the per-law selection loop. A mutation that moves it after the loop leaves all 83 lib tests passing, because the only test uses `quire.op.integer.add`, which has no law | crates/quire-contract-model/src/checked_package/v2/operations.rs:466-471, 3765-3780 |

## Verdict

Not mergeable as is. FND-001 is an over-admission of the exact class this PR exists to close ("more supplied leaves than text leaves refuse"), and FND-001 of SR-648 left it open by turning the non-set `result_inner` case into a skip rather than an expected count of zero. Fix: for `result_inner` with a non-set result, treat the expected count as 0 and run the same comparison. FND-002: refuse an undecidable compared type as the reference does, or list the deviation in FR-038 (see SR-662). FND-003: add a law-bearing no-source case (for example `integer.div` with its law unselected plus one leaf, which must refuse mismatch at `leaves/0`).

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@b2ba6c53bfb89f7a7bff9be7890ad92662452764. The fix commit is 489d6a8; b2ba6c5 only names the `LeafShape` type alias.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | No test pins `operation-law-unselected` ahead of the leaf mode checks (the reference runs `selected` before `mode_value_admitted`). A mutation that moves the unselected check after both mode loops leaves all 86 lib tests passing. The current order is correct: a probe with leaf 0 missing its mode under an empty lock refuses unselected at `leaves/0/laws/0/definition` | crates/quire-contract-model/src/checked_package/v2/operations.rs:2134-2168 |
| FND-005 | low | `text_profile_pin` runs on every text visit in `LeafWalk::enter` and again for every emitted leaf. It walks the alias and bounded-domain chain without charging the meter, on top of the uncharged `structural_type` walk that was already there. A probe with a 3000-alias chain under a record of 3000 fields took 7.5 s (debug) to use 1000 work units, against 2.4 s with the pin walk removed and 26 ms with a shallow chain. The uncharged chain walk is pre-existing (needs its own ticket); this PR roughly triples its cost. Fix: memoise the pin per type id, or charge the chain steps | crates/quire-contract-model/src/checked_package/v2/operations.rs:1734-1752, 1858-1859, 1973-1974 |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 489d6a8 |
| FND-002 | accepted-no-change | FR-038 now states it as an open deviation: an operand the reader does not type (untyped literal, aggregate, binding, dependency_reference, frame, application without result_type) and a set-like result whose inner type does not resolve skip the leaf comparison. This over-admits only input the reference refuses, never QSL output, and is part of the reader-wide operand-typing gap (IR-484 covers part of it). A follow-up ticket should cover it together with the float leaf deviation |
| FND-003 | fixed | 489d6a8 |

## New findings (disposition pass 2)

Reviewed at agent-ix/quire-contract-ir@ecfcb825b906e62b378ac04c43002d1ec9e7ff09, rebased on origin/main 457566c.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | No test pins that each new pin-chain step is charged to the meter. A mutation that removes the `walk.meter.charge` call in `text_profile_pin` leaves all 88 lib tests passing. `tc_048_text_profile_pin_of_an_alias_chain_is_memoised_and_charged` says the chain costs "about 600 work units", but it only fails when the memo is removed, because a 1000-unit meter is never reached either way. Fix: assert the refusal under a budget the uncharged walk would fit but the charged one exceeds (for example 450 units), or assert the units consumed | crates/quire-contract-model/src/checked_package/v2/operations.rs:1752-1754, 3891-3917 |

## Dispositions (round 2)

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed | ecfcb82 |
| FND-005 | fixed | ecfcb82 |
