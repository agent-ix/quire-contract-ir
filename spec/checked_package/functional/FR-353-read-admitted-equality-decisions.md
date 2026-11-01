---
id: FR-353
title: "Read admitted equality decisions without a checking pass"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-349
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-322
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-390
    type: references
---
# FR-353: Read admitted equality decisions without a checking pass

## Description

When a consumer reads an equality application from an admitted
`CheckedPackageV2`, Contract IR SHALL expose a public, typed, read-only view of
the equality decision made by the producing checking stage. The view SHALL
preserve the decision needed to form the consumer's checked equality node
without running a type checker in the reader. An operation identity, operand
types, structural domain projection, or catalog law alone SHALL NOT be treated
as that decision.

## Inputs

- An admitted package, the full structural `CheckedNodeId` of a local root
  `expression`/`application` node, one occurrence belonging to it, the
  QSpec-owned typed `term_path` from that root to the equality application,
  and a finite read work limit. The empty path selects the root application.
- An equality decision from the same checking stage that produced the package,
  carried into package admission under an authoritative producer contract.
  The producer decision identifies the package, root application node,
  occurrence, typed `term_path`, operator, both original operand positions and
  source types, any conversions, selected schedule,
  and the complete resolved `UnitTable` and filtered `EnumMemberIndex` facts
  retained by the checked equality. The unit facts include every compound-unit
  dimension, canonical root-unit term and exponent, and exact conversion datum
  that the selected equality schedule uses; the enum facts include declaration
  and member identities, ordered flag, cases and ranks.
  A caller-created operation name or pair of type nodes is not such a decision.

## Outputs

The public read SHALL return a complete IR-owned typed equality decision view
or a closed typed refusal. A successful view SHALL carry the admitted
root application node, occurrence, typed `term_path`, equality or inequality
operator, each operand's source type and optional conversion target, the selected text, enum, quantity
or plan schedule, and the complete resolved unit-table and filtered enum-member
closure retained for that schedule. A quantity view SHALL preserve compound
unit dimensions, canonical root-unit terms/exponents and exact conversion
data as well as top-level operand units. An enum view SHALL preserve the
compared declaration and member identities, ordered flag, cases and ranks.
The view SHALL preserve source order and the
producer's selected identities. It SHALL expose no `serde_json::Value` body,
QSL `CheckedEquality` value, or partial view.

## Behavior

The QSpec-owned `checked_term_data.equality_decisions` carrier SHALL identify
each decision by its root `application` `NodeRef`, exact `occurrence`, and typed
`term_path`. The root SHALL name one local `expression`/`application` node.
The reader SHALL follow each `term_path` step through the admission-retained
`CheckedTerm` tree: `argument(index)` selects an application argument,
`member(index)` selects an aggregate member, and `binding_value` selects a
binding's value. Each selector SHALL match the current term kind and name an
existing child; the final term SHALL be the captured equality application.
The reader SHALL reject a foreign or absent root, a root of another form,
invalid selectors or child indices, and duplicate (`application`, `occurrence`,
`term_path`) keys. The reader SHALL check the occurrence against the root and
retain it as provenance. The path is not a display name, source position, or
identity inferred from an operand type. This FR requires no new wire member.

The reader SHALL retain a producer decision only during admission of the
corresponding complete package. It SHALL join the decision to the admitted
application by package, full root node identity and typed `term_path`, check occurrence
ownership, and compare its operator, operand positions and source types to the selected admitted `CheckedTerm::Application`
and its reader-admitted operation record. A missing application, wrong
occurrence, invalid path, wrong operator, changed operand position or source type, duplicate decision, or
decision for another package SHALL refuse the decision at admission with a
named typed outcome. No decision for a different application or one of two
competing rows may be attached to the admitted application. Distinct nested
applications under one root and occurrence SHALL keep distinct decisions by
their typed paths even when they share the same operation and operand types.
The schedule, conversion targets, complete resolved `UnitTable` facts
and filtered `EnumMemberIndex` facts remain
facts of the producing checking stage. IR SHALL preserve those facts exactly.
IR SHALL NOT regenerate them by
type-checking the graph, projecting `ValueType`s, or inferring a schedule from
the operation catalog.

