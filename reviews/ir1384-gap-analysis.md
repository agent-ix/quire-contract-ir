---
id: SR-202611070951309518
title: IR-1384 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; IR-1384; changed model authority, fixture tests,
  standalone conformance lane, TC-471 and matrix status
review_set: subset
---

## Summary

Ticket: IR-1384. Frozen PR #975 source reviewed read-only, including dependency refresh and fixture conversion. No applicable AssuranceProfile exists under spec/. No source or build cache was modified. Production authority rows keep private fields, borrow byte values unchanged, and add no QSL/observation/runtime dependency. No decode/re-encode authority bridge or second plan identity was found.



## Verdict

**FAIL** — Repository static gaps remain.

Full FR-361 acceptance is not claimed: layout-context AC-3 and non-authority AC-4 remain separately owned; full two-occurrence AC-1, refusal permutations AC-2 and per-charge AC-4 are partial. AC-6 final quire-integration qualification remains with that owner. Published QSL fails compilation with CheckedJson/Value and Dependency exhaustiveness errors (compile2.log); genuine receipt tests are unrun. Workspace check and one strict fixture test pass in existing logs; focused tests and Clippy remain queued. No aggregate/Kani gate run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-019-AC-27 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-019-rust-library-interface.md: |
| FND-002 | high | FR-028-AC-4 is tagged-by-ignored-test in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-028-separate-cycle-free-contract-model.md: |
| FND-003 | high | FR-030-AC-7 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md: |
| FND-004 | high | FR-035-AC-5 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md: |
| FND-005 | high | FR-035-AC-12 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md: |
| FND-006 | high | FR-036-AC-1 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md: |
| FND-007 | high | FR-036-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md: |
| FND-008 | high | FR-036-AC-3 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md: |
| FND-009 | high | FR-036-AC-4 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md: |
| FND-010 | high | FR-036-AC-5 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md: |
| FND-011 | high | FR-038-AC-10 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-012 | high | FR-038-AC-11 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-013 | high | FR-038-AC-19 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-014 | high | FR-038-AC-48 is tagged-by-ignored-test in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-015 | high | FR-038-AC-49 is tagged-by-ignored-test in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-016 | high | FR-038-AC-50 is tagged-by-ignored-test in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-017 | high | FR-038-AC-51 is tagged-by-ignored-test in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-018 | high | FR-038-AC-53 is tagged-by-ignored-test in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-019 | high | FR-038-AC-54 is tagged-by-ignored-test in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-020 | high | FR-038-AC-57 is tagged-by-ignored-test in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-021 | high | FR-038-AC-59 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-022 | high | FR-038-AC-90 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-023 | high | FR-038-AC-92 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-024 | high | FR-038-AC-111 is tagged-by-ignored-test in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-025 | high | FR-038-AC-123 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-026 | high | FR-038-AC-126 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-027 | high | FR-038-AC-131 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-028 | high | FR-038-AC-135 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-029 | high | FR-038-AC-148 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-030 | high | FR-038-AC-183 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-031 | high | FR-038-AC-184 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-032 | high | FR-038-AC-215 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-033 | high | FR-038-AC-216 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-034 | high | FR-038-AC-235 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-035 | high | FR-038-AC-503 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-036 | high | FR-038-AC-507 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-037 | high | FR-038-AC-517 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md: |
| FND-038 | high | FR-040-AC-26 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md: |
| FND-039 | high | FR-045-AC-1 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md: |
| FND-040 | high | FR-045-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md: |
| FND-041 | high | FR-045-AC-3 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md: |
| FND-042 | high | FR-045-AC-4 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md: |
| FND-043 | high | FR-045-AC-5 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md: |
| FND-044 | high | FR-045-AC-6 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/core/functional/FR-045-derive-proof-coverage-baseline.md: |
| FND-045 | high | FR-046-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-046-derive-candidate-application-node-identity.md: |
| FND-046 | high | FR-046-AC-6 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-046-derive-candidate-application-node-identity.md: |
| FND-047 | high | FR-052-AC-7 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-052-source-obligation-correspondence.md: |
| FND-048 | high | FR-052-AC-9 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-052-source-obligation-correspondence.md: |
| FND-049 | high | FR-052-AC-10 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-052-source-obligation-correspondence.md: |
| FND-050 | high | FR-052-AC-11 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-052-source-obligation-correspondence.md: |
| FND-051 | high | FR-052-AC-26 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-052-source-obligation-correspondence.md: |
| FND-052 | high | FR-052-AC-27 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/model/functional/FR-052-source-obligation-correspondence.md: |
| FND-053 | high | FR-057-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/conformance/functional/FR-057-bind-cross-repository-conformance.md: |
| FND-054 | high | FR-057-AC-3 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/conformance/functional/FR-057-bind-cross-repository-conformance.md: |
| FND-055 | high | FR-057-AC-4 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/conformance/functional/FR-057-bind-cross-repository-conformance.md: |
| FND-056 | high | FR-057-AC-5 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/conformance/functional/FR-057-bind-cross-repository-conformance.md: |
| FND-057 | high | FR-057-AC-6 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/conformance/functional/FR-057-bind-cross-repository-conformance.md: |
| FND-058 | high | FR-057-AC-7 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/conformance/functional/FR-057-bind-cross-repository-conformance.md: |
| FND-059 | high | FR-057-AC-8 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/conformance/functional/FR-057-bind-cross-repository-conformance.md: |
| FND-060 | high | FR-057-AC-10 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/conformance/functional/FR-057-bind-cross-repository-conformance.md: |
| FND-061 | high | FR-058-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-058-admitted-clause-execution-point.md: |
| FND-062 | high | FR-058-AC-7 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-058-admitted-clause-execution-point.md: |
| FND-063 | high | FR-063-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-063-dimension-equivalence-node.md: |
| FND-064 | high | FR-066-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-066-retain-identity-bound-execution-points.md: |
| FND-065 | high | FR-067-AC-5 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-067-admit-boolean-state-transition.md: |
| FND-066 | high | FR-069-AC-6 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md: |
| FND-067 | high | FR-346-AC-10 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md: |
| FND-068 | high | FR-348-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-348-admit-extension-form-nodes.md: |
| FND-069 | high | FR-350-AC-1 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-070 | high | FR-350-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-071 | high | FR-350-AC-3 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-072 | high | FR-350-AC-4 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-073 | high | FR-350-AC-5 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-074 | high | FR-350-AC-6 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-075 | high | FR-350-AC-7 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-076 | high | FR-350-AC-8 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-077 | high | FR-350-AC-9 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-078 | high | FR-350-AC-10 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-079 | high | FR-350-AC-11 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md: |
| FND-080 | high | FR-352-AC-6 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md: |
| FND-081 | high | FR-357-AC-9 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md: |
| FND-082 | high | FR-357-AC-11 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md: |
| FND-083 | high | FR-357-AC-16 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md: |
| FND-084 | high | FR-357-AC-17 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md: |
| FND-085 | high | FR-357-AC-18 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md: |
| FND-086 | high | FR-358-AC-11 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md: |
| FND-087 | high | FR-361-AC-3 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md: |
| FND-088 | high | NFR-001-AC-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/core/non-functional/NFR-001-determinism.md: |
| FND-089 | high | NFR-001-AC-3 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/core/non-functional/NFR-001-determinism.md: |
| FND-090 | high | NFR-001-M-1 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-091 | high | NFR-001-M-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-092 | high | NFR-001-M-3 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-093 | high | NFR-002-M-1 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-094 | high | NFR-002-M-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-095 | high | NFR-003-M-1 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-096 | high | NFR-003-M-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-097 | high | NFR-003-M-3 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-098 | high | NFR-003-M-4 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-099 | high | NFR-006-M-1 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-100 | high | NFR-006-M-2 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |
| FND-101 | high | NFR-006-M-3 is untagged in the computed repository matrix; this is a repository baseline/planned-work gap, not a claim that IR-1384 owns its implementation. | spec/: |

