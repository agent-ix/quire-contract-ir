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
