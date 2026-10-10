---
id: SR-4103
title: "Evidence review of IR-360 private numeric pairing"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; IR-360; FR-014-AC-9, TC-444, TC-016, model TestMatrix"
review_set: subset
---

## Summary

Inspection is the justified method for a Rust construction invariant, despite the catalog advisor recommending behavioral tests for AC-9's public-preservation clause. The new inspection procedure lacks a complete public-contract comparison oracle.

## Examined units

- FR-014-AC-9, structural invariant and public-preservation clause (examined).
- TC-444 Procedure and Expected Results (examined).
- Model TestMatrix TC-444 Inspection, P0, planned row (examined).
- FR-014-AC-1 and TC-016 existing behavioral coverage (context only).
- Catalog advice for FR-014-AC-9: authored Inspection, mismatch due example/user-visible characteristics (context only).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-444 compares public expression signatures and invalid-operand diagnostic codes to FR-014-AC-1, which specifies neither signatures nor exact codes. Name the actual public API and diagnostic authority, and retain TC-016 behavioral evidence for the public-preservation claim. | spec/model/matrix/TC-444-private-numeric-type-range-pairing.md:26 |

## Verdict

**CONDITIONAL.** Inspection can discharge the private type-level invariant, but its public-contract comparison needs a real oracle before the criterion is review-ready.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-444 now names the public `DeclarationEnvironment::check_expression` signature and return types, FR-014 Behavior and STD-001 for diagnostic codes/spans, and TC-016 for behavioral evidence. The related AC-9 sentence uses the same authorities. No new issue arose in this narrow citation change. |
