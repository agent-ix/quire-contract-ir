---
id: SR-4603
title: "Scope-boundary review of IR-361 proof coverage"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@0eca02b43bd14db1411abff447a45eacced9752f; AD-008, FR-045, FR-029, FR-030; Codegen ADR-003, FR-004 and FR-028 as context"
review_set: subset
---

## Summary

Examined owner allocation and the boundary between capability negotiation, Kani outcomes, Codegen execution/refinement and LLVM vacuity observations. One result vocabulary crosses the FR-029/FR-030 boundary.

## Verdict

CONDITIONAL. Treat `unsupported` as a negotiation disposition joined to the source candidate and keep `refused` as a Kani/pre-execution or generation disposition with its producer named. Do not require an unsupported item to have a Kani outcome or run record.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AD-008 lists `refused/unsupported` among admissible Kani result states and FR-045-AC-5 groups them as producer states. FR-029 says `unsupported` settles at profile negotiation and emits no artifact or `KaniOutcome`; FR-030's terminal kinds omit it. A baseline that consumes only Kani result records can therefore misclassify an unsupported candidate as missing, or demand an impossible result. Specify a separate negotiation-disposition join and test it. | spec/assurance/AD-008-proof-coverage-accounting.md:68; spec/core/functional/FR-045-derive-proof-coverage-baseline.md:67 |
