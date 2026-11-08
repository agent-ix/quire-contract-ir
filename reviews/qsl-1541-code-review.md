---
id: SR-958001
title: "code-review of QSL-1541/QSL-1542 guarantee and composition reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs, crates/quire-contract-model/src/checked_package/v2/guarantee_terms.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/typed_terms.rs, shared v2 match arms"
review_set: subset
---

## Summary

Code-review (with the rust-review lane) of quire-contract-ir PR #958 at e2553549 against origin/main 22b659ef, judged against QSpec origin/main 7e63321f (FR-490 and FR-491 are absent from the local QSpec checkout at b09800f7). Ticket QSL-1541, with QSL-1542. Interop judged against QSL PR #2549 (branch qsl-1541-1542-planner) node writers.

Focused runs: `cargo test -p quire-contract-model --lib tc_225_exactly_one_kind` FAILS; the TC-474 integration tests were run separately.

## Examined scope

- FR-038 "Implementation guarantee and component composition bodies" (examined): the new reader section and FR-038-AC-531..538.
- QSpec FR-490 "Checked-package node" (examined) and QSpec FR-491 "Checked-package node" (examined).
- FR-346-AC-2 (examined): exactly one kind has the abstraction relation body.

## Verdict

FAIL. The typed terms, encode and node-id remapping are complete and consistent, and the shared-file match arms add no behaviour. But the PR breaks an existing unit test, refuses every composition whose components live in other packages, and disagrees with QSL's writer on guarantee dependencies and on supplies order.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | body_grammar now maps three kinds to BodyGrammar::AbstractionRelation; the existing FR-346-AC-2 unit test tc_225_exactly_one_kind_has_an_abstraction_relation_body fails | crates/quire-contract-model/src/checked_package/v2/mod.rs:2825-2829, 5052-5063 |
| FND-002 | high | A composition whose subject or clause targets are nodes of another package is refused invalid_semantic_graph at dependencies[i] by the graph stage (a dependency not in the local index), before the guarantee step; QSpec FR-491 resolves them through FR-322 dependency packages. Tests use only local subjects | crates/quire-contract-model/src/checked_package/v2/mod.rs:3732-3760 |
| FND-003 | high | The reader requires a guarantee's fairness environment context in dependencies; QSL's ImplementationGuaranteeTerm::targets excludes it, so a QSL guarantee with a fairness premise whose context is not also a progress context refuses at the node | crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:89-108 |
| FND-004 | medium | supplies order compares kind and granularity by enum discriminant (weak<strong, whole<each); QSL sorts by spelling bytes (strong<weak, each<whole), so two supplies on one operation emitted by QSL refuse out of order | crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:556-565 |
| FND-005 | medium | A composition's environment entries get none of FR-490's checks: no assumption-is-predicate target check and no duplicate-name conflicting-binding | crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:573-631 |
| FND-006 | medium | A guarantee's progress and fairness operations are resolved against their own context, not checked to be operations of the clause's model subject (QSpec FR-490 check 4) | crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:472-491 |


## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | Fairness kind/granularity spellings have duplicate implementations in validation ordering and typed encoding; reuse one conversion so the two cannot drift | crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:141-153; crates/quire-contract-model/src/checked_package/v2/guarantee_terms.rs:110-122 |


## Disposition scope

One replacement reviewer performed disposition pass 1 against the frozen final candidate. Original findings above are preserved verbatim. No applicable AssuranceProfile exists in spec; ordinary Rust/code review applies. No CI workflow edits or gate reductions occur.

