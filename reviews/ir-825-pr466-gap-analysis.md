---
id: SR-12741
title: "IR-825 PR 466 Traceability gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-825; PR #466; FR-052 AC-7, AC-9 through AC-11, AC-16, AC-24 through AC-27; TC-452 and model TestMatrix"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-052
    type: reviews
---

## Summary

Compared with current main at a69587f, PR head loses the AC-24 trace binding and omits the newly merged FR-051-AC-4 while adding three intentionally planned and untagged owner criteria. Main has 535/603 backed and 39 unbacked; head has 533/605 backed and 43 unbacked, with zero contradicted statuses in both.

## Scope

| ID | Path | Role | Excerpt |
| --- | --- | --- | --- |
| FR-052-AC-25 | spec/model/functional/FR-052-source-obligation-correspondence.md | examined | Given an owner-issued IR V2 reader result for a selected artifact, the model-owned adapter enumerates every node in an admitted package as its `node_id` plus native `semantic_type: CheckedNodeId` fact and every source occurrence under its node key, and issues `Complete` only after the graph reaches its normal end. |
| FR-052-AC-26 | spec/model/functional/FR-052-source-obligation-correspondence.md | examined | Given a selected QSL checked request, the QSL-owned adapter enumerates every `CheckedGraph::requirements()` key and every native pending obligation, preserving each requirement's wire node bytes, occurrence role and ordinal and each obligation's declaration, unit, expression, byte span, kind and predicate target. |
| FR-052-AC-27 | spec/model/functional/FR-052-source-obligation-correspondence.md | examined | Given a `quire-code-rs` selected production module, the owner adapter retains that module and every function fact from its complete native parse, including a parsed module with zero functions, and issues `Complete` only after the selected module's normal terminal state. |
| FR-052-AC-16 | spec/model/functional/FR-052-source-obligation-correspondence.md | context_only | With PLAT-1165's real Quire producer, normal exhaustion of the validated selected scope issues `Complete` with every authored criterion and its provenance; |
| FR-052-AC-7 | spec/model/functional/FR-052-source-obligation-correspondence.md | context_only | Complete Quire scope covers every authored acceptance criterion in the bound selected corpus and declared obligation sources, including unbound criteria; |
| FR-052-AC-9 | spec/model/functional/FR-052-source-obligation-correspondence.md | context_only | For each of the four owners, the native enumerator or typed adapter issues a `Complete` outcome only after the selected named scope reaches its normal end; |
| FR-052-AC-10 | spec/model/functional/FR-052-source-obligation-correspondence.md | context_only | For each owner, stopping before normal exhaustion issues `Incomplete` with retained partial rows and the typed cause the owner actually reports. |
| FR-052-AC-11 | spec/model/functional/FR-052-source-obligation-correspondence.md | context_only | The neutral model retains an owner-issued outcome only when its source, owner, scope and observed count match the retained rows; |
| FR-052-AC-24 | spec/model/functional/FR-052-source-obligation-correspondence.md | examined | Conflicting inventory facts (two `SemanticNode` rows with one `CheckedNodeId` node key and differing `semantic_type: CheckedNodeId` facts) receive their named deterministic disposition, `ConflictingIdentity`; each conflicting entry remains inspectable, and no occurrence under a conflicting node enters an accepted relation. |
| FR-052 status | spec/model/functional/FR-052-source-obligation-correspondence.md | examined | AC-24 stays Planned until the public type and |
| TC-452 IR V2 procedure | spec/model/matrix/TC-452-source-obligation-correspondence.md | examined | one node and verify the V2 reader refuses it as `InvalidSourceMap` before an |
| TC-452 owner status | spec/model/matrix/TC-452-source-obligation-correspondence.md | examined | AC-25 through AC-27 are also Planned. |
| TC-452 AC-24 status | spec/model/matrix/TC-452-source-obligation-correspondence.md | examined | Planned until the `SemanticNode` type change and conflicting-facts test land. |
| TC-452 matrix row | spec/model/matrix/tests.md | examined | AC-24, split from AC-3, is Planned until the `SemanticNode` type change and conflicting-facts test land. |
| SourceInventoryIdentity | crates/quire-contract-model/src/source_obligation.rs | context_only | semantic: CheckedSemanticId, |
| FR-051-AC-4 | spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md | context_only | For empty, nested, and deeply nested node bodies, and for present and absent optional projection fields, the borrowed view and equivalent owned preimage have byte-identical canonical encodings and identical typed `package_id` digests. |

## Verdict

FAIL for this head: AC-24 is newly unbacked and FR-051-AC-4 is absent relative to current main. The three new untagged rows are correctly labelled Planned and do not by themselves claim evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-24 is untagged on this PR head although current main has its traced implementation; coverage falls by one backed row beyond the three new Planned criteria. | spec/model/functional/FR-052-source-obligation-correspondence.md:485 |
| FND-002 | medium | This PR head also omits main's backed FR-051-AC-4; integrate current main so the borrowed-projection criterion and its test remain in the candidate. | spec/checked_package/functional/FR-051-encode-checked-package-v2-envelope.md:113 |


## Coverage

Plan completion: not assessed. Current main a69587f has 535/603 backed, 39 unbacked, 0 contradicted; PR head has 533/605 backed, 43 unbacked, 0 contradicted. AC-24 is the fourth newly unbacked row after AC-25 through AC-27; FR-051-AC-4 is absent altogether. Optional repository-wide semantic review was not requested; the three new criteria and TC-452 procedure were checked in this PR.
