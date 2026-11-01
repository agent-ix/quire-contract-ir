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
`CheckedPackageV2`, Contract IR SHALL return QSV's opaque
`AdmittedEqualityDecision` for the decision made by the producing checking
stage. That value SHALL preserve the decision needed to form the consumer's
`CheckedEquality` node without running a type checker in the reader. An operation identity, operand
types, structural domain projection, or catalog law alone SHALL NOT be treated
as that decision.

## Inputs

- An admitted package, the full structural `CheckedNodeId` of the equality
  `expression`/`application` node, one occurrence belonging to that node,
  and a finite read work limit.
- An equality decision from the same checking stage that produced the package,
  carried into package admission under an authoritative producer contract.
  The producer decision identifies the package, equality application node,
  occurrence, empty `term_path`, operator, both original operand positions and
  source types, any conversions, selected schedule,
  and the complete resolved `UnitTable` and filtered `EnumMemberIndex` facts
  retained by the checked equality. The unit facts include every compound-unit
  dimension, canonical root-unit term and exponent, and exact conversion datum
  that the selected equality schedule uses; the enum facts include declaration
  and member identities, ordered flag, cases and ranks.
  A caller-created operation name or pair of type nodes is not such a decision.
- Selected QSL-stage Exact identity and descriptor rows supplied alongside the
  admitted package to the IR read, not added to QSpec V2 wire. The rows name
  each equality application's original `NodeKey` and authored `Location` by
  full emitted structural node and occurrence; dimension-declaration,
  base-dimension-term and ordered conversion-edge-target `NodeKey`s by full
  structural node, selected producer document/context and unit-fact role; source declared
  `UnitId`, compound `UnitId`, model enum declaration
  `NodeKey`, model enum member `NodeKey`, Text type identity and Plan type
  identity needed by the selected equality's operand source/conversion types
  and resolved unit/enum facts. A row carries its admitted structural node,
  selected source or model document/context and admitted node owner when one
  exists, original declaration or member
  identity when present, and the QSL-issued Exact identity or descriptor. The
  application row additionally carries the exact occurrence. A bare Exact id, display
  name or caller-created row is a claim, not evidence of QSL-stage provenance.

## Outputs

The public read SHALL return a complete QSV-owned opaque
`AdmittedEqualityDecision` or a closed typed refusal. The value's read-only
typed API SHALL expose the admitted equality application node and occurrence
as provenance getters, together with the equality or inequality operator,
each operand's source type and optional
conversion target, the selected text, enum, quantity or plan schedule, and
the complete resolved unit-table and filtered enum-member closure retained
for that schedule. A quantity value SHALL preserve compound
unit dimensions, canonical root-unit terms/exponents and exact conversion
data as well as top-level operand units. An enum value SHALL preserve the
compared declaration and member identities, ordered flag, cases and ranks.
The value SHALL preserve source order and the producer's selected identities.
The read SHALL expose no `serde_json::Value` body, raw wire carrier as an
admission proof, or partial decision.

## Behavior

The QSpec-owned `checked_term_data.equality_decisions` carrier SHALL identify
each decision by the equality application's own `NodeRef` and exact
`occurrence`, with an empty `term_path`. Every composite application is its
own graph node. An `and`, `or`, aggregate or binding parent SHALL reference a
child equality application by that child's `NodeRef`; the equality row SHALL
name the child, not the parent. The reader SHALL reject a foreign or absent
application, a node of another form, a non-empty `term_path`, and duplicate
(`application`, `occurrence`) equality rows. The reader SHALL check the
occurrence against that application and retain it as provenance. It SHALL NOT
follow a parent reference or infer an equality identity from a path, display
name, source position or operand type. This FR requires no new wire member.