The public accessor SHALL read only a retained admitted decision. When the
package has no admitted decision for the requested equality, it SHALL return
`UnavailableEqualityDecision`, with the requested root node identity,
occurrence and typed `term_path`,
before exposing any schedule or operand value. An unknown or foreign node,
non-application root, invalid or absent path, non-equality final term,
ineligible operation, and occurrence not owned by that root SHALL each have a
distinct typed refusal. The reader SHALL charge every inspected application,
path segment, operand and retained decision entry against the
finite work limit before inspection. Exhaustion SHALL return the attempted
work and first failing locus without a partial view. Repeated reads and reads
from an equal clone SHALL return equal complete views or equal typed refusals,
without reparsing wire or source documents or changing package equality.

Contract IR SHALL own only the neutral admitted decision and its structural
join. QSL SHALL own construction of its `CheckedEquality` from that trusted
view and SHALL NOT call `TypeEnvironment::check_equality` in a package reader.
The model crate SHALL acquire no dependency on QSL, its checker or evaluator.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-353-AC-1 | A public external consumer reads admitted `=` and `!=` applications by full root application node identity, exact occurrence and typed `term_path` (empty for the root), and receives the producer-selected operator, original operand positions, source and conversion target types, and exact schedule in source order. Two equalities under one `and` root, one under `or`, and aggregate-member and binding-value paths remain distinct; the four TC-1070 Fixture E nested equalities are readable. Text, enum, quantity and plan cases retain their respective schedule; the enum case retains the complete filtered index for its compared declaration, including declaration/member identities, ordered flag, cases and ranks, while the quantity case retains every resolved `UnitTable` fact used by its schedule, including compound-unit dimensions, canonical root-unit terms/exponents and exact conversion data. The result contains no JSON body, top-level-only unit truncation, unrelated enum members or partial decision. | Test |
| FR-353-AC-2 | Admission refuses a decision whose package, full root application node, occurrence, typed `term_path`, operator, operand position or source type differs from the complete admitted application, and refuses duplicate (`application`, `occurrence`, `term_path`) keys. An absent or foreign root, non-application root, selector of the wrong term kind, absent indexed child, or path ending at a non-equality term is a named typed refusal at the first mismatching sidecar member; neither decision becomes readable. Distinct paths under one root and occurrence retain distinct decisions even when they share an operator and operand types. Display-name or type equality does not establish the join. | Test |
| FR-353-AC-3 | An admitted equality application with no authoritative producer decision returns `UnavailableEqualityDecision`; unknown or foreign root, non-application form, invalid or absent path, non-equality final term, ineligible operation and wrong occurrence return their distinct typed outcomes. None returns a fabricated schedule, an empty unit or enum snapshot in place of a required one, or a partial view. | Test |
| FR-353-AC-4 | A finite work limit equal to the charged read succeeds and one less returns typed exhaustion at the first uncharged application, path segment, operand or decision entry, with attempted work. Repeated reads and a package clone give equal outcomes and do not reparse package or selected source bytes. | Test |
| FR-353-AC-5 | Code inspection shows the accessor reads an admission-retained decision, performs no `TypeEnvironment::check_equality` or equivalent type-checking pass, exports no QSL checker or evaluator type from the IR model crate, and leaves construction of QSL `CheckedEquality` to QSL's trusted checked-stage boundary. | Inspection |

## Dependencies

QSpec [STD-613](https://linear.app/agent-ix/issue/STD-613) and
[STD-617](https://linear.app/agent-ix/issue/STD-617) supply the
FR-322 `checked_term_data.equality_decisions` wire, typed nested-path schema and vectors for
the authoritative, bounded carrier. Current IR admission does not yet retain
the producer's schedule, conversion choices, complete resolved unit-table
closure or filtered enum member index. This FR allocates no further V2 wire
member and grants no authority to reconstruct a decision from type or catalog data.
QSL [QSL-1390](https://linear.app/agent-ix/issue/QSL-1390) owns the emitter and
trusted construction boundary for its private `CheckedEquality` fields after
the IR read are available. QSL producer rows are needed for an end-to-end
successful read; IR can verify the carrier with QSpec vectors and locally built
fixtures meanwhile. The QSpec carrier admits nested equality terms beneath
an application root; this FR uses its closed `term_path` selectors without
adding a second identity domain.
