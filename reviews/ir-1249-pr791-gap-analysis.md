---
id: SR-202611010110
title: "Gap analysis of IR-1249 reaches population role-end comparison PR 791"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1249; PR 791 head b8f1a820; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---

## Summary

Ticket: IR-1249. Rechecked the rebased reader comparison and its test bindings against the computed Test Matrix and FR-038-AC-274 after IR-1241 merged. The changed production file and committed review artifacts are byte-identical to the originally reviewed head. The changed behavior has an owning criterion and executed tagged tests. No applicable AssuranceProfile was found under `spec/`.

## Verdict

PASS for the PR diff. No changed-code trace gap or hollow assertion found. This subset review does not claim complete repository coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format tsv` marks FR-038-AC-274 tagged by the changed private-caller test, the QSpec conformance test, and an existing full-reader integration test. The changed test asserts distinct positive and negative outcomes that would fail if the old destination-label comparison returned. The existing full-reader integration test asserts exact subtype and sibling refusals and admission over the named end type. The three QSpec positive reaches cases now pass (3/3) against QSpec `a90a4903` with the merged IR-1241 graph-index harness, so the admission path has current end-to-end evidence. No new unowned behavior, stub, vendored artifact, or inflated complete-criterion claim was found in the diff. Optional semantic review was not requested.

## Scope and bindings

Examined FR-038-AC-274: “When the selected role end type is `Order` and the producer-supplied `destination_type` is the structural id of `Shipment`, a `Set<Reference<Order>>` population admits; a population over another declared model type refuses at that pointer. Changing only `destination_type` between schema-valid structural ids does not change the population check or its result.” Read FR-038-AC-269/279 and QSpec FR-322-AC-73/74/270 as context. Correct bindings: `tc_048_relationship_population_caller_uses_retained_kind` (changed private caller) and `tc_048_a_population_of_another_object_type_than_the_receiver_end_is_refused` (existing full reader) to FR-038-AC-274. The existing `tc_048_qspec_reaches_population_cases_match_the_reader` is tagged to the criterion and its by-reference vector lane now passes through the merged IR-1241 harness.
