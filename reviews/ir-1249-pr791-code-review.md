---
id: SR-202611010109
title: "Code review of IR-1249 reaches population role-end comparison PR 791"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1249; PR 791 head b8f1a820; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---

## Summary

Ticket: IR-1249. Rechecked the rebased Rust diff on merged IR-1241 harness against FR-038-AC-274, with FR-038-AC-269/279 and QSpec FR-322-AC-73/74/270 as context. The production file and committed review artifacts are byte-identical to the originally reviewed head; no applicable AssuranceProfile was found under `spec/`. The reader compares a population's object type with the declared type of the end selected by the member role; the producer-supplied destination label remains a separate member.

## Verdict

PASS. No diff-scoped code or Rust-style defect found. The code lead owns the final aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

`SelectedEnds.receiver` is the end carrying the named role. The lookup uses that end's selected model declaration form and identity, then compares the exact typed population object key. The existing destination label parse and navigation checks remain in their prior order, while a mismatch retains `ill_typed`/`operator-ineligible` at `member.population`. No public API, wire shape, dependency, CI workflow, unsafe block, panic path, copied fixture, or hash/pin record changed. The focused model caller test passes (1/1), and the three QSpec reaches population cases pass (3/3) by reference against QSpec `a90a4903` after IR-1241 merged. `cargo fmt --all -- --check` and `git diff --check` pass. Workspace Clippy, string-edge, and the broader gate are coder/lead evidence, not rerun in this review.

## Scope and bindings

Examined FR-038-AC-274: “A reaches application whose `operation.member.population` names a `model_population` node over an object type other than the declared type of the selected relationship end role, including a subtype of that type, is refused by the reader with `ill_typed`/`operator-ineligible` at the member's `population` pointer, and one over exactly that role end type is admitted.” Examined `operations.rs` and its selected-end and population-type helpers. Read FR-038-AC-269/279 and QSpec FR-322-AC-73/74/270 as context. The changed test `tc_048_relationship_population_caller_uses_retained_kind` binds to AC-274 and asserts role-end admission with a differing destination label, invariance when that label changes, and the exact wrong-object refusal. Existing full-reader test `tc_048_a_population_of_another_object_type_than_the_receiver_end_is_refused` binds to AC-274 and checks both subtype and sibling refusals. The by-reference QSpec reaches positives now execute through merged IR-1241 and pass; their fixture data remains in QSpec.
