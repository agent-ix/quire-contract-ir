---
id: FR-064
title: "Bind each checked protocol clause to its exact model owner"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-061
    type: references
  - target: ix://agent-ix/quire-specification/FR-322-AC-130
    type: references
  - target: ix://agent-ix/quire-specification/FR-375-AC-23
    type: references
  - target: ix://agent-ix/quire-specification/FR-464-AC-16
    type: references
  - target: ix://agent-ix/quire-specification/FR-464-AC-20
    type: references
---

# FR-064: Bind each checked protocol clause to its exact model owner

## Description

When a source clause is declared over model `M`, its producer SHALL set
`owner_model` to the object-type node of `M`, the object type of the clause's
`over` input, a model owned by a selected declaration. The Contract IR model and
typed writer SHALL carry the supplied local `NodeRef`. The strict reader SHALL
require it to name a selected local model declaration. The reader uses the edge
as the authoritative clause-to-model ownership for refinement class catalogs.
IR preserves the QSpec FR-322 and FR-464 wire, graph, identity and protocol
reader order. The reader SHALL NOT infer or cross-check source ownership from
the `over` parameter's type, the clause name, or a refinement step row.

## Inputs

A typed V2 graph supplied to the public encoder of
[FR-051](./FR-051-encode-checked-package-v2-envelope.md), or untrusted V2
bytes, selections, evidence and limits supplied to the strict reader of
[FR-038](./FR-038-consume-checked-package-v2.md). The authoritative edge
shape and preimage are QSpec FR-322, FR-464 and the published
`proposals/checked-package-v2/schema.json` and
`node-identity-preimage.schema.json` at the selected QSpec source.

## Outputs

Canonical bytes and an admitted typed graph retaining the owner edge and
identities, or one typed, located refusal for the first failing check, with
no partial admitted package.

## Behavior

The public `CheckedSemanticNodeV2` SHALL carry `owner_model` as a typed local
node reference, required exactly when `node_tag` is `protocol` and
`semantic_form` is `protocol_clause`, and absent on every other node. Its raw
reader form and `CheckedNodeProjectionV2` SHALL retain the same edge. The
public V2 writer SHALL encode the edge once in the graph node and in its
occurrence-free identity projection, without adding another envelope or a
second graph. The reader SHALL retain the admitted edge through typed
read/write/read. The edge is distinct from the existing structural `owner`
member and from `semantic_type`.

For a clause, the closed `quire.application-node/v1` preimage SHALL include
`owner_model` as the node id, alongside its existing version, tag, form,
semantic type, declaration, recursion and body. Other application-node
preimages SHALL omit the member. The clause's exact local `dependencies`
include the edge target together with its body targets, once each in
ascending node-id digest order. The package identity projection SHALL include
the edge. With all dependent identities recomputed, retargeting only
`owner_model` to a different valid local model SHALL change the clause key
and `package_id`; retaining the old key SHALL refuse `invalid_package`/
`stale-node-key` at that clause's `node_id` under the application-key check.

The source producer SHALL set `owner_model` to the exact local object-type
node of the model that the clause's source `over` type names, which is owned by a
selected declaration. The public typed
encoder is a byte-construction API: it retains the supplied edge and has no
source declaration with which to check this producer obligation.

The reader SHALL apply QSpec's closed wire presence and `NodeRef` shape
checks before graph or meaning checks. An absent `owner_model` on a clause,
or a present edge on another node, SHALL refuse as `malformed_wire` at the
offending node/member; an external package key in the local-only edge SHALL
also refuse under the wire shape. The graph-shape stage SHALL join a
model/object-type target to its selected declaration at the target's own
`node_id`, refusing `missing_declaration`/`missing-selection` there if it
cannot join. It SHALL require the target in exact dependencies, refusing
`invalid_semantic_graph` at `/semantic_graph/nodes/{n}/dependencies` before
key and body checks. After the earlier wire, graph, key, declaration, state,
refinement-step and protocol member checks, the clause step SHALL check the
typed edge first. A target naming no local node SHALL refuse
`missing_declaration`/`missing-name` at `owner_model`; a local target that is
not a selected object-type declaration SHALL refuse `invalid_model_binding`/
`malformed-declaration` there, before the same clause's activation, capture or
slot checks. The reader SHALL use the admitted edge without a source-level
`over` comparison; no general key stage SHALL invent or repair the edge.

