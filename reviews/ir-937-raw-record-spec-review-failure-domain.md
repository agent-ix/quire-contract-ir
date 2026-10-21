---
id: SR-14724
title: "IR-937 raw population record failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; IR-937 raw PopulationRecord hotfix; changed FR-038 prose and AC-304; QSpec FR-322-AC-82"
review_set: subset
---

## Summary

The raw-record identity failure domain omits a stale `node` member when `identity` is present.

## Verdict

**FAIL** — The raw-record identity failure domain omits a stale `node` member when `identity` is present.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A population record can carry valid `identity` and a second `node` string with a different or equal value. FR-038-AC-304 does not specify refusal for this identity-confusion case, while QSpec requires `invalid_model_binding`/`malformed-declaration` at the selection row for any `node` member. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2404,4970; QSpec FR-322-AC-82 |

## Coverage

Exact review scope: IR hotfix 2140725..a13f03a, changed FR-038 owner-join prose and AC-304, with merged QSpec FR-322-AC-82 and its published population cases at QSpec main 6ab82c5. This reviewer did not edit the author branch or run heavy gates.

## Dispositions, round 2

Exact hotfix head `f05b6506726fdb92380ce9261153186392f20030`:

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed | FR-038-AC-304 explicitly refuses a `node` member even when `identity` is present; the direct TC-229 mutation covers the same-value duplicate-identity case. |

## Current verdict

**PASS** — The open findings for this method are fixed at the exact reviewed head.
