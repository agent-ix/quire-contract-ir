---
id: SR-1207
title: "PR #281 FR-346 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@98ed90df11a41f8c76aa1b4146b31916d56b4bf7; FR-346-AC-1..10, TC-225; tests/it/checked_package_v2_abstraction_relation.rs, crates/quire-contract-model/src/checked_package/v2/rust_spelling.rs (tests), crates/quire-contract-model/src/checked_package/v2/mod.rs (tests), crates/quire-contract-model/src/checked_package/v2/abstraction.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-346
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-225
    type: reviews
---
# SR-1207: PR #281 FR-346 gap analysis

## Summary

Ticket: IR-509. PR agent-ix/quire-contract-ir#281. Plan completion: not
assessed.

Each FR-346 criterion was mapped to the tests that carry its tag, and each
test's oracle was read:

- AC-1: `tc_225_the_minimal_abstraction_relation_body_admits_with_its_computed_key`,
  `tc_225_empty_tuple_field_and_raw_identifier_bodies_admit`,
  `tc_225_an_unrecognized_correspondence_form_still_refuses_at_semantic_form`.
  These use the AC's domain package and body. `node_id` is computed in the
  test, not by the reader.
- AC-2: `tc_225_a_body_off_its_closed_shape_refuses_without_a_cause` (every
  named shape case at its path), `..._an_occurrence_role_or_another_form_...`,
  `..._an_application_at_the_body_root_...`,
  `..._an_application_as_a_member_value_...`, `..._a_deeply_nested_member_...`,
  and the unit test `tc_225_exactly_one_kind_has_an_abstraction_relation_body`.
- AC-3: `tc_225_a_frame_entry_binds_its_pair_with_or_without_a_frame_node`.
- AC-4: `..._identity_members_that_do_not_hold_refuse_at_the_node` (stale key,
  `semantic_type`, both dependency cases, not `stale-node-key`) and
  `..._an_array_out_of_its_canonical_order_refuses_at_the_array` (objects,
  populations, frames by operation, fields, parameters).
- AC-5: `..._a_target_of_the_wrong_kind_or_unselected_owner_...`, with the
  locus at the target key.
- AC-6: `..._a_field_binding_or_rust_spelling_off_its_syntax_...` and the
  `rust_spelling.rs` unit tests. This covers `"not a name"`, `"crate"`,
  `"9lives"`, a conflicting `version` at `fields/1` and an ambiguous name.
- AC-7: `..._a_frame_operation_resolves_as_an_anchor_does_...`. This covers
  `noSuchOperation`, `version`, ambiguous, inherited, missing `next`, `a`/`a`
  at `parameters/1`, and receiver `next` at `parameters/0`.
- AC-8: `..._a_second_binding_of_one_key_refuses_at_the_second_entry`. All
  three keys are tested within one node and across two nodes. The object and
  operation pairs sit in position order opposite to node-id order, so they
  discriminate between the two orders.
- AC-9: four tests, one per ordering clause.
- AC-10: `..._a_changed_binding_changes_the_node_key_and_the_package_id`.

Coverage measured: `quire coverage --scope . --strict` gives 23 unbacked rows
(35 at base) and 0 contradicted. FR-346 is 10/10. The ✅ on the TC-225 and
FR-346 matrix rows is backed by these tests.

## Verdict

**APPROVE WITH NITS** (three low test-oracle findings; no AC is unbacked).

Every clause of AC-1 through AC-10 has a test that asserts the exact code,
cause, RFC 6901 path and, where FR-346 states one, locus. The examples in the
ACs are the ones the tests use.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-9 "two defective nodes report the lower `node_id` node's own defect" does not discriminate. The lower-digest body (`missingOne`) is also the first relation node in position order, so a reader that walked nodes in position order would pass. Swapping the bodies' order, or asserting that position and digest order disagree, makes the oracle real. | tests/it/checked_package_v2_abstraction_relation.rs:1789-1801 |
| FND-002 | low | AC-2 says a `temporal_formula` application at the body root refuses "from the temporal step". The test checks only the code and the node pointer, which the shape check that opens the abstraction step also gives. No case pairs it with a defect between those stages, for example a member defect on a lower-`node_id` abstraction node, so the stage is not pinned. | tests/it/checked_package_v2_abstraction_relation.rs:809-831 |
| FND-003 | low | AC-3's second clause ("the entry's pair equals that frame's `operation_anchor` pair") is asserted by comparing the anchor's fixture JSON with the entry's fixture JSON. Both are built from the same `context` and `"attemptUpdate"` in the test, so the assertion cannot fail. Only the admission is observed from the reader. | tests/it/checked_package_v2_abstraction_relation.rs:993-996 |

## Dispositions

Round 1 was reviewed at 0768afb609da09d3a82043b1fd63a1780b0a6afa.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 0768afb. The two defective nodes are placed in descending key order. A guard assertion checks that the lower key sits at the later position, so the oracle now separates node-id order from position order. |
| FND-002 | fixed | 0768afb. `tc_225_a_root_application_is_refused_by_the_step_that_places_it`: a temporal-class root is reported ahead of a lower-key member defect. An ordinary-class root is reported after a lower-key defect and before a higher-key one. This pins the temporal step against the abstraction shape check. |
| FND-003 | fixed | 0768afb. Both pairs are now read back from the admitted package's graph. A negative control (an anchor for `rebase`) admits and shows the pair comparison can fail (`assert_ne`). |
