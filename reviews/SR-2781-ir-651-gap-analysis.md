---
id: SR-2781
title: "IR-651 composite operand accessor gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #320; FR-038-AC-177 through FR-038-AC-182 against TC-048 and the changed source and tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Ticket: IR-651. Pull request agent-ix/quire-contract-ir#320. Gap analysis of the
composite equality accessor against the merged IR-651 contract: FR-038 section
"Typed authored domains of composite equality operands", AC-177 through AC-182,
and the TC-048 case section for those criteria. The reviewed head is recorded only
in the Linear review marker.

The computed Test Matrix (`quire matrix --scope . --format tsv`) was measured at
the head and at the merge base. All 336 criterion ids remain, and exactly AC-177
through AC-182 change from `untagged` to `tagged`. `quire coverage --scope .
--strict` reports 31 unbacked rows at the base and 25 at the head, with 0
contradicted statuses at both. The author's claim of 25 against the base's 31 is
confirmed. A tag is a binder index; it does not prove that every clause of a
criterion runs.

The new production code has an owning requirement (FR-038). No stub, copied
foreign fixture, SHA or pin catalog, local path or conflict marker appears in the
diff.

- `FR-038-AC-177` (examined): an admitted structural.eq over parameter references
  returns the application and occurrence with typed operands in argument order.
  Swapping arguments swaps the entries, and a repeated parameter keeps two entries.
  A second authentic occurrence is retained, and an absent occurrence refuses
  `MissingOccurrence`. Every clause has a killing assertion.
- `FR-038-AC-178` (examined): a closed graph literal keeps its identity with a
  `Literal` disposition. A parameter-reading subtree refuses
  `NonliteralGraphValue` and an application subterm refuses `ApplicationSubterm`.
  The defensive inline integer case refuses `InlineInteger` and other inline kinds
  refuse `InlineNonInteger`. Gaps: FND-003, FND-004, FND-005.
- `FR-038-AC-179` (examined): exact field, collection and element paths; endpoints
  beyond i128 and u64; a change to one source type changes only its descriptor;
  explicit unbounded descriptors. Every clause has a killing assertion.
- `FR-038-AC-180` (examined): optional-presence and direct Option edges; the List
  Depth key `[]` with reentry `[1,0]`; Tree reentries `[0,0]` and `[1,0]`; shared
  siblings without Depth; all eighteen scalar and bounded forms; enum order. Every
  clause has a killing assertion.
- `FR-038-AC-181` (examined): the twelve variants and their payloads, ordinal and
  type loci, the work boundary, node-order invariance and first-defect order.
  Gaps: FND-002, FND-006, FND-007.
- `FR-038-AC-182` (examined): external exhaustive matching, equality across
  repeated and cloned calls, the API inspection and the `structural.ne` gap.
  Covered.

## Verdict