## Coverage

Reconciliation: quoin matrix 0.29.1. Repository totals: tagged 1026; untagged 92; tagged-by-ignored-test 9; method-without-symbol 26. All 1153 have no run evidence. Plan completion: not assessed. Reverse-gap inspection covers the changed receipt API and fixture/conformance seam; a complete repository behavior inventory was not established, so this is not a repository-clean assertion. Optional semantic gap-analysis was skipped; code-test intent was independently checked in the mandatory code-review lane. The 101 static gaps are recorded individually, including separately owned FR-361-AC-3; they are not new PR regressions.

## Examined scope

```yaml
scope:
- id: FR-019-AC-27
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: A default-feature external consumer imports `SelectedMonitorAuthorityRow`, `SelectedMonitorAuthority
- id: FR-028-AC-4
  path: spec/model/functional/FR-028-separate-cycle-free-contract-model.md
  role: examined
  excerpt: Default, all-feature and minimum-version builds prove that no optional, dev or historical dependency
- id: FR-030-AC-7
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: The IR outcome constructor accepts the observed caller-request or timer-owner deadline source as `Ca
- id: FR-035-AC-5
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 no-hash decision under QSpec #517/#518: the six-member V1 package and its c'
- id: FR-035-AC-12
  path: spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
  role: examined
  excerpt: 'Retired with replacement by FR-035-AC-31: its five-member omission list predates FR-330''s required `'
