---
id: SR-2026110705000821
title: "code-review review of reached map row projection"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1409; spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md; spec/checked_package/matrix/TC-473-reached-map-row-projection.md"
review_set: subset
---

## Summary

The spec-only change owns a small public row extension and corresponding existing projection paths; no Rust code, dependency, lockfile, workflow or gate changes are present. Intended implementation scope is concrete and grounded in the current reader.

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

SPEC PASS. No vendored type/schema/checker, tracking machinery, compatibility path or unrelated ceremony is introduced. Rust code review and gap analysis apply after implementation; no missing-code finding is raised in this spec-first stage. Implementation and its review/final gate remain pending.

## New findings (mixed implementation review)

Static review covers the frozen implementation at 7dff46a139b2ebbbde05858b17bdedd96fe237c3 plus trace-only fix 30c76d1d40e4a57a16e442c43f50ebcdb28724dd. Dynamic checks and owner-lock adoption are pending; this is not a final-head PASS. Rust style, panic/resource bounds, wire ownership and test assertion lanes are folded into this code-review artifact.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | Two new three-ID trace attributes violate the user's exact TC-first plus one-AC attribute rule; split each into two two-ID attributes preserving both bindings. | tests/it/checked_package_v2_map_type.rs:652; crates/quire-contract-model/src/checked_package/v2/monitor_plan/reached_model_read.rs:2639 |
| FND-003 | medium | Fresh-budget map projection never initializes a native active locus, so compact resource denials lose their row/key/value location; the new cold-map handle-boundary test cannot find its required nonempty locus. The source-map key work step also leaves the generic join phase because charge_work does not set the locus form. Initialize and update the native projection/key/value locus before the relevant charges, with independently expected boundary assertions. | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:3590-3598; crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:1140-1141; tests/it/checked_package_v2_map_type.rs:723-727 |

## Static Trace Disposition Round 1

The original FND-002 text above is preserved against the first frozen implementation. The later source-only fix has been inspected: both tests now carry separate two-ID attributes for AC-19 and AC-20, preserving both bindings.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Fix 30c76d1d changes only the two named trace sites into two TC-first, one-AC attributes each. |
| FND-003 | still-open | No native active projection/key locus is initialized at the reviewed source head. Dynamic focused receipts remain pending. |

## Static Review Evidence

No source edits, Cargo execution, duplicate worktree or aggregate gate was run by the reviewer. Parent has refreshed the actual branch-main Exact dependency for the real owner copy-footprint API; the old locked missing API is not treated as a semantic defect. Native map identity joins, complete map payload/order, typed key refusals, precharge before container copies and charged cancellation handle acquisition were read. The owner logical Arc-unit rule is preserved; no byte-count unit substitution was requested.

## Static Fix Inspection Round 2

Inspected actual production changes through 9c632ce70d50d8a6b5e88b926bfdfdb07b5dcd9e, including the Exact branch-main lock refresh and the missing unary collection key_node: None initializer. The new map key/value validation calls charge_native_work with source_map_key/source_map_value before inspecting the corresponding joins. A fresh public Exact projection initializes a Package locus before its first paid lookup, then establishes GraphNode from the admitted lookup result; an existing caller locus is borrowed and restored, while the denied-locus snapshot remains sticky. This source inspection resolves the missing/stale native-locus defect. It is not a claim that the newly added tests executed.

The new oracles independently construct ReachedRow(1) for the second genuine collection input, fixed key/value phase names, the expected admitted key/value node, GraphNode(inner_position) for fresh projection and GraphNode(map_position) for cold numeric handle denial. They assert the full expected work charge, original allowance, spent prefix, empty success delta and sticky refusal. They would reject absent, constant-zero, stale or wrong-phase loci. No new static defect was found in the fix diff. Dynamic focused checks remain queued, and no final mixed PASS is claimed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Static round 1: trace-only fix 30c76d1d preserves both bindings through separate two-ID attributes. |
| FND-003 | fixed | Static round 2: actual native key/value charges set their phase before denial, and fresh projection establishes paid authentic graph context while preserving the caller context and sticky denial snapshot. Dynamic receipts remain pending. |

