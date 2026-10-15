---
id: FR-053
title: "Select a source-backed finite population binding"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: traces_to
  - target: ix://agent-ix/quire-spec-language/FR-094
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-084
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/TC-453
    type: references
---
# FR-053: Select a source-backed finite population binding

## Description

When a graph consumer selects an input population from an admitted checked
package, `quire-contract-model` SHALL expose the finite maximum only from the
exact checked population binding and source occurrence selected by that
consumer. The result SHALL retain the binding's typed identity, selected
object-type identity, inclusive extent, and admitted source provenance.
`model_object_fields` remains the sole authority for model edge-field
multiplicity.

## Inputs

One typed source selection and, when that selection is admitted, one selector.
An admitted selector names the exact `bounded_domain` / `model_population`
node identity and its `CheckedOccurrenceRole::Type` occurrence,
including the ordinal. This node identity is the selected bounded-population
binding identity. The query does not accept a parameter node as a substitute
for that identity. A declaration-only selection names a selected
domain-package PopulationRecord by its typed identity. Neither selection
names an object type alone, supplies a bound, or provides source text.

## Outputs

The model crate exposes `selected_population::select_population`,
`CheckedSelectedPopulationSource`, `CheckedSelectedPopulationSelector`,
`CheckedSelectedPopulation`, and `CheckedSelectedPopulationOutcome` in
`selected_population`. A finite result retains the selected package identity,
population binding node identity, occurrence, selected model object type
identity, inclusive extent `[0, N]`, and the authentic source regions
associated with that exact node-occurrence pair. The outcome is typed and
distinguishes `Finite`, `Unbounded`, `UnknownClosure`, `Unavailable`,
`MissingOccurrence`, `StaleBinding`, `ForeignPackage`, and `NotPopulation`.
Non-finite outcomes carry no extent.

`select_population` accepts a source selection that is either an admitted
`CheckedPackageV2` plus selector, a declaration-only population selection, an
upstream typed QSL `UnknownClosure` admission result with its selected model,
population declaration identity, source provenance and typed cause, or no
available selection. The adapter SHALL map QSL's typed
`AdmissionOutcome::UnknownClosure` directly to `UnknownClosure` and retain
those fields unchanged;
it SHALL NOT infer that outcome from a message, an absent V2 node, or an open
resource limit. This path has no admitted V2 package or binding. A declaration
selection with a known population record and no finite maximum returns
`Unbounded`. An omitted selection returns `Unavailable`. For an admitted
package, a selector from another package returns `ForeignPackage`; a
population node identity absent from that package returns `StaleBinding`; an
existing selected node without the exact source-map occurrence returns
`MissingOccurrence`; and an exact occurrence on a node that is not a
population returns `NotPopulation`.

## Behavior

### Source and selection authority

QSL resolves a checked `Population<T>[N]` type occurrence to
`ValueType::Population(Some(N))` and lowers that occurrence to a keyed V2
`bounded_domain` / `model_population` node with maximum `N`. The selector
SHALL name that exact population node and occurrence. The query SHALL verify
that the named node has the admitted `bounded_domain` / `model_population`
shape, read `N` from its own `max` binding, and require a source-map entry for
the same population node and exact occurrence before returning the extent.
It SHALL NOT follow the `semantic_type` pointer of a `value` / `parameter`
node to choose the population node: FR-038 permits that pointer to be
retargeted to another valid population node while the package is re-keyed.
The direct node-occurrence selector keeps that reader-permitted retargeting
from changing the selected source binding. It SHALL retain that exact
occurrence and its source-map regions as provenance. An equal object type
identity, or another population node under the same parameter, is not an
equivalent selector.

The domain-package `PopulationRecord` identifies a population declaration and
its member types and closure, but carries no finite maximum. A declaration-only
selection with a known record returns `Unbounded` when the declared extent is
closed; an open extent or unresolved object/subtype closure is the distinct
`UnknownClosure` input/outcome, carrying the upstream QSL source identity and
provenance and no admitted binding. QSL FR-084 owns that typed
pre-admission outcome. The neutral IR adapter preserves it without creating an
admitted package result.

