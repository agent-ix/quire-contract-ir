---
id: SR-581001
title: "Base spec review of IR-874 activation relation API clarification"
type: SpecReview
analysis: base
scope: "spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: base
---
# SR-581001: Base spec review of IR-874 activation relation API clarification

## Summary

Reviewed frozen IR #581 against QSpec FR-464-AC-13, the current IR public API and FR-019-AC-5 inventory test. The relation semantics, admitted-only exposure, field shapes, sorting and edge cases agree with QSpec. The FR-019 inventory changes prematurely classify two unimplemented types as exported, so the review is conditional.

Base checklist review used the one-pass spec diff. No active `AssuranceProfile` is present under the IR repository's `spec/assurance/`, so no enforced profile applies to this review. QSpec's node-id digest is its canonical semantic identity and the diff introduces no tracking digest, pin or manifest.

Disposition on the reviewed fix: FND-001 fixed. FR-019 now names both types and the method as planned in prose outside the parsed table, retains FR-038-AC-324 as the full owning obligation, and defers adding the two types to the implemented inventory until the code exports them. The FR-038 API contract and acceptance criterion are unchanged. Review verdict: PASS.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-019 Public items row adds `CheckedProtocolActivation` and `CheckedProtocolActivationRow` although this frozen spec-only head does not export either. TC-058 parses every backticked item in that row as an implemented export and compares it to the crate root, so this change causes the public inventory assertion to fail until code lands. Keep the types in the FR-038 planned API contract; list them as planned outside the parsed public-items table until the implementation PR exports them, then add them to the table in that PR. | FR-019-AC-5; FR-038-AC-324; tests/it/public_interface.rs:881-908; crates/quire-contract-model/src/lib.rs:126-160 |
