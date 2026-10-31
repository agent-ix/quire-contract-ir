---
id: SR-2026103041
title: "IR-1203 field read code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 745 Rust source and tests for admitted model field read_form; IR-1203"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-349
    type: reviews
---

## Summary

Ticket: IR-1203. Reviewed frozen PR #745 at 6880ab2a against origin/main. Rust review covered the changed model crate, conformance test, integration tests, and public inventory. The typed borrowed member retains Bare and ExplicitDeref, encoding retains wire data, and no QSL mirror or copied QSpec vector was added. No applicable AssuranceProfile was found.

## Verdict

**FAIL** — the repository's required `make string-edge` gate fails on three new closed-string decisions. Local reproduction matches hosted CI run 41245722533. `git diff --check` passed. No aggregate gate was run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `make string-edge` rejects three new branch-gating string decisions: the post-intake `operation_defect` identity comparison and both `CheckedFieldReadForm::from_wire` match arms. The normal `make lint` gate therefore fails. Keep the post-intake choice typed and mark only the actual wire-to-enum intake conversion with the authorized neutral marker, while preserving the refusal behavior. | crates/quire-contract-model/src/checked_package/v2/operations.rs:974; crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:204 |

## Coverage

- FR-349-AC-8, role: examined. Excerpt: “An external crate reads every admitted operation member and constraint production, identity, mode, ordered laws and ordered leaves through one borrowed typed view. The view contains no JSON payload or map-key lookup; a direct-deserialized or `test-support` fixture operation returns `NotAdmitted`, while the existing specific getters retain their prior results.” The new `read_form` field is in the public typed member.
- FR-349-AC-9, role: examined. Excerpt: “The FR-019 exact public-item inventory and external compile probes show that IR exports the admitted read views but no QSL checked-expression constructor, `TermOperation` mirror, checker, or evaluator.” The diff adds only the typed read enum and export.
- FR-019-AC-5, role: examined. Excerpt: “The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision.” The new export appears in the inventory row.
- FR-056-AC-8, role: examined. Excerpt: “The actual repository command completes with no unresolved occurrences across both roots, without removing a root, filtering diagnostics, introducing a local marker or weakening FR-064's rule checks.” FND-001 violates this gate.
- QSpec FR-322-AC-34, role: examined. Excerpt: “Its field member carries `read_form: bare` for `view.total` and `read_form: explicit_deref` for `deref(view).total`; otherwise identical operand, field and result data yield distinct member values, and a reader preserves each value.” The PR exercises both admitted forms and the published malformed outcomes.
- FR-038-AC-107, role: context_only. The existing positive fixture sweep remains outside `make ci`; QSpec's positive-state-transition fixture at the referenced upstream revision omits the newly required form. This is an upstream fixture inconsistency, not a second IR code finding.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At 2a0acc4848a782f1dab9ac1f901562f20a92de3d, `OperationWire::field_projection_identity` is a marked wire-to-enum conversion, `operation_defect` branches on its typed `FieldProjectionIdentity`, and `CheckedFieldReadForm::from_wire` is marked as the closed wire-to-enum conversion. Local `make string-edge` reports no unmarked or unlisted decisions. |

## Disposition verdict

Round 1 at 2a0acc4848a782f1dab9ac1f901562f20a92de3d: FND-001 fixed on source inspection and a focused string-edge rerun. The original review finding and verdict remain as recorded above. No new finding or still-open finding was found. The lead owns the final aggregate gate.

## Disposition pass 2 review

At e6d5aabf0db516af030b969b6887534956c7408d, the five failing integration cases from the full gate at ca12e267 received fixture corrections. `step_relation_package` now supplies `read_form: bare` on its model-object field projections. `selected_model_field_range_joins_exact_body_members` now constructs `Reference<Object> -> model.deref -> record.project` with the same form. Both tests assert `CheckedOperationMember::Field { read_form: Some(CheckedFieldReadForm::Bare), .. }` after strict reader admission. These paths satisfy QSpec FR-322-AC-34's operand and member form rule. The focused tests reported by the coder passed, and an independent `make string-edge` rerun passed. `git diff --check ca12e267..HEAD` passed. No new findings arose; FND-001 remains fixed, so round 2 adds no `## Dispositions` row. The final aggregate gate on this head remains the lead's responsibility.
