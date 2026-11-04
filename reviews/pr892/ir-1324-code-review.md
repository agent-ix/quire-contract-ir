---
id: SR-20261104133006
title: "IR-1324 PR 892 Rust code review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #892; exact_projection.rs and checked_package_v2_model_fields.rs; FR-352-AC-12..15"
review_set: subset
---

## Summary

Reviewed PR #892 at 9220275f against base f319cf84, corrected FR-352 PR #891, and QSpec FR-154. Applied code-review and rust-review with rust-style and repository idioms to the two changed Rust files. No applicable AssuranceProfile exists under spec/. The PR changes no CI workflow, wire shape, hash, digest, pin, copy, or compatibility layer.

## Coverage

- FR-352-AC-12 (examined): plain collection keeps witnessed field identity, presence, all four kinds and exact bounds.
- FR-352-AC-13 (examined): admitted Collection<Option<Integer>>, optional Collection<Integer>, required option alias, and optional native String preserve their distinct wrappers and presence.
- FR-352-AC-14 (examined): nested nominal record target is witnessed and represented by its original NodeKey; missing witness and unrepresentable leaf refuse without a partial view.
- FR-352-AC-15 (examined): wrappers are inspected and rebuilt iteratively with a charged work boundary; no recursion over the nominal record graph.
- QSpec FR-154 option and presence rules (context_only): no Option<Option<T>> through aliases or optional [1,1] option field is admitted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the implementation diff. The test-criterion gaps are recorded separately in the gap-analysis artifact. No aggregate gate was run in this review; the lead owns the final reviewed-head gate.
