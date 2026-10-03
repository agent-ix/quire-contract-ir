---
id: SR-830
title: "code review of PR 260 (IR-450 a quantity at a position requires a bound)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@e7fc7a69a1dfb6632a5a1b0e69be368918d51ffc; crates/quire-contract-model/src/checked_package/v2/lower.rs, tests/it/checked_package_v2_require_bounds.rs, spec/checked_package/matrix/tests.md"
review_set: subset
---
# SR-830: code review of PR 260

## Summary

Ticket: IR-450. Reviewed head e7fc7a69a1dfb6632a5a1b0e69be368918d51ffc. The PR was cut
from origin/main 7ed352beb994355413b9d7c572e45667aee229ba (#257, the spec). During the
review #258 merged, and main moved to a41c0e5f7360d37b8a3ce740108d8077329b36c8. The head
is now one commit behind and conflicts with main in `spec/checked_package/matrix/tests.md`
(FND-001). The rust-review lane is folded into this file. Every check ran in a separate
detached worktree, not taken from the PR body.

Predicate. `quantity_position_ends(&positions, &ordered)` is a separate function. It is
called only inside `if profile.require_bounds`. `requires_bound()` changes only in its doc
comment: `Unit` and `CompoundUnit` stay in the `false` arm, and the typed path is
unchanged. `quantity_chain` is exhaustive over every family and form:

- `unit` and `compound_unit` are a quantity;
- every `bounded_domain` form except `model_population` continues through its own
  `semantic_type`;
- anything else ends the chain with no quantity.

The walk is a `for _ in 0..=closure.len()` loop over a `BTreeMap` of the closure. It is
iterative, with no call to itself. It is bounded by the closure size. A cycle exits after
n+1 steps with no insert, and a base outside the closure breaks. It inserts the `node_id`
of the node at the end of the chain (the unit), never the position or a domain.

Positions. The positions are the existing `positions` set. They are the typing successors
of a `composite_type` node, and edge 0 (the own `semantic_type`) of a `value`/`parameter`
node. This is FR-038's definition. A `literal.type` is an `Annotation` edge, so it is
never a position. An application's `result_type` and `semantic_type` go to `typed` only.
A requested unit or compound unit has no position of its own, so it lowers.

Interaction. The quantity test is one more disjunct in the existing
`ordered.iter().find(...)`. So a single record names the least `node_id` among every
offender: recursive, quantity, unbounded at a position, or typed-and-unbounded. There is
no double reporting, and the eight forms' refusals are unchanged. `require_bounds: false`
skips the block entirely.

Rust. The `QuantityChain` enum is private, and there is no `unwrap`, `expect`, `panic`,
`unsafe` or integer cast. The match is exhaustive, so a new form must decide. The pass is
O(positions × chain length) and is not charged to the work meter. That is the same order as
the existing unmetered `is_bounded` scan inside the same `find`, so it is no new resource
class. No compatibility layer, pin, SHA or vendored copy was added, and there is no QSL
dependency. The tests build QSL-shaped nodes from this crate's own vocabulary
(`nominal_fixture_members`).

Tests. The focused run (`cargo test --test it checked_package_v2_require_bounds`) passed
14 of 14. The whole `checked_package_v2` filter passed 109 of 109, including
`tc_052_unbounded_forms_are_exactly_the_eight_declared_forms`. `make deny` passed
(advisories, bans, licenses, sources, one-copy). The gate log `ir-450-ci.log` ends
`head=e7fc7a69a1dfb6632a5a1b0e69be368918d51ffc exit=2`. Only `spec` fails, at the strict
baseline: 23 unbacked rows and 0 contradicted, none of them FR-038. The `it` suite passed
189 and lib passed 101.

The helper changes are behaviour-preserving for existing callers. `key("metre")` was used
by no test on main. `add` now forwards `"integer"` to the new `application`, which builds
the same node.

Mutation probes, each reverted (the `checked_package_v2` filter):

- `Unit`/`CompoundUnit` added to `requires_bound()`: TC-052's eight-forms test and
  `tc_050_a_unit_is_not_a_quantity_position_merely_by_being_reached` fail;
- the chain walk dropped (`Base => break`): `..._requires_a_bound_naming_the_unit` fails;
- the position named instead of the unit: the same test fails;
- the walk stopped after one domain (`0..2`): the same test fails, on `dom2_field`;
- the quantity disjunct removed: the naming test and the least-key test fail;
- the walk seeded from `typed` instead of `positions`: TC-052 and the not-merely-reached
  test fail;
- annotation edges counted as typing: five tests fail, including three new ones.

Recursion versus iteration cannot be tested: a deep chain would only show stack depth. The
code has no self-call, so this is settled by reading the code.

QSL agreement. This was run in a detached QSL worktree at origin/main
8d1deba4318d6dddf6448bd77d6d822c70c660a0, with a command-line `--config patch` to this head
and no file edits. With `--include-ignored`, all six `extent_agreement` tests pass,
including the ignored `tc_440_quantity_extent_agrees_with_ir_requires_bound` (Measure). The
same run against IR main 7ed352b fails that test. The QSL package suite otherwise has one
failure, `admission_corpus` ("CyclicEquality: IR admits it: retire the known gap"). It fails
identically against IR main 7ed352b, so this PR does not cause it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The head conflicts with current main a41c0e5 (#258 merged during review) in the FR-038 row of `spec/checked_package/matrix/tests.md`, so the PR cannot merge as is. Rebase. Resolve the row by keeping #258's AC-74 through AC-80 planned sentence and its `AC-35 through FR-038-AC-80` range, dropping the AC-73 planned sentence, and listing `AC-70 through AC-73` as implemented. Then rerun the gate, because #258's planned AC-74 through AC-80 change the strict counts. The TC-050 row merges cleanly | spec/checked_package/matrix/tests.md:15 |

## Verdict

Approve the code, subject to a rebase. The predicate, the chain walk, the positions, the
naming and the least-key selection match FR-038's quantity paragraph and FR-038-AC-73
clause for clause. AC-8 and the eight-forms test are untouched and green. Every requested
mutation is killed. The QSL Measure agreement that IR-450 exists for now passes against
this head. The only blocker is the tests.md conflict with #258.

## Dispositions

Round 1 at c3d89a01928d806bba6b0df7859e1eebb72d877b, rebased onto origin/main
a41c0e5f7360d37b8a3ce740108d8077329b36c8, which is still current. Against
e7fc7a69a1dfb6632a5a1b0e69be368918d51ffc, `lower.rs` and the test file are byte-identical.
The delta, excluding the rebase, is one FR-038 sentence (see SR-831) and the re-resolved
tests.md.

`git diff origin/main HEAD -- spec/checked_package/matrix/tests.md` changes exactly two
rows, FR-038 and TC-050. Every other row is identical to main.

The gate log ir-450-r2-ci.log ends
`head=c3d89a01928d806bba6b0df7859e1eebb72d877b exit=2`. Only `spec` fails: 23 unbacked,
0 contradicted, and none of them FR-038. `it` passed 189 and lib 101. Re-run here:
require_bounds passed 14 of 14 and the TC-052 eight-forms test passed. No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | c3d89a01928d806bba6b0df7859e1eebb72d877b |
