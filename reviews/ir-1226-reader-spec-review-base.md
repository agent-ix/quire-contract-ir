---
id: SR-202611010321
title: "Base specification review of IR-1226 correction in PR 812"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-354-project-admitted-runtime-context.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Ticket: IR-1226. Reviewed the FR-354 input, behavior and AC-11 signature-source correction and the FR-019 implemented public-item inventory and AC-21, against QSpec STD-640's effect-only row and the PR's actual public Rust exports. No applicable AssuranceProfile is declared in this repository's spec tree. Scoped Quire validation and properties extraction passed for the changed FRs. No new hash, digest, pin, copied schema or compatibility route was introduced.

## Scope

| ID | Role | Excerpt |
| --- | --- | --- |
| FR-354 | examined | For each selected object operation, Contract IR SHALL preserve its declaring object and original member identity, admission-retained ordered parameter names and projected types, admission-retained optional result type, and the QSL producer's declared effect, including an intentionally empty effect. It SHALL join the producer operation fact to the admission-retained selected document, object declaration and own operation member; inherited visibility SHALL NOT change that declaring owner or duplic |
| FR-354-AC-11 | examined | A missing, foreign, duplicate or conflicting operation/effect fact, wrong document, owner or member, a producer row with no admission-retained own operation, or unrepresentable admission-retained signature or producer effect returns a named typed outcome without any partial declaration view; an admitted signature without the required producer effect fact refuses. The operation signature comes only from the admission-retained selected `OperationDecl`; the QSpec STD-640 producer effect row contain |
| FR-354-AC-10 | context_only | A selected `Ledger` with own `pay` and `payTaxed` operations (the QSL `child_composite` signature pattern) exposes them in declaration order with their selected declaring owner and original member identities, ordered typed parameters, optional result and producer-declared effects, including an empty effect and a nonempty effect in the test case, sufficient for QSL `ObjectTypeDeclaration::with_operations`; inherited visibility does not change ownership or add an own operation to the subtype. |
| FR-019-AC-21 | examined | A default-feature external producer constructs `CheckedOperationEffectRow` inputs and passes them as optional borrowed `CheckedPackageSidecarsRef::operation_effects` to the V2 encoder without raw JSON; the fixed provenance origin is emitted with the supplied member identity. A default-feature external consumer obtains `None` from `CheckedPackageV2::operation_effects` when the optional carrier is absent and borrowed `CheckedOperationEffectRef` values when present. It reads selected document, decl |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the changed specification statements. The correction removes a producer-signature conflict that STD-640 cannot express, preserves concrete typed effect refusals and makes the public-item inventory agree with the implemented exports. Code and test findings remain governed by their separate review artifacts.
