---
id: SR-2026110705000823
title: "ears-conformance review of reached map row projection"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; IR-1409; spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md; spec/checked_package/matrix/TC-473-reached-map-row-projection.md"
review_set: subset
---

## Summary

Reviewed new requirement statements and acceptance wording for concrete actor, trigger and response. No new EARS defect found.

Frozen spec head 16dc150ccc00ead0ebf776f1ec7b151bd4b78824, PR 952. Direct review; no enforced workflow or human acknowledgement claimed. No applicable AssuranceProfile exists under the repository spec tree.

## Examined Scope

```yaml
{
  "scope": [
    {
      "id": "FR-357-AC-19",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "examined",
      "excerpt": "The public `SelectedSourceCollectionRow` carries `key_node: Option<CheckedNodeId>`. With one strictly admitted reached map whose key is scalar and whose value is a record, the collection row join and Exact projection return its complete producer `ValueType::Map`, preserving structural/base/key/value nodes and key-then-value order. The monitor closure retains that map row and follows both type dependencies; an unrelated map row is not selected. Existing sequence, set, bag and ordered-set rows retain `key_node: None` and their existing complete values. A row with a non-collection/non-map Exact v"
    },
    {
      "id": "FR-357-AC-20",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "examined",
      "excerpt": "Map row selection, acquisition and projection compose on one caller-owned `NativeJoinBudget`. Their measured exact combined work and storage allowances return the same complete map facts as ample limits; one unit below either refuses before its first unpaid operation with the original bound, native kind, full amount, row/key/value locus, consumed prefix and no partial output or success delta. Zero storage refuses before any owned allocation, including diagnostic material. A selected row after an unrelated earlier row retains its original `ReachedRow` index; external adapter work retains its `A"
    },
    {
      "id": "FR-357-AC-16",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "context_only",
      "excerpt": "A reached admitted `composite_type`/`map` with a scalar key and a record value joins a collection row carrying its key node and value node in key-then-value order, and the closure retains the map's complete Exact `ValueType::Map`. A row whose `key_node` is absent, names another node, or is present on a non-map collection refuses `SelectedMonitorTypeClosureErrorCode::CollectionKey` at the collection node, with charged work and no partial closure; an unreached map row does not refuse. The closure with the map succeeds at its exact measured work and storage allowance, and one unit below either re"
    },
    {
      "id": "FR-357-AC-17",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "context_only",
      "excerpt": "The closure read requires the same-emission authority row of FR-361 and returns either a complete closure carrying the receipt and layout context or one typed first refusal; no read exists without that row, and no partial closure escapes an authority refusal."
    },
    {
      "id": "FR-357-AC-18",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "context_only",
      "excerpt": "The closure read seeds the plan's `tick_unit` premise with its unit and dimension facts and exposes `SelectedMonitorTypeClosure::tick_unit_dimension() -> &CheckedNodeId`, the admitted dimension node of that unit. For a source tick unit and for a model tick unit, it returns that unit's own admitted dimension; a tick unit whose admitted dimension is not the base Time dimension refuses at strict admission before any closure exists; the seeded facts are charged and succeed at the exact allowance and refuse one unit below at the first unpaid seed charge."
    },
    {
      "id": "FR-357",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "examined",
      "excerpt": "`SelectedSourceCollectionRow` SHALL expose the optional public `key_node`\nalongside its structural node, base node, element node and complete Exact\nvalue type. For a map, `key_node` SHALL name the admitted key type and\n`element_node` SHALL name the admitted value type; the admitted base references\nSHALL be joined in key-then-value order. For an ordinary sequence, set, bag or\nordered set, `key_node` SHALL be `None` and `element_node` SHALL retain its\nexisting element meaning. A map row's value SHALL be the complete producer\n`ValueType::Map`, not a unary collection approximation.\n\nThe existing s"
    },
    {
      "id": "TC-473",
      "path": "spec/checked_package/matrix/TC-473-reached-map-row-projection.md",
      "role": "examined",
      "excerpt": "Verify that reached map rows expose and retain the admitted key and value\nstructural nodes and complete producer Exact map value, and charge selection,\nrow materialization and nested projection to one native budget."
    },
    {
      "id": "FR-358",
      "path": "spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md",
      "role": "context_only",
      "excerpt": "When a consumer assembles selected Exact identity witnesses from one admitted\nchecked package, Contract IR SHALL expose one caller-owned, finite work and\nstorage budget that composes across stage-row acquisition and all five native\nsource joins. The budgeted path owns no QSL identity judgment: the admitted\nstructural joins and the producer-supplied opaque identities remain governed by\n[FR-352](./FR-352-project-admitted-exact-value-types.md). This requirement is\nthe native-join prerequisite for [FR-357](./FR-357-project-selected-generated-type-closure.md)."
    }
  ]
}
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

