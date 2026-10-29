---
id: SR-12178
title: "IR-862 gap analysis of IR PR 449"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR 449..; selected-variant test traces and computed coverage matrix"
review_set: subset
---

# IR-862 gap analysis of IR PR 449

## Summary

The seven existing criterion citations remain on the same tests. The new TC-048 citations bind those tests to their owning test case, without changing assertions or production behavior. Computed traceability is stable at the exact PR base and head.

## Verdict

**PASS for the PR diff** — no new traceability gap or status contradiction. Existing unbacked criteria remain outside this slice.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder). | - |

## Coverage

Independent `quire coverage --scope. --format json` runs at and both report 527/584 backed rows, 39 unbacked rows, and 0 status lies. All 550 computed obligations and their criterion statuses and binders are byte-identical. Only `shared_trace_ids` changes: TC-048 gains the seven selected-variant test symbols. FR-038-AC-236, FR-038-AC-237, and FR-038-AC-238 all belong to TC-048 in the authored matrix. No coverage decrease occurs.
