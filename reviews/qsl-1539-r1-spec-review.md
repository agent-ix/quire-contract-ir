---
id: SR-2026110283839
title: "spec-review of QSL-1539 FR-052 TypeAdmissionUnfinished scoping"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #854 at 193f7993; spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md (context)"
review_set: subset
---

## Summary

Spec review of quire-contract-ir PR #854 (QSL-1539), which keeps `SourceInventoryIncompleteReason::TypeAdmissionUnfinished` and scopes it to QSL's composed-lane inventory. Ticket: QSL-1539. Base: IR origin/main bb3b8956. QSL claims were re-measured on quire-spec-language origin/main fc091fb1e (read-only, `git show`).

Claim verification (all re-measured, none taken from ticket or PR text):

- QSL `src/checking/source_inventory.rs` `stop_reason` maps `TypeDisposition::Unfinished` to `TypeAdmissionUnfinished` after checking `types.exhaustion()`: confirmed (lines 97-117).
- QSL test `an_unfinished_declaration_is_distinct_from_exhaustion` (tests/it/source_inventory.rs:248) asserts `exhaustion().is_none()` and `Incomplete(TypeAdmissionUnfinished{Pending})`: confirmed. Its trigger is `BindingLimits { bindings: 0 }`. Admitted reference cycles also stay `Unfinished` with no exhaustion (solver/dependencies.rs: unsettled callers never leave `Unfinished`): confirmed.
- QSL Cargo.toml:28 depends on `quire-contract-model` by `git ... branch = "main"`: confirmed, so dropping the variant breaks QSL main's build.
- The spine `qsl_inventory` (QSL FR-514) never produces the cause: true as specified, but no spine `qsl_inventory` exists on QSL main yet; the only definition is the composed-lane one at src/checking/source_inventory.rs:247 (QSL-1533 is Specced).
- Nothing in IR consumes the variant besides `owner()` (source_obligation.rs:127) and synthetic tests (tests/it/source_obligation.rs:252, 436, 497, 980): confirmed.
- The ticket's own premise "QSL never produces TypeAdmissionUnfinished" (untrusted ticket text) is false for current QSL main; it is true only of the spine.

## Examined scope

- FR-052-AC-7 (examined): "... Normal exhaustion is required for `Complete`; Quire skipped/load/scan failures, QSL exhaustion and, on a QSL owner path that reports it, `Unfinished`, early stop, failed parse/request, unavailable selected module, or scope/source mismatch produce a typed incomplete or error result and no census; ..."
- FR-052-AC-10 (examined): "... QSL type-work exhaustion retains dimension, limit, prior, requested charge and site; on a QSL owner path that reports `TypeDisposition::Unfinished` (the composed lane), it retains its declaration handle and is distinct, and a QSL path with no such stop (the spine, QSL FR-514) never issues `TypeAdmissionUnfinished`. ..."
- FR-052-AC-26 (examined): "... Native type-work exhaustion instead issues `Incomplete` with dimension, limit, prior, requested charge and site; where the selected QSL path reports `TypeDisposition::Unfinished`, it issues a distinct `Incomplete` with the native declaration handle, and a path with no such stop never issues `TypeAdmissionUnfinished`. ..."
- FR-052 Behavior, stop-projection paragraph lines 210-227 (examined): "QSL has two inventory paths. Its composed-lane `checking::source_inventory::qsl_inventory` projects ... `TypeAdmissionUnfinished` is retired, with no replacement, by the change that deletes the composed-lane inventory (QSL-868, QSL-830)."
- FR-052 Behavior, typed-cause paragraph lines 157-163 (examined): "Where a QSL owner path reports `TypeDisposition::Unfinished`, that stop shall be a different typed cause keyed by its native declaration handle."
- FR-052 Behavior, adapter paragraph lines 180-188 (examined, unchanged): "Any early return, ... or unfinished type admission shall issue `Incomplete` ... In particular, QSL type-work exhaustion and `TypeDisposition::Unfinished` remain separate causes."
- TC-452 FR-052-AC-26 procedure lines 130-136 (examined, unchanged): "Trigger native type-work exhaustion and unfinished admission separately, and compare every retained typed cause field and partial row with the owner report."
- TC-452 FR-052-AC-6 procedure line 252 (context only): synthetic `TypeAdmissionUnfinished` cause for the QSL position.

## Verdict

CONDITIONAL. Keeping the cause is the right call: it is not a compatibility layer but the typed cause a live producer on QSL main actually emits, and dropping it would force QSL's composed lane either to fabricate a `TypeWorkExhausted` with no native exhaustion fields or to report a false `Complete`, in both cases breaking FR-052-AC-10 ("the typed cause the owner actually reports"), and would break QSL's `branch = "main"` build. The AC-7/AC-10/AC-26 rewording is coherent and testable. Two medium gaps remain: TC-452's AC-26 procedure was not updated to the conditional wording, and the stated retirement condition has no owner on either side.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-452's FR-052-AC-26 procedure still unconditionally requires triggering "unfinished admission"; the reworded AC-26 makes that conditional on the selected QSL path, and the spine adapter (FR-514) cannot produce it, so the procedure as written cannot be executed against the spine adapter | spec/model/matrix/TC-452-source-obligation-correspondence.md:133-134 |
| FND-002 | medium | Retirement condition has no owner: FR-052 says the cause "is retired ... by the change that deletes the composed-lane inventory (QSL-868, QSL-830)", but neither ticket mentions removing the IR variant or FR-052 text, and the variant lives in IR, so its removal is an IR change that must land with or after QSL-830; nothing tracks it | spec/model/functional/FR-052-source-obligation-correspondence.md:224-226 |
| FND-003 | low | "QSL has two inventory paths" is stated in the present tense in a paragraph "closed over owner paths that actually exist", but QSL main has only the composed-lane `qsl_inventory`; the spine one is specified (FR-514, QSL-1533 Specced), not built | spec/model/functional/FR-052-source-obligation-correspondence.md:210-222 |
| FND-004 | low | The unchanged adapter paragraph still states unconditionally that "unfinished type admission shall issue `Incomplete`" and that QSL exhaustion and `TypeDisposition::Unfinished` remain separate causes, not scoped like AC-7/AC-10/AC-26 and the line-160 Behavior sentence | spec/model/functional/FR-052-source-obligation-correspondence.md:183-188 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-452's FR-052-AC-26 procedure now depends on the selected QSL owner path. The composed lane triggers unfinished admission separately from exhaustion. The spine control sets a stage limit and expects `TypeWorkExhausted` and never `TypeAdmissionUnfinished`. |
| FND-002 | fixed | Filed IR-1304 (Contract IR & Codegen), blocked by QSL-830 and related to QSL-1539, to remove the variant, the FR-052 and TC-452 text and the synthetic uses. FR-052 now cites IR-1304. QSL-830's description is unchanged. |
| FND-003 | fixed | FR-052 now names the composed-lane `qsl_inventory` as QSL's implemented inventory. It describes the spine as specified by QSL FR-514 and not yet built (QSL-1533). |
| FND-004 | fixed | The adapter paragraph scopes unfinished type admission, and its separation from exhaustion, to an owner path that reports it. |