- Actual `SelectedSourceCollectionRow` currently lacks key_node, and `project_source_collection` requires Collection; the stated API and behavior remedy this exact seam without an alternate reader.
- Existing standalone Map projection explicitly requires the admitted Text key and constructs native MapType; the spec retains these restrictions and existing cardinality behavior.
- Existing selected_rows retains original ReachedRow indices, source_collection storage accounting has a single owner, and compact native_error contains no owned node diagnostic. Extending those paths supports precharge and zero-storage refusal without a new budget or shim.
- `quire properties` exposes all twenty unique FR-357 criteria, including new AC-19/20. Changed document validation exits zero with one unchanged old grammar warning; unchanged main reproduces that warning. No implementation or Cargo gate is claimed in this initial stage.
- TC-473 uses authentic strictly admitted map inputs, independently expected key/value nodes and original row loci, a paid prefix, exact and one-under combined allowances, real zero-storage allocation observation and unchanged refusal controls.

## Verdict

SPEC PASS. The one non-canonical-shape warning is on an unchanged existing criterion and reproduces in the supplied unchanged-main validation. Both new criteria are direct observable assertions; the new behavior has a named actor and explicit bounded refusal conditions. Implementation and its review/final gate remain pending.

## Static Mixed Review Stage

The map implementation source was inspected at 7dff46a1 plus trace-only fix 30c76d1d. The approved map semantics and parsed AC-19/20 are unchanged except that preceding public caller charges keep their supplied native locus; this does not broaden the map API or key domain. No new specification defect is found. The code-review resource-locus finding FND-003 remains open, and focused dynamic results plus owner-lock adoption are pending; no final mixed PASS is claimed.

## Static Fix Inspection Round 2

Inspected actual production changes through 9c632ce70d50d8a6b5e88b926bfdfdb07b5dcd9e, including the Exact branch-main lock refresh and the missing unary collection key_node: None initializer. The new map key/value validation calls charge_native_work with source_map_key/source_map_value before inspecting the corresponding joins. A fresh public Exact projection initializes a Package locus before its first paid lookup, then establishes GraphNode from the admitted lookup result; an existing caller locus is borrowed and restored, while the denied-locus snapshot remains sticky. This source inspection resolves the missing/stale native-locus defect. It is not a claim that the newly added tests executed.

The new oracles independently construct ReachedRow(1) for the second genuine collection input, fixed key/value phase names, the expected admitted key/value node, GraphNode(inner_position) for fresh projection and GraphNode(map_position) for cold numeric handle denial. They assert the full expected work charge, original allowance, spent prefix, empty success delta and sticky refusal. They would reject absent, constant-zero, stale or wrong-phase loci. No new static defect was found in the fix diff. Dynamic focused checks remain queued, and no final mixed PASS is claimed.

## Final Mixed Review

Reviewed clean pushed head c52aaebb0a35e3ae38899a36d2d44cc7308a4c10 and all subsequent implementation/fix diffs. No new findings. FND-002 and FND-003 latest outcomes are fixed. This final review supersedes earlier pending/static-only stage verdicts without rewriting their history.