The query SHALL NOT infer, substitute, or clamp `N` from `Closed` or `Open`
closure, model membership, a Codegen or IR resource limit, fixture size,
observed members, another binding of the same object type, or caller-provided
maximum. Changing only a resource limit SHALL leave the selected extent
unchanged; changing the checked source maximum SHALL change it. The query
does not alter the admitted package, reader decisions, or source map.

`model_object_fields` continues to define each model field's type and edge
multiplicity. This population query reports only a selected binding's finite
extent; it SHALL NOT add field-level multiplicities, rewrite edge bounds, or
introduce a competing graph-edge table.

### Public boundary

The public query and result types SHALL be owned by the cycle-free
`quire-contract-model::selected_population` API, listed in [FR-019](./FR-019-rust-library-interface.md),
and accessible to a default-feature model consumer without source-text
parsing, a QSL dependency, or a Codegen dependency. No item is re-exported from
the `quire-contract-ir` root. The selected population result is a typed
read-only projection and does not itself construct symbolic inputs, graph
edges, or a proof harness. The model-owned source enum and adapter introduce
no QSL dependency; the adapter matches QSL's closed typed admission outcome
and preserves its identity, provenance and cause without parsing diagnostic
text or copying the QSL admission schema.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-053-AC-1 | Through a default-feature public model consumer and an admitted QSL-produced V2 package, selecting the exact keyed `bounded_domain` / `model_population` node and its `Type` occurrence for `Population<T>[N]` returns that node, occurrence, selected `T`, authentic source regions, and inclusive `[0,N]`; `N=0` returns `[0,0]`. Changing only the checked source maximum changes the selected node and returned extent. | Test |
| FR-053-AC-2 | Two population nodes with the same selected object type but distinct source occurrences retain their own maxima and provenance. A valid package that re-points a parameter's `semantic_type` at the other population node and re-derives affected keys still returns the extent of the node-occurrence pair named by the selector; the query never follows that parameter pointer. Selecting the other node with the first occurrence returns `MissingOccurrence` unless that exact pair is source-mapped. Changing only the Codegen or IR resource ceiling or observed fixture membership does not change either extent. | Test |
| FR-053-AC-3 | An absent selector yields `Unavailable`; an absent binding identity yields `StaleBinding`, a present binding without the selected occurrence yields `MissingOccurrence`, a foreign package identity yields `ForeignPackage`, and a present non-population binding yields `NotPopulation`; each has no extent. A selector with the right object type but a different binding or occurrence cannot satisfy the request. | Test |
| FR-053-AC-4 | A selected domain-package PopulationRecord with closed extent but no binding maximum yields `Unbounded`; an open extent or unresolved object/subtype closure maps QSL's typed pre-admission `UnknownClosure` to the distinct typed outcome with no package, binding or finite extent. A consumer can distinguish both from `Unavailable`. Source inspection finds no finite-bound path from closure, resource limits, fixture or member count, or caller-provided maximum. | Test, Inspection |
| FR-053-AC-5 | A compiled default-feature consumer reads the typed result through the public model API from a real QSL-produced admitted V2 package without parsing source text or importing QSL/Codegen; `model_object_fields` remains the only edge-field multiplicity authority, and the root crate re-exports none of the new items. | Test |

## Dependencies

[FR-038](../../checked_package/functional/FR-038-consume-checked-package-v2.md)
owns V2 reader admission, checked identities, and source-map correspondence.
[FR-019](./FR-019-rust-library-interface.md) owns the model crate's public
item inventory. QSL [FR-094](ix://agent-ix/quire-spec-language/FR-094) owns
checked `Population<T>[N]` typing and its V2 lowering; QSL
[FR-084](ix://agent-ix/quire-spec-language/FR-084) owns the typed
pre-admission `UnknownClosure` result. The domain-package `PopulationRecord`
owns declaration identity and closure, not a maximum. IR-268 consumes this
projection for symbolic graph inputs. The projection does not implement that
consumer or claim a Kani proof.
