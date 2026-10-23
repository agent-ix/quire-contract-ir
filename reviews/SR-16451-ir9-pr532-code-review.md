---
id: SR-16451
title: "IR-9 PR 532 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@819df8e88b7ae56a318cd938f25f08748251c894; Cargo.lock; crates/quire-contract-model/Cargo.toml; crates/quire-contract-model/src/checked_package/v2/model_members.rs; crates/quire-contract-model/src/checked_package/v2/model_members/tests.rs"
review_set: subset
---
# SR-16451: IR-9 PR 532 code and Rust review

## Summary

Independent diff-scoped review of the two selected-model intake markers and focused conversion tests. The neutral macro is imported directly from `ix-trace-rs` as a normal model-crate dependency, with one locked source revision. No QSL or scanner dependency, rules, Make gate, production branch, or public API changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope examined

- `SelectedTypeRef::from_admitted_name` classifies open unit and declared names into owned enum variants, maps closed native names through the typed converter, and refuses empty names. Its `#[string_edge]` marks an actual selected-model admission conversion.
- `NativeTypeRef::from_name` maps exactly ten native names to closed enum variants and refuses unknown names. Its `#[string_edge]` marks an actual typed conversion; later `DomainModel::element_type` dispatches on the enum.
- New TC-460 tests assert every native arm, unknown/refusal cases, ownership of open names, and the selection converter's typed result. The assertions would fail if the converter silently widened or returned the wrong variant.
- The lock updates only `ix-trace-rs` from 0.1.0 at `83c970b` to 0.1.2 at `e29cc07`; the manifest moves the already declared crate from dev to normal dependency. Locked offline Cargo metadata resolves it without a QSL edge. `git diff --check` passes.
- Rust API, error, ownership, safety, and repository convention checks found no defect in the changed paths. This reviewer did not rerun the author's focused compilation or final aggregate gate.

## Verdict

PASS for code and Rust at the exact reviewed head. This is not a repository-wide clean-scan or final-gate verdict.
