---
id: SR-13342
title: "Code review of nested ContractPackage reader cause alignment"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/shared.rs, crates/quire-contract-model/src/checked_package/v2/contract_package.rs, tests/it/checked_package_v2_temporal.rs, tests/it/complete_v1_contract_package.rs"
review_set: subset
---

## Summary

Ticket: IR-896. Reviewed PR #483 at 9dd98dc7945c10dd6058b167b9832d17021797ea with the Rust review lane. The implementation rejects the removed cause spellings and retains the other pairing branches; the new test leaves one causeless-code edge unguarded.

## Scope

- FR-035-AC-17, examined: "Given an otherwise valid canonical `ContractPackageWireV1` with an `invalid_body.refusal` whose code is `unknown_contract_version` or `unknown_required_capability`, the reader admits that nested record when `cause` is absent and refuses a present `cause` as `invalid_package`/`invalid-value` at the nested `/cause` pointer, including `unsupported-wire` and `unsupported-feature` respectively, as QSpec FR-322-AC-78 and FR-330 require."
- QSpec FR-322-AC-78 and FR-330 nested refusal rule, context only.

## Verdict

**CONDITIONAL** — one medium test gap. The focused test passed (1 passed); full gates were outside this review pass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The test only tries removed cause spellings, so it does not prove either reader code refuses every present cause; add a recognized cause such as `invalid-value` for each code. | tests/it/complete_v1_contract_package.rs:1098 |

## Evidence

The loop supplies `unsupported-wire` and `unsupported-feature`. Both were removed from `CheckedPackageRefusalCause`, so `serde_json::from_value` rejects them before `cause_admitted` can enforce the no-cause pairing. A regression admitting a still-recognized cause for either reader code would leave this test green. The absent-cause admission and exact outer refusal fields are asserted correctly. No CI workflow changed, and no vendored content or compatibility layer was added.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The fix removed both explicitly named unsupported cause cases from the test; retain them alongside the recognized-cause check so FR-035-AC-17 tests its concrete pairings. | tests/it/complete_v1_contract_package.rs:1097 |

The changed loop now checks only `invalid-value`. A regression that erroneously recognizes `unsupported-wire` or `unsupported-feature` would not fail the new test, although the criterion and TC-047 explicitly name those spellings. No other integration test for this reader names either string.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The loop now supplies a recognized `invalid-value` cause for both reader codes and asserts the exact `invalid_package`/`invalid-value` refusal and `/feature_accounting/1/refusal/cause` pointer; see the after excerpt in the marked Linear disposition. |
| FND-002 | still-open | Add both `unsupported-wire` and `unsupported-feature` cases back without removing the recognized-cause checks. |
| FND-002 | fixed | The tagged test now retains the recognized-cause loop and adds exact refusal assertions for both named unsupported cause strings at head 5442aeb; see the after excerpt in the marked round-2 Linear disposition. |