The reader SHALL retain a producer decision only during admission of the
corresponding complete package. The existing public, deserializable IR
`CheckedEqualityDecision` SHALL remain the raw V2 wire carrier. The existing
`checked_term_data()` read may still expose that row.
The FR-353 admitted read SHALL NOT return a raw `CheckedEqualityDecision` or
treat construction of that raw value as proof of admission.
The reader SHALL join the wire decision to the admitted
equality application by package, full application node identity and exact
occurrence, and compare its operator, operand positions and source types to
the admitted `CheckedTerm::Application` and its reader-admitted operation
record. A missing application, wrong occurrence, non-empty path, wrong
operator, changed operand position or source type, duplicate decision, or
decision for another package SHALL refuse the decision at admission with a
named typed outcome. No decision for a different application or one of two
competing rows may be attached to the admitted application. Distinct child
equality applications referenced by one parent SHALL keep distinct decisions
by their own application nodes and occurrences, even when they share an
operator and operand types.
The schedule, conversion targets, complete resolved `UnitTable` facts
and filtered `EnumMemberIndex` facts remain
facts of the producing checking stage. IR SHALL preserve those facts exactly.
For QSV's `EqualityDecisionDescriptor.location`, IR SHALL use the original
application `NodeKey` and authored `Location` supplied for that exact emitted
equality application and occurrence. It SHALL compare the row's occurrence
and structural application with the admitted equality decision, require the
`Location`'s node to equal the row's application `NodeKey`, and preserve
the original Exact `Origin`. The stage row SHALL state the correspondence
between that `Origin` and the emitted structural occurrence; a mismatched
correspondence SHALL refuse the join. IR SHALL NOT mint an Exact application
key or replace a conflicting authored location from the structural occurrence.
The IR read SHALL join each selected QSL-stage row to this admitted package's
selected source or model document/context and admitted node owner where one
exists, exact structural application,
type/unit/dimension/enum node and unit-fact role,
original declaration or member identity, and the equality decision's
referenced source/conversion type or resolved unit/enum fact. It SHALL reject
foreign, duplicate, conflicting, wrong-form or mismatched rows at the first
structural locus. A missing selected identity or descriptor needed by the
chosen Text, Enum, Quantity or Plan schedule SHALL return typed
`UnavailableEqualityWitness` with the missing structural node and witness
kind, without minting a QSV value. No caller-supplied row becomes
authoritative merely because its Exact payload is well formed.

For each QSV unit descriptor, the read SHALL use the original witnessed
`UnitId`. For a declared form it SHALL use the producer's dimension
declaration key, ordered dimension-term keys and ordered conversion-edge
target keys. For a compound form it SHALL use the ordered dimension-term
keys. IR SHALL match each supplied key to the corresponding admitted selected
unit fact and structural node, including the sidecar's original edge order
and exact scale/offset. A missing supplied key SHALL return
`UnavailableEqualityWitness` naming its structural node and role before
QSV minting. A changed edge target, dimension term or location SHALL refuse
the join at that fact's structural locus.
QSV's private admitted construction SHALL unwrap the declared unit key and
compound root terms from the already witnessed `UnitId`.
QSV SHALL obtain a declared root from the last retained ordered path target,
or that declared key for an empty path.
QSV SHALL NOT select a conversion route or repeat semantic checking in these
steps, and IR SHALL NOT mint an identity. An Exact quantity-type key,
duplicate declared-unit key, explicit root key and duplicate
compound-root-term keys are not required descriptor inputs;
IR may retain the structural quantity-type fact in the sidecar.

