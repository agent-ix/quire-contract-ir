---
id: SR-627
title: "PR #231 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@a567ba99e8601849c77984f19c723a000e664fca; crates/quire-contract-model/src/checked_package/v2/lower.rs, tests/it/checked_package_v2_require_bounds.rs, tests/it/complete_v1_contract_package.rs (git diff origin/main...HEAD)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---
# SR-627: PR #231 code and Rust review

## Summary

Ticket: IR-283 (and IR-284). PR agent-ix/quire-contract-ir#231. Code review
with the rust-review lane folded in.

Examined units:

- `lower.rs` `lower_one`: the new `named_by_composite` set, its insertion in
  the successor loop, and the rewritten `require_bounds` predicate.
- `lower.rs` `is_recursive_type`.
- `lower.rs` `requires_bound` (unchanged; quantity/unit still `false`).
- `mod.rs` `validate_recursion` (context: what a `recursion_group` label
  guarantees).
- Four new TC-050 tests and the rewritten
  `tc_047_a_refused_request_is_represented_only_as_reached_meaning`.

Measured by the reviewer:

| Check | Result |
| --- | --- |
| `cargo test --test it` (require_bounds + complete_v1 filters) at a567ba9 | 16 passed |
| QSL `tc_440*` (quire-spec-language origin/main a2837b6f, `-- --include-ignored`) with IR patched to this PR | Flags, Mixed, RangedTree now agree; `tc_440_an_unbounded_application_record_requires_a_bound_in_ir_pending_ir_283` still FAILS (outer `+` of `(x + 1) + n` lowers); Measure not compared (emitter omits it, pre-existing) |
| Mutation M1: position rule off | red: `tc_050_a_bound_over_the_shared_integer_does_not_cover_an_unbounded_field`, `tc_050_a_collection_of_unranged_integers_requires_a_bound` |
| Mutation M2: recursion rule off | red: `tc_050_a_recursive_record_requires_a_bound` only |
| Mutation M3a: closure-wide domain exemption off | red: 11 tests incl. `tc_050_x_plus_one_over_a_bounded_parameter_lowers_under_require_bounds`, `tc_050_bounded_collections_and_ranged_fields_lower` |
| Mutation M3b: typed-at rule off | red: 5 tests incl. `tc_050_a_value_typed_at_an_unbounded_type_still_requires_a_bound`, `tc_052_unbounded_forms_are_exactly_the_eight_declared_forms` |

## Verdict

**CHANGES REQUESTED** (one high finding).

These checks passed:

- The composite-position rule is correct for the record fixtures: QSL's own
  emitter output for `Flags {xs: Sequence<Boolean>[0,3]}` now lowers and
  `Mixed {n: Integer, k: Int[0,9]}` now refuses naming `integer`.
- The recursion rule makes QSL's `RangedTree` refuse, as ADR-014 §4 / QSpec
  FR-143 require (recursive depth is unbounded; no `bounded_domain` form
  bounds depth).
- A `bounded_domain`'s base type and its bound literals are not positions:
  domains are not `composite_type`, and literal `type` is an annotation edge.
- Reporting stays deterministic: `find` runs over the key-ordered closure, so
  the least key satisfying either rule is named.
- Each of the three branches has its own oracle: each mutation turns a
  distinct test red.
- `tc_047_a_refused_request_is_represented_only_as_reached_meaning` keeps its
  intent. Its assertions are unchanged. The old fixture (an `option` over
  `integer` reached beside an `integer_range`) became a real refusal under
  the position rule, so the fixture had to change.
- No new `unwrap`/`expect`, no `unsafe`, no integer casts.

