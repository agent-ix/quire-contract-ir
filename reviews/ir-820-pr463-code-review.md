---
id: SR-12675
title: "IR-820 PR 463 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-820; PR #463; FR-052, FR-052-AC-24, TC-452, spec/model/matrix/tests.md, V2 node and inventory types"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-052
    type: reviews
---

## Summary

The diff changes only FR-052 and TC-452 prose. The specified key and fact match the V2 node fields; the public enum still uses `CheckedSemanticId`, which the PR correctly leaves planned for a separate code change. No code, test body, dependency or workflow changed.

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

PASS. No code-review defect in the changed diff; the stale matrix status is recorded in the integrity review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
