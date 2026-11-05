---
id: FR-352
title: "Project admitted checked-package types to Exact value types"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-349
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-390
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-400
    type: references
---
# FR-352: Project admitted checked-package types to Exact value types

## Description

When a consumer requests the type of a node in an admitted `CheckedPackageV2`,
Contract IR SHALL expose a public, read-only projection from that node's
`semantic_type: CheckedNodeId` to `quire_exact::ValueType`. The result SHALL
retain the identity and domain of the admitted type or return a typed refusal
or incomplete result before exposing any partial value-type view. This is an
IR-model read over [FR-038](./FR-038-consume-checked-package-v2.md)'s admitted
graph and selected declarations, not another admission path. The dependency
direction is IR model to Exact; Exact gains no checked-package or model
admission authority.

## Inputs

- An owner-admitted `CheckedPackageV2`, a `CheckedNodeId` identifying a node in
  that package, and a finite projection work limit.
- For an identity-bearing type, a read-only `SelectedExactIdentities` input
  produced by the QSL checking/model identity stage for the selected documents
  used to construct this admitted package. Its entries associate each selected
  document identity, original IR declaration identity and declaration form with
  the stage's Exact `NodeKey` or `EffectiveId`; record field entries associate
  the selected IR field identity with its Exact `MemberId`. The caller obtains
  this input from that stage alongside the checked package. It is a trusted
  producer input, not a value minted or authenticated from arbitrary bytes by
  Contract IR; a caller-created table has no authoritative status.
- For an admitted model-owned state operation anchor, a QSL checking/model stage
  `SelectedOperationMemberRow` from the same selected documents. The row
  carries the full anchor `CheckedNodeId`, selected domain document identity,
  declaring model's original IR identity and closed declaration form, the
  selected operation member's original IR identity, and the stage's selected
  operation identity. The latter is the stage's existing operation identity;
  this contract does not mint an Exact key for an operation. The row is
  producer input alongside the package, not a new QSpec wire member.
- For a source-owned selected union, one QSL checking/emission-stage
  `SelectedSourceUnionRow` from the same final checked-package emission. It
  carries the full union structural node, selected source authority and
  identity, original qualified declaration, QSL-issued union type handle
  (`AdmittedComposite::key()`), settled union key
  (`AdmittedComposite::node()`), and every QSL-issued sealed `UnionMember`
  binding with its ordered final-emission IR payload `CheckedNodeId`s and
  corresponding QSL-issued payload `ValueType`s. The type handle and settled
  key are distinct roles even when their values happen to agree.
- For a source-owned selected tuple, one QSL checking/emission-stage
  `SelectedSourceTupleRow` from the same final emission. It carries the final
  admitted tuple node, selected source authority and identity, original
  qualified declaration, QSL-issued `AdmittedComposite::key()` type handle,
  and every position in declaration order. Each `SelectedTuplePositionRow`
  carries its zero-based original ordinal, final emitted IR type node and
  QSL-issued opaque Exact `ValueType`. A tuple position has no invented field
  name or `MemberId`; `AdmittedComposite::node()` is not substituted for the
  type handle.
- For an admitted equality application, QSL checking/emission-stage identity
  rows from the same package construction. An application row carries the full
  emitted `CheckedNodeId`, its exact authored occurrence, the selected producer
  document/context and QSL's original application `NodeKey` and `Location`. Quantity
  identity rows carry the full emitted structural nodes and selected producer
  documents/contexts for
  the dimension declaration, base-dimension terms and each ordered
  conversion-edge target, together with their original QSL-issued `NodeKey`s.
  The rows distinguish each occurrence and each unit fact role;
  they are producer input alongside the package, not QSpec wire members.

The public IR-model entry point SHALL take the admitted package, requested
`CheckedNodeId`, `&SelectedExactIdentities`, and finite work limit, and return
`ExactTypeProjection` or the closed typed outcome. The QSL-facing
`SelectedExactIdentities` acquisition interface SHALL accept the stage's
selected-document identity plus declaration-form/original-identity/Exact-id
rows and record-field original-identity/`MemberId` rows. It SHALL expose no
constructor that treats a bare Exact id or display name as a selected row.
The producing QSL stage supplies these rows from the same selected documents
it used for checked-package construction; IR uses their original identities
for the admitted-declaration join. The opaque Exact identities themselves
remain trusted producer facts: IR cannot derive their provenance from their
payloads.

