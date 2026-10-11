---
id: SR-8061
title: IR-491 AC-180 correction base spec review
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@7d0adeb864110b696a568f8983f64e766ac0615c; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; IR-491"
review_set: subset
---

## Summary

The correction preserves all eighteen type-walk mappings. AC-180 and TC-048 distinguish fifteen reader-admitted public projections from three private IEEE descriptor controls, and require public refusals for the IEEE forms. The code evidence is explicitly planned and unrun.

## Scope

- `FR-038` type-walk mapping and reader-admission paragraph (examined): eleven scalar and seven bounded-domain forms are mapped, with three IEEE forms excluded from public structural-comparison admission.
- `FR-038-AC-180` (examined): fifteen public positive controls, three isolated private controls and public IEEE refusal controls are specified.
- `TC-048` AC180 paragraph (examined): requires independently enumerated keys and separately constructed public refusals.
- `FR-038-AC-231` (context_only): supplies the IEEE refusal rule.

## Verdict

**PASS** — The corrected acceptance and test obligations are falsifiable and preserve the requested 18-form scope. The column-heading inconsistency is recorded in the integrity review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
