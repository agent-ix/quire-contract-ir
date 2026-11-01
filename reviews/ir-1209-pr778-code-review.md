---
id: SR-202610310135
title: "Code review of IR-1209 admitted equality PR 778"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 778 frozen head 5178bd8f; equality projection, admission, tests, Cargo boundary; IR-1209"
review_set: subset
---

## Summary

Ticket: IR-1209. Reviewed the frozen Rust diff with the Rust style and code review checks against FR-353-AC-2 through AC-7 and FR-352-AC-11. No applicable AssuranceProfile was found under `spec/`. The new admission joins application, occurrence, operator and source operand type; the primitive Plan read uses retained decisions and QSV's opaque mint. Text, enum and quantity still return typed witness refusals as declared by this PR's scoped delivery.

## Verdict

CONDITIONAL. One work-budget error maps to the generic materialization code and loses the attempted charge; the focused test permits that result. The branch is frozen and the reviewer ran no aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A QSV work-budget denial is returned as generic Materialization with attempted_work zero, and the one-less test permits it. | crates/quire-contract-model/src/checked_package/v2/equality_projection.rs:827 |

## Evidence

`mint_admitted_equality_decision` receives the remaining Exact meter budget. QSV reports a denied charge as `EqualityMaterializationError::Resource { locus, cause: Incomplete }`, where `cause` contains the consumed and next charge values. The read maps every QSV error to `AdmittedEqualityReadErrorCode::Materialization` and sets `attempted_work` to zero. FR-353-AC-4 and AC-7 require typed exhaustion and the attempted charge at the first denied work item. `admitted_equality_joins_application_and_bounds_read` accepts either `WorkLimit` or `Materialization`, so it cannot catch this defect.

## Scope and bindings

Examined FR-353-AC-2, FR-353-AC-3, FR-353-AC-4, FR-353-AC-5, FR-353-AC-7, FR-352-AC-11; FR-353-AC-1, AC-6 and AC-8 were context for the intentionally incomplete positive schedules. Examined changed Rust files in `crates/quire-contract-model/src/checked_package/v2/`, crate root exports and Cargo metadata, `tests/it/cycle_free_model.rs` and `tests/it/nominal_reads.rs`. The four new test functions carry tags for AC-2, AC-3, AC-4, AC-5, AC-7 and FR-352-AC-11; the AC-4 assertion is the failing binding described above. No copied upstream artifacts, new hash/pin records, QSL dependency or CI gate change appears in this diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, local rebased head 041cac92: `materialization_refusal` maps QSV `Resource` at WorkUnits to `WorkLimit`, retains the nested `Incomplete`, and reports consumed and attempted work. The focused mapping test asserts both work and non-work resource cases. |
