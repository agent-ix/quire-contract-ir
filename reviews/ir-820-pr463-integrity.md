---
id: SR-12677
title: "IR-820 PR 463 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-820; PR #463; FR-052, FR-052-AC-24, TC-452, spec/model/matrix/tests.md, V2 node and inventory types"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-052
    type: reviews
---

## Summary

FR-052 and TC-452 now consistently identify the node key as `node_id: CheckedNodeId` and the extra fact as `semantic_type: CheckedNodeId`. However, the TC-452 row in the companion Test Matrix still says IR-820 has not settled that fact.

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

CONDITIONAL. Update the matrix status reason to say AC-24 awaits implementation and its conflict test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The TC-452 matrix row still says AC-24 waits for IR-820 to settle the SemanticNode fact; this PR settles it and leaves only the code and conflict test planned. | spec/model/matrix/tests.md:34 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 5bbd8790dfda2caebd5acdef574f62c07694262a, the TC-452 matrix row says AC-24 awaits the `SemanticNode` type change and conflicting-facts test. The row no longer claims the spec decision is pending. |
