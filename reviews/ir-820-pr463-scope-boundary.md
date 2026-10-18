---
id: SR-12716
title: "IR-820 PR 463 scope-boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir; IR-820; PR #463; FR-052, FR-052-AC-24, TC-452, spec/model/matrix/tests.md, V2 node and inventory types"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-052
    type: reviews
---

## Summary

IR owns the selected V2 `SemanticNode` inventory fact and its neutral census conflict result. The checked package supplies `node_id` and `semantic_type`; the IR V2 owner-native adapter remains planned in IR-825. The PR does not add QSL/CG ownership or a package-level semantic id to a node.

## Scope

| ID | Path | Role | Excerpt |
| --- | --- | --- | --- |
| FR-052 | spec/model/functional/FR-052-source-obligation-correspondence.md | examined | `differing semantic type keys conflict, so each conflicting entry shall remain` |
| FR-052-AC-24 | spec/model/functional/FR-052-source-obligation-correspondence.md | examined | Conflicting inventory facts (two `SemanticNode` rows with one `CheckedNodeId` node key and differing `semantic_type: CheckedNodeId` facts) receive their named deterministic disposition, `ConflictingIdentity`; each conflicting entry remains inspectable, and no occurrence under a conflicting node enters an accepted relation. |
| TC-452 | spec/model/matrix/TC-452-source-obligation-correspondence.md | examined | Planned until the `SemanticNode` type change and conflicting-facts test land. |
| TC-452 matrix row | spec/model/matrix/tests.md | examined | AC-24, split from AC-3, is Planned until Linear IR-820 settles the `SemanticNode` fact. |
| CheckedSemanticNodeV2 | crates/quire-contract-model/src/checked_package/v2/mod.rs | context_only | `pub semantic_type: CheckedNodeId,` |
| SourceInventoryIdentity | crates/quire-contract-model/src/source_obligation.rs | context_only | `semantic: CheckedSemanticId,` |

## Verdict

PASS. The amended boundary aligns with the existing V2 node fields and retains the adapter as follow-on work.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
