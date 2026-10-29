---
id: SR-7861
title: "IR-743 integrity review of FR-035 and TC-047"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md; spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md"
review_set: subset
---
# SR-7861: IR-743 integrity review

## Summary

Ticket: IR-743. Reviewed the PR head against QSpec main FR-330 AC-8 and TC-936. The revised FR-035 outputs and AC-5 agree with TC-047 and with the pinned QSpec identity rule. The digest projection and source-map validation duties are distinct and observable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean for this method. Examined FR-035 Outputs and AC-5 and TC-047 Test Procedure and Expected Results. The two separate QSpec defensive-payload and reader-input questions are outside this narrow diff.
