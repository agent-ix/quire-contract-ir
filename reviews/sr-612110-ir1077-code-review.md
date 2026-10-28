---
id: SR-612110
title: "IR-1077 PR 651 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1077; PR #651; refinement.rs, checked_package_v2_refinement.rs; FR-061-AC-14/15/16, TC-062"
review_set: subset
---

## Summary

Reviewed PR #651 head c3023198048a827a3595db4678c72b12e40b09b4 against merged FR-061-AC-14/15/16 and TC-062, QSpec FR-342/FR-375, and IR-874 catalog-present code. Both changed files were examined; no workflow or dependency files changed, no copied artifacts were introduced, and no applicable AssuranceProfile was found. The FR-040 process/state-machine amendment remains an owner-held draft PR #650.

## Verdict

PASS for the scoped Rust diff. The new catalog-free join checks local anchor kind, typed anchor context, declaring model kind, selected model identity and operation name. Existing present-catalog dispatch stays in place. Process/state-machine admission remains owner-held at FR-040 PR #650 and is not credited; test gaps are in SR-612111.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-061-AC-14 (examined): In a model-only refinement without `class_catalog`, a step whose `source_class` names an operation and whose local `source_node` is that operation's anchor admits when the anchor `context` is the object type, process or state machine declaring the operation in the selected concrete model subject. The same holds when the selected type inherits the operation: the anchor stays at its declaring type, while the refinement `semantic_type` stays at the selected concrete model's canonical `model_root`. The admitted row and identities survive read/write/read.
- FR-061-AC-15 (examined): For a catalog-free operation step, an absent `source_node` refuses `invalid_semantic_graph` at the step entry in the earlier closed-body check; an unresolved `source_node` refuses `missing_declaration`/`missing-name` at `source_node`. A present wrong-kind node or an anchor for a foreign or wrong selected model subject refuses `invalid_model_binding`/`malformed-declaration` at `source_node`; an anchor whose operation name differs from `source_class` refuses the same pair at `source_class`. None admits a partial package.
- FR-061-AC-16 (examined): With a catalog-free step defect and an earlier wire, identity, canonical-order, header or catalog defect, the earlier FR-375 stage wins; with two step-row defects, the first row in canonical order wins. A refinement with `class_catalog` continues to join a step to its concrete catalog entry, including the existing missing-class and wrong-node refusals, independently of the catalog-free anchor rule.
- TC-062 (examined): For FR-061-AC-14 through AC-16, construct catalog-free model-only refinements with operation steps anchored at declaring object-type, process and state-machine nodes of the selected concrete model.
