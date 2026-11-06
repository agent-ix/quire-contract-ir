---
id: SR-2026110414560075
title: "IR-1352 PR 919 FR-038 base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 919; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1352. Reviewed the new slot-scope rule and FR-038-AC-473 against QSpec FR-440's case-arm shape, binder levels, and IR's existing protocol-slot and operation checks. Merged PR #918 owns FR-038-AC-472, preserved on the base. No applicable AssuranceProfile exists under `spec/`.

## Verdict

PASS. The positive names a real QSL producer case that presently reaches BinderScope. Each adverse class can be built as a QSpec-shaped IR wire control that passes case shape and type joins before slot scope: sibling arms can have distinct same-name binder nodes of equal payload type; a recursive union payload can type a scrutinee reference; matching payload/result types can type cross-arm and outer reads. The expected typed refusal and protocol-owner locus distinguish these cases from earlier `ill_typed` operation failures. The requirement preserves let/query controls and barred model-state precedence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

QSpec FR-440 gives each arm a binder-reference aggregate and a body, with arm-local levels and separate sibling scopes. IR's current `union_arms_hold` validates arm order, binder count/payload typing, and body result typing; it does not decide lexical scope. The later `BinderScope::slot` is the owning decision and currently traverses case arms as ordinary terms. A test of scrutinee leakage must choose a union-typed binder, such as a recursive union payload, so that the operation join does not preempt the scope refusal; the other adverse controls likewise need type-correct references.

## Coverage

- FR-038 case-arm slot paragraph near line 1850, role: examined. Identity-specific arm scope and refusal precedence.
- FR-038-AC-473, role: examined. QSL positive, same-name sibling positive, leakage/shadowing adverse cases and existing-control retention.
- FR-038-AC-472, role: context_only. Merged PR #918's literal-body admission criterion remains on the base and is not changed by this PR.
- FR-038-AC-465/466, role: examined. Existing lexical let/query and protocol-stage nonleak rules preserved by the new text.
- QSpec FR-440-AC-3/4/5, role: context_only. Case wire, operation joins, and binder levels.
- IR `protocol_body.rs` slot walk and `operations.rs::union_arms_hold`, role: context_only. The reader's current validation order and reachable adverse control route.
