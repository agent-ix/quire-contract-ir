---
id: SR-958002
title: "gap-analysis of QSL-1541/QSL-1542 guarantee and composition reader"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-474-implementation-guarantee-and-component-composition-bodies.md, tests/it/checked_package_v2_guarantee_composition.rs"
review_set: subset
---

## Summary

Manual AC-to-test gap check (Quoin gap-analysis not run as a full matrix pass; diff-scoped) of PR #958 at e2553549. Ticket QSL-1541, with QSL-1542. Plan completion: not assessed.

## Examined scope

- FR-038-AC-531..538 (examined), each bound to one TC-first `#[trace("TC-474", ...)]` test.
- QSpec FR-491 static checks 2 to 5 and its reader paragraph (examined).

## Verdict

FAIL on coverage of refusal paths and on an undeferred QSpec gap. Trace tags are TC-first and each AC has one test; admission cases are covered.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | QSpec FR-491 static checks 4 and 5, and check 2's clause-of-component lookup, are not made; FR-038 and TC-474 disclose this but no follow-up ticket is named | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5776-5786 |
| FND-002 | medium | Untested refusals: out-of-order supplies, out-of-order composition environment, a malformed node key (malformed_wire), and a cross-package composition target; FR-038-AC-535 omits supplies although FR-038 step 3 orders it | tests/it/checked_package_v2_guarantee_composition.rs:496-566 |
| FND-003 | low | FR-038 prose says the reader joins composition subject and clause targets of component packages to dependencies; the graph stage refuses them (SR-958001 FND-002), so the text claims behaviour the code lacks | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5776-5781 |


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
| FND-001 | fixed | Round 1: Reader now checks clause model subject, polarity-aware FR-363 safety/liveness classification, and scheduling operations through admitted component model declarations. |
| FND-002 | fixed | Round 1: Out-of-order supplies/environment, malformed structural body key, and cross-package target cases now assert the reader refusal code, cause and pointer; AC535 includes supplies. |
| FND-003 | fixed | Round 1: FR038 and TC474 now describe implemented foreign package resolution with no deferred static checks. |

## Disposition verdict

PASS for examined FR038 AC531..538. Plan completion: not assessed. All original gaps are fixed; computed matrix marks all eight criteria tagged by nonignored tests.

## Disposition validation

Reviewed real test setups and exact refusal oracles; no mocks or admission bypasses. Coder log /tmp/qsl1541-fix-focused4.log records 15 passing TC474 tests, the exactly-one-abstraction-kind regression and warnings-denied workspace checks. Scoped Quire validation exits 0; its sole grammar warning concerns pre-existing criterion outside this review. No aggregate gate was run. Matrix output: /tmp/titan-codex2-958-matrix.json.