**CONDITIONAL.** AC-177, AC-179, AC-180 and AC-182 are backed by assertions that
kill the named mutants. AC-178 and AC-181 have real coverage gaps, and the
specification's status prose still calls all six criteria planned and unrun while
the matrix now tags them. Code defects are recorded in SR-2780.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-177 through AC-182 rows, the TC-048 section and both tests.md rollups still say PLANNED / UNRUN although the matrix now tags all six criteria; the IR-680 code PR updated its AC-183 through AC-185 rows | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3808-3813 |
| FND-002 | medium | No refusal is ever exercised at an enclosing ordinal other than 0, because every fixture passes the same operand twice; a mutant that hard-codes `ordinal: 0` survives | tests/it/checked_package_v2_composite_operands.rs:600-660 |
| FND-003 | medium | The ApplicationSubterm reason has no public admitted case; the inline application-argument branch has no test; TC-048's graph value with an application member is absent | crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:419-425 |
| FND-004 | medium | TC-048's separately authored integer-inline structural.eq input, whose reader-admission refusal must stand, is not tested, so the measurement premise for the defensive InlineInteger rule is not asserted | tests/it/checked_package_v2_composite_operands.rs:563-661 |
| FND-005 | low | AC-178 and TC-048 name closed record, tuple, collection and Option graph literals; only `record_value` is tested | tests/it/checked_package_v2_composite_operands.rs:569-589 |
| FND-006 | low | The one-below-budget WorkLimit check uses a weak `matches!` with `cost>limit`, not an exact payload; no independently computed charge backs TC-048's "expected charges come from the specified semantic visits" | tests/it/checked_package_v2_composite_operands.rs:493-495 |
| FND-007 | low | The seven new integration tests do not use the `tc_048_` name prefix that every other `checked_package_v2_*` integration file uses for TC traceability | tests/it/checked_package_v2_composite_operands.rs:366 |
| FND-008 | low | Pre-existing on IR main (from #318), outside this diff: the FR-038-AC-185 row ends with merge-conflict residue `>>>>>>> <commit-id> (IR-680: ...)`, in the table that FND-001's status update must edit | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3816 |

### FND-001 detail

The matrix TSV at the head shows AC-177 through AC-182 as `tagged`, but each
statement still begins "PLANNED / UNRUN". TC-048 line 543 says "All cases remain
PLANNED / UNRUN". `spec/checked_package/matrix/tests.md:16` and
`spec/tests.md:15` carry the matching flag. In the adjacent rows, AC-176 says
"IMPLEMENTED (IR-654 code ...)" and AC-183 says "Implemented and verified by
TC-048 ... (IR-680 code)". A reader cannot tell from the specification that this
accessor exists.

### FND-002 detail

AC-181 reads "Unsupported union, malformed optional wrapper and unanchored cycle
carry the actual enclosing ordinal". FR-038 adds "Every `ordinal` below is the
enclosing argument position". Every integration fixture builds `structural.eq(x, x)`
or `structural.eq(none, none)`, and every unit mutation changes argument 0 or a
node that both arguments share. A test with a valid first operand and a defect in
argument 1 would kill the mutant.

### FND-003 detail

FR-038 has two ApplicationSubterm branches. The inline `term: application`
argument branch (lines 419-425) is never reached by any test. The graph-reference
branch (lines 433-438) is reached only by a unit test that rewrites a parameter
node's body. TC-048 asks for "a graph value whose member reads a parameter and an
application subterm". Only the parameter member is present. An admitted
`structural.eq` over a reference to an application node of structural result type,
for example a collection-producing operation, would give a public case.

### FND-004 detail

TC-048 reads: "retain reader admission's original refusal on a separately authored
integer-inline structural.eq input". The codex2 measurement ruling, under which
the catalog excludes Integer from `structural_kind`, is the reason InlineInteger
is defensive only. No test in this PR, and none found in the existing
`structural.eq` tests, authors that input and asserts the reader's refusal.

### FND-006 detail

`assert!(matches!(..., Err(Error::WorkLimit {limit, consumed: cost}) if limit ==
consumed-1 && cost > limit))` accepts any overshoot. The expected `consumed` comes
only from the accessor's own successful run, so the charge formula itself is never
checked (see SR-2780 FND-003).

## Bindings

| Test | AC | Trace |
| --- | --- | --- |
| positional_children_and_selected_occurrences_survive_repeated_calls_and_clone | FR-038-AC-177 | correct |
| positional_children_and_selected_occurrences_survive_repeated_calls_and_clone | FR-038-AC-182 | correct |
| exact_authored_decimal_bounds_and_paths_have_a_reproducible_work_boundary | FR-038-AC-179 | correct |
| exact_authored_decimal_bounds_and_paths_have_a_reproducible_work_boundary | FR-038-AC-181 | correct |
| closed_graph_values_preserve_identity_while_free_values_and_unions_refuse | FR-038-AC-178 | correct |
| closed_graph_values_preserve_identity_while_free_values_and_unions_refuse | FR-038-AC-181 | correct |
| optional_wrapper_paths_and_all_reentries_preserve_distinct_source_routes | FR-038-AC-180 | correct |
| every_scalar_and_bounded_form_keeps_its_authored_position_and_source_kind | FR-038-AC-180 | correct |
| every_scalar_and_bounded_form_keeps_its_authored_position_and_source_kind | FR-038-AC-182 | correct |
| collection_kinds_remain_unbounded_and_ordered_enums_keep_semantic_order | FR-038-AC-179 | correct |
| collection_kinds_remain_unbounded_and_ordered_enums_keep_semantic_order | FR-038-AC-180 | correct |
| external_error_consumer_retains_authentic_node_and_application_loci | FR-038-AC-181 | correct |
| external_error_consumer_retains_authentic_node_and_application_loci | FR-038-AC-182 | correct |
| defensive_target_failures_never_salvage_or_fabricate_a_child_identity | FR-038-AC-181 | correct |
| defensive_operand_and_catalog_failures_keep_their_original_typed_loci | FR-038-AC-178 | correct |
| defensive_operand_and_catalog_failures_keep_their_original_typed_loci | FR-038-AC-181 | correct |
| malformed_optional_wrappers_unanchored_cycles_and_union_metadata_refuse_exactly | FR-038-AC-181 | correct |
| malformed_optional_wrappers_unanchored_cycles_and_union_metadata_refuse_exactly | FR-038-AC-180 | correct |
| attempted_meter_overflow_is_a_refusal_even_at_the_maximum_limit | FR-038-AC-181 | correct |
| path_width_refusal_keeps_actual_ordinal_and_containing_type | FR-038-AC-181 | correct |

## Coverage

Plan completion: not assessed. Measured with quire 0.36.1 (engine 0.50.1).
Matrix and coverage runs exited with the installed module's
`semantic.inline-data-schema`, `DuplicateArchetype` and `DuplicateInverseEdge`
notices, which are the same at the head and at the base.