- id: FR-036-AC-1
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: Generated symbolic ranges equal model-derived inclusive domains, including boundary values.
- id: FR-036-AC-2
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: Missing bounds, absent exact encodings, and unsupported meaning produce a per-item non-success and n
- id: FR-036-AC-3
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: Every generated item keeps its precondition, postcondition, invariant and frame clauses separate.
- id: FR-036-AC-4
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: A provider completes accounting for every requested item before exposing generated bytes or a result
- id: FR-036-AC-5
  path: spec/kani/functional/FR-036-exact-backend-negotiation-and-emission.md
  role: examined
  excerpt: A requested claim for which the provider manifest advertises no matching FR-290 capability settles `
- id: FR-038-AC-10
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: the lock holds no domain-package row and no digest, so there is no `lock.model_selections'
- id: FR-038-AC-11
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: the lock holds no domain-package row and no digest, so there is no `lock.model_selections'
- id: FR-038-AC-19
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: the lock holds no domain-package row and no digest, so there is no `lock.model_selections'
- id: FR-038-AC-48
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1141). A definition reference at a lock, diagnostics.catalog or operation-law site
- id: FR-038-AC-49
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'An `operation.laws[].definition` that carries a member beyond `authority` and `identity`, lacks one '
- id: FR-038-AC-50
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.sources` row, `source_map` region `source` or `diagnostics.entries[].loci[].source` that car
- id: FR-038-AC-51
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1141). A definition reference at a lock, diagnostics.catalog or operation-law site
- id: FR-038-AC-53
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.sources` row carrying `digest_domain` or `digest` refuses as `unknown_member` at the first e
- id: FR-038-AC-54
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `lock.sources` row with an empty `authority` or `identity` refuses as `malformed_wire` at that mem
- id: FR-038-AC-57
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An `operation.laws[].definition` of a profile role (`temporal_profile` or `protocol_profile`) admits
- id: FR-038-AC-59
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: no identity preimage mirrors definition selections and no stale-selection refusal exists.'
- id: FR-038-AC-90
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: `NominalIdentityPreimage::digest` and its byte ceiling do not exist. Canonical envelope b'
- id: FR-038-AC-92
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the preimage types and their'
- id: FR-038-AC-111
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package document holding an inexact numeric JSON token in a node body refuses noncanonical_wire be
- id: FR-038-AC-123
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision: changing the body of a node under a retained '
- id: FR-038-AC-126
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision: the former preimage-key tamper control for in'
- id: FR-038-AC-131
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not re-deriv'
- id: FR-038-AC-135
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1141). Each current QSpec `adverse.json` structural and body-grammar mutation refu
- id: FR-038-AC-148
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: valid body or bound edits un'
- id: FR-038-AC-183
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader computes no expec'
- id: FR-038-AC-184
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader computes no expec'
- id: FR-038-AC-215
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: a selected domain package document has no digest to recompute, compare or refuse as `cont'
- id: FR-038-AC-216
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Withdrawn: a selected domain package document has no digest, so there is no `byte-digest-mismatch`; '
- id: FR-038-AC-235
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Test controls implemented (IR-1003); qualified Inspection remains planned until IR-1006 lands. Indep
- id: FR-038-AC-503
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: ModelDocument stage. For a package with two selected models, a read-storage limit one below the cumu
- id: FR-038-AC-507
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Retained stage. For a genuine package with direct dependencies, a read-storage limit one below the r
- id: FR-038-AC-517
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The existing public types `CheckedPackageV2ReadResult`, `CheckedPackageDispatchResult`, `CheckedPack
- id: FR-040-AC-26
  path: spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recomput'
