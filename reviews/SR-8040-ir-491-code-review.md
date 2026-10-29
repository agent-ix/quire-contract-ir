---
id: SR-8040
title: "Code and Rust review of IR-491 residual leaf sources"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/composite_operands.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_composite_operands.rs, tests/it/checked_package_v2_recursive_leaves.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md; IR-491"
review_set: subset
---

## Summary

Reviewed the exact local candidate diff for IR-491 with the Rust review checklist. The operand guard, required leaf source refusal, and IEEE reachability propagation match the stated behavior. The public-reader evidence gaps are recorded in the separate gap analysis.

## Verdict

**PASS** — No distinct source defect found in the changed implementation. This source verdict does not discharge the acceptance evidence gaps.

## Scope

- FR-038-AC-180, FR-038-AC-228, FR-038-AC-229, FR-038-AC-230 and FR-038-AC-231: examined against changed code and tests.
- Rust idioms, panic and unsafe surface, numeric conversions, work bounds, test seams, duplication, and CI workflow changes: examined in the diff; no workflow or public API change occurred.
- The leader reported final candidate `make ci`, formatting, lint, full tests, doctests, corpus, spec validation, deny, audit-unsafe and conformance as passing; this review did not rerun those full gates.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
