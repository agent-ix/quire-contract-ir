---
id: SR-202610310131
title: "IR-1209 PR 777 spec-review/ears-conformance review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md"
review_set: subset
---

# IR-1209 PR 777 spec-review/ears-conformance review

## Summary

Ticket IR-1209, PR #777 at frozen head c7ba7205. Changed requirement statements have named actors and testable responses; scoped Quire validation reports zero grammar findings. No applicable AssuranceProfile was found under spec/.

## Examined scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-352 | context_only | When a consumer requests the type of a node in an admitted `CheckedPackageV2`, Contract IR SHALL expose a public, read-only projection from that node's `semantic_type: CheckedNodeId` to `quire_exact::ValueType`. The result SHALL retain the identity and domain of the admitted type or return a typed refusal or incomplete result before exposing any partial value-type view. This is an IR-model read over [FR-038](./FR-038-consume-checked-package-v2.md)'s admitted graph and selected declarations, not another admission path. The dependency direction is IR model to Exact; Exact gains no checked-packag |
| FR-352-AC-11 | examined | QSL checking/emission supplies one original application `NodeKey` and authored `Location` per emitted equality application occurrence, and original dimension-declaration, base-dimension-term and ordered conversion-edge-target `NodeKey`s for every selected quantity fact. Each row carries a full emitted structural node, selected producer document/context and its fact role; application rows also carry exact occurrence. The FR-353 join refuses absent, foreign, wrong-form, duplicate, conflicting or mismatched rows with a typed structural locus and bounded work. Neither `CheckedNodeId`, display name |
| FR-353 | context_only | When a consumer reads an equality application from an admitted `CheckedPackageV2`, Contract IR SHALL return QSV's opaque `AdmittedEqualityDecision` for the decision made by the producing checking stage. That value SHALL preserve the decision needed to form the consumer's `CheckedEquality` node without running a type checker in the reader. An operation identity, operand types, structural domain projection, or catalog law alone SHALL NOT be treated as that decision. |
| FR-353-AC-6 | examined | With selected QSL-stage application `NodeKey`/authored `Location` per exact equality occurrence, dimension-declaration/base-term/ordered edge-target `NodeKey`s, source and compound `UnitId`s, model enum declaration/member `NodeKey`s, Text type identity and Plan type identity rows joined to their exact admitted nodes and selected producer documents/contexts, the reader maps the sidecar's source/conversion types and schedule into QSV admitted descriptors without a checking pass. QSV privately obtains declared keys, declared roots and compound root terms from the witnessed UnitId and retained pat |
| FR-353-AC-7 | examined | For each required witness kind, including application/location, dimension declaration/base terms and ordered edge targets, an absent row returns typed `UnavailableEqualityWitness` naming the missing structural node and kind before QSV minting; a foreign-document, wrong-form, duplicate, conflicting or mismatched row returns a typed refusal at its first structural locus. A work limit equal to the complete application/type/witness/unit/enum representation mapping admits, and one less returns typed exhaustion before the next charge with no partial descriptor or opaque value. Neither path fabricate |
| FR-353-AC-8 | context_only | For a declared quantity whose selected unit has an ordered exact affine conversion path, QSV's private admitted construction folds that path to the required canonical scalar with exact arithmetic and preserves the selected root and edge order. Changing an edge or root refuses the admitted join; arithmetic impossibility or one-less work returns a typed failure before an opaque decision exists. The fold does not choose another route, invent a unit identity, change a conversion target or recompute the selected equality schedule. |

## Verdict

**PASS** — Changed requirement statements have named actors and testable responses; scoped Quire validation reports zero grammar findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
