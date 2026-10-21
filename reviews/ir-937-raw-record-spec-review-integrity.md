---
id: SR-14723
title: "IR-937 raw population record integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-937 raw PopulationRecord hotfix; changed FR-038 prose and AC-304; QSpec FR-322-AC-82"
review_set: subset
---

## Summary

The edited criterion is narrower than its upstream owner-join contract.

## Verdict

**FAIL** — The edited criterion is narrower than its upstream owner-join contract.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-304 names missing identity, unresolved kind and wrong meaning but omits a `node` member coexisting with identity; the nearby prose only couples `node` with missing identity. Thus the requirement has two valid interpretations of a mixed old/new record even though QSpec FR-322-AC-82 makes either `node` presence or missing identity independently malformed. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2404,4970; QSpec FR-322-AC-82 |

## Coverage

Exact review scope: IR hotfix 2140725..a13f03a, changed FR-038 owner-join prose and AC-304, with merged QSpec FR-322-AC-82 and its published population cases at QSpec main 6ab82c5. This reviewer did not edit the author branch or run heavy gates.

## Dispositions, round 2

Exact hotfix head `f05b6506726fdb92380ce9261153186392f20030`:

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed | FR-038 now states two independent conditions, no `identity` string or any `node` member, so the mixed record has a single specified refusal consistent with QSpec FR-322-AC-82. |

## Current verdict

**PASS** — The open findings for this method are fixed at the exact reviewed head.
