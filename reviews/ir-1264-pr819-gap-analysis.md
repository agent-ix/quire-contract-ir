---
id: SR-2026110101270
title: "IR-1264 PR 819 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1264 PR #819; FR-038-AC-420; tests/it/{checked_package_v2_model_fields,checked_package_v2_owners}.rs; crates/quire-contract-model/src/checked_package/v2/{owner,model_members}.rs"
review_set: subset
---

## Summary

Audited the changed requirement, tagged tests and production owner path. The computed matrix binds FR-038-AC-420 to two real integration tests; no new traceability gap was found in the PR diff.

## Verdict

PASS for the scoped PR diff. This is not a repository-wide completion claim: the full computed matrix still has preexisting untagged criteria outside this review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- Reconciliation: `quire matrix --scope . --format json` (Quire CLI 0.36.2, engine 0.50.2), no run evidence read.
- Plan completion: not assessed
- PR criterion FR-038-AC-420: tagged by two test functions, zero ignored tests.
- Full matrix context: 946 criteria, 791 tagged, 132 untagged, 22 method-without-symbol, 1 tagged-by-ignored-test; the latter gaps predate this PR and are outside the user-directed diff review.
- Reverse gap: changed `owner.rs` and `model_members.rs` behavior is owned by FR-038-AC-420 and QSpec FR-322-AC-68.
- Semantic review: skipped as optional under the gap-analysis skill; code-review separately checked the concrete owner-join behavior.

## Evidence

The new TC-281 test uses the public strict reader with a selected event construct and checks admission, exact EV-02 refusal code/cause and `node_id` path for record, object and absent declaration owners. The declared-source-owner test checks public-reader admission. The same TC-281 test checks absent owner and SourceOwner wire refusals with exact primary paths. Neither test replaces the reader under test or stores QSpec's vectors as a copied fixture.
