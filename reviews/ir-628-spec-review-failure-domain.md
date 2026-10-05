---
id: SR-1563
title: "Failure-domain review of quire-contract-ir PR #296 (IR-628 typed model object fields accessor)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@7ffa956c25fe491767cb790d8168e958929000e4; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (items 6, 9, 10 and 15, IR-628-Q2, FR-038-AC-138, FR-038-AC-139, FR-038-AC-143)"
review_set: subset
---

## Summary

Ticket: IR-628. I looked for failure modes the decided items leave unstated: resource bounds, numeric
range, identity, and partial results.

- Identity: `UnknownNode` and `NotModelObjectType` are kept apart. Equality of the package excludes
  the retained models (item 15). That is consistent: equal wire means equal lock digests, which
  means equal documents, which means equal models, so equal packages give equal accessor results.
- Numeric: the bound type is `i128`, the same as the reader's `IntegerBounds`, and a bound outside
  `i128` gives `None` and is never truncated (re-measured, SR-1559). AC-138 does not test that case
  (FND-002).
- Resource: the accessor charges nothing (item 15, Q2). The justification given is that
  "admission's limits bound the retained model". Admission charges `resolve`'s work only for
  objects some read names (FND-001).

## Verdict

Changes requested: one medium finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Item 15 and IR-628-Q2 say the accessor needs no work limit because "admission's limits bound the retained model". The model's size is bounded; the accessor's work is not proportional to it. `resolve`'s most-derived-redefiner pass is quadratic in the redefiners of one target, and memoizes one ancestor `BTreeSet` per redefining owner (`model_members.rs:729-752`). AC-143's own document (a 10000-type chain with 1000 redefinitions) makes that about 10^6 pair checks and about 10^7 retained set entries per call. Admission charged this work only for objects a read names; an unread object never paid it. The accessor would also rebuild `ModelOwners`, one key hash per declaration, on every call, because `ModelOwners` borrows the models and cannot easily be stored beside them. A package a producer controls can therefore make a "total", unmetered call cost far more than admission did. Either charge a caller-supplied limit (reopen Q2), or state a bound on the accessor's work in terms of admitted quantities and pin it in AC-143. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2081, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2155, spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2795 |
| FND-002 | low | No criterion pins the out-of-`i128` bound. The reader parses a string bound with `parse::<i128>()`, so `Int[0, 170141183460469231731687303715884105728]` gives `None` today. AC-138 tests only the `i128` extremes, so a mutation that saturates or wraps a past-`i128` bound into `IntRange` would pass. Add a past-`i128` bound, expecting `None` and the call `Ok`, to AC-138 and TC-227's cases. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2790 |
| FND-003 | low | Item 6 refuses the whole call with `AmbiguousField` when any two effective fields share a name. The section's own rationale ("Why a typed enum with `None` and not a refusal") rejects a whole-object refusal, because it "would hide the range of the `Int` field from a consumer that wants only the ranges". An ambiguous name on one field hides every other field's range in the same way. Either justify the asymmetry, or report the ambiguous name per field so the other fields still return. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2040 |


## Dispositions

Round 1, reviewed at bd47f6aa58501e5d88afff2657f634f9f2b4f382.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
| FND-002 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
| FND-003 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