The QSL-facing operation-member acquisition interface SHALL emit one row per
selected admitted model-owned anchor, including distinct admitted execution
points for one operation. It SHALL NOT use a matching operation name alone to identify a
selected member. The operation-member rows are a separate stage-row kind;
the current `SelectedExactIdentities` declaration and field rows do not
contain them. A source-declared anchor whose context is not a selected model
declaration has no FR-040 model member and SHALL NOT be assigned a model-member
row. The callable view SHALL return a typed unavailable-witness outcome for
that anchor until the QSL source stage supplies a distinct source-operation
identity witness.

The QSL-facing equality identity acquisition interface SHALL emit the
application row for each produced equality occurrence and the complete
quantity identity rows used by its selected unit facts. The application key
and authored `Location` SHALL come from the checking stage's retained
application-key/location correspondence. Quantity keys SHALL come from the
stage's retained unit and dimension nodes. `TypeEnvironment` SHALL NOT be treated
as a source of these quantity identities. An IR reader SHALL join every row
by full structural node, occurrence where applicable, selected producer
document/context and admitted node owner where one exists,
unit-fact role and original declaration identity where one exists. It SHALL
refuse a missing, foreign, wrong-form, duplicate, conflicting or mismatched
row at its first structural locus before constructing an Exact descriptor.
It SHALL charge each inspected row against the finite work limit. No
`CheckedNodeId`-to-`NodeKey` conversion or display-name lookup establishes an
identity. The already witnessed `UnitId` supplies its own declared key and
compound root terms to QSV's private admitted construction; QSV obtains a
declared root from the last retained path target or declared key when the path
is empty. IR SHALL NOT mint these keys. The equality read and
its closed `UnavailableEqualityWitness` refusal are owned by
[FR-353](./FR-353-read-admitted-equality-decisions.md), not the general
`project_exact_type` API.

## Outputs

- A public `ExactTypeProjection` result containing a complete
  `quire_exact::ValueType` and the selected structural `CheckedNodeId` to Exact
  identity correspondence. For a selected record it also contains a public,
  read-only `SelectedRecordView`: its witnessed record `NodeKey` and every
  effective field in selected declaration order, each with its selected IR
  field identity, name, witnessed Exact `MemberId`, projected Exact
  `ValueType`, and required/optional presence. The view is sufficient to form
  Exact `FieldDeclaration` inputs for complete supplied-record checking.
- Otherwise, a closed typed outcome distinguishing unknown or foreign node,
  unavailable or conflicting identity witness, unresolved selected declaration,
  unsupported Exact type shape, and exhausted work. It contains the requested
  structural node and the first failing type node or declaration as applicable.

## Behavior

For an operation-member row, Contract IR SHALL join the full anchor
`CheckedNodeId` to an admitted [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md)
operation anchor and its selected declaring model. It SHALL compare selected
document identity, model form, original model identity, and original operation
member identity with the admission-retained selected model table. It SHALL
reject a foreign or dangling anchor, a member of another selected model,
wrong declaration form, absent member, or competing rows for one anchor
before exposing a selected operation read. Two admitted model-owned anchors for one
operation at distinct execution points MAY carry the same selected operation
identity; the anchor identity distinguishes their rows. Contract IR SHALL
NOT infer the stage identity from an operation display name, parse source
again, or construct a QSL operation checker. The callable read of
[FR-354](./FR-354-project-admitted-runtime-context.md) consumes this joined
row and returns a typed refusal while the QSL stage does not supply it.
Contract IR SHALL NOT match a source-declared anchor to a model operation
member because their names happen to agree.

The projection SHALL resolve the requested node through the admitted package's
structural-key lookup, then its retained `semantic_type` and selected declaration
index. It SHALL use the complete package, path and ordinal for each
`CheckedNodeId`; a matching name, ordinal alone, graph position, or alias is
not an identity match. A witness SHALL bind that exact structural node to the
selected declaration and its Exact identity in the same selected package
closure. The projection SHALL join the trusted producer's entries against the
admission-retained selected-document identity, declaration form and original
IR declaration or field identity; a supplied Exact identity alone does not
prove that correspondence. The producer's acquisition contract SHALL carry
the selected document and declaration identities used for this admission and
expose absent or ambiguous identity results as such. The projection SHALL
refuse a witness for another package, a dangling
declaration, two competing identities for one structural node, or two structural
nodes claiming one selected declaration without an unambiguous owner. The projection SHALL
neither synthesize an Exact identity from a display name nor select a
candidate arbitrarily. It SHALL reuse admission-retained graph and declaration
facts without reparsing the wire or selected source documents.