The IR read SHALL perform bounded representation mapping from those admitted
structural facts and joined QSL-stage identities to QSV-owned operand
`ValueType` descriptors, the selected schedule, a complete held-identity
`UnitTable`, and the compared enum's filtered `EnumMemberIndex`. It SHALL
preserve the QSL-issued identities, original operand and member order, enum
cases/ranks/ordered flag, unit dimensions, canonical root terms/exponents and
conversion data exactly. The read SHALL charge each visited type, witness,
unit, conversion edge and enum member before use and return typed exhaustion
without a partial descriptor or admitted value. The read SHALL NOT derive an
Exact identity from a structural node or name, infer a schedule from the
operation catalog, run semantic type checking, or treat a general FR-352
`project_exact_type` call as a substitute for a missing witness. Mapping an
admitted representation with joined producer identities is permitted; it
does not select a new semantic type, conversion or comparison plan.
The IR read SHALL include QSV descriptor materialization in the same finite
work allowance, passing the remaining allowance through QSV's Exact meter.
It SHALL preserve QSV's original typed `Incomplete` outcome and first failing
locus on exhaustion, and the QSV typed materialization cause and locus on an
arithmetic or admitted `UnitId` form failure, without returning a partial
descriptor or opaque value.
QSV SHALL mechanically fold the sidecar's already selected ordered exact
affine conversion-edge path with exact arithmetic when its private
`Unit.canonical` representation requires a scalar. The fold SHALL preserve that
path's order and result without selecting another root or conversion route.
An arithmetic impossibility or exhausted work allowance SHALL return a typed
failure before minting; no identity or schedule is synthesized by the fold.

QSV SHALL own `AdmittedEqualityDecision` with private fields and no public
constructor from raw decision fields. QSV SHALL expose a `#[doc(hidden)]`
mint requiring a QSV-defined `ReaderSeal`.
The admitted Contract IR reader SHALL call that mint only for a decision it
has joined, mapped and validated. QSV SHALL construct the opaque value from
joined admitted descriptors through its hidden mint, retaining the supplied
Exact identities and building the full held-ID unit table and filtered enum
member index using its private construction boundary, without exposing
raw-field constructors. The public read SHALL return the QSV value. QSV
`CheckedEquality::from_admitted_decision(&AdmittedEqualityDecision)` SHALL
construct the checked value from the retained decision without calling
`TypeEnvironment::check_equality` or another checking pass. IR SHALL pass its
joined `EqualityDecisionDescriptor`, QSV `ReaderSeal` and remaining Exact
`Meter` to QSV's published
`reader_admission::mint_admitted_equality_decision` boundary and preserve
QSV's typed materialization failure. This FR assigns no opaque wrapper to IR.

The hidden mint is a public Rust item and can be called by another crate that
obtains a `ReaderSeal`; `#[doc(hidden)]` does not make it inaccessible. The
admission provenance claim SHALL be an audited API discipline, not a claim of
language-level cross-crate unforgeability.
A structural public-API audit SHALL establish that the admitted value's fields
and raw-field constructors are private, that the hidden mint and seal are not re-exported through IR's
public API, and that no other public admission constructor is exposed.
Cargo metadata and public-API evidence SHALL establish the dependency
direction QSV -> Contract IR model -> Contract IR root, with no QSL dependency
in either IR package. The IR audit SHALL use visibility, API surface and
dependency data; it SHALL NOT substitute text search or a hand-maintained
caller ledger for that evidence. QSL's corresponding layering check is owned
downstream.

The public accessor SHALL read only a retained admitted decision. When the
package has no admitted decision for the requested equality, it SHALL return
`UnavailableEqualityDecision`, with the requested equality application
identity and occurrence, before exposing any schedule or operand value. An
unknown or foreign node,
non-application node, non-equality application, ineligible operation, and
occurrence not owned by that application SHALL each have a
distinct typed refusal. The reader SHALL charge every inspected application,
operand and retained decision entry against the
finite work limit before inspection. Exhaustion SHALL return the attempted
work and first failing locus without a partial view. Repeated reads and reads
from an equal clone SHALL return equal complete views or equal typed refusals,
without reparsing wire or source documents or changing package equality.

Contract IR SHALL own the raw wire carrier, the admitted structural join and
the read returning QSV's opaque value. QSV SHALL own construction of its
`CheckedEquality`; QSL's package reader SHALL consume that checked value
without calling `TypeEnvironment::check_equality`. The model crate SHALL
acquire no dependency on QSL, its checker or evaluator.

