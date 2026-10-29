---
id: SR-612119
title: "IR-1093 protocol class catalog path specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; FR-038-AC-329 and TC-048 item 16"
review_set: base
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1093. Reviewed the FR-038-AC-329 and TC-048 item 16 edits against QSpec origin/main FR-432, FR-464 and FR-375, with the existing IR catalog requirements as context. The revised source class spelling uses the complete selected clause path and the member's authored name while excluding the fixed declaration-kind segment and control ancestors.

## Verdict

**PASS.** The changed criterion remains falsifiable: it requires admission of QSL-shaped finish and nested parallel classes and refusal of dropped clause segments, substituted clause or member names, and included fixed-kind or ancestor segments. TC-048 item 16 preserves the two-clause, variable-depth path oracle and node associations. The change does not narrow another criterion. Quire validation of both edited documents passed. Quoin evidence advice found no mismatch for the changed criterion; unrelated repository-wide mismatches were outside this review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-329, role: examined. Excerpt: “For selected clauses whose package-local `declaration.qualified_name` paths have different depths, the catalog uses each full clause path and the member's authored name to reconstruct the FR-432 alias-free source class.” Its admission and refusal clauses were checked in full.
- TC-048 item 16, role: examined. Excerpt: “Require the complete alias-free FR-432 source class spellings for each clause and member, using the complete clause path and the member's authored name while omitting the FR-464 fixed kind segment and intervening control ancestors.” Its admission, association and rekeyed refusal cases were checked in full.
- FR-038-AC-325, FR-038-AC-328 and FR-038-AC-330, role: context_only. Their catalog completeness, model-only and compensation obligations remain unchanged.
- QSpec FR-432, FR-464 and FR-375, role: context_only. Their class spelling, declaration path and complete catalog rules supply the controlling contract.

## Review scope

This was a spec-only review. No Cargo build or test suite was run. Existing tagged IR tests still use the former class spelling and require a separate code change; their runtime outcome was not claimed here.
