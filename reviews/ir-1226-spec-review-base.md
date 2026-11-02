---
id: SR-202611010117
title: "Base specification review of IR-1226 PR 809"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-354-project-admitted-runtime-context.md"
review_set: subset
---

## Summary

Reviewed the added FR-354 operation input, behavior, AC-1, AC-10, AC-11 and QSL/QSpec dependencies against current QSL operation declarations and the admitted IR model. The required upstream carrier is stated honestly, but AC-11 omits a signature mismatch oracle for the new producer fact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Require refusal when a selected operation fact's parameter or result signature disagrees with the admission-retained operation | FR-354-AC-11; FR-354:90-99; IR model_members.rs:318-325 |

## Verdict

Changes requested. QSL FR-103 owns the effect and QSpec STD-640 is correctly identified as an unshipped producer dependency. The Ledger example is a signature pattern; its test case will need to add a nonempty effect because the historical child-composite fixture uses empty frames.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: FR-354 Behavior now compares the producer fact's ordered parameter names and types and optional result with the retained signature; AC-11 names matching-key disagreement and typed conflicting-witness refusal. |
