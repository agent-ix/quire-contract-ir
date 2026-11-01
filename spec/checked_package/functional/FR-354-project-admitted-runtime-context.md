---
id: FR-354
title: "Project admitted runtime context from checked-package decisions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-349
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-390
    type: references
---
# FR-354: Project admitted runtime context from checked-package decisions

## Description

When a consumer builds the sealed runtime adapter for an admitted
`CheckedPackageV2`, Contract IR SHALL expose read-only, typed projections of
the package's selected type declarations, callable bodies and frames, dispatch
candidates and effective preconditions, units, and enum members.
Contract IR SHALL return complete admitted facts or a typed refusal before exposing a
partial view. Contract IR SHALL neither create a second checker nor reconstruct QSL
`Scope` or `CheckedGraph`. QSL [FR-390](ix://agent-ix/quire-spec-language/FR-390)
owns runtime interpretation and supplied-state admission.

## Inputs

- An owner-admitted `CheckedPackageV2`, requested admitted node or model
  identity, and finite projection work limit.
- Where an Exact identity is required, the selected-document identity rows of
  [FR-352](./FR-352-project-admitted-exact-value-types.md), produced by the
  same QSL selection stage as the admitted package.
- Additional QSL-stage rows for selected tuple declarations, source-owned
  declared units, enum declarations, and enum members. Each row carries the
  selected source/document identity, original declaration or member identity,
  admitted structural node identity, closed form, and the stage's Exact
  `NodeKey` or declared `UnitId` as applicable. An enum member row also names
  its selected enum declaration and case. These rows extend the stage's
  acquisition contract; current `SelectedExactIdentities` does not contain
  them, so a read requiring one SHALL refuse until the producer supplies it.
  A bare name or Exact key does not establish selection.

## Outputs

- Public typed views of the admitted runtime context. An index built by QSL
  over these views is a lookup cache, not a new admission decision.
- A closed typed outcome identifying an absent, foreign, ambiguous, malformed,
  or unsupported selected join, or the exhausted limit and first failing
  locus. No outcome contains a partial authoritative table.

## Behavior

When the consumer requests the runtime type environment, Contract IR SHALL
project the complete selected composite and object declarations, effective
field identities and types, inheritance edges, and unit identities needed by
semantic-value `TypeEnvironment`. It SHALL preserve [FR-352](./FR-352-project-admitted-exact-value-types.md)'s
selected-document and structural-key joins. For tuples, source-owned units,
and enums, Contract IR SHALL check each additional QSL-stage row against the
admission-retained selected source/document, original identity, node form,
and structural node; the opaque Exact identity remains a producer fact.
Contract IR SHALL reject a member row that names another enum declaration or case.
Contract IR SHALL reject one Exact identity claimed by competing selected nodes.
Contract IR SHALL return a typed refusal if any required declaration, member, parent, or unit lacks a unique witnessed
identity or a representable type. QSL may construct its `TypeEnvironment`
from the complete view, but Contract IR SHALL NOT claim that a newly constructed
environment was itself the producer's checked result.

When the consumer requests callables, Contract IR SHALL expose a table keyed
by admitted callable and operation identities, with each callable's ordered
parameters, body handle, model context and admitted effect frame where
applicable. A handle SHALL resolve only to a term of that admitted package.
It SHALL preserve an intentionally empty frame and distinguish it from a
missing frame. The table SHALL NOT treat a call expression or a similarly
named declaration as proof of a callable definition.

When the consumer requests dispatch, Contract IR SHALL expose the admitted
candidate set, its order and selection keys, and each candidate's effective
precondition body handle. It SHALL reject missing or competing selected joins
without choosing a candidate. The consumer SHALL evaluate those preconditions
at runtime; IR SHALL NOT evaluate them or infer a dispatch decision from names.
If the checked package does not retain a producer decision required by this
view, Contract IR SHALL return a typed unsupported outcome until QSpec and its
producer carry that fact. A newly inferred value is not an admitted decision.

When the consumer requests units, Contract IR SHALL expose each selected
unit's witnessed identity, dimension, target unit, exact scale and offset, and
the admitted links needed to form a semantic-value `UnitTable`. It SHALL keep
declared and derived identities distinct. When the consumer requests enum
members, Contract IR SHALL expose the selected enum declaration, ordered flag,
case order, and each member's witnessed identity and position, sufficient to
form a semantic-value `EnumMemberIndex`. Neither view SHALL derive identity
from case text or display names.

All projections SHALL use admission-retained typed graph and selected-document
facts. They SHALL NOT parse source or wire again, attach an arbitrary source
package, broaden the package's admitted vocabulary, or hide absent producer
facts behind default values. Contract IR SHALL bound projection work and report
exhaustion before disclosing an incomplete authoritative view. Equal admitted
packages and equal selected identity rows SHALL return equal views or equal
typed outcomes.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-354-AC-1 | With the FR-352 and additional QSL-stage rows for selected tuples and source-owned units, a public IR-model read supplies complete record, tuple and object declarations, effective fields, ancestry and units from which QSL constructs the expected `TypeEnvironment`; no source or JSON is reparsed. Before those additional producer rows exist, a tuple or source-owned unit request refuses rather than claiming a complete view. | Test |
| FR-354-AC-2 | A missing, foreign, duplicated or conflicting selected declaration, field, parent or unit identity, including an additional tuple or source-unit row, returns a named typed outcome before any partial type-environment view; matching display names do not make a join. | Test |
| FR-354-AC-3 | The callable projection returns each selected function or operation's ordered parameters, admitted body handle, context and frame, including an intentionally empty frame; a dangling body, wrong owner or absent required frame returns a typed outcome. | Test |
| FR-354-AC-4 | The dispatch projection preserves every admitted candidate in producer order with its effective precondition term reference; a missing or ambiguous candidate/precondition join refuses without choosing a candidate or evaluating a predicate. | Test |
| FR-354-AC-5 | With QSL-stage unit rows for source-owned declarations, a selected root and converted unit yield exact dimension, target, scale, offset and witnessed identity sufficient for `UnitTable`; absent rows, a dangling target, foreign identity or unresolved conversion return a typed outcome without a partial resolver. | Test |
| FR-354-AC-6 | With QSL-stage enum declaration and member rows, an ordered and an unordered selected enum each yield exact case order, member identity and position sufficient for `EnumMemberIndex`; absent, duplicate, mismatched-case or foreign member witnesses refuse before any partial index. | Test |
| FR-354-AC-7 | Repeated bounded reads of an admitted package and its clone agree; an insufficient limit reports the charged work and locus without a partial view, source recheck, wire reparse or stack growth with term depth. | Test |
| FR-354-AC-8 | Public API and dependency inspection finds the runtime projections in IR model, no QSL `Scope`, `CheckedGraph`, evaluator, supplied-state admission or mirror expression type in IR, and no claimed success for a producer decision absent from the package. | Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns V2 reader admission and
  selected package closure.
- [FR-349](./FR-349-retain-typed-checked-v2-terms.md) owns typed retained terms.
- [FR-352](./FR-352-project-admitted-exact-value-types.md) owns Exact type and
  selected identity joins; its remaining obligations precede complete AC-1.
- QSL [FR-390](ix://agent-ix/quire-spec-language/FR-390) owns the sealed runtime
  adapter and evaluation. QSpec FR-322 owns any producer decision not retained
  by the current checked package.
