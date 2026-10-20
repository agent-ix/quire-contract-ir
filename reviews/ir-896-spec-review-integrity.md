---
id: SR-13345
title: "Integrity analysis of nested reader cause criterion"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md"
review_set: subset
---

## Summary

Ticket: IR-896. Checked FR-035-AC-17 and TC-047 for internal consistency, atomicity, and observable outcomes on PR #483.

## Scope

- FR-035-AC-17, examined: "Given an otherwise valid canonical `ContractPackageWireV1` with an `invalid_body.refusal` whose code is `unknown_contract_version` or `unknown_required_capability`, the reader admits that nested record when `cause` is absent and refuses a present `cause` as `invalid_package`/`invalid-value` at the nested `/cause` pointer, including `unsupported-wire` and `unsupported-feature` respectively, as QSpec FR-322-AC-78 and FR-330 require."
- TC-047 nested refusal test definition, examined.

## Verdict

**PASS** — the paired positive and negative cases have observable reader outcomes, and the cross-repository authority agrees with the criterion. The test's narrower implementation is recorded in the code and gap reviews.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
