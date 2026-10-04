---
id: SR-1237
title: "gap analysis of PR 285 (IR-495 FR-038-AC-114 through AC-118 to tests)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@f3d41bca80fe145bdabd345e1d13749f6be1fcaf; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/functional/FR-346-admit-abstraction-relation-body.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/assurance/AD-004-checked-package-seam.md, crates/quire-contract-model/src/checked_package/**, tests/it/checked_package_v2_flat_wire.rs, tests/conformance_qspec/main.rs"
review_set: subset
---
# SR-1237: gap analysis of PR 285

## Summary

Ticket: IR-495. Plan completion: not assessed. This review checks FR-038-AC-114 through
FR-038-AC-118, FR-038-AC-9, FR-040-AC-10, FR-038 "Reading", "The flat wire" and the order
of checks, and TC-048 ("Flat wire and no depth limit") against the code and tests at
f3d41bca80fe145bdabd345e1d13749f6be1fcaf. It reads them against merged QSpec FR-322 (Body
grammar, Identity and validation, AC-39 to AC-41) at quire-specification 2f846f8.

Bindings checked:

- AC-114: `tc_048_an_application_nested_in_any_position_refuses_malformed_wire_at_it`
  covers all five classes (call, state.clause, temporal, temporal_formula,
  temporal_fairness) in the three positions plus a body-aggregate member.
  `tc_048_a_body_outside_the_strata_refuses_malformed_wire_at_the_value` covers all six
  stratum shapes at exact pointers. `tc_048_an_application_in_a_frame_body_refuses_at_the_application`
  and `tc_048_a_body_inside_the_strata_is_admitted` are the positive controls. Correct.
- AC-115: `tc_048_the_first_offending_construct_in_pre_order_is_refused` covers every
  clause: nested case at the operator, both swap orders, the two `details` pre-order
  pointers, and non-temporal classes in `details`. `tc_048_a_misplaced_body_root_application_refuses_at_the_node`
  covers the body-root clause. Correct.
- AC-116: `tc_048_the_body_grammar_is_checked_ahead_of_every_identity` makes all
  identities stale (projection emptied, package id zeroed) and checks each class and the
  nested case, plus a flattened control refusing at `stale_dependency` and at
  `stale-node-key`. Correct, both directions.
- AC-117: `tc_048_a_hundred_thousand_node_chain_is_admitted_and_lowered_on_a_small_stack`,
  `tc_048_no_read_limit_and_no_limit_kind_names_a_depth`,
  `tc_048_the_v2_reader_sources_hold_no_depth_ceiling_or_growing_stack`,
  `tc_048_a_document_nested_past_the_parse_limit_refuses_at_the_parse` (300 levels with no
  pointer, ahead of canonical bytes; 20 levels at the first value outside the grammar).
  Correct for the clauses as written.
- AC-118: `tc_048_qspec_adverse_mutations_refuse_as_recorded`, run with `make
  conformance-qspec`; it passes against 2f846f8.
- AC-9 (six limits) and FR-040-AC-10: tables and `tc_056_a_state_clause_application_stands_only_as_a_clause_body_root`.
  Correct.

`quire coverage --strict`: 23 unbacked rows, the same as main. FR-038 is 115/119; the 4
unbacked rows are the planned AC-119 through AC-122 (IR-551). `quire validate` passes
with 1 grammar finding (FR-014, pre-existing).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038 "Reading" now says, in the present tense, that no walk of the V2 reader after the strict parse recurses on the call stack at any depth, and the matrix marks it implemented. The closed-schema decode (`decode_closed`, after the parse in the order of checks) recurses on the call stack once per JSON level of a body. A body nested to between about 91 and 127 JSON levels overflows a 256 KiB debug thread (SR-1236 FND-002). The statement and its "implemented" status are not true of the code. AC-117's two cases (20 and 300 levels) both miss the window, so every test passes | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:167-177 |
| FND-002 | low | AC-114's last clause ("the same meaning with each composite subterm as its own node reached by `reference` is not refused `malformed_wire`") is backed in `make test` only by admitting the unchanged `v2_all_families` fixture. No refused shape in the test has a flattened counterpart built next to it. QSpec's own `flattened` controls do cover the five body-grammar mutations, but only under `make conformance-qspec`, which is outside `make test` and `make ci`. Add a flattened control next to at least the nested-call and binding-root cases | tests/it/checked_package_v2_flat_wire.rs:333-335 |

## Verdict

AC-114, AC-115, AC-116 and AC-118 are fully backed by tests with exact pointers. They are
hand-written expectations, not oracles copied from the implementation. AC-117 is backed
as written. The FR-038 "Reading" claim it summarises is not met (FND-001, same root cause
as SR-1236 FND-002). The planned-to-implemented flips are otherwise each backed by a
tagged test.

## Dispositions

Round 1, reviewed at b7dd914b9c995dc9979d31cebfea5feafb4822ad.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | b7dd914. FR-038 "Reading" now says the closed decode does not read bodies, projection bodies or `details`, and names the 90-126-level window that refuses at the grammar on a 256 KiB debug stack. Reproduced. See SR-1236 FND-007 for one remaining omission |
| FND-002 | fixed | b7dd914. `tc_048_the_flattened_form_of_each_refused_body_is_admitted` admits the reference form of each refused position and stratum shape |
