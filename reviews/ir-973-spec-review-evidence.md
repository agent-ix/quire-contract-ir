---
id: SR-15730
title: "Evidence analysis — FR-035-AC-22 typed key association"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; IR-973 PR #523; FR-035-AC-22, TC-047, tests/it/complete_v1_contract_package.rs"
review_set: subset
---

## Summary

Checked whether the declared Test evidence can fail on a wrong source-to-lowered key association, ordinal lookup, missing-key behavior, or reader/producer disagreement.

## Verdict

**PASS** — TC-047 traverses represented nodes and compares both packages' term/key and node/key results with the original admitted source; separate assertions cover unrepresented existing source keys and absent keys. Existing adjacent tests cover the unchanged wire and refusal claims.

## Scope

- FR-035-AC-22 (examined): Producer/reader keyed node and term lookup, absence, and wire-preservation evidence.
- TC-047 (examined): Tagged integration test `lowered_package_retains_typed_terms_by_admitted_node_key`.
- FR-035-AC-5 and FR-035-AC-21 (context only): Existing canonical identity and retained-view tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
