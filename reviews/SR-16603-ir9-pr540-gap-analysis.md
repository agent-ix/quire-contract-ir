---
id: SR-16603
title: "Incremental gap analysis — typed retained-term conversion causes"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #540; crates/quire-contract-model/src/checked_package/v2/typed_terms.rs, mod.rs, contract_package.rs; FR-349-AC-4, FR-038, TC-048"
review_set: subset
---

## Summary

Compared the changed conversion behavior with FR-349-AC-4, FR-038 reader precedence and TC-048 trace evidence. The new unit controls assert each typed cause and first-child propagation; existing public reader tests assert unchanged refusal code, cause and path for abstraction, frame and extension bodies.

## Verdict

**PASS** for the incremental PR scope. The review does not credit IR-9 AC-8 as clean: the new private enum and tests are a typed-cause slice, and the published string-edge campaign remains open.

## Coverage

| Unit | Role | Evidence |
| --- | --- | --- |
| `FR-349-AC-4` | examined | "A malformed or unknown term still refuses at the same first code, cause and JSON pointer as the existing reader, before an admitted typed package is returned." Public tests tagged to this criterion exercise abstraction, frame and extension refusal paths. |
| `FR-038` | context_only | Reader order and public refusal envelope are unchanged in both modified call sites. |
| `TC-048` | examined | Two new tagged unit tests check absent and unknown tags, malformed members, order within a term, and recursive child-cause propagation. |

Plan completion: not assessed. Incremental diff only; repository-wide strict coverage and IR-9 AC-8 remain outside this review's PASS claim. No test matrix file was edited.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
