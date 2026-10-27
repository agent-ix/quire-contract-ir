---
id: SR-610019
title: "spec-review/failure-domain of STD-498 FR-064 owner_model wording"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-064-bind-protocol-clause-to-owner-model.md"
review_set: subset
---

## Summary

Independent spec-review/failure-domain of quire-contract-ir PR #600 at 48dd790dac48f36b526f6f47dfe3a0b4f2b2bbb1 against QSpec main 46c7a46582f86c177d339f7934db9919bc042cae. Ticket STD-498.

## Examined scope

- FR-064-AC-1 (examined): An external consumer constructs a complete typed V2 package with two selected local model declarations and one protocol clause whose `owner_model` names the exact model over which it was declared, calls `encode_checked_package_v2`, and reads the bytes through the public strict reader. The admitted typed graph retains the edge as a local `NodeRef`, exact dependencies include it, the occurrence-free projection includes it, and canonical read/write/read retains the same clause key and package id.
- FR-064-AC-2 (examined): A clause's `quire.application-node/v1` preimage contains `owner_model` as its local node id; an otherwise identical non-clause application preimage omits it. Retargeting the edge to a second valid local model and recomputing dependencies, clause key, downstream keys and package id changes the clause key and package id; retaining the old clause key while recomputing the envelope identity refuses `invalid_package`/`stale-node-key` at that clause's `node_id`.
- FR-064-AC-3 (examined): A missing `owner_model` on a protocol clause, a present `owner_model` on a non-clause node, or an external `{package, node}` value in the local-only edge refuses at closed wire validation before graph or protocol checks. A clause whose otherwise valid local edge target is omitted from, duplicated in, or out of order in `dependencies` refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{n}/dependencies` before stale-key or clause-meaning checks.
- FR-064-AC-4 (examined): With wire, graph and identities valid, an `owner_model` pointing at a non-model node, a source-local declaration, or a model declaration owned by no selected declaration refuses at the edge under QSpec FR-464's typed model-binding check before clause interpretation. A separate earlier wire, graph or stale-key fault wins when paired with such a meaning defect. A source producer given a clause declared `over` model A emits the edge to A's object-type node even when selected model B exists; a candidate B edge fails producer conformance, without claiming an IR reader source-level comparison. The I
- FR-064-AC-5 (examined): A protocol refinement whose complete concrete catalog includes classes from an admitted clause whose `owner_model` is the object-type node of a model owned by its selected concrete declaration is admitted even when no step row names those classes. A clause whose edge names a model owned by another declaration contributes no class to that catalog, regardless of similar declaration names or binder types. A catalog missing an owned clause's class or including a foreign clause's class refuses under FR-038-AC-325 after protocol-body checks; an earlier refinement-step row/catalog defect still wins.
- FR-064-AC-6 (examined): The public typed writer and reader carry `owner_model` through `CheckedSemanticNodeV2`, its raw form and `CheckedNodeProjectionV2` in the existing V2 graph and envelope. No other node accepts the member, and no copied QSpec schema, fixture, parallel untyped writer, compatibility reader or fallback owner inference is added.
- FR-064 (examined): When a source clause is declared over model M, its producer SHALL set owner_model to the object-type node of M, the object type of the clause over input, a model owned by a selected declaration.

## Verdict

PASS — no distinct finding in this method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
