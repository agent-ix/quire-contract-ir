---
id: SR-15711
title: "Gap analysis — IR-275 selected model type references"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #520; selected member type source and tagged model tests"
review_set: subset
---

## Summary

The new selected native type test checks retained closed metadata and derived member types. Existing tagged tests cover selected integer value types, inherited members, and unit typed dispatch calls through the public reader. Computed matrix marks the examined criteria tagged.

## Verdict

**PASS** — no diff-scoped coverage, unowned behavior, or hollow-test gap found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Examined FR-038-AC-29 and FR-038-AC-300. New native test is tagged TC-048 and FR-038-AC-29. Existing selected document and unit tests are tagged to the two criteria. The new test asserts both retained enum value and derived result rather than a fixture constant alone. Optional semantic review was not requested.
