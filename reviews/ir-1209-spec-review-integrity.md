---
id: SR-202610310107
title: "IR-1209 PR 748 integrity analysis"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR #748; spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-353
    type: references
---

## Summary

Ticket: IR-1209. Checked completeness, consistency, atomicity and testability of FR-353 against QSpec FR-322 and QSL CheckedEquality. FR-353 traces to StR-001, has five verification criteria, separates IR admission/read from QSL construction, and leaves the missing producer carrier to STD-613. Base review FND-001 records the cross-source application-identity ambiguity.

## Verdict

CONDITIONAL on base review FND-001. No separate integrity finding was identified.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-353-AC-1 | examined | A public external consumer reads admitted `=` and `!=` applications with distinct occurrences and receives the producer-selected operator, original operand identities, source and conversion target types, and exact schedule in source order. Text, enum, quantity and plan cases retain their respective schedule; the enum case has only its compared declaration's member index and the quantity case has the resolved top-level units. The result contains no JSON body or partial decision. |
| FR-353-AC-2 | examined | Admission refuses a decision whose package, application, occurrence, operator or operand identity differs from the complete admitted application, and refuses two decisions for one application occurrence. Each refusal has a named typed code and the first mismatching structural locus; neither decision becomes readable. Display-name or type equality does not establish the join. |
| FR-353-AC-3 | examined | An admitted equality application with no authoritative producer decision returns `UnavailableEqualityDecision`; unknown or foreign node, non-application, ineligible operation and wrong occurrence return their distinct typed outcomes. None returns a fabricated schedule, an empty unit or enum snapshot in place of a required one, or a partial view. |
| FR-353-AC-4 | examined | A finite work limit equal to the charged read succeeds and one less returns typed exhaustion at the first uncharged application, operand or decision entry, with attempted work. Repeated reads and a package clone give equal outcomes and do not reparse package or selected source bytes. |
| FR-353-AC-5 | examined | Code inspection shows the accessor reads an admission-retained decision, performs no `TypeEnvironment::check_equality` or equivalent type-checking pass, exports no QSL checker or evaluator type from the IR model crate, and leaves construction of QSL `CheckedEquality` to QSL's trusted checked-stage boundary. |