The high finding: values and expressions keep the closure-wide rule, so the
masking IR-283 names still happens for a value position. QSL's TC-440
application fixture shows it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Values and expressions keep the closure-wide rule, so a `bounded_domain` over the shared `integer` node still masks an unbounded value position. In `function f using v(x: Int[0,9], n: Integer): Integer pure { (x + 1) + n }`, the outer `+` is typed at `integer` and reaches parameter `n`, which is also typed at `integer`. `x`'s `integer_range` covers `integer`, so IR returns `Lowered`. QSL classifies that record `Unbounded` at `n`. Measured: QSL's `tc_440_an_unbounded_application_record_requires_a_bound_in_ir_pending_ir_283` fails with this PR patched in. This is the IR-283 defect on the value path, and a Kani request over an infinite `n` would read as supported. For `x + 1` to still lower, a `value`/`parameter` node's own `semantic_type` has to count as a position, while an application `result_type` and a literal annotation stay non-positions. | crates/quire-contract-model/src/checked_package/v2/lower.rs:464-468 |
| FND-002 | low | `is_recursive_type` checks only `recursion_group.is_some()`. `validate_recursion` checks labels only on cyclic components, so the reader admits a label on an acyclic scalar or composite, and the lowerer then refuses it as recursive. The rule also fires for a labelled type that is reached only through a `literal.type` annotation, although FR-038 says `requires_bound` tests only a type some reachable node is typed at. QSL labels only real SCCs, so no current producer hits this. The assumption that a label means recursion is still undocumented and untested. | crates/quire-contract-model/src/checked_package/v2/lower.rs:461; crates/quire-contract-model/src/checked_package/v2/lower.rs:527-535; crates/quire-contract-model/src/checked_package/v2/mod.rs:1864-1869 |
| FND-003 | low | The `require_bounds` predicate is now a four-level nested boolean inside a `find` closure, with the O(n) `any` domain scan inlined. Two named helpers (`uncovered_position`, `uncovered_value_type`) or a precomputed `BTreeSet` of domain `semantic_type`s would make each rule readable, and each could be mutated separately. | crates/quire-contract-model/src/checked_package/v2/lower.rs:460-469 |
| FND-004 | low | The rewritten tc_047 fixture turns `bbbb` into a self-typed `sequence` with no element type and no dependencies. No producer emits that shape. The test's intent holds, but a realistic element type (for example a `boolean` scalar) would keep the fixture on a shape the reader and QSL both produce. | tests/it/complete_v1_contract_package.rs:358-364 |

## Dispositions

Round 1, reviewed at edb862819349fae5ba5bf2a19ea565ab1a3f96dd (c58e154 is
a567ba9 rebased onto origin/main 7c70041; edb8628 is the fix round).

What the reviewer measured at edb8628, on a clean tree:

- **QSL TC-440** (quire-spec-language origin/main e4ac2e8d, IR patched to
  edb8628, `--include-ignored`). These pass, including the ignored ones:
  - `tc_440_an_unbounded_application_record_requires_a_bound_in_ir_pending_ir_283`,
    so `(x + 1) + n` now refuses;
  - `tc_440_operation_application_records_agree_with_ir_per_node`, so the
    inner `x + 1` still lowers;
  - `tc_440_qsl_extent_agrees_with_ir_requires_bound`.

  `…_pending_ir_283_284` reports only "Measure: not compared: the emitter
  omits it". That is pre-existing, and QSL-side.
- **Mutations**, one per branch. Each turns a distinct test red:
  - composite position off: two TC-050 position tests;
  - parameter position off: only
    `tc_050_a_bound_over_one_parameter_does_not_cover_another_of_the_same_type`;
  - recursion off: only `tc_050_a_recursive_record_requires_a_bound`;
  - domain exemption off: 11 tests;
  - typed-at rule off: 3 tests.
- **Gates:** `make fmt-check lint test corpus` at edb8628 exits 0.
- **Let-bound parameter probe (informational).** For `let t = x + 1 in t * 2`,
  IR returns `RequiresBound` naming `integer`, while QSL says `Bounded`. IR
  origin/main returns the same, so this is not a regression from this PR, and
  QSL scopes let-rooted records out of the agreement.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | edb8628 |
| FND-002 | fixed | edb8628 |
| FND-003 | fixed | edb8628 |
| FND-004 | fixed | edb8628 |
