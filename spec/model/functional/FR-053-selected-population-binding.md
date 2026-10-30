---
id: FR-053
title: "Select a source-backed population binding"
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
# FR-053: Select a source-backed population binding

## Description

When a graph consumer selects an input population from an admitted checked
package, `quire-contract-model` SHALL expose a finite maximum only from the
exact checked population binding and source occurrence selected by that
consumer. The result SHALL retain the binding's typed identity, selected
object-type identity, inclusive extent when finite, and admitted source provenance.
`model_object_fields` remains the sole authority for model edge-field
multiplicity.

## Inputs

One typed source selection and, when that selection is admitted, one selector.
An admitted selector names the admitted V2 package by its exact top-level
`name`, the exact `bounded_domain` / `model_population` structural node ID,
and its `CheckedOccurrenceRole::Type` occurrence, including the ordinal.
That node ID is the selected bounded-population binding identity. The query
does not accept a parameter node as a substitute for that identity. A
declaration-only selection names a selected
domain-package PopulationRecord by its typed identity. Neither selection
names an object type alone, supplies a bound, or provides source text.

## Outputs

The model crate exposes `selected_population::select_population`,
`CheckedSelectedPopulationSource`, `CheckedSelectedPopulationSelector`,
`CheckedSelectedPopulation`, and `CheckedSelectedPopulationOutcome` in
`selected_population`. An admitted finite or unbounded result retains the exact selected V2 package
`name` as `package_name`, population binding structural node ID, occurrence,
selected model object type identity, and the authentic source regions
associated with that exact node-occurrence pair. A finite result also carries
inclusive extent `[0, N]`: `maximum` is
`CheckedCanonicalPopulationMaximum`, a validated canonical nonnegative
decimal-text newtype (no leading zero except `0`, no `u64` cap), and `minimum`
remains zero. `AdmittedUnbounded { package_name, binding, occurrence,
object_type, provenance }` carries the same selected identity and source
regions as `Finite` and no extent. The admitted outcome uses the package's
exact `name` and structural node ID, never a `package_id` content identity.
The outcome is typed and
distinguishes `Finite`, `AdmittedUnbounded`, declaration-only `Unbounded`, `UnknownClosure`, `Unavailable`,
`MissingOccurrence`, `StaleBinding`, `ForeignPackage`, and `NotPopulation`.
Non-finite outcomes carry no extent.

The admitted V2 projection reached through FR-038's `reaches_population`
accessor exposes `Finite` for a valid `max` and `AdmittedUnbounded` for an
admitted `model_population` with no `max` (QSpec FR-322-AC-74). The latter
retains the same package name, structural binding, exact `Type` occurrence,
selected object type and source regions as a finite selection. It is distinct
from declaration-only `Unbounded`, whose input names a closed domain-package
population record without an admitted V2 node or source-map occurrence.

`select_population` accepts a source selection that is either an admitted
`CheckedPackageV2` plus selector, a declaration-only population selection, an
upstream typed QSL `UnknownClosure` admission result with its selected model,
population declaration identity, source provenance and typed cause, or no
available selection. The adapter SHALL map QSL's typed
`AdmissionOutcome::UnknownClosure` directly to `UnknownClosure` and retain
those fields unchanged;
it SHALL NOT infer that outcome from a message, an absent V2 node, or an open
resource limit. This path has no admitted V2 package or binding. An admitted
selection with a valid no-`max` population node returns
`AdmittedUnbounded`. A declaration selection with a known population record
and no finite maximum returns
`Unbounded`. An omitted selection returns `Unavailable`. For an admitted
package, a selector whose `package_name` differs from that package's exact
`name` returns `ForeignPackage`; a selected structural node ID absent from
that named package returns `StaleBinding`; an
existing selected node without the exact source-map occurrence returns
`MissingOccurrence`; and an exact occurrence on a node that is not a
population returns `NotPopulation`.

## Behavior

### Source and selection authority

