---
id: SR-12683
title: "IR-820 PR 465 Specification integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-820; PR #465; FR-052-AC-24, TC-452, FR-019 public items, source-obligation model and tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-052
    type: reviews
---

## Summary

FR-052, TC-452 and the matrix row consistently mark AC-24 implemented. The FR-019 public-items table still attributes the exported source-obligation API only through AC-23.

## Scope

| ID | Path | Role | Excerpt |
| --- | --- | --- | --- |
| FR-052-AC-24 | spec/model/functional/FR-052-source-obligation-correspondence.md | examined | Conflicting inventory facts (two `SemanticNode` rows with one `CheckedNodeId` node key and differing `semantic_type: CheckedNodeId` facts) receive their named deterministic disposition, `ConflictingIdentity`; each conflicting entry remains inspectable, and no occurrence under a conflicting node enters an accepted relation. |
| FR-052 status | spec/model/functional/FR-052-source-obligation-correspondence.md | examined | AC-24 is Implemented by the public type and |
| TC-452 | spec/model/matrix/TC-452-source-obligation-correspondence.md | examined | Implemented by a traced synthetic conflicting-facts test. |
| TC-452 matrix row | spec/model/matrix/tests.md | examined | AC-24, split from AC-3, is Implemented with a traced synthetic conflicting-facts test. |
| SourceInventoryIdentity | crates/quire-contract-model/src/source_obligation.rs | examined | semantic_type: CheckedNodeId, |
| census_endpoints | crates/quire-contract-model/src/source_obligation.rs | examined | let conflicting_nodes: BTreeSet<_> = node_facts |
| public consumer | crates/quire-contract-model/tests/source_obligation_public.rs | examined | fn default_feature_consumer_borrows_public_census_and_rows() |
| AC-24 synthetic test | tests/it/source_obligation.rs | examined | fn conflicting_semantic_type_facts_remain_visible_and_block_correspondence() |
| CheckedSemanticNodeV2 | crates/quire-contract-model/src/checked_package/v2/mod.rs | context_only | pub semantic_type: CheckedNodeId, |
| FR-019 public-items row | spec/model/functional/FR-019-rust-library-interface.md | examined | FR-052-AC-17 through AC-23 |
| FR-019-AC-5 | spec/model/functional/FR-019-rust-library-interface.md | examined | The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists |

## Verdict

CONDITIONAL. Extend the existing FR-019 source-obligation row criterion range to include AC-24 without changing the item inventory.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-019 source-obligation public-items row still attributes its API through FR-052-AC-23 although this PR implements AC-24; the criterion attribution is stale. | spec/model/functional/FR-019-rust-library-interface.md:199 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 9ac75d9a889054ac61363bcd0a48711a7e31669c, the FR-019 source-obligation row attributes through AC-24; its public item list is unchanged. |
