---
id: SR-8690
title: "QSL-737 quire-contract-ir PR 376 code review: conforming records at operand positions"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/operations.rs; crates/quire-contract-model/src/checked_package/v2/model_members.rs; crates/quire-contract-model/src/checked_package/v2/structural.rs; tests/it/checked_package_v2_operation_arguments.rs"
review_set: subset
---

## Summary

Ticket: QSL-737. Code review with the rust-review lane folded in, over the
diff of quire-contract-ir PR 376 (one commit based on main).
Reviewer-only: no build or test run. The coder's gate log at the PR head
(`make ci`: fmt, both clippy lanes, 528 + 211 + 21 + 8 tests green, the 14
`tc_448` tests among them; stop at `make spec` strict coverage) was read, not
re-run. The coder worktree sits with a clean tree.

Checked:

- `conforms_to_type` (operations.rs): an exact digest match returns true
  with no walk and no charge. Otherwise the expected key must name a selected
  `record_value_type` (`ModelOwners::record_by_key`), the actual type node
  must be a `model`/`record_value_type` node that `ModelOwners::recover`
  recovers, and both owners must share one `DomainModel` (`ptr::eq`). The
  walk is `DomainModel::conforms_to`, i.e. the existing `ancestors` walk
  (the FR-038-AC-29 walk), charged through a `Budget` at the expected
  record's selection row. Every non-record expected type falls back to
  exact comparison. The comparison is digest-only, but `visit_reference`
  already refuses any node id outside `quire.checked-semantic-node/v1` at
  intake, so this matches the old full `CheckedNodeId` comparison.
- Work accounting: `ancestors` charges one unit per popped supertype edge and
  expands an ancestor only on first insert. GiftTaxedMoney gives 2 at Money
  and at TaxedMoney, because the walk does not stop at the target, and the
  diamond D over B and C over A gives 4 (C, A, B, A). These match FR-347-AC-8.
  The row: a conforming argument shares the parameter's document, so the
  expected record's row is also the argument's and the declaring owner's
  row. A cross-document argument returns before the `Budget` is built and
  charges nothing.
- `record_by_key`: `by_key` holds one entry per selected object, record and
  relationship declaration key. The filter on `RecordValueType` is correct,
  and declaration keys of different forms cannot collide because their
  preimages differ.
- `check_model_member`: the field operand check now goes through
  `conforms_to_type` against the declaring node's key, so it stays exact for an
  object-type declaring node. The owner lookup moved from `object_type()`
  to `field_owner()`, and a record owner refuses an `operation` member at
  `name`. `object` is read only in the operation branch (`interface`). The
  order of refusals is unchanged: `declaration`, then `arguments/0`, then
  `name`.
- `union_arms_hold` was rewritten from `all()` to a loop so that it can
  propagate the meter error. Its empty-arms and missing-`result_type`
  outcomes match the old expression, and binders stay exact.
- `union_defect` now returns `Result<Option<_>>`. The refusal order
  (not-a-union, member lookup, payload count, payload type) is unchanged.
- `body_root_defect` is new and runs on every `function` and `expression`
  node, so it is a new exact check even for non-record types, as FR-347
  "Result positions" requires. Before relying on that, every
  function/expression root in the nine QSpec positive fixtures
  (`proposals/checked-package-v2/fixtures/positive-*.json` at
  quire-specification main ) was read statically: 26 roots checked,
  0 mismatches. Their only `function.call` callee has no `parameters`
  binding, so `check_function_call` does not apply to it.
  `make conformance-qspec` is outside `make ci` and was not run by the coder.
- `check_function_call`: the callee must be a `reference` to a `function`
  node, so a `dependency_reference` callee stays unchecked. The malformed
  binding refuses at `arguments/0` in five shapes: not first, not an
  aggregate, a non-reference entry, a dangling entry, and a
  non-`value`/`parameter` entry. Arity refuses at `arguments` before any
  type check. `arguments[position]` is in bounds because the arity check
  runs first.
- The coder widened `parameter_defect` (structural.rs) and `is_type_node`
  (operations.rs). Each change adds exactly one kind,
  `Model(RecordValueType)`, and each function has one call site. Merged QSpec
  FR-322 "Record conformance" ("Calls": parameters typed at a record type
  node; "Slots": a union member payload position) needs both. No existing
  FR-038 criterion or test refuses a record-typed binder or payload, and no
  existing test expectation changed: apart from the new file and its `mod`
  line, the only test edits add a `ModelOwners::default()` argument to two
  unit-test calls.
- Rust: no `unsafe`, `unwrap`, `expect` or panic in the production diff.
  Indices come from `index` positions or are bounded by the arity check, and
  `saturating_add` is used throughout. Production tags use `Implements:
  FR-347` (the CR-061 non-binding-tag warnings in the coder's earlier coverage
  log are gone in the final CI log). Tests carry `/// Trace:` plus
  `#[trace("TC-448", ...)]`.
- Public-repo rules: SPDX header, fixture identities `acme/ledger` and
  `acme/vendor`, no private text, no local paths, no artifacts.

## Verdict

PASS WITH FINDINGS (one low). The behaviour matches FR-347 at every position.
Non-record operands keep exact comparison. The new body-root check is exact
for non-record types and does not refuse any QSpec positive fixture. The
widening of the two predicates is right and minimal, and it belongs in this
PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Three rustdoc blocks still describe exact comparison where the code now accepts a conforming record. `check_model_member` says a field is read "over an operand of the declaring node's type" and each argument is "of its parameter's type node". `union_arms_hold` says "every arm body has the application's `result_type`". `union_defect` says payload terms "are that member's payload in count and in type". Only the inline `Implements: FR-347` comments state the new rule. | crates/quire-contract-model/src/checked_package/v2/operations.rs:2288-2294, 1399-1405, 1541-1547 |