- id: FR-045-AC-1
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: From a fixture whose inventories contain two criteria, three obligations and two production function
- id: FR-045-AC-2
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: A proved production Kani result credits only its exactly matched claim and declared finite domain wh
- id: FR-045-AC-3
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: 'An effective bound narrower than declared is reported only for that narrower domain; a shadow proof '
- id: FR-045-AC-4
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: Requirement, module, function and obligation reports have separately derived eligible denominators a
- id: FR-045-AC-5
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: An FR-029 `unsupported` negotiation disposition remains visible with its capability and reason and r
- id: FR-045-AC-6
  path: spec/core/functional/FR-045-derive-proof-coverage-baseline.md
  role: examined
  excerpt: Repeating a baseline on unchanged authoritative inventories, policy and results produces the same ro
- id: FR-046-AC-2
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision: an application path is an identity label; the'
- id: FR-046-AC-6
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision: the former candidate-key, retained-body stale'
- id: FR-052-AC-7
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Complete Quire scope covers every authored acceptance criterion in the bound selected corpus and dec
- id: FR-052-AC-9
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: For each of the four owners, the native enumerator or typed adapter issues a `Complete` outcome only
- id: FR-052-AC-10
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: For each owner, stopping before normal exhaustion issues `Incomplete` with retained partial rows and
- id: FR-052-AC-11
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: The neutral model retains an owner-issued outcome only when its source, owner, scope and observed co
- id: FR-052-AC-26
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given a selected QSL checked request, the QSL-owned adapter enumerates every checked requirement and
- id: FR-052-AC-27
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'Given a `quire-code-rs` selected production module, the owner adapter retains that module and every '
- id: FR-057-AC-2
  path: spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
  role: examined
  excerpt: A consumer accepts byte-equal selected published inputs; changing, omitting or adding a selected inp
- id: FR-057-AC-3
  path: spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
  role: examined
  excerpt: A nonempty complete producer run with exact fixture identities, matching native outcomes, observed i
- id: FR-057-AC-4
  path: spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
  role: examined
  excerpt: Empty, truncated, malformed, duplicated, extra or foreign fixture results, a missing fixture or requ
- id: FR-057-AC-5
  path: spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
  role: examined
  excerpt: A selected independent consumer obligation is discharged only by that consumer's actual public-seman
- id: FR-057-AC-6
  path: spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
  role: examined
  excerpt: A matching result with correct shared definition/candidate/execution/input associations qualifies; r
- id: FR-057-AC-7
  path: spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
  role: examined
  excerpt: 'Contract-conformance intake preserves supplied trace IDs; an accepted legacy row without traces, an '
- id: FR-057-AC-8
  path: spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
  role: examined
  excerpt: Missing, failed, vacuous, suspect, stale or unreadable required shared evidence prevents qualificati
- id: FR-057-AC-10
  path: spec/conformance/functional/FR-057-bind-cross-repository-conformance.md
  role: examined
  excerpt: 'An all-obligations-satisfied contribution and the human completion decision remain distinct; absent '
- id: FR-058-AC-2
  path: spec/checked_package/functional/FR-058-admitted-clause-execution-point.md
  role: examined
  excerpt: 'SUPERSEDED by FR-058-AC-9 under QSpec #449 / IR-1040. Original planned text: For each of an invarian'
- id: FR-058-AC-7
  path: spec/checked_package/functional/FR-058-admitted-clause-execution-point.md
  role: examined
  excerpt: 'SUPERSEDED by FR-058-AC-10 under QSpec #449 / IR-1040. Original planned text: Two equal or conflicti'
- id: FR-063-AC-2
  path: spec/checked_package/functional/FR-063-dimension-equivalence-node.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recomput'
- id: FR-066-AC-2
  path: spec/checked_package/functional/FR-066-retain-identity-bound-execution-points.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recomput'
- id: FR-067-AC-5
  path: spec/checked_package/functional/FR-067-admit-boolean-state-transition.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recomput'
- id: FR-069-AC-6
  path: spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md
  role: examined
  excerpt: 'Withdrawn: this historical numbered intake and wire replay criterion included a stale tuple key refu'
- id: FR-346-AC-10
  path: spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recomput'
- id: FR-348-AC-2
  path: spec/checked_package/functional/FR-348-admit-extension-form-nodes.md
  role: examined
  excerpt: 'Withdrawn by the IR-1141 structural-identity decision under QSpec #517: the reader does not recomput'
