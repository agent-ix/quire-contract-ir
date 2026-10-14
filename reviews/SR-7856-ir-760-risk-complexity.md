---
id: SR-7856
title: "Risk-complexity analysis — IR-760 selected population specification"
type: SpecReview
analysis: risk-complexity
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-019-rust-library-interface.md, spec/model/functional/FR-053-selected-population-binding.md, spec/model/matrix/TC-453-selected-population-binding.md, spec/spec.md"
review_set: subset
---

## Summary

The projection depends on a cross-repository source encoding and authenticated occurrence/type association; direct end-to-end QSL fixture verification is planned in TC-453. The base review records the concrete source-association gap.

## Verdict

**CONDITIONAL** — The boundary can be made explicit with the reported clarification.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The highest-risk association is stated but lacks an adverse-source control | spec/model/functional/FR-053-selected-population-binding.md:102 |
