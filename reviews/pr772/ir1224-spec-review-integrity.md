---
id: SR-202610310135
title: "IR-1224 spec-review/integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-1224; TextBounds form/base join and public-reader regression controls"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Independent scoped review of PR #772: structural.rs, the new FR-038 paragraph and AC-382, and two public-reader tests in checked_package_v2_structural_keys.rs. The change refuses a text_bounds form over a non-Text scalar base while retaining a valid Text base and its own body.

## Verdict

PASS for the source/specification review, with no findings. Merge remains conditional on the user-required full IR integration target or a successful CI Test. Formatting, focused checks and lint are separately required and recorded below. Focused green evidence is two public-reader tests and workspace all-target Clippy; the negative public-reader control also has an intended red result against the original guard. No full-gate or CG success is claimed by this artifact. Reviewer ran no Cargo command or proof and edited no source.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

- FR-038-AC-382 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: A public-reader mutation that relabels an integer-range node as `text_bounds` while retaining its Integer semantic base and body refuses `invalid_package`, no cause, at `/semantic_graph/nodes/<n>/body`, with that node as locus. A `text_bounds` node over Text retains its own minimum, maximum and text profile unchanged.
- FR-038-AC-124 (context_only), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: PLANNED/UNRUN (IR-1141). Each `scalar_type`/`boolean` or `/integer` body is the empty aggregate; `reference`, `option`, `set`, `bag`, `sequence` and `ordered_set` have exactly one reference; `integer_range` and `collection_bounds` have ordered `min` and `max` integer-literal bindings of their specified decimal grammar. A missing, extra, reordered, wrongly typed or malformed member refuses `invalid_package`/`invalid-derived-shape` at that node without using a content-derived key. For `composite_type`/`reference`, a valid body target must be a model `object_type`, `process`, `state_machine` or `
- FR-038-AC-125 (context_only), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: PLANNED/UNRUN (IR-1141). A `scalar_type` or `composite_type` in those ten shapes is self typed; an `integer_range` is typed at the exact Integer node and `collection_bounds` at a collection node. A wrong semantic type or a bound literal typed at Boolean refuses `invalid_package`/`invalid-derived-shape` at the offending node; a valid structural id alone does not bypass the body/type check.
- FR-038 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: A `bounded_domain`/`text_bounds` node SHALL have a `scalar_type`/`text`
semantic base.

## Analysis

The existing sorted structural-key traversal remains the refusal order. The new arm performs one lookup through the existing graph kind resolver and requires exactly scalar_type/text as the semantic base. A mismatch returns the specified InvalidPackage code, absent cause, body pointer and original node locus without inferring from its authored path or another bound. The positive path continues ordinary reader admission; no node rewrite, copied schema, mirror type, dependency, compatibility form, unsafe block, evaluator or resource-limit change is introduced. The established IntegerRange/CollectionBounds validators remain unchanged. This is the assigned form/base correction, not a claim that every TextBounds body grammar is newly validated.

The adverse test relabels the existing admitted integer-range fixture only, preserving its integer semantic type, body and structural node ID. It invokes the public reader and asserts every requested refusal component. The documented red run admitted that mutation before the guard. The positive test supplies an admitted Text base, independent min/max values and text_profile nfc; it asserts the retained base and complete own body, rejecting a blanket TextBounds refusal. Both controls carry the actual new criterion attribute. No assertion is bypassed by a guard or ignored marker.

AGENTS/CLAUDE and applicable Rust API, safety, wire and test guidance were examined. No manifest or CI workflow is changed, no required gate is narrowed, and no active repository AssuranceProfile was found. The repository's existing AC-364 grammar advisory lies outside this diff and is not attributed to the new criterion.

The new criterion ID is unique, the table and document validate, and existing relationship/verification edges remain intact. Its observable refusal code, optional cause, exact body locus and successful-retention counterpart match the implementation and test oracle. The new ubiquitous SHALL statement has a concrete subject and directly observable type constraint; no new EARS advisory is emitted. The integrity lens finds one coherent form/base obligation with adverse and positive controls, no duplicate criterion, conflicting rule or hidden external-service assumption. No dependency-edge or domain-object analysis is applicable to this diff.

## Validation status

Inspected parent logs: the focused IR run reports two passed tests, and workspace Clippy completed with warnings denied. The required unchanged CG regression attempt currently stops before the test with a task-local dependency API mismatch: selected QSL constructs a boxed node-path operand while current IR exposes Arc. This integration condition is reported to the parent; it is not evidence of a defect in the twelve-line TextBounds guard. Final merge must use the actual passing full IR integration or CI Test receipt, not this review's clean source conclusion. The separate unchanged CG AC16 attempt is blocked before execution by the reported cohort API mismatch; it is not counted as a pass or made an additional invented release condition.

## Additional same-head validation

The complete touched structural module reports 33 passed tests, including the two added controls. Formatting output is empty with parent-reported exit zero. The official direct scanner reports a pass. CI Rust Checks has passed lint; its Test step remains in progress. Either the complete local IR integration target or a successful CI Test must finish before merge, as authorized by the user. This reviewer ran no Cargo or scanner command.