## Dynamic Finding Extension: FND-003

The earlier source-only disposition did not cover a later actual closure denial. At 926c26a4 the selected-row footprint preflight changed the active form to native_type_footprint and the subsequent row-storage charge reused that form. The executed selected_map_row_keeps_original_index_and_precharges_its_actual_copy control refused with ReachedRow(1)/native_type_footprint instead of its independently stated ReachedRow(1)/source_collection_selection. This is an additional manifestation of FND-003, reopening that finding after the earlier static fix; it is not a baseline failure. Receipt: /tmp/ir1409-closure-stale-form.log (2 passed, 1 failed).

## Dynamic Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | still-open | Round 3 at 926c26a4: actual late row-storage denial retained the preceding footprint phase instead of the collection selection phase. |
| FND-003 | fixed | Round 4 at c52aaebb: source restores source_collection_selection only after successful preflight and before row storage. The actual final three closure tests pass, including independent original row index and allocation boundary assertions. Preflight failures keep their first preflight locus. |

## Final Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Two exact two-ID attributes per dual-criterion test remain in final source. |
| FND-003 | fixed | Fresh projection and key/value loci are established before charge; successful selection preflight restores its channel before row storage. Independent native locus tests execute green. |

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

## New findings (pre-gate root verification)

Root supplied a concrete cold bounded-quantity equality concern, then this reviewer traced it against the actual locked Exact a08e1b2 and current IR source. This supersedes the earlier PASS until resolved.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The generic ValueType equality fallback can compare distinct cold BoundedQuantity payloads before the nested map leaf is projected, allocating Integer caches outside native precharge. Preserve the existing unsupported-leaf boundary or use an owner-bounded equality seam; do not add a shadow numeric comparator or warm caches. | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:3469; crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:3591-3599 |

### FND-004 Grounding

The strict IR reader has genuine scalar_type/bounded_quantity admission in structural.rs and existing bounded quantity conformance fixtures. The current Exact projection has no BoundedQuantity leaf projection arm, so this finding does not claim that a full Map<BoundedQuantity> projection succeeds. However, an outer Map<Text,Map<Text,BQ>> row and its inner Map<Text,BQ> row can pass structural collection-row acquisition: validation checks map base/key/value handles, map bounds and outer shape, not the opaque numeric leaf. The nested-row path compares the outer map element value against the next map row BEFORE projecting the inner BQ leaf. Thus the comparison peels Map then reaches the generic fallback with two BQ leaves before the eventual UnsupportedExactType.

Locked Exact ValueType BoundedQuantity equality compares its Arc-backed BoundedQuantityType. Distinct but equal owner payloads reach derived Quantity/Rational/Integer equality; Integer::cmp on cold LargeDigits calls as_big and allocates the lazy cache. Two ValueType clones sharing the same Arc may short-circuit pointer equality and hide this counterexample, so the focused oracle must construct independently owned equal cold payloads before measurement. No dynamic allocation reproduction is claimed by this static review; the source reachability and locked owner chain are concrete. Direct Map<BQ> is not the reproducer because the unsupported leaf is encountered before comparison.

The same coder should add the genuine nested-map refusal-path cold-bound/late-storage test and fix the fallback without broadening unsupported scalar projection or implementing private numeric equality. Zero-storage refusal remains required, but the decisive comparison lies after a genuine paid prefix.

## Pre-gate Current Verdict

HOLD: FND-004 open. Prior findings remain fixed, and prior green receipts remain true for their tested inputs. No final gate or fresh Cargo test was run by the reviewer.

## FND-004 Companion: Compound Unit Payload

The same open fallback finding includes `ValueType::Quantity(UnitId::Compound(...))`. Actual locked Exact a08e1b2 `identity.rs:190-195` derives PartialEq for the owned compound unit terms `(NodeKey, Integer)`. Distinct cold copied exponent Integers therefore reach `Integer::cmp` and `as_big` through generic Quantity ValueType equality. This does not depend on a shared Arc identity shortcut.

