---
id: SR-10849
title: "Evidence review — IR-759 source-obligation correspondence"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; FR-052-AC-1 through FR-052-AC-5, TC-452"
review_set: subset
---

## Summary

TC-452 supplies a plausible typed fixture and adverse cases for the principal mapping behaviors. AC-4 asks for a test to vary values excluded from the constructor, and AC-5 combines compile-time consumer behavior with inspection while declaring only Inspection. No applicable AssuranceProfile was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-4's specified test cannot vary proof, method or profile inputs that the API excludes | FR-052-AC-4 |
| FND-002 | medium | AC-5's downstream borrow claim needs consumer compile evidence, but its declared method is Inspection only | FR-052-AC-5, TC-452 |
