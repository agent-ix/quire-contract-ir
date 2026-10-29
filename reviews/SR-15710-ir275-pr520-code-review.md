---
id: SR-15710
title: "Code and Rust review — IR-275 selected model type references"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR #520; model_members.rs and model_members/tests.rs"
review_set: subset
---

## Summary

Reviewed selected type reference classification, validation, later member type derivation, and the new native type test at the exact PR head. The change retains admitted names as typed metadata without altering public wire shape or refusal ordering.

## Verdict

**PASS** — no diff-scoped code or Rust defect found. Full gates await the final reviewed head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Rust review

The closed native enum replaces the prior native string table. The validation path checks the same native names and still rejects unknown native names; selected Unit and Declared names remain lookup keys. A malformed reference records a defect before the selected model can be used. The new Option for a refused type reference is consumed only after successful admission. The changed code adds no new unsafe, panic, unchecked conversion, blocking, recursion, or unbounded loop. The test-first commit precedes implementation. Cargo formatting and diff whitespace checks pass.

## Scope

Examined FR-038-AC-29 and FR-038-AC-300. No spec or plan file changed.
