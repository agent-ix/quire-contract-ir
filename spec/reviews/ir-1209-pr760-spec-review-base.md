---
id: SR-202610310117
title: "spec-review/base review of PR #760 FR-353"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md; IR-1209"
review_set: subset
---

## Summary

Reviewed FR-353 nested-path identity, producer authority, refusal behavior, work charges and acceptance criteria against merged QSpec #558 and current IR FR-038. One high soundness defect makes the positive nested-read criterion impossible. No applicable AssuranceProfile was found under `spec/`.

## Coverage

- `FR-353` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): - An admitted package, the full structural `CheckedNodeId` of a local root   `expression`/`application` node, one occurrence belonging to it, the   QSpec-owned typed `term_path` from that root to the equality application,   and a finite read work limit. The empty path selects the root application. - An equality decision from the same checking stage that produced the package,   carried into package admission under an authoritative producer contract.   The producer decision identifies the package, root application node,   occurrence, typed `term_path`, operator, both original operand positions a
- `FR-353` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): The QSpec-owned `checked_term_data.equality_decisions` carrier SHALL identify each decision by its root `application` `NodeRef`, exact `occurrence`, and typed `term_path`. The root SHALL name one local `expression`/`application` node. The reader SHALL follow each `term_path` step through the admission-retained `CheckedTerm` tree: `argument(index)` selects an application argument, `member(index)` selects an aggregate member, and `binding_value` selects a binding's value. Each selector SHALL match the current term kind and name an existing child; the final term SHALL be the captured equality app
- `FR-353-AC-1` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): A public external consumer reads admitted `=` and `!=` applications by full root application node identity, exact occurrence and typed `term_path` (empty for the root), and receives the producer-selected operator, original operand positions, source and conversion target types, and exact schedule in source order. Two equalities under one `and` root, one under `or`, and aggregate-member and binding-value paths remain distinct; the four TC-1070 Fixture E nested equalities are readable. Text, enum, quantity and plan cases retain their respective schedule; the enum case retains the complete filtere
- `FR-353-AC-2` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): Admission refuses a decision whose package, full root application node, occurrence, typed `term_path`, operator, operand position or source type differs from the complete admitted application, and refuses duplicate (`application`, `occurrence`, `term_path`) keys. An absent or foreign root, non-application root, selector of the wrong term kind, absent indexed child, or path ending at a non-equality term is a named typed refusal at the first mismatching sidecar member; neither decision becomes readable. Distinct paths under one root and occurrence retain distinct decisions even when they share a
- `FR-353-AC-3` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): An admitted equality application with no authoritative producer decision returns `UnavailableEqualityDecision`; unknown or foreign root, non-application form, invalid or absent path, non-equality final term, ineligible operation and wrong occurrence return their distinct typed outcomes. None returns a fabricated schedule, an empty unit or enum snapshot in place of a required one, or a partial view.
- `FR-353-AC-4` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): A finite work limit equal to the charged read succeeds and one less returns typed exhaustion at the first uncharged application, path segment, operand or decision entry, with attempted work. Repeated reads and a package clone give equal outcomes and do not reparse package or selected source bytes.
- `FR-353-AC-5` (context_only, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): Code inspection shows the accessor reads an admission-retained decision, performs no `TypeEnvironment::check_equality` or equivalent type-checking pass, exports no QSL checker or evaluator type from the IR model crate, and leaves construction of QSL `CheckedEquality` to QSL's trusted checked-stage boundary.
- `FR-353` (examined, `spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md`): QSpec [STD-613](https://linear.app/agent-ix/issue/STD-613) and [STD-617](https://linear.app/agent-ix/issue/STD-617) supply the FR-322 `checked_term_data.equality_decisions` wire, typed nested-path schema and vectors for the authoritative, bounded carrier. Current IR admission does not yet retain the producer's schedule, conversion choices, complete resolved unit-table closure or filtered enum member index. This FR allocates no further V2 wire member and grants no authority to reconstruct a decision from type or catalog data. QSL [QSL-1390](https://linear.app/agent-ix/issue/QSL-1390) owns the e
- `FR-038` (context_only, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): An application of any operator class other than `case` nested inside a node   body or inside another term (an application's `arguments`, an `aggregate`'s   members, a `binding` value) refuses `malformed_wire` at the nested   application.
- `FR-322` (context_only, `spec/objects/interfaces/FR-322-checked-package-artifact.md`): Every composite subterm is its own node, reached by `reference`: an application never stands inside another application's arguments, an aggregate's members or a binding's value.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-353-AC-1 requires readable nested equality applications, but the merged QSpec V2 body grammar/schema and current IR FR-038 reject such nested terms before an admitted read. | spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md:121 |

## Evidence

FR-353-AC-1 requires two equalities under one `and` root, one under `or`, and aggregate/binding paths to be readable. Merged QSpec FR-322 Body grammar still requires every composite subterm to be a separate node reached by reference; its schema MemberTerm does not admit application, and IR FR-038 explicitly refuses nested applications as malformed_wire. The new QSpec `term_path` sidecar selector cannot make those bodies admissible. The positive cases cannot pass until the authoritative body grammar/schema and IR admission rule are extended, or the AC is scoped to valid top-level terms.

QSpec #558 merged at `6ce98f66`: FR-322 Body grammar says applications never stand inside application arguments, aggregate members or binding values. The schema `MemberTerm` accepts only leaf, group or binding terms. IR FR-038 lines 632-641 refuses nested non-`case` applications during admission.

## Verdict

FAIL: FND-001 blocks the nested positive acceptance cases.
