---
id: SR-5690
title: 'IR-484 PR #351 Code and Rust review'
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/operations.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, tests/it/checked_package_v2_canonical_encoding.rs, tests/it/checked_package_v2_flat_wire.rs, tests/it/checked_package_v2_frame_entries.rs, tests/it/checked_package_v2_operand_families.rs, tests/it/checked_package_v2_reader.rs, tests/it/checked_package_v2_require_bounds.rs, tests/it/main.rs
review_set: subset
---

## Summary

Ticket: IR-484. The reader now resolves declared units as quantities and aggregate terms as aggregates, validates literal kind against type, and applies result and operand mode pins. Public-reader controls and fixture repairs exercise the changed behavior. The Rust checklist found no defect in the PR diff.

## Verdict

**PASS** — No finding in the changed scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The repo conventions, panic and unsafe surface, typed error causes, source/test alignment, identity refresh, CI workflow diff, and bounded term traversal were checked. The pre-PR same-head run reported fmt, lint, 466 integration and 204 model/corpus tests passing, with only the unchanged strict coverage backlog; deny and unsafe audit also passed.

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-210 | examined | Implemented (IR-484). For a valid self-typed declared `unit` node, references to distinct values typed by that node resolve to `quantity`: all twelve catalogued `quire.op.quantity.*` entries resolve each required quantity position and reject a Boolean at the first one, independent of their later result, member and dimension obligations; `quantity.add` and `quantity.eq` with valid same-unit operands also admit end to end. Replacing a required quantity argument by Boolean refuses `ill_typed`/`operator-ineligible` at that argument; placing a declared-unit quantity in the first `integer.add` posit |
| FR-038-AC-211 | examined | Implemented (IR-484). `quire.op.control.if` over a Boolean condition and two values of the same declared-unit quantity type admits, while replacing the second branch with an Integer value refuses `ill_typed`/`operator-ineligible` at `arguments/2` under its `same_family` constraint. A valid `aggregate` in each of the three aggregate positions of `quire.op.temporal.clause` and in the first position of `quire.op.state.clause` fits `aggregate` at the operation step after earlier shape checks. Placing an aggregate in the first `boolean.and` argument refuses `ill_typed`/`operator-ineligible` at `arg |
| FR-038-AC-212 | examined | Implemented (IR-484). A body-root or argument literal whose `value_kind` is `integer` and whose `type` names an Integer node admits when the enclosing operation is otherwise valid; retaining that value kind while changing only its declared type to a Boolean node and re-deriving identities refuses `ill_typed`/`operator-ineligible` at the literal's `type`, even if the enclosing operand position accepts either family. An enum literal accepts either `enum` or `ordered_enum` type according to its nominal declaration, and a `none` literal accepts an option type; each mismatched kind/type pair, inclu |
| FR-038-AC-213 | examined | Implemented (IR-484). `quire.op.quantity.convert` over a declared-unit quantity and a selected quantity `type_argument` admits mode `{kind: rounding, value: exact}`; `toward-zero` on otherwise identical operands and result refuses `invalid_package`/`operation-mode-type-mismatch` at `operation.mode/value`, even though the unit type has no explicit rounding binding. The result type alone pins this conversion to the declared-unit default `exact`. The same mode check applies to `quire.op.collection.sum.quantity` when its quantity binder and member are valid. |
| TC-048 | examined | FR-038-AC-210 through FR-038-AC-213 have tagged public-reader admission and adverse controls for the published operation catalog's declared-unit quantity, aggregate, same-family, literal-kind and rounding-mode rules. |
