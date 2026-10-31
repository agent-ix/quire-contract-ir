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
and type node. In particular,
it SHALL NOT relabel a dimension, alias, or union as an unrelated scalar, nor
claim a standalone `Dimension`, `Alias`, or `Union` Exact variant. An admitted
union may use `Composite(NodeKey)` only when its selected union declaration
and payload identity have an authoritative exact witness. The projection SHALL
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
| FR-352-AC-6 | Every other admitted V2 type form either projects to an Exact variant with its full bounds, mode, members and identity, or returns `UnsupportedExactType` before exposing a partial view. No unmatched form maps to Boolean, Integer, Text, Composite or Reference as a default. | Test, Inspection |
| FR-352-AC-7 | Repeated reads of one admitted package and its clone return equal complete values or equal typed outcomes. A nested or cyclic type either completes within the finite limit or reports typed exhaustion with the charged work and locus; no call reparses wire/source, grows the call stack with type depth, or grants admission to an unadmitted fixture. | Test |
| FR-352-AC-8 | The public-item and dependency inventories show the projection in `quire-contract-model`, its cycle-free dependency on `quire-exact`, and no QSL checked-expression constructor, checker, evaluator or mirror expression type in IR; Exact has no IR admission dependency. QSL FR-390 AC-6 and AC-11 remain separately verified in QSL. | Test |
| FR-352-AC-9 | A QSL-side selected-model identity producer supplies `SelectedExactIdentities` alongside a checked package; an external caller passes it to the public projection API. The IR join accepts identities only for matching admitted selected-document, declaration form and original IR declaration or field identities. Absent, conflicting, foreign-document and wrong-form entries return the named typed outcome; a caller-supplied bare `NodeKey`, `EffectiveId`, `MemberId`, or node-name pair cannot enter the projection as an authoritative witness. | Test, Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns V2 admission,
  structural-key resolution, selected declarations, limits, and first refusal.
- [FR-349](./FR-349-retain-typed-checked-v2-terms.md) owns typed term retention
  and read-only boundaries.
- QSL [FR-390](ix://agent-ix/quire-spec-language/FR-390) owns evaluator and
  supplied-value parity; QSL [FR-400](ix://agent-ix/quire-spec-language/FR-400)
  owns checked-term construction.
