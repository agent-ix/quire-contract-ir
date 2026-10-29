---
id: SR-8683
title: QSL-737 spec-review-base of operation-call argument record conformance
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md,
  spec/checked_package/matrix/TC-448-checked-package-v2-operation-argument-conformance.md,
  spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md,
  spec/spec.md, spec/tests.md
review_set: subset
---

## Summary

Ticket: QSL-737 (QSL FR-359 cross-repo requirement 4). PR quire-contract-ir#375 adds FR-347 (eight ACs), TC-448, a pointer in FR-038 "Model-owned members" step 4, and index rows. The base checklist was run over the new and edited artifacts and checked against the reader code on origin/main (`check_model_member`, `DomainModel::ancestors`/`conforms_to`, `ModelOwners::recover`, `validate_owner_joins`).

## Verdict

**CONDITIONAL**: one high and one medium AC defect, and one low index gap.

What holds. The rule matches QSL FR-359 item 6, AC-8 and cross-repo requirement 4, and the pending QSpec FR-322 rule (FR-322-AC-31 as amended, FR-322-AC-69): a record-typed parameter admits a conforming record, the count is checked before the types, and everything else stays exact. The ancestry source is sound and agrees with FR-359 item 3. The checked package carries no `supertypes`, and the relation is reached through the argument node's `ModelOwner` into the digest-pinned selected document. The reader already does this for receiver and reference conformance (`conforms_to` at operations.rs:1835/1880; `ancestors` at model_members.rs:826, whose walk covers record types through `field_owner`). The same-document test matches the `std::ptr::eq(package)` precedent. The refusal cause and locus match today's exact check: `ineligible(argument_at(position))` gives `/semantic_graph/nodes/{n}/body/arguments/{position}`, and arity is checked first at `arguments` (operations.rs:2127-2144). Owner recovery cannot fail for an argument that is a `model`/`record_value_type` node, because `validate_owner_joins` and derived-key checks admit every such node first. So the "exact otherwise" fallback does not conflict with the QSpec statement that a failed lookup carries step 2's refusal. The scope rule for event types matches FR-359 cross-repo requirement 7.

