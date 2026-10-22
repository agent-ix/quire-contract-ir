---
id: SR-15653
title: "Gap analysis — IR-276 independent V2 record lowering PR 517"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-309, FR-019-AC-5; crates/quire-contract-model/src/checked_package/v2/lower.rs and lower/ceiling_tests.rs"
review_set: subset
---

## Summary

This planless, diff-scoped audit reconciled the new FR-038-AC-309 and public-surface inventory claim with their changed tests and source. Ticket: IR-276. No changed-scope gap or hollow trace binding was found.

## Verdict

**PASS** for the changed scope — the byte-ceiling divergence, record order/equality, zero repeated body validation, and one graph/source/SCC index construction are all asserted by traced tests and correspond to the production path.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

- Examined FR-038-AC-309: one record per requested key in input order, exact equality to separate one-key `lower` calls under the same profile and ceiling, one graph/source and component closure build, no extra body validation, and preservation of per-key records when the union package exceeds its byte ceiling.
- Examined FR-019-AC-5 as context: default-feature public items match the Public items table. The new method is listed there and tested through its public type in changed unit tests; the existing inventory test itself is outside the diff.
- `independently_lowered_records_share_one_closure_and_equal_single_requests` binds FR-038-AC-309/TC-050 and checks ordering, exact record equality, body walks, and graph/source/SCC build counts. `independently_lowered_records_do_not_inherit_a_union_package_byte_failure` binds FR-038-AC-309/TC-048 and distinguishes union failure from valid one-key results.
- Reverse gap: the new public method has FR-038-AC-309 and FR-019 inventory ownership. No changed production stub was found. The optional repository-wide semantic audit was not requested; this review checked the changed intent, test assertions, and source directly.