The refinement's concrete `semantic_type` SHALL resolve to the canonical
`model_root` of its selected concrete model package. That root identifies the
selected package as a whole; it is not an object-type declaration node. For
each admitted clause, the reader SHALL resolve `owner_model` through the
selected declaration and its package to an exact object-type node owned by
that declaration. The refinement class-catalog graph join SHALL select every
admitted clause whose resolved owner belongs to the refinement's selected
concrete declaration, including a clause with no named step row, and SHALL
exclude clauses whose edge names a model owned by another declaration or an
unselected package. It SHALL NOT compare the root node id directly to the
owner node id, treat a matching name as ownership, or accept a selected
declaration's non-object-type node as an owner.
It SHALL compare the complete selected model class inventory under
[FR-038](./FR-038-consume-checked-package-v2.md) and QSpec FR-375. A
refinement-step row/catalog defect SHALL precede the protocol step; a
protocol-body defect SHALL precede the later class-catalog graph join. A
catalog made complete only by a foreign clause or by an inferred `over` type
refuses under the existing class-catalog rules, with no synthesized
class or default owner.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-064-AC-1 | An external consumer constructs a complete typed V2 package with two selected local model declarations and one protocol clause whose `owner_model` names the exact model over which it was declared, calls `encode_checked_package_v2`, and reads the bytes through the public strict reader. The admitted typed graph retains the edge as a local `NodeRef`, exact dependencies include it, the occurrence-free projection includes it, and canonical read/write/read retains the same clause key and package id. | Test |
| FR-064-AC-2 | A clause's `quire.application-node/v1` preimage contains `owner_model` as its local node id; an otherwise identical non-clause application preimage omits it. Retargeting the edge to a second valid local model and recomputing dependencies, clause key, downstream keys and package id changes the clause key and package id; retaining the old clause key while recomputing the envelope identity refuses `invalid_package`/`stale-node-key` at that clause's `node_id`. | Test |
| FR-064-AC-3 | A missing `owner_model` on a protocol clause, a present `owner_model` on a non-clause node, or an external `{package, node}` value in the local-only edge refuses at closed wire validation before graph or protocol checks. A clause whose otherwise valid local edge target is omitted from, duplicated in, or out of order in `dependencies` refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{n}/dependencies` before stale-key or clause-meaning checks. | Test |
| FR-064-AC-4 | With wire, graph and identities valid, a genuinely unselected `model`/`object_type` target refuses `missing_declaration`/`missing-selection` at the target's own `node_id` in the owner join before any body step. A local edge naming no node refuses `missing_declaration`/`missing-name` at `owner_model` in the clause step. A non-model node, a source-local declaration, a selected declaration's model node of any form other than `object_type` (including `state_machine`, `process`, `record_value_type`, a relation node or the selected package's `model_root`) refuses `invalid_model_binding`/`malformed-declaration` at `owner_model` as the clause's first check. An earlier wire, graph, stale-key, refinement-step or protocol member defect wins when paired with a typed edge defect; the typed edge defect wins over the same clause's activation, capture or slot defect. A source producer given a clause declared `over` model A emits the edge to A's object-type node even when selected model B exists; a candidate B edge fails producer conformance, without claiming an IR reader source-level comparison. The IR reader admits a selected B edge as a changed clause identity and does not infer or cross-check it from the `over` parameter. | Test |
| FR-064-AC-5 | A protocol refinement whose concrete `semantic_type` is its selected package's canonical `model_root` and whose complete concrete catalog includes classes from an admitted clause whose distinct `owner_model` is the object-type node of a model owned by its selected concrete declaration is admitted even when no step row names those classes. A clause whose edge names a model owned by another declaration or an unselected package contributes no class to that catalog, regardless of similar declaration names or binder types. A catalog missing an owned clause's class or including a foreign clause's class refuses under FR-038-AC-325 after protocol-body checks; an earlier refinement-step row/catalog defect still wins. | Test |
| FR-064-AC-6 | The public typed writer and reader carry `owner_model` through `CheckedSemanticNodeV2`, its raw form and `CheckedNodeProjectionV2` in the existing V2 graph and envelope. No other node accepts the member, and no copied QSpec schema, fixture, parallel untyped writer, compatibility reader or fallback owner inference is added. | Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns strict V2 admission,
  check order, identity and refinement class-catalog graph joins.
- [FR-051](./FR-051-encode-checked-package-v2-envelope.md) owns the public
  typed writer; [FR-061](./FR-061-admit-refinement-nodes.md) owns the closed
  refinement node. Its Behavior "Keys, graph and identity" and AC-1/AC-3/AC-7
  require the same canonical selected `model_root` for `semantic_type` and
  `abstract`; the clause's object-type `owner_model` remains distinct.
- QSpec FR-322-AC-130, FR-375-AC-23 and FR-464-AC-16 own the published
  wire and preimage shapes, exact owner relation and catalog semantics.

## Status

Partially implemented. The typed `owner_model` carrier, writer and basic
reader admission are present. The reader still needs the selected-declaration
owner join and complete class-catalog reconstruction specified above; this
status does not claim those checks or their test evidence.
