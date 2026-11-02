---
id: SR-202611010322
title: "Integrity analysis of IR-1226 correction in PR 812"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-354-project-admitted-runtime-context.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Ticket: IR-1226. Checked the changed requirement statements for completeness, consistency, atomicity and testability. QSpec STD-640 has an effect-only selected row; the amended FR-354 now names the admission-retained signature as the single source and does not imply a second producer signature. FR-019 lists the five Rust types actually exported. No applicable AssuranceProfile is declared in the spec tree.

## Scope

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-354 | examined | For each selected object operation, Contract IR SHALL preserve its declaring object and original member identity, admission-retained ordered parameter names and projected types, admission-retained optional result type, and the QSL producer's declared effect, including an intentionally empty effect. It SHALL join the producer operation fact to the admission-retained selected document, object declaration and own operation member; inherited visibility SHALL NOT change that declaring owner or duplic |
| FR-354-AC-11 | examined | A missing, foreign, duplicate or conflicting operation/effect fact, wrong document, owner or member, a producer row with no admission-retained own operation, or unrepresentable admission-retained signature or producer effect returns a named typed outcome without any partial declaration view; an admitted signature without the required producer effect fact refuses. The operation signature comes only from the admission-retained selected `OperationDecl`; the QSpec STD-640 producer effect row contain |
| FR-019-AC-21 | examined | A default-feature external producer constructs `CheckedOperationEffectRow` inputs and passes them as optional borrowed `CheckedPackageSidecarsRef::operation_effects` to the V2 encoder without raw JSON; the fixed provenance origin is emitted with the supplied member identity. A default-feature external consumer obtains `None` from `CheckedPackageV2::operation_effects` when the optional carrier is absent and borrowed `CheckedOperationEffectRef` values when present. It reads selected document, decl |
| FR-322-AC-303 | context_only | A package carries `operation_effects` or omits it; when omitted the effects are unavailable, not empty, and no other member changes. When present it has exactly one closed `{document, declaring, operation, effect, provenance}` row per operation member of every admitted `object_type`, `process` and `state_machine` declaring node, keyed and sorted by `(document, declaring, operation)`, with all four `effect` members always present. Each of `modifies`, `creates`, `deletes` and `deletes_scoped` is c |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the amended statements. They have one source of truth for signature and one for effect, and their negative cases remain observable through typed admission or projection outcomes. Criterion-to-test coverage is separately assessed in the gap-analysis artifact.
