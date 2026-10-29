---
id: SR-612078
title: spec-review/failure-domain review of IR-1077
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-061-admit-refinement-nodes.md;
  spec/checked_package/matrix/TC-062-refinement-reader-writer.md
review_set: subset
---

## Summary

Reviewed PR #642 against main and QSpec . Wrong kind, missing anchor, wrong operation name, foreign owner, inheritance, and stage precedence are specified. No applicable AssuranceProfile or enforced review policy was found in the changed module.

## Verdict

**PASS** — Wrong kind, missing anchor, wrong operation name, foreign owner, inheritance, and stage precedence are specified.

## Scope

- FR-061-AC-14 (examined): In a model-only refinement without `class_catalog`, a step whose `source_class` names an operation and whose local `source_node` is that operation's anchor admits when the anchor `context` is the object type, process or state machine declaring the operation in the selected concrete model subject. The same holds when the selected type inherits the operation: the anchor stays at its declaring type, while the refinement `semantic_type` stays at the selected concrete model's canonical `model_root`. The admitted row and identities survive read/write/read.
- FR-061-AC-15 (examined): For a catalog-free operation step, an absent `source_node` refuses `invalid_semantic_graph` at the step entry in the earlier closed-body check; an unresolved `source_node` refuses `missing_declaration`/`missing-name` at `source_node`. A present wrong-kind node or an anchor for a foreign or wrong selected model subject refuses `invalid_model_binding`/`malformed-declaration` at `source_node`; an anchor whose operation name differs from `source_class` refuses the same pair at `source_class`. None admits a partial package.
- FR-061-AC-16 (examined): With a catalog-free step defect and an earlier wire, identity, canonical-order, header or catalog defect, the earlier FR-375 stage wins; with two step-row defects, the first row in canonical order wins. A refinement with `class_catalog` continues to join a step to its concrete catalog entry, including the existing missing-class and wrong-node refusals, independently of the catalog-free anchor rule.
- TC-062 (examined): For FR-061-AC-14 through AC-16, construct catalog-free model-only refinements with operation steps anchored at declaring object-type, process and state-machine nodes of the selected concrete model.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
