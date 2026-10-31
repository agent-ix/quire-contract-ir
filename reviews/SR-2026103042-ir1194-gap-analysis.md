---
id: SR-2026103042
title: "IR-1194 PR 744 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #744; FR-352 scalar projection and tests in crates/quire-contract-model/src/checked_package/v2/exact_projection.rs"
review_set: subset
---

## Summary

The two traced unit tests verify scalar values and several refusal codes through the new projection method. They construct `CheckedPackageV2` directly with private fields and do not exercise the public reader or an external-crate caller. Quoin matrix was available; this review maps changed tests to FR-352 manually because the full repository matrix contains pre-existing open criteria outside this partial PR.

## Verdict

**CONDITIONAL** — One evidence gap in the delivered scalar slice. FR-352 AC2, AC3, AC9 and full admitted-package acceptance remain open pending the QSL identity producer; this partial PR does not satisfy them.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Scalar tests bypass public admission and external caller path | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:283-351 |

## Coverage

- **FR-352-AC-1 (examined):** The test checks Boolean, Integer, String, Uuid, Timestamp, and three inclusive ranges; external caller and reader admission are untested.
- **FR-352-AC-4 (examined):** The test checks changed path, ordinal, and package; declaration witness cases remain open.
- **FR-352-AC-5 (examined):** The test checks three scalar variants and one alias refusal; declared units, union, dimension cases remain open.
- **FR-352-AC-6 (examined):** One unsupported form refuses; other admitted forms use the fallback branch.
- **FR-352-AC-7 (examined):** One clone equality and exhausted-work outcome are checked; owner admission is untested.
- **FR-352-AC-8 (examined):** Cargo dependency and public exports inspected; no QSL mirror is added.
- **FR-352-AC-2, AC-3, AC-9 (context only):** Identity-bearing acceptance remains open.
- Plan completion: not assessed.

## Evidence

The fixture at lines 299-351 deserializes a private wire and directly initializes `CheckedPackageV2` with empty `structural_keys`, default retained models, and empty closure index. That bypasses `CheckedPackageV2::read` and its admission checks. Both new tests use this fixture. Consequently, the tests can pass if the new method works only on an unadmitted internal package and fails for a package returned by the public reader. A focused integration test through the public reader and public projection API would close the scalar-slice evidence gap. Semantic review beyond the changed code and test intent was not run.
