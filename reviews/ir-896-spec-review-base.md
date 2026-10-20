---
id: SR-13344
title: "Base spec review of FR-035-AC-17 and TC-047"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-896. Reviewed the changed requirement, test case, and matrix prose against the QSpec FR-322-AC-78 and FR-330 authority at origin/main.

## Scope

- FR-035-AC-17, examined: "Given an otherwise valid canonical `ContractPackageWireV1` with an `invalid_body.refusal` whose code is `unknown_contract_version` or `unknown_required_capability`, the reader admits that nested record when `cause` is absent and refuses a present `cause` as `invalid_package`/`invalid-value` at the nested `/cause` pointer, including `unsupported-wire` and `unsupported-feature` respectively, as QSpec FR-322-AC-78 and FR-330 require."
- TC-047 nested refusal case and computed matrix row, examined.

## Verdict

**PASS** — the added criterion is concrete, correctly sourced, and has a TC-047 case and computed trace binding. The test implementation gap is recorded in SR-13342 and SR-13343.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
