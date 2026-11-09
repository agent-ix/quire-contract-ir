---
id: SR-202611081512300007
title: IR-1404 PR 1004 gap analysis
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-contract-ir; IR-1404 PR #1004 head 8ca9b6a1; crates/quire-contract-model/src/checked_package/v2 flat_wire.rs, operations.rs, protocol_body.rs, stage_storage.rs, state.rs'
review_set: subset
---

## Summary

Ticket: IR-1404. Scoped to the PR diff. The PR changes accounting and refusal construction in code owned by FR-038 and FR-358; it adds no behavior outside those requirements and no spec text. The computed matrix was read from `quire matrix` at the head.

## Verdict

PASS for this PR's scope. Every criterion the change touches is tagged, the new helpers have owning requirements, and the diff adds no stub. This is a repository-assurance claim over the PR scope only. The unpaid flat-wire refusal is recorded as FND-001 of the code review, SR-202611081512300006, and is not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No gaps found (placeholder) | - |

## Coverage

- FR-038-AC-510: tagged, 25 binders.
- FR-038-AC-511: tagged, 100 binders.
- FR-038-AC-512: tagged, 20 binders.
- FR-038-AC-516: tagged, 6 binders.
- Reverse gap: `refuse_at`, `refuse_unlocated`, `node_path`, `Refusal::in_document`, `flat_refusal`, `flat_refusal_at`, `refuse_detailed` and the `MemberCheck` refusal helpers all serve FR-038's budget-charged strict read. None adds unowned behavior.
- Stubs: none in the diff.
- Plan completion: not assessed
- Semantic review: skipped, not opted in.
