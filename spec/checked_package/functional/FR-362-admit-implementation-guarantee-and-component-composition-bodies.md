---
id: FR-362
title: "Admit the V2 implementation guarantee and component composition bodies"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-346
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-490
    type: references
  - target: ix://agent-ix/quire-specification/FR-491
    type: references
  - target: ix://agent-ix/quire-specification/FR-492
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---
# FR-362: Admit the V2 implementation guarantee and component composition bodies

## Description

When the `quire.checked-package/v2` reader ([FR-038](./FR-038-consume-checked-package-v2.md))
reads a `correspondence` node of `semantic_form` `implementation_guarantee` or
`component_composition`, the reader shall admit that node's body as QSpec FR-490
and FR-491 define it, retain it as a typed term, and refuse a body whose joins
do not hold, in the order below. The QSpec text and the published
`proposals/checked-package-v2/schema.json` (`ImplementationGuaranteeBody`,
`ComponentCompositionBody`) are normative. Where they fix no code, cause or
path, this requirement states the reading Contract IR takes and marks it "IR
reading". The reader reads the declaration from the node alone and reads nothing
from QSL. Settling the declaration's premise slots is the planner's (QSpec
FR-492), not the reader's.

## Inputs

Untrusted V2 package bytes and the evidence FR-038 already takes.

## Outputs

An admitted package whose guarantee and composition nodes expose their typed
bodies (`CheckedTerm::ImplementationGuarantee`, `CheckedTerm::ComponentComposition`),
or one typed refusal with its code, cause and RFC 6901 path. No partial package.

## Behavior

### Form gate

The `correspondence` form enum adds `implementation_guarantee` and
`component_composition`, matched exhaustively with no catch-all arm. Both bodies
are flat, as the abstraction relation body is (FR-346): node keys, identifiers,
Rust paths and strings, with no application, aggregate, binding or literal term.

### Body shape

An `implementation_guarantee` body is `{term, clause, relation, initial,
progress, environment}`: `initial` a `RustPath`; a `progress` entry
`{context, operation, measure}` with `measure` exactly `{decreases: RustPath}`
or `{bound: n}`, `n` from 1; an `environment` entry `{name, supplier, supplies}`
with `supplies` exactly `{fairness: {kind, granularity, context, operation}}` or
`{assumption: NodeRef}`, `kind` `weak` or `strong` and `granularity` `whole` or
`each`. A `component_composition` body is `{term, rule, components, guarantees,
relies, liveness_dependencies, supplies, environment}`: `rule` one of
`assume-guarantee`, `protocol-refinement`, `refinement-transfer`, `interface`;
`components` entries `{name, package, subject}`, at least two; `guarantees`
entries `{component, clause}`; `relies` and `liveness_dependencies` entries
`{guarantee: {component, clause}, on: {component, clause}}`, kept as separate
lists; `supplies` the scheduling entries above. Every member is required and
every shape is closed. A body, entry or member off its shape refuses
`invalid_semantic_graph` with no cause at the body (IR reading).

### Reader order

The declaration step runs after the abstraction step, over every
`implementation_guarantee` node and then every `component_composition` node, in
ascending structural node-id order. For one node the first failing check is its
defect:

1. **Identity.** `semantic_type` is the node itself and `dependencies` are
   exactly the unique, ascending set of body targets that are nodes of this
   graph; else `invalid_semantic_graph` at the node. IR reading: a composition
   target that is no node of this graph is a component's own, in another
   package, is checked only for well-formedness, and is not listed.
2. **Order.** `progress` by `context` then `operation`; `environment` by `name`;
   `components` by `name`; `guarantees` by `component` then `clause`; `relies`
   and `liveness_dependencies` by the first member's `component` and `clause`,
   then the second's; else `invalid_semantic_graph` at the array. The order is
   non-strict. IR reading: QSpec FR-491 states no order for `supplies`, so it is
   not checked.
3. **Targets,** in body order, each joined to a declared dependency of its kind,
   else `missing_declaration`/`missing-name`, or `invalid_model_binding`/
   `malformed-declaration` for another kind, at the entry: `clause` a
   `temporal`/`temporal_clause` node, and for a guarantee one selecting the
   infinite-trace profile; `relation` an `abstraction_relation` node; a `context`
   a `model`/`object_type` node; an `assumption` a `function`/`predicate` node; a
   component `subject` a `model` node.
4. **Members,** in body order: a `progress` or scheduling `operation` that is
   not an operation the context declares, `missing_declaration`/`missing-name`
   (the operation resolution of FR-346); a Rust path that is not of FR-450's
   syntax, `invalid_model_binding`/`malformed-declaration`; two `progress`
   entries for one operation, two `environment` entries of one name, or two
   components of one name or one subject, `invalid_model_binding`/
   `conflicting-binding` at the second entry.

The source-level checks of QSpec FR-490 and FR-491 that need the unit's
temporal-clause classification (a `rely` on a liveness guarantee, a `depends` on
a safety guarantee, a mutual `depends` cycle) belong to the compiler, which
refuses them before it emits the node; the reader does not classify clauses.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-362-AC-1 | A package holding an `implementation_guarantee` node over an infinite-trace temporal clause and an abstraction relation, with two `progress` entries (one `bound`, one `decreases`) and one scheduling `environment` entry, admits; the node exposes the typed body, which serializes back to the wire body. | Test (TC-474) |
| FR-362-AC-2 | A `component_composition` node with two components (one in another package), two guarantees, a `liveness_dependencies` entry and a `supplies` entry admits; the typed body exposes the four premise lists separately and serializes back to the wire body. | Test (TC-474) |
| FR-362-AC-3 | A guarantee body with an extra member, no `initial`, an empty `initial` path, `bound` 0, a measure carrying both members, a non-identifier operation, an empty supplier or a `term` of the other form, and a composition body with one component, an unknown rule or no `supplies`, each refuse `invalid_semantic_graph` at the body. | Test (TC-474) |
| FR-362-AC-4 | A node whose `dependencies` omit a body target refuses `invalid_semantic_graph` at the node; reversed `progress`, `environment` and `components` refuse `invalid_semantic_graph` at their array. | Test (TC-474) |
| FR-362-AC-5 | A `relation` naming an object type, a `clause` naming the abstraction relation, a `progress` context naming the temporal clause, an `assumption` naming an object type and a composition `clause` naming an object type each refuse `invalid_model_binding`/`malformed-declaration` at the entry. | Test (TC-474) |
| FR-362-AC-6 | A `progress` operation the context does not declare refuses `missing_declaration`/`missing-name`; an `initial` path with the segment `crate` refuses `invalid_model_binding`/`malformed-declaration`; two `progress` entries for one operation, two `environment` entries of one name and two components of one name each refuse `invalid_model_binding`/`conflicting-binding` at the second; a `progress` context that only inherits the operation refuses `invalid_model_binding`/`malformed-declaration`. | Test (TC-474) |

## Dependencies

[FR-038](./FR-038-consume-checked-package-v2.md) owns the reader, its refusal
codes and pointers, the closed vocabularies and the flat wire this requirement
reuses; [FR-346](./FR-346-admit-abstraction-relation-body.md) owns the
abstraction step this step follows and the operation resolution it shares.
QSpec FR-490 and FR-491 own the normative bodies, FR-492 the settlement the
planner performs over them, and FR-322 the graph and structural node ids. Open
QSpec questions: FR-491 lists cross-package targets among `dependencies`, which
FR-322 resolves only within the graph, and states no order for `supplies`.
