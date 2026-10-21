---
id: SR-15319
title: "IR-571 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038, FR-346; spec/checked_package/matrix/TC-225; current TC-225 implementation; QSpec FR-154-AC-14"
review_set: subset
---

## Summary

The existing TC-225 test asserts the former frame-entry refusal and conflicts with the new selection-row requirement.

## Verdict

**CONDITIONAL** — The existing TC-225 test asserts the former frame-entry refusal and conflicts with the new selection-row requirement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Current TC-225 test asserts malformed-declaration at the abstraction frame and says the malformed domain document admits, while the changed FR-346-AC-7 and TC-225 require refusal at the domain selection row. Keep the spec-only change scoped, but mark implementation pending and make the contradictory test a named follow-up before claiming conformance. | tests/it/checked_package_v2_abstraction_relation.rs |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The existing test remains a code defect outside this specification-only PR. TC-225 now explicitly marks the new selection-row cases and the contradictory frame-entry assertion as pending until the separate IR-571 code change. |
