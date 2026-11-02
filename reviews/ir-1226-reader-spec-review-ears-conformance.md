---
id: SR-202611010323
title: "EARS conformance of IR-1226 correction in PR 812"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-354-project-admitted-runtime-context.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Ticket: IR-1226. Scoped Quire validation of the two changed FRs returned no new EARS diagnostic, and the changed obligations retain a named Contract IR subject and concrete refusal or output. The amended AC-11 is a test criterion rather than a new free-standing SHALL statement. No applicable AssuranceProfile is declared in the spec tree.

## Scope

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-354 | examined | For each selected object operation, Contract IR SHALL preserve its declaring object and original member identity, admission-retained ordered parameter names and projected types, admission-retained optional result type, and the QSL producer's declared effect, including an intentionally empty effect. It SHALL join the producer operation fact to the admission-retained selected document, object declaration and own operation member; inherited visibility SHALL NOT change that declaring owner or duplic |
| FR-354-AC-11 | examined | A missing, foreign, duplicate or conflicting operation/effect fact, wrong document, owner or member, a producer row with no admission-retained own operation, or unrepresentable admission-retained signature or producer effect returns a named typed outcome without any partial declaration view; an admitted signature without the required producer effect fact refuses. The operation signature comes only from the admission-retained selected `OperationDecl`; the QSpec STD-640 producer effect row contain |
| FR-019-AC-21 | context_only | A default-feature external producer constructs `CheckedOperationEffectRow` inputs and passes them as optional borrowed `CheckedPackageSidecarsRef::operation_effects` to the V2 encoder without raw JSON; the fixed provenance origin is emitted with the supplied member identity. A default-feature external consumer obtains `None` from `CheckedPackageV2::operation_effects` when the optional carrier is absent and borrowed `CheckedOperationEffectRef` values when present. It reads selected document, decl |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the changed statements. No new grammar or semantic-trigger defect was found. Existing unrelated specification warnings are outside this PR diff.
