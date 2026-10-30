---
id: FR-069
title: "Admit model tuple declarations and payload-bearing variant types"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-specification/FR-154
    type: references
  - target: ix://agent-ix/quire-specification/FR-208
    type: references
  - target: ix://agent-ix/quire-specification/FR-440
    type: references
---
# FR-069: Admit model tuple declarations and payload-bearing variant types

## Description

When a checked-package V2 selects a domain document containing a tuple value
type or a variant with a resolving payload type, the Contract IR reader SHALL
retain and validate the declaration's type information under QSpec FR-322's
model-owner and identity rules. The typed V2 writer SHALL preserve the same
declaration nodes and package identity when a producer supplies them.

## Inputs

Canonical checked-package V2 bytes and the selected Semantic IR document
supplied under the identity of the package's `model_root`; for writing, the
existing typed V2 envelope members and an explicit canonical encoding limit.

## Outputs

An admitted `CheckedPackageV2`, its existing typed refusal or incomplete
outcome, and canonical V2 bytes from the existing typed envelope encoder.
No second wire schema or parallel declaration store is introduced.

## Behavior

### Selected declarations and node identity

The selected-document intake SHALL recognize FR-208
`quire.meaning.model.tuple-value-type/v1` declarations in `tuple_types` and
retain their ordered positions `p0` through `pN-1`, `N >= 1`, each with its
zero-based ordinal and admitted `typeRef`. A position is a member of its tuple,
not a graph node or an occurrence. Its identity is `<tuple node identity>/p<i>`.
The reader SHALL admit the V2 declaration node `model`/`tuple_value_type` with
family `tuple`, required `ModelOwner`, self `semantic_type`, empty aggregate
body and dependencies, and absent `declaration`, `nominal_identity_preimage`
and `recursion_group`. Owner recovery SHALL match the owner identity to the
selected model row and its node identity to that row's tuple declaration.

The tuple node carries its authored structural id and exact selected `ModelOwner`. Reordering positions or changing a position type changes the selected document's complete typed content; the reader checks its model-root owner and position type joins independently of the tuple node id. The package name and valid declared path can remain unchanged.
The reader SHALL use that selected document for a tuple
position's type. A `quire.op.tuple.project` member of kind `position` SHALL
resolve only an existing position of its named tuple declaration and derive the
result from that position's `typeRef` under required presence and `[1, 1]`
multiplicity. An anonymous `composite_type`/`tuple` remains a different type.

### Variant payloads and uses

The selected-document intake SHALL retain each variant case's optional
`payloadType` instead of reducing the declaration to a set of case names. A
`model`/`variant_type` declaration remains one owner-keyed, self-typed node:
its family is `enum` when no case has a payload and `union` when at least one
case does. In the latter case the reader SHALL derive member order by ascending
UTF-8 case name, arity zero for a nullary case and arity one for a payload case;
the payload type is the case's resolved `payloadType` under required presence
and `[1, 1]` multiplicity at position zero. Changing the payload changes the selected document and complete package value, while the declared variant path and package name remain unchanged. The reader SHALL apply QSpec FR-440's
`union_value`, `case`, equality and structural-leaf rules to this model union,
including a nullary `union_value` with an empty payload and a recursive type's
`recursion:<d>` leaf only when text is reachable. An `enum_value` SHALL refuse
for a union family; a `union_value` SHALL refuse for an enum family. Tuple
positions and variant payloads MAY refer to other tuples or variants,
including recursive declarations, without graph `recursion_group`s.

### Refusal order, writer and conformance

The reader SHALL keep FR-038 and QSpec FR-322's stage order and bounded work
charges. A malformed tuple node owner or wire shape refuses at the closed
schema stage; a selected-document defect refuses in step 1 at the model
selection row before graph admission; a malformed or duplicate structural id refuses at the graph stage; and an owner referring to a different or absent kind
refuses at the owner join. Within step 1 the reader SHALL check tuples in
document order before variants, with each tuple's duplicate ordinal,
declaration shape, position sequence and type reference in QSpec FR-322's
published order. It SHALL preserve FR-154's typed code and cause through the
existing public refusal conversion, including `unsupported_construct` with
`declaration-form` for an unresolved or unsupported `Variant.payloadType`.
For the published `TK-11` through `TK-35` intake cases, this includes native
and package-bound `Rational`, `Decimal` and bare or unbounded `Text`, their
resolving aliases, admitted bounded `Text`, admitted `Float32` and `Float64`
aliases (including an alias chain), and admitted unbounded `Bytes` and
`JsonObject`. Tuple positions and variant payloads retain the different
QSpec refusal codes, causes and loci. A tuple defect precedes a later variant
defect, including when the tuple's type has no admitted V2 mapping.

The writer of [FR-051](./FR-051-encode-checked-package-v2-envelope.md) SHALL
encode supplied `model`/`tuple_value_type` and payload-bearing
`model`/`variant_type` nodes through the same typed envelope, retaining each
node's semantic form, owner, family, body, node key, source occurrence and
selection values. The reader, not the writer, decides whether those bytes are
admissible. The writer SHALL NOT synthesize a model document, derive missing
positions or payloads, or rewrite the supplied package name.

