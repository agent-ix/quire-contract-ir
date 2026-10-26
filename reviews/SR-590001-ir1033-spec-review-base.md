---
id: SR-590001
title: "IR-1033 owner-model edge spec review for PR 590"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 590 diff: spec/checked_package/functional/FR-064-bind-protocol-clause-to-owner-model.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/spec.md"
review_set: base
---

# SR-590001: IR-1033 owner-model edge spec review

## Summary

Ticket: IR-1033. Reviewed PR #590 at 8f7864cbc503275885f82b98a6a85599cf5aae18 against QSpec 9f10eddf044a08145a307379aae6fdedcaf69e30 (FR-464-AC-16, FR-322-AC-130, FR-375-AC-23 and both closed V2 schemas). Base checklist, EARS, integrity, object ownership and dependency semantics were checked over the diff. No applicable AssuranceProfile was found under IR spec/. No QSpec schema or fixture was copied.

## Examined scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-064-AC-1 | examined | An external consumer constructs a complete typed V2 package with two selected local model declarations and one protocol clause whose `owner_model` names the exact model over which it was declared, calls `encode_checked_package_v2`, and reads the bytes through the public strict reader. The admitted typed graph retains the edge as a local `NodeRef`, exact dependencies include it, the occurrence-free projection includes it, and canonical read/write/read retains the same clause key and package id. |
| FR-064-AC-2 | examined | A clause's `quire.application-node/v1` preimage contains `owner_model` as its local node id; an otherwise identical non-clause application preimage omits it. Retargeting the edge to a second valid local model and recomputing dependencies, clause key, downstream keys and package id changes the clause key and package id; retaining the old clause key while recomputing the envelope identity refuses `invalid_package`/`stale-node-key` at that clause's `node_id`. |
| FR-064-AC-3 | examined | A missing `owner_model` on a protocol clause, a present `owner_model` on a non-clause node, or an external `{package, node}` value in the local-only edge refuses at closed wire validation before graph or protocol checks. A clause whose otherwise valid local edge target is omitted from, duplicated in, or out of order in `dependencies` refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{n}/dependencies` before stale-key or clause-meaning checks. |
| FR-064-AC-4 | examined | With wire, graph and identities valid, an `owner_model` pointing at a non-model node, a model declaration outside the selected local model, or a model different from the clause's source `over` model refuses at the edge under QSpec FR-464's typed model-binding check before clause interpretation. A separate earlier wire, graph or stale-key fault wins when paired with such a meaning defect; no edge is inferred from the `over` parameter. |
| FR-064-AC-5 | examined | A protocol refinement whose complete concrete catalog includes classes from an admitted clause owned by its selected concrete model is admitted even when no step row names those classes. A clause whose edge names another model contributes no class to that catalog, even when its `over` parameter or name resembles the selected model. A catalog missing an owned clause's class or including a foreign clause's class refuses under FR-038-AC-325 after protocol-body checks; an earlier refinement-step row/catalog defect still wins. |
| FR-064-AC-6 | examined | The public typed writer and reader carry `owner_model` through `CheckedSemanticNodeV2`, its raw form and `CheckedNodeProjectionV2` in the existing V2 graph and envelope. No other node accepts the member, and no copied QSpec schema, fixture, parallel untyped writer, compatibility reader or fallback owner inference is added. |
| FR-064 Description and Behavior | examined | The owner edge is required on protocol clauses, identity-bound, and used for class catalogs; the reader is also told to match the clause source `over` model. |
| FR-038 protocol and key paragraphs | examined | The new prose uses the identity-bound edge for catalog ownership and adds it to the application preimage only for protocol clauses. |
| spec/spec.md FR-064 index | examined | The new index entry links FR-064. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-064-AC-4 and Behavior require a reader to compare `owner_model` with the clause source `over` model and refuse a mismatch. QSpec FR-464 defines the producer edge as the exact source owner, but its reader check only resolves a selected local model declaration and exact dependency; FR-375 uses the edge as authoritative. The reader cannot verify source intent from this package, and FR-064-AC-5 itself requires a foreign edge to exclude a clause even when its `over` parameter resembles the selected concrete model. Remove the invented reader match while preserving the producer equality, selected-model check, exact dependency and foreign-clause exclusion. | spec/checked_package/functional/FR-064-bind-protocol-clause-to-owner-model.md:84,105-106 |

## Verdict

Changes needed. FND-001 is a spec contradiction with QSpec and between AC-4 and AC-5. The remaining scoped identity, dependency, wire-shape, and catalog requirements match the cited QSpec requirements and published schemas.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@fa9fa4b5850e03a8fe0bb0afbbf50894229bbc84. The same-PR fix separates the source producer obligation from the typed encoder and strict reader obligations. QSpec FR-464 source equality remains explicit; the reader checks selected local model declaration, exact dependency, identity and class-catalog ownership without a source-level over comparison. No original adverse case was dropped; AC-4 retains non-model and unselected-model refusals and source producer A/B conformance.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-4 and Behavior no longer require an unavailable source-over reader join; AC-5 uses authoritative owner edge and remains consistent. |

## Final verdict

PASS for this scoped spec diff after FND-001 fix, subject to the review artifact being committed on the final PR head. FR-064 criteria remain Planned until implementation evidence exists.
