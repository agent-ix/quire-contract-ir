---
id: SR-16452
title: "IR-9 PR 532 incremental gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-056-AC-3/6/7/8; TC-460; two selected-model intake functions and changed tests"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-056", type: references }
---
# SR-16452: IR-9 PR 532 incremental gap analysis

## Summary

Planless, diff-scoped reconciliation of FR-056's marker, dependency, intake, and whole-repository scan obligations with the two changed conversions and TC-460 tests. The PR supports a narrow twelve-row scan reduction, not completion of FR-056-AC-8.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-056-AC-3: The two real intake functions carry the neutral marker and focused TC-460 tests preserve conversion and refusal. Adjacent unmarked and look-alike-marker controls belong to the shared scanner qualification; these new unit tests do not claim them.
- FR-056-AC-6: The model crate directly resolves `ix-trace-rs` in locked offline Cargo metadata; neither changed manifest nor lock adds QSL or scanner dependencies.
- FR-056-AC-7: Both markers are on admission-to-typed-value conversions. No branch rules, file exemption, copied scanner, or post-intake suppression is introduced. The unmarked post-intake cases remain reported and owned elsewhere.
- FR-056-AC-8: The actual whole-repository command remains red. The author reports 261 to 249 syntactic rows, which is consistent with exactly the twelve previously classified selected-model intake rows. This review does not certify the author's scan execution or a clean repository.
- Plan completion: not assessed. No spec or matrix file changed. The final reviewed-head gate remains with the author.

## Verdict

PASS for the incremental slice; FR-056-AC-8 and the other scanner owners remain open by design.
