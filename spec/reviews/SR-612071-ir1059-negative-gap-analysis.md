---
id: SR-612071
title: "IR-1059 PR 638 intake refusal trace review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 638 tests/conformance_qspec/model_tuple_union.rs and crates/quire-contract-model/src/checked_package/v2/model_members.rs; FR-069-AC-6"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-069
    type: reviews
---

## Summary

Ticket: IR-1059. PR-diff-scoped trace review of the intake refusal slice against FR-069-AC-6 and its QSpec TK-01..TK-10 rows. The PR adds no acceptance-criterion trace tag and does not claim to back the full wire and combined-defect matrix.

## Verdict

**PASS for trace scope** — the partial test leaves FR-069-AC-6 open and does not inflate computed coverage. The mutation-oracle defects are in SR-612070. The repository-wide Quoin matrix did not complete during this review, so this verdict is limited to the changed files and a direct trace-tag search.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-069-AC-6, role: examined. Excerpt: “Every published intake and wire mutation returns its exact QSpec code, cause and location at its specified stage.” No test in this diff carries its trace tag, correctly leaving the criterion open.
- FR-069-AC-8, role: context_only. Excerpt: “`make conformance-qspec` loads the positive fixture, `model-tuple-union-vectors.json`, its full Semantic IR `selected_document` ... and runs every tuple/union case.” Full conformance remains open.
- Reverse trace: the changed intake locator has an owning FR-069 refusal obligation; no unowned production behavior was identified in this diff.
- Quoin repository matrix: not completed; manual PR-diff trace scan used. Optional semantic review: not requested.
- Plan completion: not assessed.
