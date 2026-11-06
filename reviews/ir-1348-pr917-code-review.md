---
id: SR-2026110414560072
title: IR-1348 PR917 code review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR917; crates/quire-contract-model/src/checked_package/v2/lower.rs, tests/it/checked_package_v2_require_bounds.rs; FR-038-AC-471, FR-038-AC-73"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Ticket: IR-1348. Independent diff review of PR #917 at ab9c97f3870226fa0917a0a88e615684699c4fa8. Rust review is folded into this code-review artifact. No applicable AssuranceProfile was found. The diff has no CI, configuration, dependency, public API, unsafe, schema, or wire change.

## Verdict

PASS for the narrow IR lowering slice. The admitted Integer literal's own typing edge is ignored for the bound-required predicate while its Integer and literal keys remain reachable. The reference to the same Integer from a parameter or record field still triggers `RequiresBound`; unit and compound-unit positions retain their refusal. Final scoped gates are owned by the code lead on the reviewed head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Rust Review

The new condition is restricted to `ValueForm::Literal`, a body-root `CheckedTerm::Literal` with `LiteralKind::Integer`, equality between body type and node semantic type, and the first successor's scalar Integer kind. It changes only the insertion into the bound-required `typed` set; the successor remains traversed and therefore stays in lowered dependencies. The existing edge builder places the node's semantic-type successor first. The public-reader test asserts the exact Integer value `2`, compound result type, both dependency keys, successful lowering, and three distinct bound-required controls. The test uses the repository's public read path and trace attribute. No new panic is reachable from caller-controlled input; test panics are deterministic fixture assertions.

## Coverage

Inspected the complete two-file diff, its edge construction and bound predicate, the QSL-shaped package fixture and control assertions, repository Rust guidance, FR-038-AC-471 and FR-038-AC-73. `git diff --check origin/main...HEAD` passed. The coder reports 15/15 focused tests, scoped Clippy and format checks passed; I did not rerun Cargo while the host's single heavy-build slot was held by the CG lane. The separate strict `value/literal` body-admission rule remains tracked by IR-1350 and is not claimed by this PR.

## Examined scope

```yaml
scope:
  - id: FR-038-AC-471
    path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
    role: examined
    excerpt: "Under a bounds-required profile, a reader-admitted QSL-shaped `quantity.pow` application with a quantity-literal base, an exact `compound_unit` result and a referenced `value`/`literal` Integer exponent whose body is the canonical literal `2` lowers without `requires_bound` from that exponent"
  - id: FR-038-AC-73
    path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
    role: context_only
    excerpt: "Under a bounds-required profile, a quantity position declared directly at a `unit` or `compound_unit` returns `requires_bound` naming that exact underlying type node"
```