- Actual receipts: /tmp/ir1409-map-fixed.log has 13/13 map tests passing; /tmp/ir1409-closure-final.log has 3/3 selected-map closure tests passing; /tmp/ir1409-clippy-final.log is the required ordinary strict workspace lane and passes; /tmp/ir1409-fmt-final.log is empty with successful formatting status; channel summary records closure/clippy/fmt exit0. Retained unchanged unary controls pass 4/4, numeric comparison handle 1/1, and allocation-free cold footprint 1/1.
- The real closure fixture follows admitted monitor State -> activations -> the declared activation record and actual Inputs identity. Its retained by_key record member carries the genuine complete Map value, not an Integer stand-in. This corrects the fixture without replacing production traversal or weakening the value oracle.
- Zero-storage tests measure constructed inputs and assert zero allocations, including compact diagnostics. Late denial tests preserve actual paid-prefix allocations; caller row clones occur before measurement. Admitting the native row reservation adds actual native row/container allocations, and cold copied integer payloads add no hidden cache allocations relative to the same native prefix. Shared Map/Collection payload handles are not falsely described as deep clones.
- Independent original row and graph indices, fixed key/value/selection phase names, original bounds, full attempted charge, retained prefix and no success delta are asserted. The new selection reset occurs only after successful preflight, so a failure inside preflight keeps the original denied locus. Fresh public projection initializes Package before lookup, promotes to an admitted GraphNode only after the paid lookup and restores the caller active context without clearing the first-denial snapshot.
- The actual branch-main Exact owner refresh supplies copy_footprint/try_eq_bounded; no owner branch pin, shim, shadow numeric type, fresh join budget or charge replay was introduced. Existing standalone native Map Text-key restrictions and ordinary non-map collection/refusal controls remain. The model remains cycle-free and no build/test workflow is narrowed.
- Computed matrix at this final head reports AC-19 tagged by six real tests and AC-20 by nine. All fifteen binders were read against assertions. New trace attributes retain exactly TC-first plus one AC; separate attributes preserve dual coverage. The old unchanged FR-357 grammar warning is not an introduced finding. IR-1399/PR951 bounded-wrapper closure enumeration remains a separate feature; it is not a blocker for the map criteria delivered here.

### Final Examined Scope