Leaving the `Reference<P>` open question to FR-322 is the right call: FR-322 owns reference conformance, and the pending QSpec call rule keeps references exact. The coverage claim was measured, not inferred. `quire coverage --strict` reports 42 unbacked rows on main and 52 at the PR head. The 10 new rows are FR-347, TC-448 and FR-347-AC-1..8, all PLANNED. `quire validate` passes at the head: 749/749 documents are grammar-clean. FR-347 and TC-448 are unique across every origin branch and both open PRs (#368, #375). The PR adds no hash, digest, pin or tracking record. The public-repo leak check is clean: no private QSpec text appears in the diff, the commit message or the PR body.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-347-AC-4 (and TC-448's procedure) needs an operation parameter typed `Option<Money>`, and a domain package cannot declare one. An operation parameter is a `TypedSlot` with no presence. `slot_type` yields the element type or a `Collection`, and only `field_type` wraps `Option` (model_members.rs:1023-1051). A `[0, 1]` parameter becomes `Set<Money>[0, 1]` (FR-322-AC-31). So the two `Option<Money>` clauses of AC-4 cannot be set up and no test can run them. Replace them with a `[0, 1]` (`Set<Money>[0, 1]`) parameter, or drop them, and remove `Option<X>` from the parameter list under "Exact kinds stay exact". | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:127; spec/checked_package/matrix/TC-448-checked-package-v2-operation-argument-conformance.md:28,39 |
| FND-002 | medium | FR-347-AC-7 says a receiver typed at a `Reference` of a subtype "is refused or admitted exactly as before". That gives no outcome a test can assert, so two implementers would write different tests. Today the receiver must match `Reference<declaring>` exactly, and a subtype reference refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{n}/body/arguments/0` (operations.rs:2127-2129). State that outcome. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:130 |
| FND-003 | low | spec/tests.md adds FR-347 to the Checked package row's requirement list, but the row's status text does not mention it. Every other planned item in that cell gets a status clause. Add "FR-347 (TC-448) planned/unrun, QSL-737". | spec/tests.md:15 |

## Coverage

Base checklist: ID formats, duplicates (checked across all origin refs and open PRs), link targets (FR-038, StR-001 and the QSpec/QSL FR references resolve by id), US/FR/TC quality, AC-to-TC coverage (TC-448 verifies all eight ACs, and the matrix row and TC row list them), and the hash/pin check. EARS: the FR-347 statement is a well-formed event-driven requirement ("When the reader checks..., the reader SHALL admit... and SHALL keep..."). No EARS finding, so no separate ears-conformance document. Separate lenses were run for failure-domain (SR-8684) and scope-boundary (SR-8685).

Review-only: no edits, no cargo builds or tests. `quire validate` and `quire coverage --strict` were run read-only in detached worktrees at origin/main and PR (quire 0.36.2, engine 0.50.2). The PR is based on current main (merge-).

## Examined units

```yaml
scope:
- id: FR-347
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: When the quire.checked-package/v2 reader (FR-038) checks a model-owned operation member call, the reader SHALL admit an argument whose type is a record value type that conforms to the record value type its parameter declares, and SHALL keep every other argument exact.
- id: FR-347-AC-1
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: Over a selected document declaring Money, TaxedMoney with supertype Money, GiftTaxedMoney with supertype TaxedMoney, an unrelated record Voucher, and an object type with the operation charge(m Money), a call whose argument is typed at Money admits (control, exact), one typed at TaxedMoney admits, and one typed at GiftTaxedMoney (grandchild) admits.
- id: FR-347-AC-2
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: In the same document a call to charge(m Money) whose argument is typed at Voucher (unrelated) refuses ill_typed/operator-ineligible at /semantic_graph/nodes/{n}/body/arguments/1.
- id: FR-347-AC-3
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A call to settle(t TaxedMoney) with an argument typed at Money (the ancestor) refuses ill_typed/operator-ineligible at that argument, and so does a call to grandsettle(g GiftTaxedMoney) with an argument typed at TaxedMoney (an intermediate ancestor) or at Money. settle with a GiftTaxedMoney argument admits.
- id: FR-347-AC-4
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A parameter typed Integer, Int[0, 9], Boolean, Reference<O>, Option<Money> or Sequence<Money> is compared by node key exactly; an argument typed TaxedMoney at the Option<Money> or Sequence<Money> parameter, an argument typed Option<TaxedMoney> at the Option<Money> parameter, an Int[0, 5] argument at an Int[0, 9] parameter and a Reference of a subtype at a Reference<O> parameter each refuse ill_typed/operator-ineligible at the argument; the same argument at its exact type admits.
- id: FR-347-AC-5
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: An argument typed at a composite_type/record node with a source owner, with the same name and fields as TaxedMoney, refuses ill_typed/operator-ineligible at charge(m Money); ancestry is read from the selected document only. A TaxedMoney record of a second selected document does not conform to the first document's Money and refuses the same way.
- id: FR-347-AC-6
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A call with two record-typed parameters, a conforming first and a non-conforming second argument, refuses at argument 2 (the second parameter); with both non-conforming it refuses at argument 1. A call with the wrong argument count refuses at arguments before any argument is judged, as before.
- id: FR-347-AC-7
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: The receiver of a call to charge typed at a Reference of a subtype of the declaring object is refused or admitted exactly as before this requirement (FR-347 changes no receiver check), and an admitted call's result_type that differs from the operation's declared result type by key still refuses ill_typed/operator-ineligible at result_type.
- id: FR-347-AC-8
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: Resolving the conformance of a GiftTaxedMoney argument at a Money parameter charges one work unit per supertypes edge followed (two), an exact Money argument charges none, and a work limit one below the measured total returns incomplete for work; the exact total admits.
- id: TC-448
  path: spec/checked_package/matrix/TC-448-checked-package-v2-operation-argument-conformance.md
  role: examined
  excerpt: Verify FR-347-AC-1 through FR-347-AC-8 - the V2 reader admits an operation-call argument whose record type conforms to the parameter's declared record type, refuses an unrelated, ancestor or foreign-document record with ill_typed/operator-ineligible at the argument, and keeps every non-record parameter exact.
- id: FR-038-model-owned-members-step-4
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: An operation call's argument types are compared with its parameters' by node key, except that an argument whose record conforms to a record-typed parameter's record is admitted (FR-347).
- id: tests-md-checked-package-row
  path: spec/tests.md
  role: examined
  excerpt: Checked package | FR-035, FR-038, FR-040, FR-344, FR-346, FR-347 | checked_package
- id: FR-038-AC-29
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: an inherited field resolves on a subtype and a subtype conforms to its supertype in either operand order
- id: FR-038-AC-30
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: Reading a selected domain package and resolving a model-owned member are charged to the work limit; a limit one below the work a read used returns incomplete for work with the pointer /lock/model_selections/<i> of the row, and the exact work admits.
bindings: []
```

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The new "Field projection over a conforming operand" section says the field "is resolved by name among `P`'s effective members as before", and FR-347-AC-9 expects an exact `OrderPlaced` operand to be admitted. Today every model-owned record field read refuses at the member's `name` before resolution runs: `check_model_member` requires `owner.object_type`, which reads only `object_types`, so a `record_value_type` declaring node returns `ineligible(member_at("name"))` (operations.rs:2108-2110, model_members.rs:1226-1228). No test on main admits a known field read over a model record; TC-056 tests only the unknown-field refusal. So "as before" is false, and the requirement does not state the second change it needs: for a `field` member, the object-type gate is lifted for a record declaring node. State that change in the section and in "It changes nothing else". QSL FR-359-AC-3's read-back depends on it. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:118-129,174 |
| FND-005 | low | Public-repo leak check: the "Function-call arguments" section says "QSpec FR-322 is adding that parameter list". That describes content of an unmerged private QSpec change (a `function` node parameter list) that no public source states. Cite the dependency by FR/AC id or ticket only, with no design description. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:143-153 |

## Dispositions

Round 1; reviewed (fix commit on top of, merge-base origin/main ).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
| FND-002 | fixed | Fixed in the reviewed PR. |
| FND-003 | fixed | Fixed in the reviewed PR. |

## Disposition round 1 verdict

**CONDITIONAL**: FND-001..003 are fixed. FND-001: AC-4 now uses `Set<Money>[0, 1]`, which `slot_type` yields for a `[0, 1]` unique, unordered parameter (`CollectionKind::of(false, true)` = `Set`, bounds `(0, 1)`), and the hand-built `Option<Money>` argument at a `Money` parameter falls to the exact comparison and refuses. FND-002: AC-7 states the `arguments/0` refusal that operations.rs:2127-2129 gives. FND-003: the spec/tests.md row now carries the FR-347 status clause. New FND-004 (medium) and FND-005 (low) are open. The title change, the TC-448 and matrix updates, and the IDs (unique across origin refs and open PRs #368 and #375) are fine. `quire validate` passes 752/752 at the head. Strict coverage gives 53 unbacked rows: main's 42 plus 11 planned (FR-347, TC-448, AC-1..9).

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | high | "Result positions" and FR-347-AC-12 make the reader admit a function whose body root is not conforming: a `Voucher` body under declared `Money` is admitted "as before", and the text says this is "stated so that it stays true". QSpec FR-322-AC-69, which this requirement says it follows, refuses a non-conforming body root. So AC-12 pins a known divergence from the producer rule as tested behavior, and a reader fixed to conform would fail the test. The reader can now reach the function body's bindings, since the function-call rule reads the `parameters` binding, so the `body` binding is just as reachable. Either add the body-root check (exact, or conforming for a record result type) or drop AC-12's `Voucher` clause and record the gap as a tracked limit. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:154-160,226 |
| FND-007 | medium | The case-arm refusal locus is wrong. The text says a failing arm refuses "at the `case` application's arguments", but `union_arms` refuses at the `case` node itself, `/semantic_graph/nodes/{n}` (operations.rs:1341-1345). Fix the text, and pin the locus in FR-347-AC-10. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:141-152,224 |
| FND-008 | low | FR-347 says the callee's body `parameters` binding comes from QSpec FR-322-AC-70. That criterion's current text in the open QSpec change does not describe a body binding. The binding shape the reader reads is QSL FR-092's "Function nodes" body (`parameters` = `aggregate{[reference(p0)...]}`). Cite QSL FR-092 as the shape's source and keep FR-322-AC-70 as a pending dependency. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:174-201 |
| FND-009 | low | The one-line summaries still describe only the operation-call position: spec/spec.md:93-94 ("admits a conforming record as an operation-call argument"), the FR-347 status in spec/checked_package/matrix/tests.md:15, and the TC-448 title. Update them to the five positions. | spec/spec.md:93-94 |

## Dispositions round 2

Round 2; reviewed (commits and on top of ).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed | Fixed in the reviewed PR. |
| FND-005 | fixed | Fixed in the reviewed PR. |

## Disposition round 2 verdict

**CONDITIONAL**: FND-004 and FND-005 are fixed. FND-004: "Field projection over a record operand (two changes)" now states the record-owner change, which lifts the `owner.object_type` check at operations.rs:2108-2110 for a `field` member, keeps it for `operation`, and resolves among `P`'s effective members as FR-038 step 3 does. AC-9 adds the exact control, an inherited read and the operation-on-record refusal. FND-005: the private design description is gone, and QSpec is cited by FR-322-AC-69/70 ids only. New: FND-006 (high), FND-007 (medium), FND-008 and FND-009 (low). `quire validate` passes 752/752. Strict coverage gives 57 unbacked rows: main's 42 plus 15 planned (FR-347, TC-448, AC-1..13). The matrix rows and TC-448 list AC-1..13 consistently. IDs are unique across origin refs and open PRs #368 and #375. The commit messages are clean.

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-010 | medium | The new body-root check refuses with `ill_typed`/`type-mismatch`, chosen to match the neighbouring `union_value` payload comparison. The open QSpec rule this requirement follows (FR-322-AC-69 and its body-root vectors) refuses a non-conforming function or expression body root with `ill_typed`/`operator-ineligible` at the node. The reader would report a different cause from the producer specification for the same package. Use `operator-ineligible`, or record the divergence and why. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:155-165,244 |
| FND-011 | low | FR-038's `structural.eq` operand-type table (FR-038:3768) says an expression node's `semantic_type` "must equal the body's `result_type`". FR-347 now admits a conforming record there. Add a pointer from that row to FR-347, so FR-038 does not state an exact rule FR-347 relaxes. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3768 |
| FND-012 | low | The body-root rule defines the root of a `function` node only through its `body` binding. FR-038 also admits a `function`-tagged node whose body root is an application directly (FR-038-AC-88). For that shape, "a node whose body names no root takes no check" does not say whether the root application's `result_type` is compared. State which. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:155-165 |

## Dispositions round 3

Round 3; reviewed (one commit on top of ).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-006 | fixed | Fixed in the reviewed PR. |
| FND-007 | fixed | Fixed in the reviewed PR. |
| FND-008 | fixed | Fixed in the reviewed PR. |
| FND-009 | fixed | Fixed in the reviewed PR. |

## Disposition round 3 verdict

**CONDITIONAL**: FND-006..009 are fixed.

- FND-006: AC-12 now requires the body-root check. The function root is the node the `body` binding references. The expression root is the root application's `result_type`, which the reader can reach at the node body, since an expression node's body root is its application. The definitions match the open QSpec Results rule. The reader makes no admission-time body-root comparison today (the only `semantic_type` against `result_type` comparison is in the comparison-operand accessor, comparison_operands.rs:546-551).
- FND-007: AC-10 pins the `case` node locus, matching operations.rs:1341-1345.
- FND-008: QSL FR-092 is cited as the source of the binding shape.
- FND-009: spec.md, the matrix status and the TC-448 title name all positions.

New: FND-010 (medium), FND-011 and FND-012 (low). `quire validate` passes 752/752. Strict coverage gives 58 unbacked rows: main's 42 plus 16 planned (FR-347, TC-448, AC-1..14). The matrix and TC-448 list AC-1..14. IDs are unique across origin refs and open PRs #368 and #375. The leak check of the new text and commit is clean.
