---
id: SR-612090
title: "IR-874 PR 647 scoped gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-874; PR #647; FR-038-AC-325/328/329/330, refinement catalog tests"
review_set: subset
---

## Summary

Planless acceptance-chain audit of the refinement catalog code and public-reader tests at PR head 4b2c7aec. The computed Quire matrix marks FR-038-AC-325/328/329/330 tagged; the findings concern specific required cases behind those tags, not absent tags.

## Verdict

CONDITIONAL. Reader code has no identified gap for the examined catalog classes, but two specified adverse-case groups are not asserted. Add focused public-reader mutations in the same PR before claiming the full AC-325/329 evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Effect and Commit class-specific adverse refusals are untested despite the positive case and QSpec FR-375-AC-28. | tests/it/checked_package_v2_refinement.rs:3348 |
| FND-002 | medium | The two-clause path test has no cross-clause substitution or dropped-ancestor refusal. | tests/it/checked_package_v2_refinement.rs:3741 |

## Coverage

Examined FR-038-AC-325/328/329/330, QSpec FR-375-AC-23/26/27/28, TC-048 items 14 through 17, the changed reader and all changed tests. The matrix reports all four IR ACs tagged, and the positive abstract-model and cend cases exercise the public reader. Plan completion: not assessed. Optional repository-wide semantic review was not requested; this scoped review checked the changed AC/test/code chain. No applicable AssuranceProfile was found.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At bb89d019, the public-reader test mutates Commit and ForwardEffect independently for omission, duplicate, wrong node and wrong binder; it asserts code, cause and locus. |
| FND-002 | still-open | At bb89d019, the dropped-ancestor mutation is added, but `Nested::OtherFlow` names no other selected clause; the actual two selected clause names are `OrderFlow` and `Nested::OrderFlow`. A cross-clause name substitution remains untested. |
