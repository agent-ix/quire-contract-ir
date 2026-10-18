---
id: SR-12681
title: "IR-820 PR 465 Traceability gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-820; PR #465; FR-052-AC-24, TC-452, FR-019 public items, source-obligation model and tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-052
    type: reviews
---

## Summary

The computed coverage rises from 533/602 to 534/602 backed. AC-24 is the only row newly backed, leaving 39 unbacked and zero contradicted status rows. The new test tags AC-24 and TC-452, exercises the production census, and verifies two inspectable conflict rows and a rejected occurrence relation.

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

PASS for the changed scope. Owner-native adapters remain planned under AC-7, AC-9 through AC-11, and AC-16; no owner-issued completeness is claimed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |


## Coverage

Plan completion: not assessed. Base 533/602 backed, 40 unbacked, 0 contradicted; head 534/602 backed, 39 unbacked, 0 contradicted. AC-24 is the only newly backed row. Optional repository-wide semantic review was not requested; the changed AC-24 intent, test and code were checked for this PR.
