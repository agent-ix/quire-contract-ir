---
id: SR-1562
title: "EARS conformance review of quire-contract-ir PR #296 (IR-628 typed model object fields accessor)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@7ffa956c25fe491767cb790d8168e958929000e4; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (section 'Typed accessor for a model object type's fields' items 1-15)"
review_set: subset
---

## Summary

Ticket: IR-628. The section states "Each of the following is its own requirement", so each of the
fifteen numbered items was checked as one EARS statement. Items 3, 4, 5, 6 and 10 use event-driven
`When ... shall` grammar. Items 1, 2, 7, 8 and 9 are ubiquitous with a single `shall`. Items 3, 6
and 7 name a concrete response. The items with more than one `shall` are 10, 11, 12, 13 and 15:
item 10 has two, item 11 three, item 12 two, item 13 two, and item 15 four plus a second sentence.

## Verdict

Approved with low findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Item 15 is compound. "shall be pure and total", "shall not panic", "shall charge no work limit", "shall return equal results ... for a clone" and "Retained models shall not take part in `PartialEq`" are five obligations in one item. Items 10, 11, 12 and 13 are compound too, with two to three `shall` clauses each. Split each into atomic statements, one `shall` apiece. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2081, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2058, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2064, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2067, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2072 |
| FND-002 | low | Item 14 ("The code change shall add the five new type names to FR-019's Public items table ... in the same change") is a process obligation on a change, not a behavior of the system. It is not an EARS requirement and belongs in the plan or the code change's checklist (see also SR-1559 FND-001). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2079 |


## Dispositions

Round 1, reviewed at bd47f6aa58501e5d88afff2657f634f9f2b4f382.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
| FND-002 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
