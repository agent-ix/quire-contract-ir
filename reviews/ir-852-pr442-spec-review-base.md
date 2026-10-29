---
id: SR-12162
title: "IR-852 spec review (base) of the spec changes in IR PR #442"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Typed population of a reaches application: the amended reader-check max rule; AC-269..280 rows), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md (Typed population section), spec/checked_package/matrix/tests.md (FR-038 row, AC-269..280 rows), spec/model/functional/FR-019-rust-library-interface.md (Public items row)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
---

# IR-852 spec review (base) of the spec changes in IR PR #442

## Summary

Ticket: IR-852. This code PR makes four kinds of spec change:

- It flips FR-038-AC-269..280 from PLANNED / UNRUN to Implemented, in FR-038, tests.md and the TC-048 section.
- It adds the FR-019 Public items row for `CheckedReachesPopulationError`.
- It amends one rule. A `max` member that is not a `binding` is no longer listed among the `max` defects the population check refuses `ill_typed`/`operator-ineligible`. The amended text says the graph reader refuses it earlier as `invalid_semantic_graph` at the member. AC-279 and its tests.md row become "each of the five", plus that earlier refusal.

The authority checked against is FR-322 "Population link of a `reaches`" and FR-322-AC-73 in quire-specification origin/main, which is read-only here.

**The amended rule is true.** `tc_048_a_population_whose_max_selection_cannot_read_is_refused` builds a population node whose only member is `{"term": "literal", "name": "max"...}`, keyed over that body by `structural_key`, so the refusal cannot come from a stale key. It asserts `invalid_semantic_graph`, no cause, at `/semantic_graph/nodes/{at}/body/members/0`, and passes. Body terms have closed member sets, and only `binding` carries `name` (`common.rs:1037`). So every `max`-named member that is not a binding is refused by the term grammar, before the operation stage. The population check's binding test therefore cannot fail on an admitted package. That makes the original AC-279 clause untestable as written, and the amendment is the right correction. The test matches the amended text.

**Status rows match the implemented local behavior.** Each of AC-269..280 has at least one test carrying its `#[trace("TC-048"...)]`: one each, and two each for AC-271 and AC-275. The focused review run passed all thirteen integration tests and the crate unit test. `quire coverage --strict` reports 43 unbacked and 0 contradicted on this branch; the four extra unbacked rows against current main are AC-193, AC-194, AC-195, and AC-201, whose Verification cells were updated by IR-861 after this branch forked. The TC-048 QSpec-vector step and the AC-275 member-kind test gap are recorded in gap-analysis SR-12161. The FR-019 row is exact by source and export comparison.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038 reader check, max rule (amended) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | When the node's `max` is one that FR-053's `population_maximum` rejects (the member is missing or duplicated, is not an integer `literal`, or the value is negative or exceeds `u64`), the reader shall refuse `ill_typed`/`operator-ineligible`. A `max` member that is not a `binding` is not a valid body term, so the graph reader refuses it earlier as `invalid_semantic_graph` at the member (`/semantic_graph/nodes/N/body/members/0`) and it never reaches this check. |
| FR-038-AC-279 (amended) | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md |... is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, for each of the five...; a population node whose `max` member is not a `binding` is refused earlier by the graph reader as `invalid_semantic_graph` at `/semantic_graph/nodes/N/body/members/0` and never reaches the population check; a `max` of 0 and a `max` given as a decimal string are admitted and yield `Finite` |
| FR-038-AC-269..278, AC-280 status flips | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Implemented (IR-852). (criterion text unchanged) |
| TC-048 "Typed population of a reaches application" status and max sentence | examined | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md | Implemented (IR-852); tagged tests.... Refuse each `max` defect of AC-279 (missing, two `max` members, non-integer literal, negative, above `u64`); a non-binding `max` member is refused earlier as `invalid_semantic_graph` at the member |
| tests.md FR-038 row and AC-269..280 rows | examined | spec/checked_package/matrix/tests.md | ✅ AC-269 through AC-280 (IR-852) typed reaches population accessor, `population` member key reader check and dependency join are implemented with tagged TC-048 tests. |
| FR-019 Public items row | examined | spec/model/functional/FR-019-rust-library-interface.md | `checked_package` (V2 reaches population) \| type `CheckedReachesPopulationError` \| FR-038 |
| FR-322 "Population link of a `reaches`" | context_only | quire-specification:spec/objects/interfaces/FR-322-checked-package-artifact.md | The node's `max` is read as for any `model_population` node and a negative `max` refuses as it does there; `max` `0` is admitted. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS** — no base-checklist findings. The amended text is true in the tested fixture. The criterion count ("each of the five") and the TC-048 and tests.md wording agree with FR-038. The PLANNED-to-Implemented flips cover exactly AC-269..280, and each is backed by a passing tagged test. No other criterion's status moved. The wording consistency finding is in integrity review SR-12163.

