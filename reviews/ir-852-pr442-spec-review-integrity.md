---
id: SR-12163
title: "IR-852 specification integrity review of IR PR #442"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@6f0452297d487e0adf8b93a5f53e1e6d4b984e4c; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Typed population reader rule and FR-038-AC-269..280), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-852 specification integrity review of IR PR #442

## Summary

Ticket: IR-852. The review examined the amended maximum rule, AC-269 through AC-280, their TC-048 procedure, tests.md status rows, and the FR-019 public item entry for consistency, completeness, and testability. The maximum amendment reflects the reader's earlier body-term refusal, but its pointer and cross-authority comparison need correction.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038 Typed population reader rule | examined | A non-binding `max` is refused earlier as `invalid_semantic_graph` at the member; FR-322's different derived-shape refusal is discussed below it. |
| FR-038-AC-269..278, AC-280 | examined | Status changed to Implemented; the requirement text and verification methods otherwise retain their previously reviewed form. |
| FR-038-AC-279 | examined | The five maximum defects reach `member.population`; a non-binding `max` is refused by the graph reader at `body/members/0`; maximum 0 and a decimal-string maximum admit. |
| TC-048 Typed population section | examined | Repeats the earlier non-binding refusal and names the published QSpec vectors as a conformance obligation. |
| tests.md FR-038 rows | examined | AC-269 through AC-280 are marked Implemented with TC-048 tests. |
| FR-019 Public items row | examined | Adds `CheckedReachesPopulationError` as the public type exported for FR-038. |
| QSpec FR-322 Population link and AC-73 | context_only | The upstream population link and published adverse vectors provide the cross-authority comparison. |

## Verdict

**CONDITIONAL** — the amended rule is implementable, with one low-severity consistency defect.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The amended rule and AC-279 fix the earlier non-binding `max` refusal at `body/members/0`, but 0 is only the test fixture's member index; the reader reports the offending member's actual index. Use `members/M`. The following comparison to FR-322 says a non-binding `max` is FR-322's absent case, yet the list of this reader's different outcomes omits its new `invalid_semantic_graph` refusal at the member. State that difference alongside the other code, pointer, and stage differences. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3423 |

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
| FR-038-AC-277 | examined | Implemented (IR-852). When `reaches_population` is called twice on an admitted reaches application and on a clone of the admitted package, the results are equal, the package equals its pre-call self and no work is charged; every package of ... |
| FR-038-AC-278 | examined | Implemented (IR-852). An external Rust API fixture calls `reaches_population` with `&CheckedNodeId`, binds the result as `Result<Option<CheckedSelectedPopulationSelector>, CheckedReachesPopulationError>` and exhaustively matches `CheckedRea... |
| FR-038-AC-279 | examined | Implemented (IR-852). A reaches application whose `operation.member.population` names a `model_population` node over the receiver end's declared type with a missing `max`, two `max` members, a `max` that is not an integer `literal`, a negat... |
| FR-038-AC-280 | examined | Implemented (IR-852). A reaches application with an invalid `population` (a node id absent from the graph, listed in its `dependencies`) and an earlier defect in `check_relationship_member` (an operand 1 that does not conform to the receive... |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in 2b798f2; see the after excerpt in the Linear disposition. |
