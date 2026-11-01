---
id: SR-202611010110
title: "Gap analysis of IR-1249 reaches population role-end comparison PR 791"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1249; PR 791 head 49dcfb31; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---

## Summary

Ticket: IR-1249. Checked the changed reader comparison and its test bindings against the computed Test Matrix and FR-038-AC-274. The changed behavior has an owning criterion and executed tagged tests. No applicable AssuranceProfile was found under `spec/`.

## Verdict

PASS for the PR diff. No changed-code trace gap or hollow assertion found. This subset review does not claim complete repository coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format json` marks FR-038-AC-274 tagged by the changed private-caller test, the existing QSpec conformance test, and an existing full-reader integration test. The changed test asserts distinct positive and negative outcomes that would fail if the old destination-label comparison returned. The existing full-reader integration test asserts exact subtype and sibling refusals and admission over the named end type. The three QSpec positive reaches cases require IR-1241's graph-index harness repair before they can serve as current end-to-end evidence; this PR does not claim they passed. No new unowned behavior, stub, vendored artifact, or inflated complete-criterion claim was found in the diff. Optional semantic review was not requested.

## Scope and bindings

Examined FR-038-AC-274: “When the selected role end type is `Order` and the producer-supplied `destination_type` is the structural id of `Shipment`, a `Set<Reference<Order>>` population admits; a population over another declared model type refuses at that pointer. Changing only `destination_type` between schema-valid structural ids does not change the population check or its result.” Read FR-038-AC-269/279 and QSpec FR-322-AC-73/74/270 as context. Correct bindings: `tc_048_relationship_population_caller_uses_retained_kind` (changed private caller) and `tc_048_a_population_of_another_object_type_than_the_receiver_end_is_refused` (existing full reader) to FR-038-AC-274. The existing `tc_048_qspec_reaches_population_cases_match_the_reader` is tagged to the criterion but its current external vector lane awaits IR-1241; its tag alone is not counted here as passing runtime evidence.