- id: FR-350-AC-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: Given an admitted package and a QSL `Supported` classification, projection takes its target from the
- id: FR-350-AC-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: Timestamped-event (finite-window), mixed past-and-future and infinite-trace classifications return `
- id: FR-350-AC-3
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Mutating each correspondence-record member''s source input independently (source entry, declaration, '
- id: FR-350-AC-4
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: Fixed-sample projection with a nanosecond unit retains epoch, period and unit and matches them to th
- id: FR-350-AC-5
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: For each of R1 to R8 a genuine owner-produced pair reaches that row with the stated decision, compar
- id: FR-350-AC-6
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: Each binding failure returns its assigned code and premise; an unknown native atom key returns `tl-p
- id: FR-350-AC-7
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Withdrawn: the earlier fixed byte vector, SHA-256 known answers and derived correspondence reference'
- id: FR-350-AC-8
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: For each of the seven limits, `TlLimits::default()` returns the stated default; with the other six s
- id: FR-350-AC-9
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: Given the authenticated binding, exact occurrence and original request maps, projection selects shar
- id: FR-350-AC-10
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: The projection and join retain the exact supplied package admission; the `models` member equals `sel
- id: FR-350-AC-11
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The test constructs the expected correspondence record and join record independently from the owner '
- id: FR-352-AC-6
  path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md
  role: examined
  excerpt: Every other admitted V2 type form either projects to an Exact variant with its full bounds, mode, me
- id: FR-357-AC-9
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: 'Implemented (`quire-integration` PR #47; explicit selected-fact scope only). A `quire-integration` t'
- id: FR-357-AC-11
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: 'PLANNED/UNRUN (`quire-integration` same-emission acceptance). A genuine QSL final emission supplies '
- id: FR-357-AC-16
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: A reached admitted `composite_type`/`map` with a scalar key and a record value joins a collection ro
- id: FR-357-AC-17
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: The closure read requires the same-emission authority row of FR-361 and returns either a complete cl
- id: FR-357-AC-18
  path: spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md
  role: examined
  excerpt: 'The closure read seeds the plan''s `tick_unit` premise with its unit and dimension facts and exposes '
- id: FR-358-AC-11
  path: spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md
  role: examined
  excerpt: '`NativeJoinLocusSource::PackageRead(stage, index)` exists for every `PackageReadStage` of FR-038 in '
- &id001
  id: FR-361-AC-3
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: The root, entry and clause keys equal the plan's resolved structural keys. `package_lock()` and `occ
- id: NFR-001-AC-2
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: For each fixed admitted corpus input and supported profile that yields canonical output, Linux, macO
- id: NFR-001-AC-3
  path: spec/core/non-functional/NFR-001-determinism.md
  role: examined
  excerpt: For each fixed rejected corpus input and profile, repeated runs and supported operating systems prod
- id: NFR-001-M-1
  path: spec/
  role: examined
  excerpt: NFR-001-M-1
- id: NFR-001-M-2
  path: spec/
  role: examined
  excerpt: NFR-001-M-2
- id: NFR-001-M-3
  path: spec/
  role: examined
  excerpt: NFR-001-M-3
- id: NFR-002-M-1
  path: spec/
  role: examined
  excerpt: NFR-002-M-1
- id: NFR-002-M-2
  path: spec/
  role: examined
  excerpt: NFR-002-M-2
- id: NFR-003-M-1
  path: spec/
  role: examined
  excerpt: NFR-003-M-1
- id: NFR-003-M-2
  path: spec/
  role: examined
  excerpt: NFR-003-M-2
- id: NFR-003-M-3
  path: spec/
  role: examined
  excerpt: NFR-003-M-3
- id: NFR-003-M-4
  path: spec/
  role: examined
  excerpt: NFR-003-M-4
- id: NFR-006-M-1
  path: spec/
  role: examined
  excerpt: NFR-006-M-1
- id: NFR-006-M-2
  path: spec/
  role: examined
  excerpt: NFR-006-M-2
- id: NFR-006-M-3
  path: spec/
  role: examined
  excerpt: NFR-006-M-3
