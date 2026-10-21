---
id: SR-14722
title: "IR-937 raw population record base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-937 raw PopulationRecord hotfix; changed FR-038 prose and AC-304; QSpec FR-322-AC-82"
review_set: subset
---

## Summary

The edited FR-038 criterion omits one independently required malformed-record shape.

## Verdict

**FAIL** — The edited FR-038 criterion omits one independently required malformed-record shape.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A selected raw population record with both a valid `identity` and a `node` member is not required to refuse by FR-038-AC-304 or its nearby prose. Merged QSpec FR-322-AC-82 requires refusal for any `node` member, independently from missing identity, at the model selection before graph checking. As written, IR can treat that record as valid and still satisfy its own edited AC. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2404,4970; QSpec FR-322-AC-82 |

## Coverage

Exact review scope: IR hotfix 2140725..a13f03a, changed FR-038 owner-join prose and AC-304, with merged QSpec FR-322-AC-82 and its published population cases at QSpec main 6ab82c5. This reviewer did not edit the author branch or run heavy gates.

## Dispositions, round 2

Exact hotfix head `f05b6506726fdb92380ce9261153186392f20030`:

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed | FR-038 prose and AC-304 now say a selected population record with any `node` member refuses at the model selection, independently of missing `identity`; this matches merged QSpec FR-322-AC-82. |

## Current verdict

**PASS** — The open findings for this method are fixed at the exact reviewed head.
