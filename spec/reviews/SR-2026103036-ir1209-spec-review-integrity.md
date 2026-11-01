---
id: SR-2026103036
title: "spec-review/integrity review of PR #755 FR-353"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md; IR-1209"
review_set: subset
---

## Summary

Reviewed identity uniqueness, producer authority, lossless equality facts, failure cases, and AC traceability in the changed FR-353 text. The local application NodeRef plus exact occurrence is consistent with current QSpec FR-322 and FR-038 grammar. No applicable AssuranceProfile was found under `spec/`.

## Coverage

- `FR-353` (examined; Inputs): - An admitted package, the full structural `CheckedNodeId` of an `expression`/`application` equality node, one occurrence belonging to it, and a finite read work limit. - An equality decision from the same checking stage that produced the package, carried into package admission under an authoritative producer contract. The producer decision identifies the package, application node, occurrence, operator, both original operand positions and source types, any conversions, selected schedule, and the complete resolved `UnitTable` and filtered `EnumMemberIndex` facts retained by the checked equality
- `FR-353` (examined; Outputs): The public read SHALL return a complete IR-owned typed equality decision view or a closed typed refusal. A successful view SHALL carry the admitted application node and occurrence, equality or inequality operator, each operand's source type and optional conversion target, the selected text, enum, quantity or plan schedule, and the complete resolved unit-table and filtered enum-member closure retained for that schedule. A quantity view SHALL preserve compound unit dimensions, canonical root-unit terms/exponents and exact conversion data as well as top-level operand units. An enum view SHALL pre
- `FR-353` (examined; Behavior): The QSpec-owned `checked_term_data.equality_decisions` carrier SHALL identify each decision by its `application` `NodeRef` and exact `occurrence`. The application SHALL name one local `expression`/`application` node. The reader SHALL reject a foreign or absent node, a node of another form, and duplicate rows with one (`application`, `occurrence`) key. The occurrence SHALL be checked against that node and retained as provenance. No body-relative term path exists in the current V2 carrier, and this FR requires no new wire member. The reader SHALL retain a producer decision only during admission 
- `FR-353-AC-1` (examined; Acceptance Criteria): A public external consumer reads admitted `=` and `!=` applications by full application node identity and exact occurrence, and receives the producer-selected operator, original operand positions, source and conversion target types, and exact schedule in source order. Text, enum, quantity and plan cases retain their respective schedule; the enum case retains the complete filtered index for its compared declaration, including declaration/member identities, ordered flag, cases and ranks, while the quantity case retains every resolved `UnitTable` fact used by its schedule, including compound-unit
- `FR-353-AC-2` (examined; Acceptance Criteria): Admission refuses a decision whose package, full application node, occurrence, operator, operand position or source type differs from the complete admitted application, and refuses duplicate (`application`, `occurrence`) keys. An absent or foreign node, or a node that is not an `expression`/`application`, is a named typed refusal; neither decision becomes readable. Distinct application nodes retain distinct decisions even when they share an operator and operand types. Display-name or type equality does not establish the join.
- `FR-353-AC-3` (examined; Acceptance Criteria): An admitted equality application with no authoritative producer decision returns `UnavailableEqualityDecision`; unknown or foreign node, non-application form, ineligible operation and wrong occurrence return their distinct typed outcomes. None returns a fabricated schedule, an empty unit or enum snapshot in place of a required one, or a partial view.
- `FR-353-AC-4` (context_only; Acceptance Criteria): A finite work limit equal to the charged read succeeds and one less returns typed exhaustion at the first uncharged application, operand or decision entry, with attempted work. Repeated reads and a package clone give equal outcomes and do not reparse package or selected source bytes.
- `FR-353-AC-5` (context_only; Acceptance Criteria): Code inspection shows the accessor reads an admission-retained decision, performs no `TypeEnvironment::check_equality` or equivalent type-checking pass, exports no QSL checker or evaluator type from the IR model crate, and leaves construction of QSL `CheckedEquality` to QSL's trusted checked-stage boundary.
- `FR-353` (examined; Dependencies): QSpec [STD-613](https://linear.app/agent-ix/issue/STD-613) supplies the FR-322 `checked_term_data.equality_decisions` wire, schema and vectors for the authoritative, bounded carrier. Current IR admission does not yet retain the producer's schedule, conversion choices, complete resolved unit-table closure or filtered enum member index. This FR allocates no further V2 wire member and grants no authority to reconstruct a decision from type or catalog data. QSL [QSL-1390](https://linear.app/agent-ix/issue/QSL-1390) owns the emitter and trusted construction boundary for its private `CheckedEquality

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. The changed specification is internally consistent with the cited current producer contract. This is a spec-only review; implementation and end-to-end producer behavior remain future work.
