---
id: SR-16626
title: "Incremental gap analysis — IR-645 TC-223"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-645; FR-030-AC-4/5, TC-223, src/kani/outcome.rs, tests/it/kani_*.rs"
review_set: subset
---

## Summary

Incremental PR-diff trace audit of the FR-030-AC-4/5 to TC-223 to source chain. The zero-check and invalid-cause paths are exercised, but the positive three-check path does not go through the shared run classifier named by TC-223.

## Verdict

**CONDITIONAL** — the positive classification count path needs one focused test assertion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-223 positive three-check control bypasses `proved_from_checks`, leaving its positive count propagation untested | tests/it/kani_outcome_tc223.rs:10 |

## Coverage

Plan completion: not assessed.
FR-030-AC-4 (examined): `KaniOutcome::proved_from_checks(0)` is tested, but the three-check test calls `proved(3)` directly. TC-223's Test Procedure requires building both through FR-030 classification. A defect dropping the positive count inside `proved_from_checks` would leave the three-check tagged test green; the older shared test only compares the one-check result without checking the accessor.
FR-030-AC-5 (examined): fixed cause constructors, invalid broad requests and field privacy probe are present; the privacy probe defect is recorded separately in SR-16625.
TC-223 (examined): tagged cases cover most clauses; the count propagation gap is scoped above. Planless incremental gap review only; no claim of whole-repository strict coverage.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At aa3cdd1, the tagged three-check positive case calls `KaniOutcome::proved_from_checks(3, ...)` and retains assertions for `success_checks() == Some(3)`, Boolean true, and serialization round trip. |
