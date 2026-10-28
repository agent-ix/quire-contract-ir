---
id: SR-612111
title: "IR-1077 PR 651 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1077; PR #651; refinement.rs, checked_package_v2_refinement.rs; FR-061-AC-14/15/16, TC-062"
review_set: subset
---

## Summary

Reviewed PR #651 head c3023198048a827a3595db4678c72b12e40b09b4 against merged FR-061-AC-14/15/16 and TC-062, QSpec FR-342/FR-375, and IR-874 catalog-present code. Both changed files were examined; no workflow or dependency files changed, no copied artifacts were introduced, and no applicable AssuranceProfile was found. The FR-040 process/state-machine amendment remains an owner-held draft PR #650.

## Verdict

CONDITIONAL. The changed object-type path has public reader/writer evidence, but AC-14 process/state-machine and AC-15/16 adverse controls remain unbacked as listed. Plan completion: not assessed. Optional repository-wide semantic review was not requested; this pass inspected the scoped AC/test/code chain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The only positive catalog-free operation step vectors call the helper with object_type; process and state_machine admission and read/write/read are untested. Those forms also encounter the owner-held FR-040 seam in PR #650, so AC-14 cannot be credited from this PR. | tests/it/checked_package_v2_refinement.rs:1025-1051 |
| FND-002 | medium | The foreign-anchor test covers a different selected model identity, but no vector uses an anchor from another selected concrete subject within the model-selection boundary. Add the wrong-selected-subject refusal separately with exact code, cause and source_node pointer. | tests/it/checked_package_v2_refinement.rs:1136-1205 |
| FND-003 | medium | The step-order test checks an out-of-order array without a simultaneous anchor/name row mismatch. AC-16 and TC-062 require canonical-order versus later row-defect precedence, then repair of only the order defect to expose the row refusal. The other four earlier-stage combined controls are present. | tests/it/checked_package_v2_refinement.rs:1209-1353 |
| FND-004 | medium | Existing present-catalog tests assert a missing row class and a wrong catalog entry association, but no valid present-catalog baseline with a step whose source_node is changed away from its matching concrete catalog entry. Add the exact wrong-node row refusal to establish that the catalog-present branch remains independent of the new catalog-free join. | tests/it/checked_package_v2_refinement.rs:3173-3250 |
| FND-005 | medium | Computed Quire matrix leaves FR-061-AC-14/15/16 untagged; new tests carry only TC-062 trace tags. | tests/it/checked_package_v2_refinement.rs:1010 |

## Coverage

- FR-061-AC-14 (examined): In a model-only refinement without `class_catalog`, a step whose `source_class` names an operation and whose local `source_node` is that operation's anchor admits when the anchor `context` is the object type, process or state machine declaring the operation in the selected concrete model subject. The same holds when the selected type inherits the operation: the anchor stays at its declaring type, while the refinement `semantic_type` stays at the selected concrete model's canonical `model_root`. The admitted row and identities survive read/write/read.
- FR-061-AC-15 (examined): For a catalog-free operation step, an absent `source_node` refuses `invalid_semantic_graph` at the step entry in the earlier closed-body check; an unresolved `source_node` refuses `missing_declaration`/`missing-name` at `source_node`. A present wrong-kind node or an anchor for a foreign or wrong selected model subject refuses `invalid_model_binding`/`malformed-declaration` at `source_node`; an anchor whose operation name differs from `source_class` refuses the same pair at `source_class`. None admits a partial package.
- FR-061-AC-16 (examined): With a catalog-free step defect and an earlier wire, identity, canonical-order, header or catalog defect, the earlier FR-375 stage wins; with two step-row defects, the first row in canonical order wins. A refinement with `class_catalog` continues to join a step to its concrete catalog entry, including the existing missing-class and wrong-node refusals, independently of the catalog-free anchor rule.
- TC-062 (examined): For FR-061-AC-14 through AC-16, construct catalog-free model-only refinements with operation steps anchored at declaring object-type, process and state-machine nodes of the selected concrete model.

Computed Quire matrix reports FR-061-AC-14, FR-061-AC-15 and FR-061-AC-16 all untagged at this PR head. The new tests use TC-062 tags only. Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | FR-040 process/state-machine admission is owner-held in draft IR #650 pending Peter; PR #651 tests object_type only. AC-14 remains uncredited and must remain untagged until the full process/state-machine oracle passes. |
| FND-002 | rejected | Already covered at c3023198: the test adds a second model selection and foreign model evidence, constructs that selected model’s anchor, keeps refinement semantic_type on the original model_root, and asserts invalid_model_binding/malformed-declaration at steps/0/source_node. Fix head 9b27ece5 adds explicit two-selection and original-root assertions. |
| FND-003 | rejected | Already covered at c3023198: save then other share one source_node, so the canonical sort key differs only by class and save > other; index 1 is out of order while other also mismatches anchor operation save. Reversing the rows repairs order and exposes invalid_model_binding at index 0/source_class. Fix head 9b27ece5 makes these preconditions explicit. |
| FND-004 | fixed | 9b27ece5 adds an admitted present-catalog protocol refinement, then changes only steps/0/source_node to the model root, rekeys, and asserts invalid_model_binding/malformed-declaration at the exact row pointer. |
| FND-005 | deferred | At 9b27ece5, tests now directly trace FR-061-AC-15 and AC-16; the remaining AC-14 trace is deliberately withheld because process/state-machine admission is owner-held under draft IR #650 and the whole criterion is not yet backed. |
