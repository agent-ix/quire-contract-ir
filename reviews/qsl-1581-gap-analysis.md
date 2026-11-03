---
id: SR-2026110283846
title: "QSL-1581 PR 857 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; QSL-1581; FR-052-AC-28 through AC-30; crates/quire-contract-model/src/source_obligation.rs, tests/it/source_obligation.rs"
review_set: subset
---

## Summary

Reviewed the FR-052-AC-28 through AC-30 trace chain on PR #857 at 283884c00c02174a080bfdbf0bc2493ceab9df07. The computed Quire matrix marks all three criteria tagged by public model tests. The separate code-review artifact records the invalid Complete outcome used by the AC-28 test.

## Verdict

**PASS for traceability.** The changed public carrier behavior has FR-052 ownership and each new criterion has one tagged test. This verdict does not resolve the code-review finding about the AC-28 test's terminal state.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

| Role | Unit | Excerpt |
| --- | --- | --- |
| examined | FR-052-AC-28 | "SourceRequirementFormId ... distinct QSL-owned RequirementForm inventory identities." |
| examined | FR-052-AC-29 | "CapabilityKind values with two different admitted FR-057 labels ... remain distinct." |
| examined | FR-052-AC-30 | "S1 and S3 spine limits and a composed-lane cause retain only owner-measured fields." |
| context_only | FR-052-AC-26 | "The QSL-owned adapter enumerates every checked requirement and native pending obligation." |

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format tsv` computes FR-052-AC-28, AC-29 and AC-30 as tagged at `tests/it/source_obligation.rs:288`, `:340` and `:395`. The QSL owner adapter remains Planned under AC-26. No evidence-store run or optional semantic review is claimed; the scoped source/test semantic concern is recorded in SR-2026110283845.
