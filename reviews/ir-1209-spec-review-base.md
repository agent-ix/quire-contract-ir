---
id: SR-202610310106
title: "IR-1209 PR 748 base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #748; spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-353
    type: references
---

## Summary

Ticket: IR-1209. Reviewed new FR-353 at frozen PR #748 head `0a2c7eaa9ecc79af2c2a7c079adc70088db14351` against main `154227dc`, including admitted IR term data, QSpec FR-322, QSL private `CheckedEquality`, STD-613 and QSL-1390 dependencies. The requirement states that current V2 data lacks the producer decision, assigns the carrier to QSpec and emitter/trusted constructor to QSL, and invents no wire member. No applicable AssuranceProfile was found. Targeted Quire validation reports grammar-clean, zero findings; property extraction reports 3/5 extractable, with AC2 and inspection AC5 classed as examples.

## Verdict

FAIL. FND-001 leaves the admitted-decision identity ambiguous for multiple equality terms in one node.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | A checked V2 node can contain more than one equality application in its term tree, but FR-353 identifies a producer decision only by package, structural node id and node occurrence. QSpec FR-322 permits applications wherever they occur in a node body, and `CheckedTerm::Application` has recursive argument terms. Two equality terms in one node share that key; AC2 would reject their two decisions as duplicates or attach one decision ambiguously. Specify a producer-owned way to distinguish each application term within a node, without assigning a wire member in this IR requirement. | spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md:26-30,43-51,92-93; QSpec FR-322:273-277; typed_terms.rs:1042-1048 | wrong-requirement |

## Coverage

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-353-AC-1 | examined | A public external consumer reads admitted `=` and `!=` applications with distinct occurrences and receives the producer-selected operator, original operand identities, source and conversion target types, and exact schedule in source order. Text, enum, quantity and plan cases retain their respective schedule; the enum case has only its compared declaration's member index and the quantity case has the resolved top-level units. The result contains no JSON body or partial decision. |
| FR-353-AC-2 | examined | Admission refuses a decision whose package, application, occurrence, operator or operand identity differs from the complete admitted application, and refuses two decisions for one application occurrence. Each refusal has a named typed code and the first mismatching structural locus; neither decision becomes readable. Display-name or type equality does not establish the join. |
| FR-353-AC-3 | examined | An admitted equality application with no authoritative producer decision returns `UnavailableEqualityDecision`; unknown or foreign node, non-application, ineligible operation and wrong occurrence return their distinct typed outcomes. None returns a fabricated schedule, an empty unit or enum snapshot in place of a required one, or a partial view. |
| FR-353-AC-4 | examined | A finite work limit equal to the charged read succeeds and one less returns typed exhaustion at the first uncharged application, operand or decision entry, with attempted work. Repeated reads and a package clone give equal outcomes and do not reparse package or selected source bytes. |
| FR-353-AC-5 | examined | Code inspection shows the accessor reads an admission-retained decision, performs no `TypeEnvironment::check_equality` or equivalent type-checking pass, exports no QSL checker or evaluator type from the IR model crate, and leaves construction of QSL `CheckedEquality` to QSL's trusted checked-stage boundary. |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 at head `f510c91ad3a6e6b715a16d203347476333ba904a`: FR-353 now requires a producer-owned `CheckedApplicationTermId` containing the owning structural node and a body-relative typed-term path. The reader joins each decision to exactly one retained `CheckedTerm::Application`; occurrence is provenance, not the application selector. AC2 distinguishes two applications with the same operation, operand types and node occurrence, and rejects duplicate, absent and non-application paths. The path belongs to QSpec's producer carrier; FR-353 assigns no new V2 node-body member. Targeted Quire validation is grammar-clean. |
