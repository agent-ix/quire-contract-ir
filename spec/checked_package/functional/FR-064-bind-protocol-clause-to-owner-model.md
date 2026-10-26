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
---

# FR-064: Bind each checked protocol clause to its exact model owner

## Description

When a checked-package V2 graph contains a `protocol`/`protocol_clause` node,
the Contract IR model and typed writer SHALL carry its `owner_model` local
`NodeRef`, and the strict reader SHALL admit the clause only when that edge
names the exact selected local model declaration it belongs to. The edge
determines clause-to-model ownership for refinement class catalogs. IR
preserves the QSpec FR-322 and FR-464 wire, graph, identity and protocol
reader order; it SHALL NOT infer this edge from the `over` parameter's type,
the clause name, or a refinement step row.

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

The reader SHALL apply QSpec's closed wire presence and `NodeRef` shape
checks before graph or meaning checks. An absent `owner_model` on a clause,
or a present edge on another node, SHALL refuse as `malformed_wire` at the
offending node/member; an external package key in the local-only edge SHALL
also refuse under the wire shape. The graph-shape stage SHALL require the
target in exact dependencies, refusing `invalid_semantic_graph` at
`/semantic_graph/nodes/{n}/dependencies` before key and clause checks. After
the earlier wire, graph, key, declaration, state and other body steps, the
protocol step SHALL require the target to resolve to a selected local model
declaration node and to match the clause's source `over` model exactly.
Its QSpec typed edge or model-binding refusal SHALL precede clause
interpretation. No general key stage SHALL invent or repair the edge.

The refinement class-catalog graph join SHALL select every admitted clause
whose `owner_model` equals the selected concrete model, including a clause
with no named step row, and SHALL exclude clauses owned by another model.
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
| FR-064-AC-4 | With wire, graph and identities valid, an `owner_model` pointing at a non-model node, a model declaration outside the selected local model, or a model different from the clause's source `over` model refuses at the edge under QSpec FR-464's typed model-binding check before clause interpretation. A separate earlier wire, graph or stale-key fault wins when paired with such a meaning defect; no edge is inferred from the `over` parameter. | Test |
| FR-064-AC-5 | A protocol refinement whose complete concrete catalog includes classes from an admitted clause owned by its selected concrete model is admitted even when no step row names those classes. A clause whose edge names another model contributes no class to that catalog, even when its `over` parameter or name resembles the selected model. A catalog missing an owned clause's class or including a foreign clause's class refuses under FR-038-AC-325 after protocol-body checks; an earlier refinement-step row/catalog defect still wins. | Test |
| FR-064-AC-6 | The public typed writer and reader carry `owner_model` through `CheckedSemanticNodeV2`, its raw form and `CheckedNodeProjectionV2` in the existing V2 graph and envelope. No other node accepts the member, and no copied QSpec schema, fixture, parallel untyped writer, compatibility reader or fallback owner inference is added. | Inspection |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns strict V2 admission,
  check order, identity and refinement class-catalog graph joins.
- [FR-051](./FR-051-encode-checked-package-v2-envelope.md) owns the public
  typed writer; [FR-061](./FR-061-admit-refinement-nodes.md) owns the closed
  refinement node.
- QSpec FR-322-AC-130, FR-375-AC-23 and FR-464-AC-16 own the published
  wire and preimage shapes, exact owner relation and catalog semantics.

## Status

Planned. The measured IR model has no `owner_model` member on
`CheckedSemanticNodeV2` or `CheckedNodeProjectionV2`, so its closed reader
and typed writer cannot yet round-trip the merged QSpec edge.