The native IR identity boundary excludes Compound from declared unit identities: SelectedExactDeclarationForm::matches accepts DeclaredUnit only with UnitId::Declared, the stage join rejects Compound as a unique declared identity, and ScalarType::Unit Exact projection only emits a Declared quantity. ScalarType::CompoundUnit has no Exact projection arm. The outer Map row shape validator does not check its opaque leaf unit identity, so nested Map<Quantity(Compound)> values can still reach this comparison before the unsupported/invalid actual unit leaf is projected. Budget/refusal guarantees apply on this path even though it cannot produce a complete supported compound-quantity projection.

The FND-004 fix/control must reject or validate this unsupported/invalid compound-unit leaf before derived Eq without rejecting supported `Quantity(Declared)` or adding custom limb comparison, cache warming or new unit semantics. A nested-map cold exponent refusal-path control belongs in this same feature PR. No new ticket or finding ID is allocated; original finding text is preserved. No reviewer Cargo or source edits occurred.

## FND-004 Actual Runtime Reproduction

The same coder now supplies genuine runtime RED in /tmp/ir1409-bq-red.log: `unsupported_nested_bounded_quantities_refuse_without_comparing_cold_payloads` reaches the intended refusal on a strictly admitted nested BQ graph, then observes 342 whole-call allocations for independently owned cold payloads versus 330 for independently parsed equal payloads. The 12 additional equality-cache allocations confirm the locked-source counterexample. This is not the earlier expanded-ID fixture admission failure, which is preserved separately in /tmp/ir1409-bq-fixture-expanded-id.log and is not regression evidence. FND-004 remains open pending the narrow guard and actual green BQ/compound controls.

Read-only compound fixture/API grounding was sent to the same coder and recorded in /tmp/ir1409-compound-grounding.md; it distinguishes direct CompoundUnit UnsupportedExactType from source-unit-row MismatchedIdentityWitness and preserves the supported declared-unit record-field path.

## FND-004 Disposition Round 5

Inspected the complete actual source/test diff through 7692aae2 and the finished serialized focused receipts. The three same-variant unsupported numeric leaf cases (BoundedQuantity, CarriedQuantity and Quantity with Compound UnitId) now return the existing typed UnsupportedExactType through source_collection_error before derived equality. The diagnostic still follows exact_diagnostic's native storage precharge. Supported Quantity(Declared) is untouched. No new BQ/compound leaf projection, numeric equality implementation, cache warming, fresh budget or compatibility path is introduced.

The BQ fixture strictly admits genuine bounded_quantity and nested map graph nodes, builds two independently owned equal cold payloads and checks their distinct Arcs before measurement. Actual RED at /tmp/ir1409-bq-red.log is342 versus330 allocations; actual GREEN is /tmp/ir1409-bq-green.log and the final full map suite. The compound control reuses the existing strictly admitted Metre^2 package helper, changes its canonical exponent coherently, and adds real nested maps before strict admission. Its independently owned compound exponent buffers produce actual RED356 versus350 in /tmp/ir1409-compound-red.log, then pass in /tmp/ir1409-fnd004-map-green.log. Earlier expanded-ID fixture failure is separately retained, not counted as regression proof.

Both controls compare real complete typed refusals, cold-versus-independent equal native-prefix allocation counts/bytes, consumed native charges, original limits, fixed expected GraphNode context and absent success delta. Zero/late resource controls construct row payloads before measurement. Existing strict zero-storage and paid-prefix row/cancel-handle controls remain unchanged and passing. The new compound test also asserts the separate MismatchedIdentityWitness at the genuine source-unit node when a compound identity is supplied as a declared unit. Existing supported declared-unit record-field projection passes1/1 through its actual admitted_source_units path.