```yaml
scope:
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: When the strict V2 reader reads a `correspondence` node of `semantic_form`
    `implementation_guarantee` or `component_composition`, it SHALL admit the body
    exactly as QSpec FR-490 and FR-491 "Checked-package node" define it, as FR-346
    admits the `abstraction_relation` body, and expose it as a typed term.
- id: FR-038-AC-531
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package holding one `correspondence`/`implementation_guarantee` node
    over a clause selecting the infinite-trace profile, an `abstraction_relation`
    node, a `predicate` node and the `Order` object type admits for a body with `initial`,
    a `decreases` and a `bound` progress member and a fairness and an assumption environment
    member, and the typed term reproduces the body; a body with empty `progress` and
    `environment` lists admits.
- id: FR-038-AC-532
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package holding one `correspondence`/`component_composition` node admits
    for a body with two components, two guarantees, one `relies`, one `liveness_dependencies`,
    one `supplies` and one `environment` entry, the typed term keeps the premise lists
    separate, and each of the four rule spellings admits.
- id: FR-038-AC-533
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A missing or extra body member, a `term` of the other form, an empty `initial`
    path, a measure of two members, an unknown fairness kind, a `supplies` of neither
    form, an unknown composition rule and a component without its package each refuse
    `invalid_semantic_graph` at the offending entry or the body; a malformed structural
    node key refuses `malformed_wire` at the key; an `application` at the body root
    refuses `ill_typed`/`operator-ineligible` at the node.
- id: FR-038-AC-534
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For each form, a `semantic_type` that is not the node, a `dependencies`
    list that omits a body target and one that adds a target the body does not hold
    each refuse `invalid_semantic_graph` at the node.
- id: FR-038-AC-535
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An out-of-order `progress`, `environment`, `components`, `guarantees`,
    `relies`, `liveness_dependencies` or `supplies` array refuses `invalid_semantic_graph`
    at that array.
- id: FR-038-AC-536
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A guarantee whose `clause` is no temporal clause or selects another profile,
    whose `relation` is no abstraction relation, whose `progress` or fairness `context`
    is no object type or whose `assumption` is no predicate refuses `invalid_model_binding`/`malformed-declaration`
    at the member.
- id: FR-038-AC-537
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A guarantee whose `progress` or fairness operation its context does not
    declare refuses `missing_declaration`/`missing-name`; an operation the context
    only inherits, a non-Rust `initial` or `decreases` path and `bound` 0 refuse `invalid_model_binding`/`malformed-declaration`;
    two `progress` members for one operation and two `environment` members of one
    name refuse `invalid_model_binding`/`conflicting-binding` at the second.
- id: FR-038-AC-538
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A composition with two components of one name or over one subject refuses
    `invalid_model_binding`/`conflicting-binding` at the second; a guarantee or premise
    naming an undeclared component refuses `missing_declaration`/`missing-name`; a
    premise whose guarantee is undeclared, whose two sides name one component, a rely
    on liveness or a depends with a non-liveness side refuses `invalid_model_binding`/`malformed-declaration`;
    component subjects and clauses resolve through admitted dependency packages, clauses
    must belong to the component model subject, scheduling operations must belong
    to a compon
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: FlatCorrespondence now distinguishes these forms from AbstractionRelation; existing exactly-one-kind regression passes. |
| FND-002 | fixed | Round 1: Foreign component targets are joined through admitted selected dependency packages; shared graph exception is restricted to ComponentComposition plus holds_node and later exact target validation. |
| FND-003 | rejected | Round 1: The reader requirement is correct: QSpec FR-490 explicitly includes every context target, including scheduling environment contexts. The QSL writer omission is owned by QSL-1541/QSL PR2549 and has been routed to its planner; weakening this reader would violate FR-490. |
| FND-004 | fixed | Round 1: Canonical ordering now compares kind and granularity spelling bytes, shared with the encoding convention. |
| FND-005 | fixed | Round 1: Composition environments validate predicate targets before member checks and reject repeated names. |
| FND-006 | fixed | Round 1: Operation resolution first compares the context model identity to the temporal clause model subject, including reference-type recovery. |
| FND-007 | still-open | Round 1: New duplicate kind_spelling/granularity_spelling mappings exist in guarantee_composition.rs and guarantee_terms.rs; reuse one authoritative conversion for encoding and order. |
| FND-007 | fixed | Round 2: guarantee_composition imports the single kind/granularity spelling conversions from guarantee_terms; no duplicated mapping remains. |


## Disposition verdict

FAIL: original reader findings resolved or rejected on owning-spec evidence, but new FND-007 remains open; PR is not mergeable.

## Disposition validation

Reviewed real test setups and exact refusal oracles; no mocks or admission bypasses. Coder log /tmp/qsl1541-fix-focused4.log records 15 passing TC474 tests, the exactly-one-abstraction-kind regression and warnings-denied workspace checks. Scoped Quire validation exits 0; its sole grammar warning concerns pre-existing criterion outside this review. No aggregate gate was run. Matrix output: /tmp/titan-codex2-958-matrix.json.

## Disposition pass 2 verdict

PASS. The shared spelling conversions now serve both canonical ordering and typed encoding. Inspected the entire fix diff and found no new regression. All findings have a latest outcome of fixed or rejected; PR #958 is mergeable from review, subject to the lead’s final aggregate gate and merge policy. Historical round 1 findings, outcomes and verdict remain unchanged above. Coder log /tmp/qsl1541-fix-round2.log records 15 passing TC474 tests, formatter and Quire validation; no unaffected checks were repeated by this reviewer.