```yaml
{
  "scope": [
    {
      "id": "FR-357-AC-19",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "examined",
      "excerpt": "The public `SelectedSourceCollectionRow` carries `key_node: Option<CheckedNodeId>`. With one strictly admitted reached map whose key is scalar and whose value is a record, the collection row join and Exact projection return its complete producer `ValueType::Map`, preserving structural/base/key/value nodes and key-then-value order. The monitor closure retains that map row and follows both type dependencies; an unrelated map row is not selected. Existing sequence, set, bag and ordered-set rows retain `key_node: None` and their existing complete values. A row with a non-collection/non-map Exact v"
    },
    {
      "id": "FR-357-AC-20",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "examined",
      "excerpt": "Map row selection, acquisition and projection compose on one caller-owned `NativeJoinBudget`. Their measured exact combined work and storage allowances return the same complete map facts as ample limits; one unit below either refuses before its first unpaid operation with the original bound, native kind, full amount, row/key/value locus, consumed prefix and no partial output or success delta. Zero storage refuses before any owned allocation, including diagnostic material. A selected row after an unrelated earlier row retains its original `ReachedRow` index; preceding caller charges retain thei"
    },
    {
      "id": "FR-357-AC-16",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "context_only",
      "excerpt": "A reached admitted `composite_type`/`map` with a scalar key and a record value joins a collection row carrying its key node and value node in key-then-value order, and the closure retains the map's complete Exact `ValueType::Map`. A row whose `key_node` is absent, names another node, or is present on a non-map collection refuses `SelectedMonitorTypeClosureErrorCode::CollectionKey` at the collection node, with charged work and no partial closure; an unreached map row does not refuse. The closure with the map succeeds at its exact measured work and storage allowance, and one unit below either re"
    },
    {
      "id": "FR-357-AC-17",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "context_only",
      "excerpt": "The closure read requires the same-emission authority row of FR-361 and returns either a complete closure carrying the receipt and layout context or one typed first refusal; no read exists without that row, and no partial closure escapes an authority refusal."
    },
    {
      "id": "FR-357-AC-18",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "context_only",
      "excerpt": "The closure read seeds the plan's `tick_unit` premise with its unit and dimension facts and exposes `SelectedMonitorTypeClosure::tick_unit_dimension() -> &CheckedNodeId`, the admitted dimension node of that unit. For a source tick unit and for a model tick unit, it returns that unit's own admitted dimension; a tick unit whose admitted dimension is not the base Time dimension refuses at strict admission before any closure exists; the seeded facts are charged and succeed at the exact allowance and refuse one unit below at the first unpaid seed charge."
    },
    {
      "id": "TC-473",
      "path": "spec/checked_package/matrix/TC-473-reached-map-row-projection.md",
      "role": "examined",
      "excerpt": "Verify that reached map rows expose and retain the admitted key and value\nstructural nodes and complete producer Exact map value, and charge selection,\nrow materialization and nested projection to one native budget."
    },
    {
      "id": "crates/quire-contract-model/src/checked_package/v2/exact_projection.rs",
      "path": "crates/quire-contract-model/src/checked_package/v2/exact_projection.rs",
      "role": "examined",
      "excerpt": "Map keys and values are separately joined before row retention; native comparison/storage charges and fresh projection loci precede their operations."
    },
    {
      "id": "crates/quire-contract-model/src/checked_package/v2/monitor_plan/selected_type_closure.rs",
      "path": "crates/quire-contract-model/src/checked_package/v2/monitor_plan/selected_type_closure.rs",
      "role": "examined",
      "excerpt": "Selected map rows retain original ReachedRow indices; after successful footprint preflight the selection form is restored before storage admission."
    },
    {
      "id": "crates/quire-contract-model/src/checked_package/v2/native_join_storage.rs",
      "path": "crates/quire-contract-model/src/checked_package/v2/native_join_storage.rs",
      "role": "examined",
      "excerpt": "Cold Integer copy footprints use the allocation-free owner inspector before storage admission."
    },
    {
      "id": "tests/it/checked_package_v2_map_type.rs",
      "path": "tests/it/checked_package_v2_map_type.rs",
      "role": "examined",
      "excerpt": "The real strict reader exercises complete map values, malformed rows, nested and bounded maps, exact/one-under shared budgets, independent key/value/graph loci and zero or paid-prefix allocation scopes."
    }
  ],
  "bindings": [
    {
      "test_id": "tests::reached_map_rows::selected_closure_retains_only_reached_map_and_composes_exact_budget",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "tests::reached_map_rows::selected_map_key_refusal_is_typed_and_unreached_bad_key_is_ignored",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "source_map_row_joins_and_projects_its_complete_value",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "source_map_rows_preserve_key_and_non_collection_refusals",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "nested_and_bounded_source_maps_preserve_all_value_rows_and_bounds",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "cold_integer_map_rows_inspect_and_compare_before_only_admitted_allocations",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "tests::numeric_map_comparison_handle_is_charged_before_its_single_allocation",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "tests::reached_map_rows::selected_closure_retains_only_reached_map_and_composes_exact_budget",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "tests::reached_map_rows::selected_map_row_keeps_original_index_and_precharges_its_actual_copy",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "map_copy_tests::cold_map_integer_footprint_inspection_allocates_nothing_and_matches_owned_copy",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "source_map_rows_compose_at_exact_budget_and_preserve_first_denial",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "source_map_zero_storage_refuses_without_allocating_rows_or_diagnostics",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "nested_and_bounded_source_maps_preserve_all_value_rows_and_bounds",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "source_map_copy_allocation_follows_its_storage_charge_after_a_paid_prefix",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "cold_integer_map_rows_inspect_and_compare_before_only_admitted_allocations",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    }
  ]
}
```

### Final Verdict

PASS in reviewed PR scope. No open review findings. The lead still owns the ONE final combined-tree gate on the final integrated head; no aggregate gate or merge PASS is claimed by this review.

## Pre-gate Reopened Review

