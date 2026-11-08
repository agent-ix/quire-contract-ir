---
id: SR-202611070951309496
title: "base review of QSL-1541 guarantee and composition reader specification"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-474-implementation-guarantee-and-component-composition-bodies.md"
review_set: subset
---

## Summary

The new guarantee/composition reader section and AC531..538 define concrete admission, ordering, reference joins and refusal results; TC474 and the computed matrix bind every added AC. No applicable AssuranceProfile was found and no structural change warrants an integrity lens.

## Examined scope

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

## Verdict

PASS within this selected diff scope. No enforced policy applies; this is a direct review, with base and EARS selected by the dispatching lead. No new source-tracking, pin, digest or provenance mechanism is introduced. The existing repo TC-first trace convention is retained.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Validation

Quire scoped validation of FR038 and TC474 exited 0. The only advisory grammar warning is an unchanged AC outside the selected section. Computed matrix statuses for FR038 AC531..538 are all tagged; 15 nonignored integration functions carry those eight IDs. SpecReview authoring pack was resolved through quoin write.