Merge note: origin/main (IR-861, #441) rewrote FR-038 Verification cells next to these rows, and `git merge-tree` reports a content conflict in FR-038. The flips and the AC-279 amendment must be re-applied on top of IR-861 when the branch is rebased.

## Acceptance criteria examined

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-269 | examined | Implemented (IR-852). When `reaches_population` is called with the id of an admitted `quire.op.model.reaches` application whose `operation.member` carries `population` naming a `bounded_domain` / `model_population` node of the receiver end'... |
| FR-038-AC-270 | examined | Implemented (IR-852). When `reaches_population` is called with an admitted reaches application whose `operation.member` has no `population` key it returns `Ok(None)`, also in a package that holds a `model_population` node of the receiver en... |
| FR-038-AC-271 | examined | Implemented (IR-852). `reaches_population` over an id absent from the graph refuses `UnknownNode`; over an admitted `quire.op.model.navigate` application, an application of `quire.op.integer.add`, the relationship declaration node, a `model... |
| FR-038-AC-272 | examined | Implemented (IR-852). A reaches application whose `operation.member.population` is not a node id (a number, an object and `null`), or is a node id with no node in the graph and listed in `dependencies`, is refused by the reader with `ill_ty... |
| FR-038-AC-273 | examined | Implemented (IR-852). A reaches application whose `operation.member.population` names a node that is not a `bounded_domain` / `model_population` node, or a `model_population` node with no `Type` occurrence, is refused by the reader with `il... |
| FR-038-AC-274 | examined | Implemented (IR-852). A reaches application whose `operation.member.population` names a `model_population` node over an object type other than the declared type of the selected receiver end, including a subtype of that type, is refused by t... |
| FR-038-AC-275 | examined | Implemented (IR-852). A `quire.op.model.navigate` application whose `operation.member` carries a `population` key that names a node id its `dependencies` list, whatever the node is (an absent node, an integer-range node or a valid populatio... |
| FR-038-AC-276 | examined | Implemented (IR-852). A reaches or navigate application whose `operation.member.population` is a node id is admitted by the dependency join only when its `dependencies` hold that id in the unique, digest-ascending list; a package that omits... |
| FR-038-AC-277 | examined | Implemented (IR-852). When `reaches_population` is called twice on an admitted reaches application and on a clone of the admitted package, the results are equal, the package equals its pre-call self and no work is charged; every package of... |
| FR-038-AC-278 | examined | Implemented (IR-852). An external Rust API fixture calls `reaches_population` with `&CheckedNodeId`, binds the result as `Result<Option<CheckedSelectedPopulationSelector>, CheckedReachesPopulationError>` and exhaustively matches `CheckedRea... |
| FR-038-AC-279 | examined | Implemented (IR-852). A reaches application whose `operation.member.population` names a `model_population` node over the receiver end's declared type with a missing `max`, two `max` members, a `max` that is not an integer `literal`, a negat... |
| FR-038-AC-280 | examined | Implemented (IR-852). A reaches application with an invalid `population` (a node id absent from the graph, listed in its `dependencies`) and an earlier defect in `check_relationship_member` (an operand 1 that does not conform to the receive... |