## Acceptance Criteria

The positive equality cases below have the selected QSL-stage witnesses
required by their operand and schedule shapes. Missing witnesses follow
FR-353-AC-7 rather than yielding a partial admitted value.

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-353-AC-1 | A public external consumer reads admitted `=` and `!=` applications by each equality application's full node identity and exact occurrence and receives QSV `AdmittedEqualityDecision`; its read-only typed API exposes the producer-selected operator, original operand positions, source and conversion target types, and exact schedule in source order. `CheckedEquality::from_admitted_decision` consumes that value without a checking pass. Two equalities referenced by one `and` parent, one referenced by `or`, and equality children referenced by aggregate-member and binding-value forms remain distinct, with an empty `term_path` in each equality row. The four TC-1070 Fixture E expressions at rows 8, 12, 15 and 17 are readable as four separate child application rows. Text, enum, quantity and plan cases retain their respective schedule; the enum case retains the complete filtered index for its compared declaration, including declaration/member identities, ordered flag, cases and ranks, while the quantity case retains every resolved `UnitTable` fact used by its schedule, including compound-unit dimensions, canonical root-unit terms/exponents and exact conversion data. The result contains no JSON body, top-level-only unit truncation, unrelated enum members or partial decision. | Test |
| FR-353-AC-2 | Admission refuses a decision whose package, equality application node, occurrence, operator, operand position or source type differs from the complete admitted application, and refuses duplicate (`application`, `occurrence`) equality rows. An absent or foreign application, non-application node, non-equality application, or non-empty `term_path` is a named typed refusal at the first mismatching sidecar member; neither decision becomes readable. Distinct equality application nodes retain distinct decisions even when they share an operator and operand types. A parent reference, display name or type equality does not establish the join. | Test |
| FR-353-AC-3 | An admitted equality application with no authoritative producer decision returns `UnavailableEqualityDecision`; unknown or foreign application, non-application node, non-equality application, ineligible operation and wrong occurrence return their distinct typed outcomes. A non-empty sidecar `term_path` refuses at admission. None returns a fabricated schedule, an empty unit or enum snapshot in place of a required one, or a partial view. | Test |
| FR-353-AC-4 | A finite work limit equal to the charged read succeeds and one less returns typed exhaustion at the first uncharged application, operand or decision entry, with attempted work. Repeated reads and a package clone give equal outcomes and do not reparse package or selected source bytes. | Test |
| FR-353-AC-5 | Structural public-API evidence shows that IR's public `CheckedEqualityDecision` remains a raw wire carrier, while the admitted read returns QSV `AdmittedEqualityDecision` with private fields, no public raw-field constructor and a `#[doc(hidden)]` mint requiring QSV `ReaderSeal`; neither mint nor seal is re-exported through IR's public API. Cargo metadata and public-API evidence show QSV below the IR model, no QSL crate dependency in either IR package, and no IR-owned opaque wrapper. The audit records that the public hidden mint remains callable from another crate with a seal, so provenance is an API discipline rather than language-level cross-crate unforgeability; it uses no grep/source scan or hand-maintained caller ledger. QSL's corresponding public-API layering check belongs downstream. | Inspection |
| FR-353-AC-6 | With selected QSL-stage application `NodeKey`/authored `Location` per exact equality occurrence, dimension-declaration/base-term/ordered edge-target `NodeKey`s, source and compound `UnitId`s, model enum declaration/member `NodeKey`s, Text type identity and Plan type identity rows joined to their exact admitted nodes and selected producer documents/contexts, the reader maps the sidecar's source/conversion types and schedule into QSV admitted descriptors without a checking pass. QSV privately obtains declared keys, declared roots and compound root terms from the witnessed UnitId and retained path, keeping all full native unit facts in QSV. The QSV opaque value's read-only provenance getters return the original Exact application and occurrence, and its fact getters retain the issued identities, operand/member order, enum cases/ranks/ordered flag, complete held unit facts and conversion data. `CheckedEquality::from_admitted_decision` consumes the value without rebuilding any identity or schedule. | Test |
| FR-353-AC-7 | For each required witness kind, including application/location, dimension declaration/base terms and ordered edge targets, an absent row returns typed `UnavailableEqualityWitness` naming the missing structural node and kind before QSV minting; a foreign-document, wrong-form, duplicate, conflicting or mismatched row returns a typed refusal at its first structural locus. A work limit equal to the complete application/type/witness/unit/enum representation mapping admits, and one less returns typed exhaustion before the next charge with no partial descriptor or opaque value. Neither path fabricates an Exact key from `CheckedNodeId`, substitutes a display name, uses an unsupported general FR-352 projection, or runs semantic checking. QSV's use of the admitted UnitId and path for its native unit fields does not mint a new identity. | Test |
| FR-353-AC-8 | For a declared quantity whose selected unit has an ordered exact affine conversion path, QSV's private admitted construction folds that path to the required canonical scalar with exact arithmetic and preserves the selected root and edge order. Changing an edge or root refuses the admitted join; arithmetic impossibility or one-less work returns a typed failure before an opaque decision exists. The fold does not choose another route, invent a unit identity, change a conversion target or recompute the selected equality schedule. | Test |

