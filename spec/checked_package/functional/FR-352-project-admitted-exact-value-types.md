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
- For an identity-bearing type, the exact identity witness issued by the
  authoritative selected model or Exact identity stage for that same admitted
  package and declaration. A consumer cannot construct an admitted witness by
  supplying arbitrary identity bytes or an unverified node-name pair.

## Outputs

- A complete `quire_exact::ValueType` for the requested node, with the
  authenticated structural `CheckedNodeId` to Exact identity correspondence
  available to the caller where the value type carries an identity.
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
closure. The projection SHALL refuse a witness for another package, a dangling
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
`Attempt`, `Flag`, and `Tick`; the witness SHALL retain their selected member
identities and types so a consumer can check complete supplied records,
including `Tick.count: Int[0,9]` even when an expression does not read it.
A selected `model/object_type` SHALL contribute its authenticated
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
| FR-352-AC-2 | Admitted selected `Receipt`, `Attempt`, `Flag`, and `Tick` record types project to distinct `Composite(NodeKey)` identities matching authoritative selected declaration witnesses. Their complete selected member names, identities, and projected types agree with the admitted records, including `Tick.count`; a witness for a similarly named declaration in another package refuses. | Test |
| FR-352-AC-3 | An admitted reference to selected `Node` projects to `Reference(EffectiveId)` matching its authoritative object-type witness. The object declaration alone is not returned as a standalone Exact value-type variant. A reference to another selected object type, a foreign package, or a dangling target never produces Node's identity. | Test |
| FR-352-AC-4 | Exact structural identity is matched by package, full resolved path, and ordinal. A changed path segment, ordinal, owner package, or selected declaration refuses with a named typed outcome; duplicate or conflicting witnesses and ambiguous selected owners refuse before any value-type result. Alias equality and display-name equality cannot establish a join. | Test |
| FR-352-AC-5 | Admitted String, Uuid, and Timestamp scalar types map to their respective Exact variants when present. A selected declared unit maps through a witnessed declared `UnitId`; a compound unit without its authoritative identity refuses. A selected union uses `Composite` only with a verified union witness; dimension and alias shapes without faithful Exact representations return `UnsupportedExactType` with their own form and node. | Test |
| FR-352-AC-6 | Every other admitted V2 type form either projects to an Exact variant with its full bounds, mode, members and identity, or returns `UnsupportedExactType` before exposing a partial view. No unmatched form maps to Boolean, Integer, Text, Composite or Reference as a default. | Test, Inspection |
| FR-352-AC-7 | Repeated reads of one admitted package and its clone return equal complete values or equal typed outcomes. A nested or cyclic type either completes within the finite limit or reports typed exhaustion with the charged work and locus; no call reparses wire/source, grows the call stack with type depth, or grants admission to an unadmitted fixture. | Test |
| FR-352-AC-8 | The public-item and dependency inventories show the projection in `quire-contract-model`, its cycle-free dependency on `quire-exact`, and no QSL checked-expression constructor, checker, evaluator or mirror expression type in IR; Exact has no IR admission dependency. QSL FR-390 AC-6 and AC-11 remain separately verified in QSL. | Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns V2 admission,
  structural-key resolution, selected declarations, limits, and first refusal.
- [FR-349](./FR-349-retain-typed-checked-v2-terms.md) owns typed term retention
  and read-only boundaries.
- QSL [FR-390](ix://agent-ix/quire-spec-language/FR-390) owns evaluator and
  supplied-value parity; QSL [FR-400](ix://agent-ix/quire-spec-language/FR-400)
  owns checked-term construction.
