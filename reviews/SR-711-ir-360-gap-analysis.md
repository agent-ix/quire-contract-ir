---
id: SR-711
title: "gap analysis of PR 245 (IR-360 numeric operand ranges)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@9f57b4518882cb6bbd0dba6c6be6988336cacc80; crates/quire-contract-model/src/expression.rs, spec/core/non-functional/NFR-003-diagnostic-integrity.md, spec/model/functional/FR-014-expression-semantics.md"
review_set: subset
---
# SR-711: gap analysis of PR 245

## Summary

Ticket: IR-360. Reviewed head 9f57b45 against origin/main 4233b56. Quire 0.33.0 (engine
0.47.1). This was a manual gap analysis scoped to the diff. The PR changes no `spec/` or
`plan/` files and has no observable behaviour change, so the full planless audit adds nothing
here.

The code changed is the numeric-operator dispatch in `expression.rs`. Two requirements own
it. FR-014 (expression semantics) owns the operator results, and existing tests already cover
them. The whole workspace test suite passes, and the corpus output is byte-identical to the
base. NFR-003-AC-1 ("Every declared invalid-input class returns a stable diagnostic code and
no public panic") is the no-panic intent behind IR-360. The PR moves toward it: four panic
arms become a stable `IllTypedExpression` refusal. No requirement, AC or matrix row is added,
removed or re-traced. The `make spec` failure (23 planned unbacked rows) is the same as on
main and does not involve this file.

The PR adds no test for the new fall-through arms. As SR-710 shows, no input reaches them.
They are dead code, so a test could not reach them either without breaking the invariant from
inside the module. That is a design point, and SR-710 FND-001 records it. It is not a
coverage gap against any AC.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. Nothing in the diff lacks an owning requirement, and no requirement loses coverage.
