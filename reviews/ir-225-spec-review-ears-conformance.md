---
id: SR-11673
title: IR-225 EARS conformance review of the catalogued member-result requirement text
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-contract-ir (PR 408, diff vs origin/main, merge-); spec/checked_package/functional/FR-038-consume-checked-package-v2.md
review_set: subset
---

## Summary

Ticket: IR-225. EARS reading of the new "Catalogued member results" bullet and FR-038-AC-247. Review-only.

## Verdict

**CONDITIONAL**: one low finding. The bullet's two SHALL clauses are conditional on an observable trigger ("When that kind is `type_argument`", "when the entry has a `member_family` constraint"). Each has a single response and a stated refusal, so the bullet is well formed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-247 is compound. One row carries the general entry-driven rule, the selection-mechanism claim, eight to_rational cases, the generalization to the other entries and the exclusion of five member kinds. A failure of any one marks the whole AC failed, and the matrix cannot show which part is evidenced. Failure: a code PR tags AC-247 on the to_rational tests alone. The matrix then reports AC-247 tagged while the generalization (SR-11670 FND-003) is untested. Split AC-247 into (a) the to_rational cases, (b) the entry-driven rule with at least one non-IEEE adverse case, and (c) the exclusion of the other member kinds. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4326 |

## Coverage

Read the new bullet (3526-3536) and AC-247 (4326) for EARS pattern, trigger, single response and measurability.

## Examined units

```yaml
scope:
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "When that kind is `type_argument`, the application's `result_type` SHALL name exactly the type node named by `operation.member.declaration`; when the entry has a `member_family` constraint, that declared type SHALL satisfy the catalogue's family."
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "The same entry-driven rule applies to the other catalogued `type_argument` member-result entries, including non-IEEE conversions; entries whose member kind is `field`, `position`, `element`, `relationship_end` or `operation` retain their member-specific result rules."
bindings: []
```

## Dispositions

Round 1; reviewed (diff ..).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |

## Disposition round 1 verdict

**PASS**. AC-247 is split into AC-247 (equality and ordering), AC-248 (resolved family and the deferral), AC-249 (member shape), AC-250 (non-IEEE entries) and AC-251 (catalogue-shape fail-closed). Each has its own tests.md row and TC-048 paragraph, so the matrix can show which part is evidenced. Each states a trigger and a single typed response. No new findings.

## Disposition round 2 verdict

Round 2 reviewed (two follow-up commits and a merge of origin/main). **PASS**: the round-1 `fixed` outcome still holds on the merged tree, no row was added and there are no new findings.