## Dependencies

QSpec [STD-613](https://linear.app/agent-ix/issue/STD-613) and
[STD-619](https://linear.app/agent-ix/issue/STD-619) supply the
FR-322 `checked_term_data.equality_decisions` wire and vectors for the
authoritative, bounded carrier. FR-322-AC-288/289/292 require each equality
row to identify its own application node and occurrence with an empty
`term_path`; parent bodies reference child application nodes. Current IR
admission does not yet retain
the producer's schedule, conversion choices, complete resolved unit-table
closure or filtered enum member index. This FR allocates no further V2 wire
member and grants no authority to reconstruct a decision from type or catalog data.
QSL [QSL-1398](https://linear.app/agent-ix/issue/QSL-1398) supplies the
selected source `UnitId` and model enum declaration/member key rows.
QSL [QSL-1390](https://linear.app/agent-ix/issue/QSL-1390) supplies selected
equality application `NodeKey`/authored `Location` rows,
dimension-declaration/base-term/ordered edge-target `NodeKey` rows, compound
`UnitId`, Text type identity and Plan type identity rows from the same
checking/emission stage as the equality sidecar. This producer-row delivery
follows QSL-1398 and is a prerequisite to the complete IR read. After the IR
read is available, QSL-1390 separately integrates its package reader with
QSV's `CheckedEquality::from_admitted_decision`.
These stage rows travel alongside the admitted package to the IR read, not in
a new QSpec wire member. A missing row leaves the corresponding successful
read unavailable; IR cannot fill it by inventing an Exact key.
QSV [IR-1223](https://linear.app/agent-ix/issue/IR-1223)
owns the opaque admitted value, hidden mint, `ReaderSeal`, no-recheck
constructor and admitted descriptor path for held unit and filtered enum
facts. Its published mint accepts `ReaderSeal`, `EqualityDecisionDescriptor`
and a shared Exact `Meter`. Merged QSV PR #83 removed the redundant
quantity-type, declared-key, explicit-root and compound-root-term descriptor
inputs while preserving QSV's complete native facts and opaque provenance;
its wrong-`UnitId` form returns typed `IdentityForm` at the unit locus. The IR
read returns QSV's `AdmittedEqualityDecision` or a typed refusal. IR can verify
the raw carrier with QSpec vectors and
locally built fixtures while the producer stage rows are implemented. This
FR uses the child's existing structural node identity for the wire join and
preserves the checking stage's separate Exact identity for QSV provenance.
