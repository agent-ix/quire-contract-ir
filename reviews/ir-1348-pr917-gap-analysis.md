---
id: SR-2026110414560073
title: IR-1348 PR917 gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR917; crates/quire-contract-model/src/checked_package/v2/lower.rs, tests/it/checked_package_v2_require_bounds.rs; FR-038-AC-471, FR-038-AC-73"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Ticket: IR-1348. Planless diff-focused matrix, code-to-spec and test-intent audit of PR #917. The computed matrix tags FR-038-AC-471 at `tests/it/checked_package_v2_require_bounds.rs:736`; the new production branch and test have FR-038 ownership. No copied QSpec fixture, stub, disabled test, or hand-written matrix was added.

## Verdict

CONDITIONAL for the full FR-038-AC-471 criterion: PR #917 implements and tests its bound-required lowering statements, while the existing strict-admission clause is still unimplemented and explicitly assigned to child IR-1350. The PR body and IR-1348 ticket state this boundary honestly; the narrow lowering slice may land without claiming complete criterion coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-471's nonliteral `value/literal` strict-admission control remains unimplemented even though the computed matrix marks the criterion tagged by this PR's lowering test; IR-1350 owns the coordinated reader and shared-fixture correction. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5871 |

## Coverage

Plan completion: not assessed.

`quire matrix --scope . --format tsv` reported FR-038-AC-471 `tagged` at the new test. The test's positive and parameter/record/unit/compound negative assertions each exercise public reader admission followed by the real lowerer. The code branch has direct FR-038 ownership; no reverse-gap or hollow-test defect was found in the changed lines. The matrix's tagged status does not establish the criterion's separate strict-admission sentence. That gap predates the code diff and has a linked child ticket; the PR does not claim to close IR-1348 or IR-1350. No plan was selected; unrelated repository criteria were outside the requested diff review.

## Examined scope

```yaml
scope:
  - id: FR-038-AC-471
    path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
    role: examined
    excerpt: "A `value`/`literal` node with a nonliteral body is refused by strict admission before this lowering rule is reached."
  - id: FR-038-AC-73
    path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
    role: context_only
    excerpt: "Under a bounds-required profile, a quantity position declared directly at a `unit` or `compound_unit` returns `requires_bound` naming that exact underlying type node"
```
