---
id: SR-611027
title: "Gap analysis of IR-874 protocol dependency exactness"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-285/324/327, TC-048, crates/quire-contract-model/src/checked_package/v2/structural.rs, protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs"
review_set: subset
---

## Summary

Reviewed quire-contract-ir PR #629 at frozen head 7ac176525c667a13a44a3fdaea28b2e60e8eee26. FR-038-AC-285/324/327 have tagged integration tests for omitted existing targets and extra dependencies across five protocol member forms; an absent listed target is tested through the public reader for a role. No applicable AssuranceProfile was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-327 explicitly covers each protocol-family form and named TC-946 missing-target vectors, but the changed tests omit a control form and test the listed-but-nonexistent target only on a role; add a control dependency case and the clause's C13a/A8a/A9a target paths. | tests/it/checked_package_v2_protocol_body.rs:3397 |

## Verdict

CONDITIONAL. The tested forms exercise graph-stage refusal and later target resolution with exact code, cause and path. The remaining named form and target-path controls are needed for AC-327's full test claim.

## Coverage

Plan completion: not assessed. The computed Quire matrix marks FR-038-AC-285/324/327 tagged; direct assertion inspection found the incomplete form and vector coverage above. This is a diff-scoped review, not a repository-wide completeness claim.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At fix head 34e3a57bb55ea70a0bc0ae33da4838da056cc46e, the exact-dependency table includes a `control` holder and asserts both extra and omitted dependency refusals. `listed_clause_targets_missing_from_graph_reach_their_body_joins` checks C13a, A8a and A9a with a listed absent target and exact `MissingDeclaration`/`MissingName` locus. C13a uses an orphan-free base; the isolated repaired test passes 1/1. |
