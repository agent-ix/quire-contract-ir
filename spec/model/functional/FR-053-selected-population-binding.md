---
id: FR-053
title: "Select a source-backed finite population binding"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: traces_to
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

One `CheckedPackageV2` admitted by `CheckedPackageV2::read` and one typed
selector. A checked binding selector names the binding identity and its exact
`CheckedOccurrenceRole::Expression` occurrence, including its ordinal. A
declaration selector names a selected domain-package PopulationRecord by its
typed identity. Neither selector names an object type alone, supplies a bound,
or provides source text.

## Outputs

The model crate exposes `CheckedSelectedPopulationSelector`,
`CheckedSelectedPopulation`, and `CheckedSelectedPopulationOutcome` in
`checked_package`. A finite result
retains the selected package identity, binding node identity, occurrence,
population type node, selected model object type identity, inclusive extent
`[0, N]`, and the authentic source regions associated with that selected
occurrence. The outcome is typed and distinguishes `Finite`, `Unbounded`,
`Unavailable`, `MissingOccurrence`, `StaleBinding`, `ForeignPackage`, and
`NotPopulation`. Non-finite outcomes carry no extent.

`CheckedPackageV2::selected_population` is a read-only query over the admitted
package. Its selector is bound to that package identity. An omitted selection
returns `Unavailable`; an identity from another admitted package returns
`ForeignPackage`. A binding identity absent from the selected package returns
`StaleBinding`; a present binding without the selected source-map occurrence
returns `MissingOccurrence`; a present binding whose selected occurrence is
not a population returns `NotPopulation`. A selected domain-package
population record with no finite maximum returns `Unbounded` and carries no
extent.

## Behavior

### Source and selection authority

QSL resolves a checked `Population<T>[N]` type occurrence to
`ValueType::Population(Some(N))` and lowers that occurrence to the admitted V2
`bounded_domain` / `model_population` shape with maximum `N`. The query SHALL
follow the selected binding and exact occurrence through the admitted V2 graph,
validate that the represented population type is the selected one, and return
the maximum from that source-backed shape. It SHALL retain the selected
occurrence and its source-map regions as provenance. An equal object type
identity on another parameter or occurrence is not an equivalent selector.

The domain-package `PopulationRecord` identifies a population declaration and
its member types and closure, but carries no finite maximum. Selecting such a
record can yield only `Unbounded`; `Closed` does not establish a count. QSL
remains the owner of rejecting a checked source population with no finite
maximum or unknown closure before it emits an admitted V2 package. This query
does not synthesize an admitted result for a source construct that QSL refused.

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
`quire-contract-model::checked_package` API, listed in [FR-019](./FR-019-rust-library-interface.md),
and accessible to a default-feature model consumer without source-text
parsing, a QSL dependency, or a Codegen dependency. No item is re-exported from
the `quire-contract-ir` root. The selected population result is a typed
read-only projection and does not itself construct symbolic inputs, graph
edges, or a proof harness.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-053-AC-1 | Through a default-feature public model consumer and an admitted QSL-produced V2 package, selecting the exact checked population binding and `Expression` occurrence for `Population<T>[N]` returns that binding, occurrence, selected `T`, authentic source regions, and inclusive `[0,N]`; `N=0` returns `[0,0]`. Changing only the checked source maximum changes the extent. | Test |
| FR-053-AC-2 | Two bindings with the same selected object type but distinct source occurrences retain their own maxima and provenance. Selecting one never returns the other's extent. Changing only the Codegen or IR resource ceiling, observed fixture membership, or unrelated model members does not change the selected extent. | Test |
| FR-053-AC-3 | An absent selector yields `Unavailable`; an absent binding identity yields `StaleBinding`, a present binding without the selected occurrence yields `MissingOccurrence`, a foreign package identity yields `ForeignPackage`, and a present non-population binding yields `NotPopulation`; each has no extent. A selector with the right object type but a different binding or occurrence cannot satisfy the request. | Test |
| FR-053-AC-4 | A selected domain-package PopulationRecord with `Closed` or `Open` extent but no binding maximum yields `Unbounded` with no finite extent. Source inspection finds no finite-bound path from closure, resource limits, fixture or member count, or caller-provided maximum. An unbounded or unknown-closure checked source refused by QSL produces no admitted-package finite projection. | Test, Inspection |
| FR-053-AC-5 | A compiled default-feature consumer reads the typed result through the public model API from a real QSL-produced admitted V2 package without parsing source text or importing QSL/Codegen; `model_object_fields` remains the only edge-field multiplicity authority, and the root crate re-exports none of the new items. | Test |

## Dependencies

[FR-038](../../checked_package/functional/FR-038-consume-checked-package-v2.md)
owns V2 reader admission, checked identities, and source-map correspondence.
[FR-019](./FR-019-rust-library-interface.md) owns the model crate's public
item inventory. QSL owns checked `Population<T>[N]` typing and its V2 lowering;
the domain-package `PopulationRecord` owns declaration identity and closure,
not a maximum. IR-268 consumes this projection for symbolic graph inputs. The
projection does not implement that consumer or claim a Kani proof.
