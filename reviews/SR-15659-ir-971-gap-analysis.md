---
id: SR-15659
title: "Gap analysis — IR-971 typed operand evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #518; ten changed crates/quire-contract-model/src/checked_package Rust files and relevant tests"
review_set: subset
---

## Summary

Compared the changed operand and population paths to their owning criteria and tagged tests. The new controls exercise retained terms after raw-body mutations; existing admission and ContractPackageV1 controls back the surrounding behavior. No diff-scoped trace or coverage gap was identified.

## Verdict

**PASS** — no defect found in this diff-scoped gap-analysis pass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. Examined FR-349-AC-1/2/4, FR-038-AC-207, FR-053-AC-3 and FR-054-AC-3. New scalar, composite, comparison, population, reaches, and relationship tests carry FR-349-AC-2 and assert retained results after raw-body changes. Existing tests/it and conformance controls carry FR-349-AC-1/4; the TC-445 synthetic first-refusal test covers FR-038-AC-207; TC-458 reader-admitted tests cover FR-054-AC-3; FR-053 selected-population controls cover the duplicate maximum and identity refusals. No optional semantic audit or plan was selected.

## Scope

Examined FR-349-AC-1, FR-349-AC-2, FR-349-AC-4, FR-038-AC-207, FR-053-AC-3, FR-054-AC-3. No spec or plan artifact changed in this PR, so spec-review methods do not apply.
