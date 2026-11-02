---
id: SR-2026110283303
title: "IR-1003 PR 833 disposition round 1"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1003 PR 833 FND-001; tests/it/support/cargo_graph.rs, tests/it/support/mod.rs, tests/it/cycle_free_model.rs, tests/it/public_interface.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-352
    type: references
  - target: ix://agent-ix/quire-contract-ir/SR-2026110283301
    type: reviews
---

## Summary

Disposition of SR-2026110283301 FND-001 only. The duplicated traversal is removed; both inventories call the shared canonical Cargo graph helper. The original code and gap artifacts remain unchanged. No additional defect was found in this fix. No applicable AssuranceProfile was found.

## Verdict

PASS — FND-001 fixed. The prior scoped gap-analysis PASS remains applicable; this disposition does not expand its scope or claim final gates passed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | implementation-bug-despite-evidence |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| SR-2026110283301/FND-001 | fixed | Removed projection_dependencies and extracted the existing traversal into support::cargo_graph::production_graph. Both inventory callers now use that implementation; the cycle_free_model wrapper supplies only its ownership predicate. |

## Scope

FR-352-AC-8 (examined): “The public-item and dependency inventories show the projection in `quire-contract-model`, its cycle-free dependency on `quire-exact`, and no QSL checked-expression constructor, checker, evaluator or mirror expression type in IR; Exact has no IR admission dependency. QSL FR-390 AC-6 and AC-11 remain separately verified in QSL.”

## Coverage

Inspected the complete fix diff and the shared DFS against the original production_graph. Normal/build edges remain included, dev-only edges excluded, and resolved IDs preserve renamed and transitive edges. Target-specific edges remain included. Ownership rejection is still controlled by the caller predicate. Root-inclusive results are handled correctly by the updated assertions; cycles produce typed Result errors rather than successful closure sets. Public AST/signature/export oracles are unchanged. The fixture controls exercise cycle rejection, acyclic renamed/build reachability, and dev-edge exclusion through the actual shared helper.

Inspected /tmp/ir1003-surface-fix-logs/focused-checks.sh and every resulting log: formatting, workspace conformance-feature Clippy, model default Clippy and six focused tests passed. These cover both projection inventories, all-feature production graphs, build/dev/cycle/owner controls, downstream-owner rejection and optional CLI confinement. No heavy compilation repeated. Final hosted Test, conformance compilation/corpus and spec gates remain pending with the parent. No production, specification, dependency or CI gate changes were introduced. Plan completion: not assessed.

Original tracker review comments: 65e5b5ef (code/Rust) and 1f4515e0 (gap-analysis) on IR-1003.
