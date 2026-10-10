---
id: SR-4602
title: "Failure-domain review of IR-361 proof coverage"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@0eca02b43bd14db1411abff447a45eacced9752f; AD-008 and FR-045 candidate and projection failure cases"
review_set: subset
---

## Summary

Examined missing mappings and results, producer refusals, inconclusive and unknown terminal states, narrower bounds, and partial projections. One edge case permits a zero-obligation entity to be credited under the stated universal rule.

## Verdict

CONDITIONAL. Require at least one eligible mapped obligation for full criterion/module/function credit, and retain zero-mapping entities as explicit gaps; add that case to FR-045-AC-4 or TC-446.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Full criterion/module/function credit is defined as every eligible mapped obligation qualifying. For an eligible source entity with zero mapped obligations, that condition is vacuously true, despite the separate promise that missing mappings remain gaps. AC-4 tests a one-of-two partial case but not zero mappings, so two conforming implementations could credit or gap the same unmapped function. | spec/assurance/AD-008-proof-coverage-accounting.md:77; spec/core/functional/FR-045-derive-proof-coverage-baseline.md:54; spec/core/functional/FR-045-derive-proof-coverage-baseline.md:66 |
