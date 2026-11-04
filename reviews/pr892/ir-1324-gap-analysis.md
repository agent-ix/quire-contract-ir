---
id: SR-20261104133007
title: "IR-1324 PR 892 acceptance evidence gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #892; FR-352-AC-12..15; exact_projection.rs and checked_package_v2_model_fields.rs"
review_set: subset
---

## Summary

Compared the computed Quoin Test Matrix, corrected FR-352 PR #891 criteria, tagged integration tests and production projection in PR #892. All four criteria have static test binders. The new tests exercise the positive mapping, optional presence, one nested missing witness, unsupported leaf, and exact work boundary. They do not exercise two specifically required adverse/deep shapes.

## Coverage

- FR-352-AC-12 (examined): static tag and assertions for four kinds, unbounded and [0,3] bounds, witnessed member IDs and original identities.
- FR-352-AC-13 (examined): static tag and assertions for Collection<Option<Integer>>, optional collection, required option alias and optional String, matching corrected QSpec admission.
- FR-352-AC-14 (examined): static tag and assertions for cyclic nominal target, missing target witness and unsupported leaf; foreign, conflicting and wrong-form target cases are missing from the nested fixture.
- FR-352-AC-15 (examined): static tag and assertions for a 64-sequence chain, 256 KiB thread stack, exact and one-less allowance, clone equality; alternating sequence/option chain is missing.
- QSpec FR-154 option rules (context_only): Option<Option<T>> and optional [1,1] option field are refused at admission.
- Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-15's deep alternating sequence/option chain is untested: the only deep positive builds 64 sequences and uses an option only in a separate shallow field. | tests/it/checked_package_v2_model_fields.rs:52 |
| FND-002 | medium | AC-14's foreign, conflicting and wrong-form nested target refusals lack a test at the first referenced declaration; the new nested fixture checks only a missing target witness. | tests/it/checked_package_v2_model_fields.rs:134 |

## Verdict

CONDITIONAL. The projection code appears to handle the named shapes, but the tagged tests do not yet back the full corrected criteria. Existing general identity-join tests cover some wrong, foreign and conflicting rows without a nested projection and its required target locus.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At head 9a5284c9, the 64 type declarations alternate sequence and option; each option's inner is a sequence. The expected Exact value uses the same 64 wrappers, is compared field by field on a 256 KiB stack, and the exact/one-less allowance and clone assertions remain. |
| FND-002 | fixed | At head 9a5284c9, the nested cyclic record fixture now feeds foreign, duplicate-conflicting and wrong-form target B rows into the stage join. Each asserts native code, requested and at locus, selected_identity form and work=0; the missing target projection still asserts B as its locus. |