Only existing test helpers parameters_package/read receive pub(crate) visibility for reuse; no production API or checker is duplicated. No source edits, Cargo, new worktree or PR were made by this reviewer.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | Actual unsupported numeric leaf guards precede equality, both genuine red controls are green, and declared-unit behavior remains passing. |

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

## New findings (final aggregate gate)

The lead's still-running combined gate at custody head 960b35e2fe01f410f838ad1805a80cfe79181ee6 executed the required string-edge lane and reported one introduced occurrence. This reopens the prior in-scope PASS; the gate is still collecting its other command results and no overall completion is inferred here.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | The new Package-to-GraphNode state transition branches on the diagnostic locus.form string, violating the required string-edge gate and coupling native provenance to a display label. Replace the stage decision with typed internal context while preserving the paid lookup, caller/row provenance and first-denial snapshot; do not allowlist or suppress it. | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:3717-3720 |

### FND-005 Evidence

Actual `/tmp/IR1409-combined-final-960b35e2f.log` records the required qsl-string-edge invocation with the repository's empty allow_list: `string-edge: 1 unmarked, unlisted occurrence(s)` at `exact_projection.rs:3719` in `CheckedPackageV2::project_exact_type_inner (branch-gating)`, command_exit=1. The offending `locus.form == "exact_projection_node"` condition is in this PR's new source, not an unchanged baseline. It decides whether the reader replaces Package context with an admitted GraphNode after lookup. The display-form label must not act as an internal state discriminator; the replacement needs a typed mode/context rather than an exception entry.

No reviewer source changes or Cargo execution occurred. The candidate remains frozen until its active gate finishes. Future disposition requires actual string-edge success and the focused independent native key/value/fresh projection/caller-context controls, retaining the budget-paid lookup and sticky first-denial properties of FND-003. Earlier cold BQ/compound regression fixes and green receipts remain valid historical evidence.

## Current Gate-Reopened Verdict

HOLD: FND-005 open. Prior findings FND-002, FND-003 and FND-004 remain fixed. No aggregate PASS or final review clearance is claimed until the typed fix and its actual checks land.

## New findings (completed candidate gate integration)

The candidate gate at 960b35e2fe01f410f838ad1805a80cfe79181ee6 has now completed with aggregate exit1. In addition to FND-005 and its matching string-edge runtime test failure, the required conformance compile command found this separate introduced integration defect. The lead confirms no source mutation occurred during the candidate gate;1312 other root integration cases passed. The failure is not an unchanged baseline.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | high | The shared map test module imports two root-IT sibling fixture modules that are absent in the required conformance_qspec crate, so its new compound refusal control makes that existing target fail to compile with E0432/E0433. Share the genuine fixture/harness context across both targets without copying fixtures, excluding the control, weakening checks or suppressing errors. | tests/it/checked_package_v2_map_type.rs:1341; tests/it/checked_package_v2_map_type.rs:1498; tests/conformance_qspec/main.rs |

### FND-006 Evidence

Actual /tmp/IR1409-combined-final-960b35e2f.log1586-1603 records `cargo test --locked --workspace --features conformance-cli --no-run --test conformance_qspec --test monitor_plan_public --test temporal_binding_public --test it` failing exit101. The already reused map test module now includes `use crate::checked_package_v2_parameters;` in its genuine compound test and calls `crate::checked_package_v2_owners::complete_witness_fixture()`, but those modules were not declared by the conformance_qspec crate at the frozen gate head. Both new references compile in the root it harness, which is why its focused controls passed. This is a missing shared target context, not an invalid cold-unit reproduction and not a reason to skip the new tests. The fix must genuinely share existing fixture providers across the targets or place the shared builder at their common support home, without copied semantic fixture definitions.

No reviewer Cargo or source changes occurred. The lead released the same coder only after the failed candidate gate finished. Disposition requires actual compile receipts for both it and conformance_qspec, plus the pending typed-string-edge/locus fix checks.

## Current Completed-Gate Verdict