QSL resolves a checked `Population<T>[N]` type occurrence to
`ValueType::Population(Some(N))` and lowers that occurrence to a V2
`bounded_domain` / `model_population` node with maximum `N`, or no `max` for an unbounded population. The selector
SHALL name that exact population node and occurrence. The query SHALL verify
that the named node has the admitted `bounded_domain` / `model_population`
shape, read exact canonical nonnegative decimal `N` from its own `max` binding
when present, and require a source-map entry for the same population node and
exact occurrence before returning a finite extent or admitted unbounded outcome.
It SHALL NOT follow the `semantic_type` pointer of a `value` / `parameter`
node to choose the population node: FR-038 permits that pointer to be
retargeted to another valid population node in a structurally valid package.
The direct node-occurrence selector keeps that reader-permitted retargeting
from changing the selected source binding. It SHALL retain that exact
occurrence and its source-map regions as provenance. An equal object type
identity, or another population node under the same parameter, is not an
equivalent selector.

For a selector returned by `reaches_population`, the admitted binding SHALL
have either one canonical nonnegative integer `max` or no `max` (QSpec
FR-322-AC-74). The former returns `Finite`, including exact decimal text above
`u64::MAX`; the latter returns `AdmittedUnbounded` with the same binding and
occurrence provenance. A negative or non-integer maximum, or a second member,
is refused `invalid_package`/`invalid-derived-shape` at the population node by
the V2 reader under FR-038-AC-279 before projection. A noncanonical JSON
number is refused at the earlier wire step. No reader or query check may
reject a valid maximum merely because it exceeds a machine integer width.

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
extent when present; it SHALL NOT add field-level multiplicities, rewrite edge bounds, or
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

The admitted-unbounded and canonical-decimal-maximum projection is
PLANNED/UNRUN (IR-1141); earlier finite `u64` controls remain historical
evidence only and do not back those corrected obligations.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-053-AC-1 | Through a default-feature public model consumer and an admitted QSL-produced V2 package, a selector naming the package's exact `name`, the exact structural ID of its `bounded_domain` / `model_population` node, and that node's `Type` occurrence for `Population<T>[N]` returns the same package name, node ID, occurrence, selected `T`, authentic source regions, and inclusive `[0,N]` with `N` as validated canonical nonnegative decimal text; `N=0` returns `[0,0]`, and decimal-string `N > u64::MAX` admits and projects unchanged. In a structurally valid body-only maximum change, the selected package name and node ID remain stable while the returned extent changes. Selecting a different admitted node and its own occurrence returns that different node ID and extent. | Test |
| FR-053-AC-2 | Two population nodes with the same selected object type but distinct source occurrences retain their own maxima and provenance. A valid package that re-points a parameter's `semantic_type` at the other population node while preserving a valid structural graph still returns the extent of the node-occurrence pair named by the selector; the query never follows that parameter pointer. Selecting the other node with the first occurrence returns `MissingOccurrence` unless that exact pair is source-mapped. Changing only the Codegen or IR resource ceiling or observed fixture membership does not change either extent. | Test |
| FR-053-AC-3 | An absent selector yields `Unavailable`; a selected structural node ID absent from the admitted package named by the selector yields `StaleBinding`, a present node without the selected occurrence yields `MissingOccurrence`, a selector whose `package_name` differs from the admitted package's exact `name` yields `ForeignPackage`, and a present non-population binding yields `NotPopulation`; each has no extent. A population binding with a second body member fails V2 admission as `invalid_package`/`invalid-derived-shape` at that node rather than producing a projection outcome. A selector with the right object type but a different node ID or occurrence cannot satisfy the request. | Test |
| FR-053-AC-4 | An admitted V2 `model_population` node with no `max` returns `AdmittedUnbounded` retaining package name, binding, exact `Type` occurrence, object type and authentic source regions, without a finite extent. A selected domain-package PopulationRecord with closed extent but no binding maximum yields `Unbounded`; an open extent or unresolved object/subtype closure maps QSL's typed pre-admission `UnknownClosure` to the distinct typed outcome with no package, binding or finite extent. A consumer can distinguish all three non-finite outcomes from `Unavailable`. Source inspection finds no finite-bound path from closure, resource limits, fixture or member count, or caller-provided maximum. | Test, Inspection |
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