Concrete locked-owner fallback finding FND-004 in the code-review artifact reopens overall acceptance: nested map row equality can reach cold BoundedQuantity derived equality before the unsupported leaf projection. No additional independent specification defect is asserted here. Prior focused green inputs and scope claims remain historical evidence; current overall verdict is HOLD pending the targeted refusal-path fix and its real test.

## Final Review After Cold-Payload Fix

Reviewed clean frozen source head 7692aae2b9dd3590494a44307d9a59dce76ceb61. This supersedes the prior pre-gate HOLD while retaining all earlier findings, PASS, reopen, actual RED and disposition history. FND-002, FND-003 and FND-004 latest outcomes are fixed; no new finding remains.

The completed serialized focused job /tmp/ir1409-fnd004-focuseds.log records map_exit=0, declared_unit_exit=0, projection_exit=0, clippy_exit=0 and fmt_exit=0. Inspected actual logs: full map module15/15, existing supported declared-unit record control1/1, exact_projection library module7/7, required ordinary strict workspace conformance-cli Clippy and rustfmt pass. Earlier final closure3/3, unary4/4, cold footprint1/1 and numeric handle1/1 receipts remain valid for unchanged code. No final aggregate gate has run under this review.

The current computed matrix contains six AC-19 binders and eleven AC-20 binders, all live and traced with exact TC-first/two-ID attributes. The two new native-refusal controls exercise the previously untested independently owned cold BQ and compound exponent paths against actual strict admission, not a private test-only bypass. The generic equality fallback is no longer used for same-variant unsupported owned numeric leaves; precharged diagnostics and supported Declared quantity semantics remain. No pending PR951 bounded-wrapper feature is invented as a prerequisite for these map obligations.

### Final Scope and Bindings