HOLD: FND-005 and FND-006 open. Prior FND-002/003/004 fixed evidence remains intact. The candidate aggregate exit1 is preserved; no baseline exemption or gate PASS is claimed.

## Candidate Fix Disposition Round 6

Inspected all changes after failed candidate960b35e2 through c15367a0. ProjectionLocusMode is a private typed InitializeRequestedNode/PreserveCaller discriminator threaded through all three inner projection calls. The node lookup is still paid first; only a successful fresh requested-node lookup promotes Package to authentic GraphNode. A caller-owned Package whose display form happens to equal the old marker now remains caller-owned, and nested/caller provenance plus sticky first-denial context remain. Actual /tmp/ir1409-locus-mode-red.log proves the old source overwrote this caller Package context; /tmp/ir1409-typed-locus.log now passes the full eight projection-module controls. Authoritative /tmp/ir1409-typed-string-edge.log reports no unmarked/unlisted branch, without allowlisting or suppression.

The conformance target now registers the same genuine parameters, owner, model-fields and native-budget test modules, with explicit relative paths for operation_census and option_types under the reused model-fields module. Existing fixtures are referenced rather than copied; no test or gate was disabled. Actual /tmp/ir1409-shared-conformance-compile.log successfully compiles both conformance_qspec and root it plus both public monitor/temporal targets. Earlier first-fix conformance101 and fmt1 receipts remain in /tmp/ir1409-typed-focuseds.log, and corrected exits0 in /tmp/ir1409-shared-focuseds.log are the latest results, not a rewrite of that failure. Full map15/15 and required ordinary strict workspace Clippy also pass.

| FND | outcome | reason |
| --- | --- | --- |
| FND-005 | fixed | Actual typed mode replaces diagnostic-string control; authoritative string-edge and eight independent projection context controls pass. |
| FND-006 | fixed | Actual required no-run command compiles the reused controls in both harnesses with genuine shared modules and explicit child paths; no copies or skips. |

## New findings (shared harness warning scope)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | Importing the full owner fixture module into conformance_qspec introduces two dead_code warnings for CompleteWitnessFixture.budget/tuples and complete_witness_long_source_fixture. Remove the newly unreachable harness surface through genuine shared fixture organization or meaningful existing consumers, without suppression, copied fixtures or dummy reads. | tests/conformance_qspec/main.rs:63-64; tests/it/checked_package_v2_owners.rs:2410-2419 |

### FND-007 Evidence

/tmp/ir1409-shared-conformance-compile.log now compiles successfully but emits exactly two new conformance_qspec warnings for the unused fields and helper. The same diagnostics are absent from the failed960b35e2 candidate before the owner module was registered; the separate monitor_plan_public unused-import warning is unchanged baseline and not this finding. This non-default test=false target is compiled by the explicit conformance lane, while the ordinary all-targets Clippy PASS does not cover these new warnings. The language-independent code-review Integrity rule requires introduced warnings to be resolved, not hidden. This is a low cleanup on the actual integration fix, not a request for a new audit, test target, gate or PR.

## Current Round 6 Verdict

HOLD on new LOW FND-007 only. FND-002 through FND-006 latest outcomes are fixed; final review cannot claim zero open findings yet. No reviewer source/Cargo changes.

## FND-007 Disposition Round 7

Inspected the only final source delta: four lines register the existing complete_identity_witness module beside the same genuine producer fixture in conformance_qspec. There are no new fixture bodies, dummy reads, suppressed warnings, skipped tests or production changes. Existing controls meaningfully consume the recorded budget and tuple rows, prove cumulative acquisition/rebind arithmetic, and exercise long-source-owner work charging.

Actual /tmp/ir1409-fnd007-compile.log compiles both conformance_qspec and root it plus public monitor/temporal targets without either introduced dead_code warning. Its sole remaining warning is the unchanged monitor_plan_public protocol_ports import, explicitly outside FND-007. Actual /tmp/ir1409-fnd007-witness.log executes all9 existing complete-witness controls in conformance_qspec and passes. /tmp/ir1409-fnd007-map.log executes all15 map controls there and passes. Typed context ownership control passes1/1 in /tmp/ir1409-fnd007-locus.log; strict workspace Clippy/fmt pass. The summaries record compile/witness/clippy/fmt/map/locus exits0.