The projection SHALL map admitted `scalar_type/boolean` to `Boolean`,
`scalar_type/integer` to `Integer`, and `bounded_domain/integer_range` to
`Int` with the admitted inclusive endpoints. Its required bounded examples
are `Int[0,1000]`, `Int[0,3]`, and `Int[0,9]`; their endpoint text SHALL be
converted exactly or refused if Exact cannot represent it. Where admitted and
faithful to Exact's current closed variants, `scalar_type/string`,
`scalar_type/uuid`, and `scalar_type/timestamp` SHALL map to `String`, `Uuid`,
and `Timestamp` respectively, without converting them to `Text` or `Integer`.

A selected `model/record_value_type` SHALL map to `Composite` with the Exact
`NodeKey` witnessed for that record declaration. This includes `Receipt`,
`Attempt`, `Flag`, and `Tick`. The record view SHALL derive its full selected
field set, inherited members, redefinitions, names, presence and member types
from the admitted selected declaration's retained field table; it SHALL join
each Exact `MemberId` from the trusted producer by selected IR field identity.
An unread member, including `Tick.count: Int[0,9]`, SHALL remain in the view.
If any field lacks a unique witnessed identity or representable complete type,
the entire projection SHALL return a typed refusal without a partial record
view. The view SHALL neither infer fields from expression reads nor substitute
display names for member identities.
For a selected record field, Contract IR SHALL project the complete
admission-retained member type into the corresponding Exact `ValueType`.
`Collection<Integer>` SHALL retain its admitted sequence, set, bag or ordered-set
kind, integer element and exact cardinality bound, including the distinction
between no bound and an admitted `[0,3]` bound. Contract IR SHALL retain an
admitted `Option<Integer>` element inside the Exact collection and an admitted
`Option` nested inside any other non-Option member type.
When the admitted field is optional, the view SHALL consume exactly the one
outer `Option` that represents field presence and report `Presence::Optional`;
it SHALL preserve any admitted inner `Option` and collection layers in the
field's Exact `ValueType`. A required field SHALL retain
`Presence::Required`. The projection SHALL NOT discard a wrapper, flatten a
collection, change its kind or bounds, or treat an admitted complete member
type as unsupported merely because it contains an `Option` or collection.
This projection consumes only QSpec FR-154 admitted member types. It SHALL NOT
admit `Option<Option<T>>`, including an alias path to the inner `Option`, or
an optional `[1,1]` field typed by an option value type; those shapes are
refused before projection.

The projection SHALL inspect nested member types with bounded iterative work,
charging each inspected wrapper, element and referenced declaration before
using it. A cyclic nominal record reference SHALL retain its witnessed Exact
`NodeKey` without recursively expanding the record's fields at that reference.
The caller's finite work allowance SHALL bound both successful projection and
the first exhausted member; nesting depth SHALL NOT grow the host call stack.
An unavailable or conflicting opaque identity for a nested referenced record,
tuple, object or unit SHALL retain the corresponding native typed witness
refusal and first failing referenced declaration. An unrepresentable nested
leaf SHALL retain `UnsupportedExactType` and its form. Either refusal SHALL
expose no partial record view. This read uses the admitted selected field and
type facts and introduces no new checked-package wire member or source recheck.
A selected `model/object_type` SHALL contribute its witnessed
`EffectiveId` as object-type identity for a `composite_type/reference` whose
admitted target is that exact object type; the reference SHALL map to
`Reference(EffectiveId)`. `model/object_type` by itself SHALL NOT be presented
as a standalone Exact `ValueType` variant. A selected declared unit SHALL use
its witnessed `NodeKey` as Exact's declared `UnitId`; a compound unit SHALL
project only if its authoritative compound identity and domain are available.

