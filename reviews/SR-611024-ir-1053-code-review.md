---
id: SR-611024
title: "IR-1053 PR #628 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_refinement.rs; PR #628 "
review_set: subset
---

## Summary

Reviewed the one-row persistence-interface addition against FR-064-AC-4 and the existing selected-owner control. The test uses the selected Store declaration already constructed by `selected_form_document`, includes its model selection and evidence, and passes Store's authentic node key into the changed clause owner edge.

## Verdict

**PASS** — no findings in this diff. The existing control asserts admission; every wrong-kind case, now including selected persistence-interface, must refuse with `InvalidModelBinding`/`MalformedDeclaration` at `/semantic_graph/nodes/{clause}/owner_model`. This review did not rerun the focused test; the coder reported 1/1 PASS and formatting PASS. No aggregate gate was run by this reviewer.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined units

- FR-064-AC-4 (examined): "With wire, graph and identities valid, a genuinely unselected `model`/`object_type` target refuses `missing_declaration`/`missing-selection` at the target's own `node_id` in the owner join before any body step. A local edge naming no node refuses `missing_declaration`/`missing-name` at `owner_model` in the clause step. A non-model node, a source-local declaration, a selected declaration node of another form (`state_machine`, `process`, `record_value_type` or a relation node), or the selected package's `model_root` refuses `invalid_model_binding`/`malformed-declaration` at `owner_model` as the clause's first check."
- FR-068-AC-5 (context_only): selected Process/StateMachine/RecordValueType declaration owner joins.
- TC-463 (context_only): selected Process and PersistenceInterface model-owner vectors.

## Rust review

The changed row adds no public API, production branch, copy, unsafe code, or gate change. The `Store` declaration's kind maps to `quire.meaning.model.persistence-interface/v1`, and the selected package has the corresponding `persistence_interface` node and digest-backed evidence. The test's `Refused` match fails if the reader admits; each code, cause and path assertion can fail independently. The same admitted control runs before all four adverse rows.
