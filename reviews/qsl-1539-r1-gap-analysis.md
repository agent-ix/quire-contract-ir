---
id: SR-2026110283841
title: "gap-analysis of QSL-1539 TypeAdmissionUnfinished scoping"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #854 at 193f7993; crates/quire-contract-model/src/source_obligation.rs, FR-052-AC-7/AC-10/AC-26, tests/it/source_obligation.rs"
review_set: subset
---

## Summary

Gap analysis of quire-contract-ir PR #854 (QSL-1539), planless. Ticket: QSL-1539. Plan completion: not assessed. The PR changes no production behaviour. The model still carries the variant and the synthetic tests that back the model-side portions of AC-6/AC-10/AC-12 still construct it (tests/it/source_obligation.rs:252, 436, 497, 980). The owner-native QSL portion of AC-26 is QSL-owned and remains Planned in TC-452; its procedure gap is recorded once, in the spec-review artifact (FND-001 there), not duplicated here.

`quire coverage --strict` was run separately on IR origin/main and the PR head (see the review comment for results); this PR touches no test file and no trace tag.

## Examined scope

- FR-052-AC-7 (examined): "... QSL exhaustion and, on a QSL owner path that reports it, `Unfinished`, ... produce a typed incomplete or error result and no census; ..."
- FR-052-AC-10 (examined): "... on a QSL owner path that reports `TypeDisposition::Unfinished` (the composed lane), it retains its declaration handle and is distinct, and a QSL path with no such stop (the spine, QSL FR-514) never issues `TypeAdmissionUnfinished`. ..."
- FR-052-AC-26 (examined): "... where the selected QSL path reports `TypeDisposition::Unfinished`, it issues a distinct `Incomplete` with the native declaration handle, and a path with no such stop never issues `TypeAdmissionUnfinished`. ..."
- crates/quire-contract-model/src/source_obligation.rs:92-100 (examined): variant and its rustdoc.

## Verdict

PASS. No code/spec/test mismatch introduced by the diff; the narrowed ACs are weaker than before, so existing IR tests still satisfy them, and the composed-lane QSL test `an_unfinished_declaration_is_distinct_from_exhaustion` still backs the producer side.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | no action | This is a placeholder with no finding. |