The QSpec checkout named by `QUIRE_SPECIFICATION_DIR` owns
`proposals/checked-package-v2/fixtures/positive-model-tuple-union.json` and
`proposals/checked-package-v2/model-tuple-union-vectors.json`. IR conformance SHALL read the current QSpec tuple and union declaration, member and application cases directly from that checkout. It SHALL read the vector's selected document as the full Semantic IR model evidence and join the declared node paths, owners, body types and member references against it. It SHALL read the published positive package through the production reader and compare observed structural ids, family, position, field-type, union-value, enum-value, equality, recursion, intake and wire outcomes with the authoritative cases. A case changing a selected document or wire package SHALL be checked as a complete structural value with its declared package name and node ids; no content-derived id is recomputed. The typed writer SHALL round trip the positive
package through the production reader without losing tuple positions or
variant payload types.

Conformance SHALL replay every published `intake_cases` entry, including
admitted controls, and every `wire_cases` entry; it SHALL compare each
refusal's code, cause, RFC 6901 location and specified reader stage. The
vector case lists are authoritative, so an added published case is not
silently skipped by an in-repo fixed count or range.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-069-AC-1 | The authoritative positive model tuple and union package admits with `model`/`tuple_value_type` declarations for `Pair`, `Swap`, `Nest` and `Cell`, owner-nested position identities, ordered zero-based positions and their exact type references; no position is a graph node or source-map key. The production reader preserves the declaration nodes' structural ids and checks their paths against their declared qualified names; aliases do not change those names. | Test |
| FR-069-AC-2 | Changing only a tuple position type or order changes the model-owned tuple declaration's content while its authored structural node id can remain unchanged. Two named tuples of identical shape have distinct owner-nested structural paths under the same owning package. A test compares each declared node path and owning package name as the tuple declaration identity, and compares the retained ordered positions and type references as its content. | Test |
| FR-069-AC-3 | A `quire.op.tuple.project` over a selected model tuple resolves each valid position to the published type, including a tuple or union type reference; an out-of-range position or wrong declaring tuple refuses `ill_typed`/`operator-ineligible` at the application member. An anonymous structural tuple is not accepted as that model declaration. | Test |
| FR-069-AC-4 | PLANNED/UNRUN (IR-1141). A payload-free variant has family enum; mixed nullary/payload variants have family union, cases sort by UTF-8 name, and payload cases retain exactly one resolved position-zero type. Adding or removing the first payload changes the selected document and complete retained model body without requiring a different structural id. | Test |
| FR-069-AC-5 | The published `union_value`, `enum_value`, equality, case-arm and structural-leaf cases pass through the production reader: a nullary union case uses an empty-payload `union_value`; union-family `enum_value` and enum-family `union_value` refuse with the specified typed mismatch; recursive tuple/union declarations admit without a graph recursion group and their leaf walk terminates with QSpec's reachable-text rule. | Test |
| FR-069-AC-6 | Withdrawn: this historical numbered intake and wire replay criterion included a stale tuple key refusal. No content-derived tuple key or stale-key comparison remains under QSpec FR-322-AC-160 and FR-322-AC-169 (withdrawn); the refusal has no subject. Its former `TK-01` through `TK-35` and `TW-00` through `TW-26` case numbers remain here for trace history. FR-069-AC-8 requires replay of every currently published tuple/union intake and wire case, including admitted and refused controls, without a fixed count. | Withdrawn |
| FR-069-AC-7 | `encode_checked_package_v2` emits the supplied tuple and payload-variant nodes under the QSpec closed V2 envelope and preserves the supplied package name and selection rows. The positive typed package encodes canonically and its bytes read back with the same admitted positions, payload types, structural node ids and package name; a malformed or duplicate node id is refused by the reader. | Test |
| FR-069-AC-8 | PLANNED/UNRUN (IR-1141). make conformance-qspec loads the current positive fixture, model-tuple-union vectors and selected document directly from QUIRE_SPECIFICATION_DIR; it replays every published tuple/union intake and wire case against production intake and reader paths, compares admission or refusal code, cause, RFC 6901 location and reader stage with the published result, and fails on an admitted partial package or panic for a malformed declaration. Missing or malformed files and omitted cases fail. No QSpec artifact is copied into IR, and no identity hash is computed. | Test, Inspection |

## Dependencies

[FR-038](./FR-038-consume-checked-package-v2.md) owns the V2 reader and refusal
surface. [FR-051](./FR-051-encode-checked-package-v2-envelope.md) owns the
typed envelope writer. QSpec FR-154 and FR-208 own the admitted source model;
QSpec FR-322 owns the V2 schema, structural ids and vectors; QSpec FR-440
owns union operations. FCD owns the Semantic IR producer and QSL owns source
intake. Their changes are separate from this IR requirement.
