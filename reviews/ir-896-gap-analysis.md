---
id: SR-13343
title: "Gap analysis of nested reader cause criterion"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, tests/it/complete_v1_contract_package.rs, crates/quire-contract-model/src/checked_package/v2/contract_package.rs"
review_set: subset
---

## Summary

Ticket: IR-896. Compared FR-035-AC-17, the computed Test Matrix binder, test assertions, and nested reader implementation on PR #483.

## Scope

- FR-035-AC-17, examined: "Given an otherwise valid canonical `ContractPackageWireV1` with an `invalid_body.refusal` whose code is `unknown_contract_version` or `unknown_required_capability`, the reader admits that nested record when `cause` is absent and refuses a present `cause` as `invalid_package`/`invalid-value` at the nested `/cause` pointer, including `unsupported-wire` and `unsupported-feature` respectively, as QSpec FR-322-AC-78 and FR-330 require."
- TC-047 nested refusal case, examined.

## Verdict

**CONDITIONAL** — the criterion has a computed binder and the code behaves as specified, but the test does not establish the universal present-cause branch.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-035-AC-17 is tagged but its test exercises only unknown cause strings; add a known cause on both causeless reader codes to verify the pairing check. | tests/it/complete_v1_contract_package.rs:1098 |

## Coverage

Plan completion: not assessed. `quire matrix --scope. --format json` returned a tagged binder for FR-035-AC-17 at the new test. The focused test passed (1 passed). This is a test intent gap, not a missing trace tag or an observed implementation defect.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | FR-035-AC-17 and TC-047 explicitly name `unsupported-wire` and `unsupported-feature`, but the fix removes both inputs from the sole tagged reader test. | tests/it/complete_v1_contract_package.rs:1097 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The tagged test now exercises a recognized `invalid-value` cause for both causeless reader codes and asserts the exact outer refusal fields; see the after excerpt in the marked Linear disposition. |
| FND-002 | still-open | Reinstate the two named unsupported cause inputs, with their exact refusal and cause pointer assertions, alongside the recognized-cause check. |
| FND-002 | fixed | The tagged test now checks both named unsupported cause strings with exact `invalid_package`/`invalid-value` and cause pointer fields, alongside the recognized-cause check at; see the after excerpt in the marked round-2 Linear disposition. |
