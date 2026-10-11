---
id: SR-7043
title: dependency review of IR-272 FR-330 ContractPackage alignment
type: SpecReview
analysis: dependency
scope: agent-ix/quire-contract-ir@c9e06ba2e99dce5852641073cf72d659d8b33170; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/assurance/AD-001-contract-ir-architecture.md, spec/assurance/AD-004-checked-package-seam.md, spec/checked_package/matrix/TC-044-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-272. FR-035 depends on its checked-package reader and refers to QSpec FR-330 and FR-322 as wire and reference authorities. The spec records IR-743 as emitter work before IR-744 consumer adoption; no dependency cycle or second package role appears in the diff.

## Verdict

**PASS** — no defect found in this method’s examined diff scope. The FR-330 emitter and backend adoption remain planned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

```yaml
scope:
- id: FR-035
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: When Contract IR lowers an admitted checked complete-V1 request, the lowerer SHALL produce one immutable QSpec FR-330 `ContractPackage` under `quire.contract-package/v1` and one exact disposition for each requested item. The package is the single published target-neutral lowering value for backend consumption. Emission of this wire is planned; the current implementation emits a different package and does not satisfy this requirement.
- id: FR-035-AC-1
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Each V1-BACK-001 through V1-BACK-004 and V1-BACK-014 requested item has exact represented meaning under its selected profile or one located record from the seven-member disposition vocabulary; no refusal creates substitute meaning.
- id: FR-035-AC-2
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: A source, type, anchor, identity, bound, dependency, or contract-version mutation that changes or invalidates meaning refuses before a backend artifact; a stale selected dependency does not resolve through an identically keyed local node.
- id: FR-035-AC-3
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Mixed supported and refused requests retain one disposition each and independent sibling results, except the package byte ceiling fails every record under FR-038-AC-95.
- id: FR-035-AC-4
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Every local reference resolves by typed key to a compatible local node with exact source correspondence; a dependency reference resolves only through its selected admitted dependency; a protocol reference targets a `protocol`-tagged node; explicit same-group recursion admits and an undeclared cycle refuses.
- id: FR-035-AC-5
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: One call emits exactly one FR-330 `quire.contract-package/v1` value with the six required properties and one key-ordered `semantic_graph` node collection; repeated equivalent construction orders have identical canonical semantic identity, and changing represented meaning, selected profile, or source correspondence changes identity.
- id: AD-001
  path: spec/assurance/AD-001-contract-ir-architecture.md
  role: examined
  excerpt: '| ContractPackage | `quire.contract-package/v1`; one key-ordered `semantic_graph` with `package`, `source_map`, `feature_accounting` and `identity` | Contract IR output to backend providers; FR-330 emitter planned | FR-035, QSpec FR-330 |'
- id: AD-004
  path: spec/assurance/AD-004-checked-package-seam.md
  role: examined
  excerpt: The target-neutral output is one published `quire.contract-package/v1` value with `package`, one key-ordered tagged `semantic_graph`, `source_map`, `feature_accounting` and `identity` (FR-035, QSpec FR-330). The seven per-item records retain independent sibling outcomes and exact refusal loci. Local semantic references resolve to compatible nodes in that graph. A `dependency_reference` retains the FR-322 selected package identity and resolves against its admitted dependency package, never a same-key local node. A protocol reference targets a `protocol`-tagged node by key. Explicit FR-322 recur
- id: TC-044
  path: spec/checked_package/matrix/TC-044-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: Verify FR-035-AC-1 through FR-035-AC-4 against the FR-330 package contract and FR-322's selected-dependency and tagged-reference rules. The FR-330 emitter cases are planned and have not run.
- id: TC-047
  path: spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md
  role: examined
  excerpt: Verify FR-035-AC-5 and the aggregate FR-330 package separately from TC-044's per-item semantic and refusal tests. The FR-330 emitter cases are planned and have not run.
- id: FR-035-matrix
  path: spec/checked_package/matrix/tests.md
  role: examined
  excerpt: '| FR-035 | FR-035-AC-1 through FR-035-AC-5 | TC-044, TC-047 | 🚧 PLANNED / UNRUN for the FR-330 `quire.contract-package/v1` wire. Current `tests/it/complete_v1_checked_package.rs` and `tests/it/complete_v1_contract_package.rs` exercise the earlier IR package and some per-item semantics, but do not establish FR-330 output, selected dependency and protocol target checks, explicit recursion, total source correspondence or canonical identity as revised here. IR-743 owns emitter implementation; IR-744 owns codegen consumer adoption. |'
```
