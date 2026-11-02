---
id: SR-202611010300
title: "IR-1251 PR 804 model-owner harness code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #804; tests/it/checked_package_v2_owners.rs; reader model selection and test support context"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---

## Summary

Ticket: IR-1251. Reviewed PR #804's test-only change at rebased head `db5459802ec96bf728dda5af09aa8024f8406e92` with the Rust review checklist. The reviewed harness supplies the QSpec domain document under the selected model-root identity, and the unchanged reader retrieves that document from host evidence rather than dependency selections. No production code or gate configuration changes.

## Reviewed units

- **FR-038-AC-304 (examined):** population owner joins the selected document's population identity; malformed document and unresolved owner cases refuse.
- **FR-038-AC-305 (examined):** `all_instances` and `lookup` admit declared population operands, preserve one shared reference node, and distinguish named applications.
- **FR-038-AC-306 (examined):** ineligible population uses refuse at the operand or member locus.
- **FR-038-AC-307 (examined):** published QSpec vectors and fixture are read by reference, with admission and adverse code, cause, and locus checks.
- **QSpec FR-322 / FR-154 (context only):** the selected domain package document is host evidence keyed by model identity.
- `tests/it/checked_package_v2_owners.rs` (examined), including `read_with_selected_model`, the QSpec positive and adverse tests, and application rekeying.
- `crates/quire-contract-model/src/checked_package/v2/model_members.rs` and `mod.rs` (context only): reader document-selection path.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS** — the changed tests call the real strict reader and make the positive admission and missing/foreign document refusals fail if the reader's model selection behavior regresses. The generated population and application identities remain internally consistent after each vector mutation. The test-only helper panics only on a malformed published fixture in the test harness. Focused TC-229 and Clippy checks were reported passing by the coder; the code lead owns the final gate.