- id: FR-361-AC-1
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: An IR-owned strict admitted Timed monitor package with two claim occurrences of one clause and a row
- id: FR-361-AC-2
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: Each of an independent strict read of equal package bytes, another package, a different clause, an a
- *id001
- id: FR-361-AC-4
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: The read succeeds at its exact measured work and storage allowance. One unit below the cumulative ch
- id: FR-361-AC-5
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: API and dependency inspection shows `SelectedMonitorClosureRows::authority` is a required field, the
- id: FR-361-AC-6
  path: spec/checked_package/functional/FR-361-bind-monitor-authority-and-runtime-layout-context.md
  role: examined
  excerpt: 'In `quire-integration`, a genuine `AdmittedPlanAuthority` row binds through the closure read on its '

```

## Method evidence

Coverage diagnostics include unmatched inspection/interface archetype declarations; the totals measure only matched declarations. Untracked tags exist in the repository coverage output and were not treated as changed-code regressions. Repository-wide gaps and diagnostics are baseline observations requiring owner classification, not a demand that IR-1384 implement unrelated requirements.

## Test bindings

Correct trace bindings support partial slices only; they do not establish complete AC coverage.

```yaml
bindings:
- test_id: genuine_qsl_authority_binds_dense_and_tick_emissions
  ac_id: FR-361-AC-1
  trace: correct
- test_id: genuine_qsl_authority_binds_dense_and_tick_emissions
  ac_id: FR-361-AC-4
  trace: correct
- test_id: genuine_qsl_authority_binds_dense_and_tick_emissions
  ac_id: FR-361-AC-6
  trace: correct
- test_id: mismatched_emissions_refuse_with_typed_authority_errors
  ac_id: FR-361-AC-2
  trace: correct
- test_id: mismatched_emissions_refuse_with_typed_authority_errors
  ac_id: FR-361-AC-6
  trace: correct
- test_id: exact_period_comparison_rejects_invalid_denominators_and_text
  ac_id: FR-361-AC-1
  trace: correct
- test_id: exact_period_comparison_reserves_work_and_scratch_at_the_authority_locus
  ac_id: FR-361-AC-4
  trace: correct
```

## Final disposition evidence (round 1)

Final disposition round 1 examines the frozen source at 14d0804dbec41811cc60f1cadf7a38766242742d. Both workspace and standalone conformance formatting passed (fix-format.log exit 0). Workspace locked all-targets check with conformance-cli, 31 monitor-focused tests (31 passed, zero failed/ignored), and workspace all-targets conformance-cli Clippy with warnings denied passed (fix-focused.log exit 0). Static final diff inspection found formatting changes only after the prior source fix. No native build was duplicated. This evidence supersedes earlier queued-check statements in this artifact. The changed IR receipt scope is review-clear and suitable for the user merge request; no aggregate or whole-ecosystem premerge gate is required by this disposition. Genuine QSL lane qualification remains externally blocked in the published QSL compiler, owned by draco/terra; it is not an IR production compilation failure and no QSL workaround or action was taken. No full FR-361, layout-context AC-3/non-authority AC-4, complete AC-1/2/per-charge AC-4, or final quire-integration AC-6 qualification is claimed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-002 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-003 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-004 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-005 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-006 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-007 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-008 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-009 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-010 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-011 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-012 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-013 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-014 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-015 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-016 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-017 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-018 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-019 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-020 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-021 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-022 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-023 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-024 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-025 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-026 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-027 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-028 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-029 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-030 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-031 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-032 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-033 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-034 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-035 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-036 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-037 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-038 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-039 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-040 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-041 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-042 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-043 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-044 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-045 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-046 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-047 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-048 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-049 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-050 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-051 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-052 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-053 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-054 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-055 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-056 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-057 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-058 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-059 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-060 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-061 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-062 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-063 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-064 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-065 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-066 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-067 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-068 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-069 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-070 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-071 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-072 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-073 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-074 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-075 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-076 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-077 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-078 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-079 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-080 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-081 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-082 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-083 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-084 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-085 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-086 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-087 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-088 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-089 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-090 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-091 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-092 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-093 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-094 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-095 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-096 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-097 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-098 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-099 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-100 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |
| FND-101 | deferred | Repository baseline/planned requirement gap lies outside the IR-1384 receipt implementation slice; no full repository or FR-361 acceptance is claimed. Existing owning work retains responsibility. |

Repository matrix FAIL remains a baseline assurance result; these dispositions do not change requirements or turn it into full coverage. No new IR-1384 regression was found in this method.
