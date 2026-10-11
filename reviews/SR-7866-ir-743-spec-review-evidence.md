---
id: SR-7866
title: "IR-743 evidence review of FR-035 and TC-047"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@1fbb066e9ce24f4eefdc6d3d320ed4bff2e43a64; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md; spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md"
review_set: subset
---
# SR-7866: IR-743 evidence review

## Summary

Ticket: IR-743. Reviewed frozen commit `1fbb066e9ce24f4eefdc6d3d320ed4bff2e43a64` against QSpec main `10a65eb9d6834b022a5076f5cbac4a41461abbb3` FR-330 AC-8 and TC-936. FR-035-AC-5 retains Test verification. TC-047 supplies an independent RFC 8785/SHA-256 oracle and both positive and refusal checks for the changed obligation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean for this method. Examined FR-035 Outputs and AC-5 and TC-047 Test Procedure and Expected Results. The two separate QSpec defensive-payload and reader-input questions are outside this narrow diff.