| FND | outcome | reason |
| --- | --- | --- |
| FND-007 | fixed | Existing genuine witness consumers are registered beside their shared fixture; actual both-target compile has no new warnings and the nine meaningful consumers pass. |

All prior FND-002 through FND-006 fixes remain intact. The failed960b35e2 aggregate, string-edge branch regression, missing shared target context and earlier cold-payload red failures remain preserved above; this is not a rewrite of the failed gate.

## Corrected Candidate Final Review

Reviewed clean pushed head 37824fbe44fb6bf8a31f608e078cfd16017cb445 after all introduced gate and integration fixes. No new findings. Latest outcomes for FND-002, FND-003, FND-004, FND-005, FND-006 and FND-007 are fixed. This supersedes prior HOLD verdicts while preserving the failed960b35e2 aggregate and every earlier finding, red receipt and disposition.

Typed private mode—not diagnostic text—now decides fresh versus caller-owned projection context at all inner call sites. Actual authoritative scanner passes; old-code regression proves caller Package context was previously overwritten when its form matched the internal label. Current projection-module8/8 and final typed ownership1/1 pass, preserving paid lookup before authentic GraphNode, nested/caller provenance, context restoration and sticky first denial.

The genuine shared harness modules and explicit model-field child paths compile under both it and conformance_qspec. The final four-line registration reuses existing complete_identity_witness controls rather than copying fixtures or hiding warnings. Its actual nine controls exercise budget/tuple acquisition and the long-source-owner bound; both-target compile now has no introduced warning. The unchanged monitor_plan_public protocol_ports warning remains explicitly baseline. Actual /tmp/ir1409-fnd007-focuseds.log and /tmp/ir1409-fnd007-controls.log report all compile/witness/map/locus/clippy/fmt exits0. Logs show conformance witness9/9, conformance map15/15 and typed ownership1/1. Earlier same-source root map15/15, projection8/8, cold footprint, unary and declared-unit controls remain valid.

Cold bounded-quantity and compound exponent guards still precede derived equality, preserve UnsupportedExactType/MismatchedIdentityWitness distinctions and the supported declared-unit path, and retain both genuine red-to-green allocation controls. No caller-cache warming, invented numeric comparator, owner shim, duplicate fixture definition, suppression, disabled test or gate narrowing was introduced.

The actual computed matrix at this corrected head records six live AC-19 binders and twelve live AC-20 binders, including the typed ownership regression. All eighteen bindings were checked against actual assertions. New attributes retain TC-first plus one AC. Existing receipt/tick criteria and unrelated bounded-wrapper PR951 scope are not overwritten or invented as blockers for these map criteria.

### Corrected Final Scope

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
    },
    {
      "id": "tests/conformance_qspec/main.rs",
      "path": "tests/conformance_qspec/main.rs",
      "role": "examined",
      "excerpt": "#[path = \"../it/complete_identity_witness.rs\"]\nmod complete_identity_witness;"
    },
    {
      "id": "crates/quire-contract-model/src/checked_package/v2/exact_projection.rs",
      "path": "crates/quire-contract-model/src/checked_package/v2/exact_projection.rs",
      "role": "examined",
      "excerpt": "enum ProjectionLocusMode {\n    InitializeRequestedNode,\n    PreserveCaller,\n}"
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
      "test_id": "tests::projection_locus_ownership_is_independent_of_diagnostic_form",
      "ac_id": "FR-357-AC-20",
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

### Corrected Final Verdict

PASS in reviewed PR scope at 37824fbe44fb6bf8a31f608e078cfd16017cb445. Zero open findings. The lead's one corrected-head combined gate remains pending; this review does not claim that aggregate gate or merge has passed. No reviewer Cargo, source edit, new worktree or PR was performed.
