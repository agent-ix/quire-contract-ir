---
id: SR-15326
title: "IR-571 PR 503 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #503 six changed Rust source/test files; IR FR-038, FR-346-AC-7, TC-225 and QSpec FR-154-AC-14"
review_set: subset
---

## Summary

Rust review found a multi-defect refusal-precedence error.

## Verdict

**CONDITIONAL** — Rust review found a multi-defect refusal-precedence error.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Row::Duplicate and Row::ParameterName both sort after Row::Reference. A selected operation parameter with an invalid name and unresolved typeRef records both defects; Defects::first chooses missing_declaration/missing-name rather than the malformed name. With a repeated parameter identity and unresolved typeRef it likewise chooses the reference instead of conflicting-binding. QSpec FR-154 requires within-operation collision, then name checks, ahead of later checks, and its declaration table places the name row before missing references. Preserve that precedence in selection without changing unrelated duplicate ordering, and test combined defects through the public reader. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:2055-2061, 2906-2917 |
## Rust review

Reviewed all six changed Rust files under the repository idioms and rust-review checklist. The public test checks selection-row code, cause and path, and the unit tests inspect private operation identity, origin and parameter path. The case-only admitted control exercises the public reader. No changed CI, unsafe, panic surface, or vendored material was found; the ordering defect above remains.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Row now has a parameter-specific collision variant before ParameterName and Reference, while generic Duplicate stays later. The two new public TC-225 combined-defect cases require malformed-declaration or conflicting-binding at the selection row, so an unresolved typeRef cannot win over either parameter defect. |
