---
id: SR-12161
title: "IR-852 gap analysis of IR PR #442"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@6f0452297d487e0adf8b93a5f53e1e6d4b984e4c; FR-038-AC-269..280, TC-048, tests/it/checked_package_v2_model_members/reaches_population.rs, tests/conformance_qspec/main.rs, crates/quire-contract-model/src/checked_package/{selected_population.rs,v2/reaches_population.rs,v2/operations.rs,v2/structural.rs,v2/relationship_end.rs}"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-852 gap analysis of IR PR #442

## Summary

Ticket: IR-852. This planless audit compared FR-038-AC-269 through AC-280, the computed matrix, the tagged tests, and the changed reader and accessor code. Plan completion: not assessed. All twelve criteria have test binders and the implementation paths are exercised, but two required test families are absent.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-269..274 | examined | Selector identity and least Type ordinal; missing key; typed accessor refusals; malformed, unresolved, non-population, and wrong-object-type links. |
| FR-038-AC-275 | examined | Navigate must refuse a population link; every other member kind keeps its existing outcome for that key. |
| FR-038-AC-276..280 | examined | Dependency join, purity and regression, public API shape, maximum boundaries, and refusal precedence. |
| TC-048 Typed population | examined | Published QSpec reaches_population_cases must run through make conformance-qspec with recorded outcomes. |
| QSpec FR-322-AC-73 and adverse.json | context_only | Seven published reaches_population_cases describe the positive and adverse producer vectors. |
| Changed production and test files in scope | examined | Reader population check, structural join, selected population helpers, both accessors, thirteen integration tests, and one crate unit test. |

## Verdict

**CONDITIONAL** — every new criterion is tagged, but two acceptance test obligations lack direct evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-048 requires every published QSpec `reaches_population_cases` descriptor to be built into an IR package and run through `make conformance-qspec`. The seven cases now exist in QSpec `adverse.json`, but `tests/conformance_qspec/main.rs` never reads that key. The local handcrafted tests cannot catch a mismatch between QSpec's published descriptors and the IR adapter, so this producer contract remains unverified. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2380; tests/conformance_qspec/main.rs:938 |
| FND-002 | medium | AC-275 names the behavior of `temporal_interval`, `fairness`, `position`, and `element` members carrying a `population` key, but its tagged test exercises only `type_argument`, `state_clause`, `field`, and `operation`. A change that rejects or admits one of the omitted member kinds contrary to AC-275 would leave the tagged test green. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4831; tests/it/checked_package_v2_model_members/reaches_population.rs:568 |

## Coverage

- Reconciliation: `quire coverage --scope . --strict --json` on PR head and current `origin/main`; the strict command exits 1 for existing unbacked rows.
- PR head: 527/584 backed, 43 unbacked, 0 contradicted. Current `origin/main`: 515/584 backed, 39 unbacked, 0 contradicted. The four additional unbacked rows are FR-038-AC-193, AC-194, AC-195, and AC-201: this branch predates IR-861, which added TC-048 to those four Verification cells on main. They are unrelated to AC-269..280; rebasing the conflicting FR-038 file restores the newer cells.
- FR-038-AC-269 through AC-280: 12/12 tagged in the computed matrix; 13 integration tests and one crate unit test passed in the focused review run.
- Plan completion: not assessed
- Reverse gap: each new production path is owned by FR-038 or FR-053; no new production stub was found.
- Semantic review: ran over FR-038-AC-269 through AC-280 and TC-048's typed population section.

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
| FND-002 | fixed | Verified in 2b798f2; see the after excerpt in the Linear disposition. |