For other admitted scalar and structural forms, including rational, decimal,
float, quantity, text, enum, option, collection, map, bounded quantity, and
population, the projection SHALL produce the corresponding Exact variant only
when its complete bounds, mode, member, unit, and identity facts can be
represented; otherwise it SHALL return `UnsupportedExactType` naming that form
and type node. Where the shape is faithfully representable but a required
authoritative opaque identity witness is absent, the projection SHALL instead
return `UnavailableIdentityWitness`; it SHALL NOT classify a missing identity
as an unsupported shape. Both refusals SHALL retain the requested node, first
failing type node and form, and charged work, without exposing a partial view.
In particular,
it SHALL NOT relabel a dimension, alias, or union as an unrelated scalar, nor
claim a standalone `Dimension`, `Alias`, or `Union` Exact variant. An admitted
union may use `Composite(NodeKey)` only when its selected union declaration
and payload identity have an authoritative exact witness. A selected union
declaration key alone does not witness its payload identity or establish a
complete union projection.
For a source-owned union, IR SHALL join the complete selected union row to the
admitted union's full structural node, selected source in the package lock,
source owner and original qualified declaration. It SHALL compare the sealed
members in admitted declaration order by identifier and every positional
payload structural node, require equal payload `ValueType` and node counts,
and require each binding's declaration and variant owner to equal the
settled union key, and refuse a missing, foreign, stale, wrong-form, duplicate,
conflicting or incomplete row before projection. The QSL-issued payload
`ValueType`s and opaque keys remain producer facts; IR SHALL NOT mint them,
recheck source, or infer payload types from display names. The validated row SHALL
retain its settled key and full ordered member bindings for a downstream
`TypeEnvironment` consumer. Projection SHALL emit `Composite` with the union
type handle only after this complete join; a bare `NodeKey` or declaration-only
row SHALL remain `UnavailableIdentityWitness`. The join and projection SHALL
be read-only, deterministic, bounded by the finite inspected row and admitted
body, and charge each inspected union member and payload position against the
caller's projection work limit. No union row is added to the QSpec wire.
The missing-witness refusal does not satisfy QSL's required successful union
reader and evaluation parity under
[FR-390](ix://agent-ix/quire-spec-language/FR-390) AC-27.
For a source-owned tuple, IR SHALL join the complete selected tuple row to the
admitted tuple's full structural node, selected source, source owner, original
qualified declaration and exact tuple form. It SHALL compare every admitted
position with one row in source order by ordinal and final type-node reference,
require a complete equal-length census, and refuse a missing, foreign, stale,
wrong-form, duplicate, reordered or incomplete row before projecting
`Composite` with the QSL-issued type handle. The row SHALL retain the ordered
opaque position `ValueType`s for a downstream `TypeEnvironment`; IR SHALL NOT
infer them, mint the handle or treat arbitrary caller rows as producer authority.
The join and projection SHALL charge every inspected row and position under a
finite allowance and expose no partial tuple view on refusal.
The projection SHALL
terminate or report work exhaustion within the caller's finite limit;
neither recursion depth nor expansion of a cyclic type may bypass the limit.

[FR-349](./FR-349-retain-typed-checked-v2-terms.md)'s read-only term boundary
continues to apply. Contract IR SHALL NOT define a mirror expression
`ValueType`, construct QSL checked terms, or implement QSL's checker or
evaluator. QSL owns supplied-value and observation admission and the result
and charge-log parity of [FR-390](ix://agent-ix/quire-spec-language/FR-390)
AC-6 and AC-11; this projection alone does not satisfy those criteria.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-352-AC-1 | An external crate reads an admitted node's semantic type through one public IR-model API and receives Exact `Boolean`, `Integer`, or `Int` for independently authored Boolean, Integer, and inclusive ranges `[0,1000]`, `[0,3]`, and `[0,9]`; it inspects no JSON and reparses no source. An endpoint outside Exact's representable domain returns a typed refusal, never a narrowed interval. | Test |
| FR-352-AC-2 | Through the public `ExactTypeProjection`, admitted selected `Receipt`, `Attempt`, `Flag`, and `Tick` return distinct `Composite(NodeKey)` identities and public `SelectedRecordView` values. Each view contains every effective member in selected declaration order with selected IR identity, name, Exact `MemberId`, projected type and presence; `Tick.count: Int[0,9]` appears even with no read of `count`. The view can supply Exact `FieldDeclaration` inputs that reject a complete record missing a required member or carrying an out-of-range `count`; a member identity absent from the producer, or a similarly named record in another package, refuses without a partial view. | Test |
| FR-352-AC-3 | An admitted reference to selected `Node` projects to `Reference(EffectiveId)` matching its authoritative object-type witness. The object declaration alone is not returned as a standalone Exact value-type variant. A reference to another selected object type, a foreign package, or a dangling target never produces Node's identity. | Test |
| FR-352-AC-4 | Exact structural identity is matched by package, full resolved path, and ordinal. A changed path segment, ordinal, owner package, or selected declaration refuses with a named typed outcome; duplicate or conflicting witnesses and ambiguous selected owners refuse before any value-type result. Alias equality and display-name equality cannot establish a join. | Test |
| FR-352-AC-5 | Admitted String, Uuid, and Timestamp scalar types map to their respective Exact variants when present. A selected declared unit maps through a witnessed declared `UnitId`; a compound unit without its authoritative identity refuses. A selected union uses `Composite` only with a verified union witness; dimension and alias shapes without faithful Exact representations return `UnsupportedExactType` with their own form and node. | Test |
| FR-352-AC-6 | Every other admitted V2 type form either projects to an Exact variant with its full bounds, mode, members and identity, returns `UnsupportedExactType` when that complete shape cannot be faithfully represented, or returns `UnavailableIdentityWitness` when a required authoritative opaque identity witness is absent. Each refusal names the requested node, first failing type node and form, and charged work, before exposing any partial view. An admitted union without its authoritative declaration and payload identity witness takes the missing-witness outcome; a declaration key alone does not establish a complete union projection. Unsupported shape and missing identity remain distinct causes; neither outcome replaces a required successful projection whose complete representation and witnesses are available. No unmatched form maps to Boolean, Integer, Text, Composite or Reference as a default. | Test, Inspection |
| FR-352-AC-7 | Repeated reads of one admitted package and its clone return equal complete values or equal typed outcomes. A nested or cyclic type either completes within the finite limit or reports typed exhaustion with the charged work and locus; no call reparses wire/source, grows the call stack with type depth, or grants admission to an unadmitted fixture. | Test |
| FR-352-AC-8 | The public-item and dependency inventories show the projection in `quire-contract-model`, its cycle-free dependency on `quire-exact`, and no QSL checked-expression constructor, checker, evaluator or mirror expression type in IR; Exact has no IR admission dependency. QSL FR-390 AC-6 and AC-11 remain separately verified in QSL. | Test |
| FR-352-AC-9 | A QSL-side selected-model identity producer supplies `SelectedExactIdentities` alongside a checked package; an external caller passes it to the public projection API. The IR join accepts identities only for matching admitted selected-document, declaration form and original IR declaration or field identities. Absent, conflicting, foreign-document and wrong-form entries return the named typed outcome; a caller-supplied bare `NodeKey`, `EffectiveId`, `MemberId`, or node-name pair cannot enter the projection as an authoritative witness. | Test, Inspection |
| FR-352-AC-10 | A QSL-stage `SelectedOperationMemberRow` for an admitted model-owned operation anchor joins its full structural node, selected document, declaring model form and original model/operation member identities to the selected operation identity; the FR-354 callable read uses the joined row to find the admitted body and frame. Two admitted anchors of one operation at distinct execution points remain distinct rows. A missing row, wrong document/form/member, foreign anchor, duplicate conflicting row, or name-only match returns a named typed outcome without a callable view. A source-declared anchor with no selected model member returns typed unavailable-witness rather than accepting a same-name model-member row; no new QSpec wire member or invented Exact operation key is used. | Test, Inspection |
| FR-352-AC-11 | QSL checking/emission supplies one original application `NodeKey` and authored `Location` per emitted equality application occurrence, and original dimension-declaration, base-dimension-term and ordered conversion-edge-target `NodeKey`s for every selected quantity fact. Each row carries a full emitted structural node, selected producer document/context and its fact role; application rows also carry exact occurrence. The FR-353 join refuses absent, foreign, wrong-form, duplicate, conflicting or mismatched rows with a typed structural locus and bounded work. Neither `CheckedNodeId`, display name nor the general FR-352 type projection produces a missing key. QSV privately unwraps the already witnessed `UnitId` for a declared key or compound root terms and obtains a declared root from the retained path, without checking or minting identity in IR. No row is added to QSpec V2 wire. | Test, Inspection |
| FR-352-AC-12 | IMPLEMENTED (IR-1324). For an admitted selected record with a plain `Collection<Integer>` member, the public record view returns a witnessed Exact `MemberId`, the original IR field identity and `Presence::Required`, and an Exact collection whose kind, integer element and cardinality bound equal the admitted member. Distinct sequence, set, bag and ordered-set kinds remain distinct, as do unbounded and `[0,3]` cardinality. The complete view can be formed from checked-package bytes and selected producer rows without a source recheck. | Test |
| FR-352-AC-13 | IMPLEMENTED (IR-1324). An admitted selected record with a required collection field whose element type is an option alias returns an Exact `Collection<Option<Integer>>` with its admitted kind and bound and the inner Exact `Option<Integer>` intact. An optional collection field of native Integer reports `Presence::Optional` after consuming its outer presence layer and retains Exact `Collection<Integer>` with its kind and bound. A required `[1,1]` field typed by an admitted option alias retains Exact `Option<Integer>`. The already supported optional field of native String reports `Presence::Optional` and Exact `String`. No positive case uses an optional `[1,1]` field typed by an option value type or an `Option<Option<T>>` alias chain. | Test |
| FR-352-AC-14 | IMPLEMENTED (IR-1324). A nested member reference to a witnessed record in a selected cyclic nominal graph retains `Composite` with that record's original Exact `NodeKey` and terminates without expanding the cycle. A missing, foreign, conflicting or wrong-form nested identity refuses with its native typed cause and first referenced declaration; an unrepresentable nested leaf refuses as `UnsupportedExactType`. Neither refusal exposes a partial record view. | Test |
| FR-352-AC-15 | IMPLEMENTED (IR-1324). A selected record with a deep admitted chain of sequence member types, including an alternating sequence and option chain in which each option's inner target is a sequence rather than an option directly or through aliases, projects using bounded iterative work and a small host stack. At the exact work allowance it returns the complete view; one less refuses with typed work exhaustion, charged work and first uncharged member locus. Equal admitted inputs and allowances, including a package clone, return equal views or equal first stops. | Test |
| FR-352-AC-16 | A complete IR-side selected source union row joins one admitted union by structural node, selected source authority and identity, source owner, original qualified declaration, QSL-issued type handle and settled key, and all sealed members with ordered final-emission payload `CheckedNodeId`s and corresponding `ValueType`s. An IR caller can inspect the validated settled key and ordered bindings; the general projection returns `Composite` with the type handle, including when it differs from the settled key. A projection at the exact finite allowance succeeds, one unit less exhausts with charged work, and equal reads of a package clone agree. This IR-side row test does not assert that QSL currently emits the row. | Test |
| FR-352-AC-17 | A missing row or a bare union declaration key returns `UnavailableIdentityWitness`. A foreign or stale structural node or payload node, unselected or mismatched source, wrong declaration form or qualified name, duplicate union row, missing/reordered/extra member, wrong member identifier, settled-key owner or payload count refuses with a named typed outcome before `Composite` or a partial union view. A caller-created row is not authenticated as QSL output by the IR join. | Test |
| FR-352-AC-18 | A complete IR-owned selected source tuple row joins the selected source, admitted tuple form and original qualified declaration, one QSL-issued `AdmittedComposite::key()` type handle, and every ordered zero-based position with its final emitted type node and opaque `ValueType`. Exact projection returns `Composite` of that handle and the complete ordered tuple view, with exact-limit and clone equality. Missing, foreign, duplicate, stale, wrong-form, reordered, changed-type or incomplete rows refuse at a typed structural locus with no partial view; a bare key and caller-created opaque values do not establish QSL producer authority. This IR-owned row qualification does not claim the final QSL producer supplies positions or satisfy FR-357's monitor closure. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns V2 admission,
  structural-key resolution, selected declarations, limits, and first refusal.
- [FR-349](./FR-349-retain-typed-checked-v2-terms.md) owns typed term retention
  and read-only boundaries.
- [FR-040](./FR-040-admit-frame-entries-and-state-clauses.md) owns operation
  anchor admission and the selected-model operation/member join.
- [FR-354](./FR-354-project-admitted-runtime-context.md) owns the callable
  view that consumes the operation-member witness.
- QSL [FR-390](ix://agent-ix/quire-spec-language/FR-390) owns evaluator and
  supplied-value parity; QSL [FR-400](ix://agent-ix/quire-spec-language/FR-400)
  owns checked-term construction.
