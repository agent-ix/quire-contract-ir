---
id: SR-2026110283301
title: "IR-1003 PR 833 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1003 PR 833; tests/it/public_interface.rs; FR-352-AC-8"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: references
---

## Summary

Rust/code review found a duplicated Cargo graph implementation. The existing production_graph traversal was located by searching tests/it for dependency helpers and dep_kinds; it already handles package IDs, build/dev policy, cycles and adverse controls. No applicable AssuranceProfile was found. This review is limited to the dispatched AC-8 surface/dependency inventory; inherited rows are not new findings.

## Verdict

FAIL — the code-review skill requires high severity for a second implementation of an existing rule.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The new projection_dependencies helper duplicates the resolved normal/build dependency traversal already implemented by cycle_free_model::production_graph, creating a second owner for Cargo package-ID reachability and dependency-kind policy. Reuse one shared graph helper for both inventories. | tests/it/public_interface.rs:1049; tests/it/cycle_free_model.rs:249 | implementation-bug-despite-evidence |

## Scope

```yaml
- id: FR-352-AC-8
  path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
  role: examined
  excerpt: "The public-item and dependency inventories show the projection in `quire-contract-model`, its cycle-free dependency on `quire-exact`, and no QSL checked-expression constructor, checker, evaluator or mirror expression type in IR; Exact has no IR admission dependency. QSL FR-390 AC-6 and AC-11 remain separately verified in QSL."
```

## Coverage

Reviewed both new tests, the existing AST parser/export walker, public root inventories, model-owned projection signatures and real Exact return type, canonical resolved package IDs, declared dependency owner checks, normal/build/dev traversal policy and renamed/transitive/cycle controls. The parser fixtures exercise the real syn walker; they are not alternate projection implementations. Existing native IR expression types are legitimate. QSL FR-390 parity is separately owned and receives no credit here.

The supplied coverage receipt marks FR-352-AC-8 backed, with two new symbols; the owner's full census reports the sole 94-to-93 unbacked reduction and zero contradictions. Other inherited rows are outside this scoped verdict. Plan completion: not assessed.

Inspected existing successful receipts in /tmp/ir1003-surface-logs: fmt, workspace conformance-feature Clippy, model default Clippy, and both focused projection inventory tests (2 passed). No compilation repeated. Hosted Test, conformance compilation/corpus and final spec gates remain the parent's responsibility. No production/spec/lockfile/CI diff exists. No separate spec review applies.
