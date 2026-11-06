---
id: SR-2026110602354628
title: "FR-358 AC2/AC3 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #934; FR-358-AC-2/AC-3; spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md"
review_set: subset
---

## Summary

Ticket: IR-1367. Reviewed the changed FR-358 criteria, trace tags, all six acquisition family test bindings, the shared limit helper, the new diagnostic-floor test, and the native denial implementation. No applicable AssuranceProfile was found under spec/.

## Verdict

**PASS** — The changed acceptance criteria have actual assertions for exact and one-below limits, zero storage, diagnostic floor, compact refusal and first unpaid charge. No reverse gap or stub is present in the changed surface.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-358-AC-2, role: examined. Exact and one-below limits, typed exhausted kind and charge, conditional first locus, compact refusal and first unpaid charge.
- FR-358-AC-3, role: examined. Logical storage precharge, diagnostic floor, zero-storage refusal and prior nonempty map copy.
- FR-358 Outputs and changed Behavior, role: examined. Owned diagnostic when affordable; compact refusal otherwise; FR-359 boundary.
- FR-358-AC-4, role: context_only. Refusal counter and first unpaid charge semantics.

## Evidence

FR-358-AC-2 and FR-358-AC-3 are tagged by the new source_join_denial_charges_complete_first_locus_or_returns_compact unit test. The shared assert_exact_join helper is called by stage, union, record, tuple, enum and collection fixtures; it asserts equal exact results, typed one-below refusal, zero-storage compact refusal and first unpaid metadata. The changed source path precharges the complete diagnostic before ownership. Matrix computation completed; its unfiltered output was too large to retain as a targeted row extract. Plan completion: not assessed. Optional semantic review: not selected.
