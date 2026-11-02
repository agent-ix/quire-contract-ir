---
id: SR-2026110283402
title: "IR-1003 PR 834 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 834; tests/it/exact_projection_limits.rs and tests/it/main.rs; FR-352-AC-7 only"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/quire-contract-ir/FR-352
---

## Summary

Scoped independent review of IR-1003 group 4. Code remains read-only. The code-review includes the Rust lane. No production, specification, CI, dependency, lockfile, capability or default changes appear in this diff. No complete spec-review was required.

## Verdict

**CONDITIONAL**. One medium test-evidence defect prevents claiming complete AC7 closure. Repeat/clone equality, typed scalar and bounded-range results, exact work limits and their one-below outcomes, strict admission, and dropping authored wire before public reads are covered. The nested/cyclic work contract remains unverified by this new test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Deep and cyclic fixtures stop at unsupported option roots, leaving nested projection work untested. | tests/it/exact_projection_limits.rs:95; crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:1111 |

The 4096-level chain and self-cycle both select composite_type/option roots and expect UnsupportedExactType at work 2. project_exact_type reads only the requested node and its semantic type, then the unsupported-kind arm returns without following either reference. Their one-below cases exhaust before inspecting the type. Removing all 4096 links or breaking the cycle would preserve every projection assertion. Consequently this test does not exercise nested or cyclic projection traversal, depth-dependent stack behavior, or exhaustion at a visited inner locus. Add an admitted supported nested/recursive projection path (with required authoritative witnesses) and assert completion or charged exhaustion after inner traversal; if the current API cannot traverse such a path, retain the residual gap rather than crediting the whole AC.

## Coverage

- FR-352-AC-7 (role: examined; excerpt): Repeated reads of one admitted package and its clone return equal complete values or equal typed outcomes. A nested or cyclic type either completes within the finite limit or reports typed exhaustion with the charged work and locus; no call reparses wire/source, grows the call stack with type depth, or grants admission to an unadmitted fixture.
- Both changed files examined, including integration module registration. Rust testing rules require real behavior and assertions that fail for the named defect; the fixtures' deep/cyclic links are never visited by projection.
- Existing receipts reused: `/home/peter/dev/qualification-logs/ir1003-projection-limits/fmt.log` PASS; `lint.log` PASS (workspace and model consumer Clippy lanes plus string-edge scanner); `focused.log` PASS (1 executed test, 1184 filtered out). No new compilation was necessary. Aggregate hosted/local gates remain the parent's responsibility.
- Computed coverage receipt `coverage.json` binds the new FR-352-AC-7 trace. The dispatch's classified census moves 94 residual criteria to 93, removing AC7 mechanically; this does not establish semantic completeness. Inherited residuals in `/tmp/ir1003-current-classification.json` are context, not new findings. This is a scoped gap review, not a repository-wide PASS.
- FR-352-AC-6 is out of scope and untraced; the missing-witness versus Unsupported ambiguity remains with owner `06db3fd4`. No cross-credit, narrowing or fix is proposed here.
- Semantic test/code inspection was explicitly requested by the review dispatch and performed for AC7 only.
- Plan completion: not assessed
