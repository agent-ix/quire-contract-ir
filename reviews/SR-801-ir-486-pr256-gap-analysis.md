---
id: SR-801
title: "gap analysis of PR 256 (IR-486 recursive equality and the recursion leaf)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@c832ea54d6528369ac8de8f554d8b0d0bcd5d51b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_recursive_leaves.rs"
review_set: subset
---
# SR-801: gap analysis of PR 256

## Summary

Ticket: IR-486. Measured against the merged spec, IR #255: FR-038 "Operation leaves",
"Recursive compared types", the deviations paragraph, amended AC-43 and AC-44, new AC-70,
AC-71 and AC-72, and the TC-048 "Recursive compared types" procedure. QSL FR-093 rule 3
and its E14, E16 and E17 vectors were read for `d`, from the sibling checkout at
origin/main. The paths and `d` match the tests: `recursion:0` for Node and A, `recursion:1`
for `Option<Node>`.

AC coverage, each clause checked against a test that asserts code, cause and pointer:

- AC-43 (amended): record-free cycle and missing node refuse, and a record cycle admits:
  `tc_048_leaf_walk_refuses_a_record_free_cycle_and_an_unresolved_node`.
- AC-44 (amended): existing AC-44 tests still pass; recursion leaves are covered by AC-70.
- AC-70: Node, the text leaf alone, the recursion leaf alone, a further text leaf at `/2`,
  `Option<Node>`, A/B at either end, Two lacking either recursion leaf, Tree2, Pair, Wrap
  (six admit, four text leaves give missing, seven give `/6`), R{a?,b?}, and an integer
  List under eq and contains. All are unit tests; Node also runs end to end in the
  integration test.
- AC-71: before the text leaf, wrong prefix, wrong `d`, a second recursion leaf, laws,
  mode, the List reentry, unpinned text inside the cycle (with leaves supplied or not), an
  unselected law, and T=Option<T>, Sequence<T> and a record holding one at work 1000
  (ineligible, not incomplete); `R { x: Option<R> }` admits.
- AC-72: 20000-record cycle on a 256 KiB thread with raised limits; ten records give
  `incomplete`/work at `operation.leaves` under default limits; 12-ring at exact work and
  one below.

The old cyclic-refusal tests were rewritten, not weakened. The old AC-43 test asserted that
a record cycle refuses. That clause is amended, so the test now asserts the amended clause:
record-free cycles refuse, and a record cycle admits. `tc_048_leaf_count_is_settled_before_leaf_modes`
now uses an option-of-itself cycle for its "ineligible before mode" case. The record cycle
it used before now admits, so it can no longer show the ordering. Its assertion is the
same, ineligible before mode-type mismatch.

Trace tags are TC-048 with FR-038-AC-43, AC-70, AC-71 or AC-72, and each matches the clause
its test asserts. None are invented. Strict coverage, measured: origin/main FR-038 42/70,
total 163/212, 23 unbacked; head FR-038 45/70, total 166/212, 23 unbacked.

Test-oracle note. The 12-ring test bisects the work a read used, then checks that this
value admits and one less is `incomplete`. Any monotone reader passes, including one that
charges a cycle nothing. That is how AC-72 is worded, and the absolute per-reentry charge is
pinned separately by `tc_048_each_reentry_edge_is_one_unit_of_work` (5 units per extra
reentry field).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The component search (`LeafWalk::analyse`/`discover`) charges one work unit per distinct reachable type node, on top of the per-edge charges the spec lists. FR-038's cost paragraph names only node visits and reentry edges, with no charge for the recursion leaf beyond its edge. It does not name this charge. No test pins it: removing the charge leaves all 100 model-lib and 183 it tests passing. This does not contradict AC-72: the 12-ring bound is relative and the ten-record case is still `incomplete`/work. But the reader's work model goes past the spec's stated model, unseen. Pin it with a test (for example, a node-count delta in the work-unit test), and have the spec lane name it in one clause of the cost paragraph | crates/quire-contract-model/src/checked_package/v2/operations.rs:2136-2162, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:897-903 |

## Verdict

Every clause of AC-43, AC-44, AC-70, AC-71 and AC-72, and every step of the TC-048
procedure, has a test that can fail. The requested mutation probes were each killed (see
SR-800). The one finding is low. "Closes IR-486" is justified: the reopened scope (admit
recursive compared types, and require the recursion leaf where text is reachable) and every
spec AC are met. The CG tc_029 flip and QSL FR-093-AC-19 are external follow-ups, not
blockers.

## Dispositions

Round 1 at e44305c9dd30c38062d7bac75af5c2a7e2c223c0. FR-038's cost paragraph now names the one-unit-per-reachable-type-node component-search charge on the same meter. That is a spec clarification found in review. It is consistent with AC-72: the 12-ring exact-work bound is relative to the work a read used, and the ten-records case is still incomplete/work. The new test tc_048_the_component_search_costs_one_unit_per_reachable_type_node (TC-048, FR-038-AC-72) is pinned. Mutation probe, removing the charge in discover: that test fails (left 2, right 3), and the probe was reverted. No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed e44305c9dd30c38062d7bac75af5c2a7e2c223c0 |