```yaml
{
  "scope": [
    {
      "id": "FR-357-AC-19",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "examined",
      "excerpt": "The public `SelectedSourceCollectionRow` carries `key_node: Option<CheckedNodeId>`. With one strictly admitted reached map whose key is scalar and whose value is a record, the collection row join and Exact projection return its complete producer `ValueType::Map`, preserving structural/base/key/value nodes and key-then-value order. The monitor closure retains that map row and follows both type dependencies; an unrelated map row is not selected. Existing sequence, set, bag and ordered-set rows retain `key_node: None` and their existing complete values. A row with a non-collection/non-map Exact v"
    },
    {
      "id": "FR-357-AC-20",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "examined",
      "excerpt": "Map row selection, acquisition and projection compose on one caller-owned `NativeJoinBudget`. Their measured exact combined work and storage allowances return the same complete map facts as ample limits; one unit below either refuses before its first unpaid operation with the original bound, native kind, full amount, row/key/value locus, consumed prefix and no partial output or success delta. Zero storage refuses before any owned allocation, including diagnostic material. A selected row after an unrelated earlier row retains its original `ReachedRow` index; preceding caller charges retain thei"
    },
    {
      "id": "FR-357-AC-16",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "context_only",
      "excerpt": "A reached admitted `composite_type`/`map` with a scalar key and a record value joins a collection row carrying its key node and value node in key-then-value order, and the closure retains the map's complete Exact `ValueType::Map`. A row whose `key_node` is absent, names another node, or is present on a non-map collection refuses `SelectedMonitorTypeClosureErrorCode::CollectionKey` at the collection node, with charged work and no partial closure; an unreached map row does not refuse. The closure with the map succeeds at its exact measured work and storage allowance, and one unit below either re"
    },
    {
      "id": "FR-357-AC-17",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "context_only",
      "excerpt": "The closure read requires the same-emission authority row of FR-361 and returns either a complete closure carrying the receipt and layout context or one typed first refusal; no read exists without that row, and no partial closure escapes an authority refusal."
    },
    {
      "id": "FR-357-AC-18",
      "path": "spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md",
      "role": "context_only",
      "excerpt": "The closure read seeds the plan's `tick_unit` premise with its unit and dimension facts and exposes `SelectedMonitorTypeClosure::tick_unit_dimension() -> &CheckedNodeId`, the admitted dimension node of that unit. For a source tick unit and for a model tick unit, it returns that unit's own admitted dimension; a tick unit whose admitted dimension is not the base Time dimension refuses at strict admission before any closure exists; the seeded facts are charged and succeed at the exact allowance and refuse one unit below at the first unpaid seed charge."
    },
    {
      "id": "TC-473",
      "path": "spec/checked_package/matrix/TC-473-reached-map-row-projection.md",
      "role": "examined",
      "excerpt": "Verify that reached map rows expose and retain the admitted key and value\nstructural nodes and complete producer Exact map value, and charge selection,\nrow materialization and nested projection to one native budget."
    },
    {
      "id": "crates/quire-contract-model/src/checked_package/v2/exact_projection.rs",
      "path": "crates/quire-contract-model/src/checked_package/v2/exact_projection.rs",
      "role": "examined",
      "excerpt": "Map keys and values are separately joined before row retention; native comparison/storage charges and fresh projection loci precede their operations."
    },
    {
      "id": "crates/quire-contract-model/src/checked_package/v2/monitor_plan/selected_type_closure.rs",
      "path": "crates/quire-contract-model/src/checked_package/v2/monitor_plan/selected_type_closure.rs",
      "role": "examined",
      "excerpt": "Selected map rows retain original ReachedRow indices; after successful footprint preflight the selection form is restored before storage admission."
    },
    {
      "id": "crates/quire-contract-model/src/checked_package/v2/native_join_storage.rs",
      "path": "crates/quire-contract-model/src/checked_package/v2/native_join_storage.rs",
      "role": "examined",
      "excerpt": "Cold Integer copy footprints use the allocation-free owner inspector before storage admission."
    },
    {
      "id": "tests/it/checked_package_v2_map_type.rs",
      "path": "tests/it/checked_package_v2_map_type.rs",
      "role": "examined",
      "excerpt": "The real strict reader exercises complete map values, malformed rows, nested and bounded maps, exact/one-under shared budgets, independent key/value/graph loci and zero or paid-prefix allocation scopes."
    }
  ],
  "bindings": [
    {
      "test_id": "tests::reached_map_rows::selected_closure_retains_only_reached_map_and_composes_exact_budget",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "tests::reached_map_rows::selected_map_key_refusal_is_typed_and_unreached_bad_key_is_ignored",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "source_map_row_joins_and_projects_its_complete_value",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "source_map_rows_preserve_key_and_non_collection_refusals",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "nested_and_bounded_source_maps_preserve_all_value_rows_and_bounds",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "cold_integer_map_rows_inspect_and_compare_before_only_admitted_allocations",
      "ac_id": "FR-357-AC-19",
      "trace": "correct"
    },
    {
      "test_id": "tests::numeric_map_comparison_handle_is_charged_before_its_single_allocation",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "tests::reached_map_rows::selected_closure_retains_only_reached_map_and_composes_exact_budget",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "tests::reached_map_rows::selected_map_row_keeps_original_index_and_precharges_its_actual_copy",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "map_copy_tests::cold_map_integer_footprint_inspection_allocates_nothing_and_matches_owned_copy",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "source_map_rows_compose_at_exact_budget_and_preserve_first_denial",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "source_map_zero_storage_refuses_without_allocating_rows_or_diagnostics",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "nested_and_bounded_source_maps_preserve_all_value_rows_and_bounds",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "source_map_copy_allocation_follows_its_storage_charge_after_a_paid_prefix",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "cold_integer_map_rows_inspect_and_compare_before_only_admitted_allocations",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "unsupported_nested_bounded_quantities_refuse_without_comparing_cold_payloads",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    },
    {
      "test_id": "unsupported_nested_compound_quantities_refuse_without_comparing_cold_exponents",
      "ac_id": "FR-357-AC-20",
      "trace": "correct"
    }
  ]
}
```

### Current Final Verdict

PASS in reviewed PR scope at 7692aae2b9dd3590494a44307d9a59dce76ceb61. No open findings. The lead still owns the single final combined-tree gate on the final integrated head; this artifact does not claim that aggregate gate or merge has passed.
