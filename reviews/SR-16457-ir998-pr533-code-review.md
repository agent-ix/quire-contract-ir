---
id: SR-16457
title: "IR-998 PR 533 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@e76875c2226178ef5d214b04d8b352491b3e51cf; crates/quire-contract-model/src/checked_package/v2/{flat_wire.rs,hybrid.rs,mod.rs}; tests/conformance_qspec/hybrid.rs"
review_set: subset
---
# SR-16457: IR-998 PR 533 code and Rust review

## Summary

Independent diff-scoped review of the four changed files at e76875c2226178ef5d214b04d8b352491b3e51cf, stacked on IR #532 cc75d144. No public API, CI gate, dependency, or vendored artifact changed. The neutral marker applies only to the strict-wire integer intake.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope examined

- `flat_wire::check` retains each decoded `CheckedNodeKind` only after successful header/body validation. `check_interval_bounds` and `hybrid_formula_nodes` use the aligned kind vector; the prior refusal order, node pointer, and raw reference traversal remain intact.
- `hybrid::validate_models` and `validate_claims` consume graph-validated `CheckedTerm` values for class, boolean initial condition, exact rational one, activation, guard, and holds selection. Existing raw term use outside the 19 targeted rows is unchanged. The closed local enums have no generic catch-all.
- The new TC-048 test actually calls the public reader, rebuilds identities, admits rational `1`, and asserts exact code/cause/locus for `2`, `+1`, and `01`. Removing the new exact-one predicate would make those assertions fail. Existing QSpec hybrid tests cover model and claim refusal paths.
- Rust ownership, bounds, error/locus, and style checks found no defect in the changed paths. `git diff --check` passed. I did not rerun the author's 22/22 focused conformance or the final aggregate gate.

## Verdict

PASS for the changed code and Rust at this draft head. Final-head gate and rebase carry remain pending.
