---
id: SR-15661
title: "Gap analysis — IR-972 retained declaration terms"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #519 at 798026771e43f0a9494fe2b5921f142b22fa3f70; FR-349 and FR-038 criteria, changed model-field, relationship and comparison tests and source"
review_set: subset
---

## Summary

The changed admitted model declaration lookups are bound to retained typed terms, and the new and existing focused tests exercise the body independence and typed refusal paths. The remaining raw declaration body comparison is used by the reader's admission-stage owner recovery.

## Verdict

**PASS** — no diff-scoped traceability, unowned behavior, stub, or coverage-inflation gap found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Examined FR-349-AC-2, FR-349-AC-3, FR-349-AC-4, FR-038-AC-140, and FR-038-AC-266. `admitted_model_fields_use_retained_declaration_term` is tagged to FR-349-AC-2 and FR-038-AC-140 and changes the raw wire after constructing the retained package. Existing relationship and comparison tests tagged FR-349-AC-2 exercise retained operation and clause terms, while the changed relationship refusal fixture mutates the retained declaration term. The source changes preserve wire identity and encoding paths. Optional semantic review was not requested.
