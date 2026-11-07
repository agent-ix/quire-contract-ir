---
id: SR-2026110705000824
title: "integrity review of reached map row projection"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-1409; spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md; spec/checked_package/matrix/TC-473-reached-map-row-projection.md"
review_set: subset
---

## Summary

The API, projection and budget statements form one consistent obligation with two extracted new criteria and TC-473 verification. All twenty FR-357 criteria parse exactly once; retained AC-16, AC-17 and AC-18 are unchanged.

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

SPEC PASS. No duplicate identifier or internal contract conflict is present in this frozen head. Concurrent PR 951 may touch this requirement and projection region; merge integration must preserve distinct existing receipt/tick and new map criteria, but unmerged external work is not a defect in this spec. Implementation and its review/final gate remain pending.
