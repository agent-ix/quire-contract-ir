---
id: SR-2026110602354619
title: "IR-1360 spec-review/dependency review"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; IR-1360; FR-352, FR-354, TC-467 and changed native APIs/tests"
review_set: subset
---

## Summary

The physical model-to-Exact dependency remains cycle-free and no Cargo graph changes exist. The changed requirement relationship introduces one logical prerequisite cycle.

## Scope

- role: examined; id: FR-352; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: - target: ix://agent-ix/quire-contract-ir/FR-358     type: depends_on
- role: examined; id: FR-358; path: spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md; excerpt: - target: ix://agent-ix/quire-contract-ir/FR-352     type: depends_on
- role: examined; id: FR-354; path: spec/checked_package/functional/FR-354-project-admitted-runtime-context.md; excerpt: This mandatory parameter is additional to the existing projection work limit; the latter continues to bound projection work, rather than silently creating a native join allowance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new FR-352 depends_on FR-358 edge closes a logical requirement cycle: FR-358 already depends_on FR-352. The enablement ordering is no longer a DAG. Break the cycle through the appropriate reference/factored prerequisite while retaining the real shared-budget prerequisite; this is a specification dependency cycle, not a Rust crate cycle. | spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md:12 |

## Verdict

FAIL. Findings require a same-PR fix and disposition. No applicable AssuranceProfile was found. This reviewer ran no Cargo or final scoped gate. Author logs were inspected: active strict/public tests, workspace Clippy and scoped specification/format checks pass; these results do not prove the missing resource properties. Runtime model metadata is unavailable and is recorded as unknown.

## Dependency Graph

FR-352 → FR-358 → FR-352 is a prerequisite cycle. FR-354 → FR-352 and FR-354 → FR-358 consume the native projection/budget foundation. The crate dependency graph is unchanged.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: FR-358 now references FR-352 for native identity semantics rather than depending on the whole evolving feature. The actual FR-352-to-FR-358 budget prerequisite remains, so the introduced two-node dependency cycle is removed without changing crate dependencies or budget behavior. |

## Round 1 Verdict

PASS for the inspected changed behavior; all findings are fixed, with no new finding. The lead's final scoped gate remains pending. No Cargo or final gate was run by this reviewer.

Evidence inspected: the original-source red log records five intended assertion failures; the fixed log records 16 complete-witness root tests and two strict model controls passing, plus the affected tuple/unit/runtime controls. Workspace Clippy with warnings denied and scoped Quire validation pass.
