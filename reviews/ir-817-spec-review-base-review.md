---
id: SR-11820
title: "spec-review/base review of IR-817 PR #418"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #418; spec/conformance/functional/FR-020-json-conformance-interface.md; spec/conformance/matrix/tests.md; spec/core/functional/FR-012-anchors-clauses-dependencies.md; spec/core/matrix/tests.md; spec/model/functional/FR-023-executable-projection-binding.md; spec/tests.md; tests/it/conformance.rs"
review_set: subset
---

## Summary

Ticket: IR-817. Base checklist over the split of FR-020-AC-3 into FR-020-AC-3 through FR-020-AC-12. IDs are sequential (AC-1..AC-12), no collision with an existing id, no renumbering of AC-1/AC-2, every new row carries `Test (TC-018)`, and `quire validate` passes over spec/plan/reviews (906/906 grammar-clean). No hash/digest/pin use is introduced. Defects found in the split are recorded by the integrity and ears-conformance lenses (SR-11821, SR-11822), not repeated here.

## Verdict

**PASS** for the base checklist. The FR-012-AC-6 row, the FR-023 prose and the core matrix FR-011 row point at FR-020-AC-5, the criterion that now holds the probe/coverage fixture-schema refusal; spec/tests.md and the conformance matrix name the AC-3..AC-12 range. Historical reviews/*.md files that cite the old compound FR-020-AC-3 record what was reviewed then and should be left unchanged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
