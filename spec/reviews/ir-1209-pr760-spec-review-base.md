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

QSpec #558 merged: FR-322 Body grammar says applications never stand inside application arguments, aggregate members or binding values. The schema `MemberTerm` accepts only leaf, group or binding terms. IR FR-038 lines 632-641 refuses nested non-`case` applications during admission.

## Verdict

FAIL: FND-001 blocks the nested positive acceptance cases.

## Author Disposition

FND-001 addressed in PR #760 after QSpec STD-619 merged in QSpec #560.
FR-322-AC-288/289/292 now require each equality record to name its own
equality application `NodeRef` and occurrence with an
empty `term_path`; an `and`, `or`, aggregate or binding parent references that
child node. FR-353 now joins directly to the child application, refuses a
non-empty path and retains TC-1070 Fixture E rows 8, 12, 15 and 17 as four
distinct child application rows. The earlier evidence above records the
contract state at review time. FR-038's existing nested-application refusal
remains in force. Independent reviewer verification of this disposition is
pending.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | FR-353 adopts QSpec's empty-path ACs while QSpec's normative equality-carrier prose still permits nested path traversal, leaving the producer/read contract contradictory. | spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md:131 |

Merged QSpec #560 (e119548e) changed FR-322-AC-288/289/292 to require each equality row to name its own application and carry an empty `term_path`, matching the V2 body grammar and revised FR-353. FR-322's `checked_term_data` prose at lines 1025-1040 was not changed: it still requires traversal through typed selectors to a nested application and describes two nested equalities below one root as separate rows. The schema still admits non-empty `term_path` values. These normative descriptions cannot both define the same producer/reader join. The QSpec owner must reconcile the prose with the empty-path ACs before the downstream contract can rely on one authoritative interpretation.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Revised FR-353-AC-1 reads each equality by its own application node and occurrence, with empty `term_path`; parent forms reference the child node. This satisfies the V2 body grammar and FR-038 admission rule. |
| FND-002 | still-open | QSpec #560's normative checked-term-data prose still permits nested path traversal while its revised AC-288/289/292 require empty paths. Upstream repair is pending. |
