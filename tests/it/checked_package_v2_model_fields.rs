// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-628: `CheckedPackageV2::model_object_fields` over packages the reader
//! admits, each built here node by node with structural identities and a real
//! Semantic IR 2.0.0 domain package document selected by the lock.

use crate::support::checked_package::{
    canonical, evidence_for, evidence_with_document, index_fixture_node_ids,
    node_id as fixture_node_id, nominal_package, rebuild_source_map, select_model, settle,
    structural_fixture_id, structural_id,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedCollectionKind, CheckedMemberType, CheckedModelFieldsError, CheckedModelObjectFields,
    CheckedModelPopulationCatalogError, CheckedModelPopulationCatalogLimitKind,
    CheckedModelPopulationCatalogLimits, CheckedModelPopulationMemberUnresolved,
    CheckedModelRecordFieldsError, CheckedNodeId, CheckedNodePathSegment, CheckedOccurrence,
    CheckedOccurrenceRole, CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, ExactTypeProjectionErrorCode, SelectedEqualityApplicationRow,
    SelectedExactDeclarationForm, SelectedExactDeclarationRow, SelectedExactFieldRow,
    SelectedExactIdentities, SelectedExactIdentity, SelectedFactEqualityId, SelectedFactFieldId,
    SelectedFactIds, SelectedFactRows, SelectedFactsErrorCode, SelectedSourceCollectionRow,
    SelectedTupleDeclarationRow,
};
use quire_exact::{
    record, CardinalityBound, CollectionKind, CollectionType, ConstructionCause, EffectiveId,
    FieldValue, Integer, MemberId, MemberOwner, NodeKey, OriginalDeclarationKey, PathSegment,
    Presence, UnitId, Value as ExactValue, ValueType,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const IDENTITY: &str = "acme/orders";
const EMPTY_MODEL: &str = "aaa/other";
const OBJECT: &str = "object_type";
const INTERFACE: &str = "systems_interface";

/// Trace: FR-038-AC-474, FR-038-AC-475, FR-038-AC-476, FR-038-AC-477.
#[trace("FR-038-AC-474", "FR-038-AC-475", "FR-038-AC-476", "FR-038-AC-477")]
#[test]
fn selected_population_catalog_keeps_unused_origins_and_full_effective_type_ids() {
    let mut selected = document(vec![
        object("Gadget", &[], &[]),
        object("Widget", &[], &[]),
        json!({"identity": orders("WidgetAlias"), "kind": "alias", "target": orders("Widget")}),
    ]);
    selected["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": IDENTITY, "name": "population"},
            "construct": {"meaning": "quire.meaning.model.population/v1"},
        }));
    selected["populations"] = json!([
        {"identity": orders("Active"), "displayName": "Active",
         "kind": {"module": IDENTITY, "name": "population"},
         "members": [orders("WidgetAlias"), orders("Gadget")], "extent": "closed",
         "origin": {"source": {"sourceIdentity": "ix://acme/orders/spec",
             "path": "spec/populations.md", "startLine": 1, "startColumn": 1}}},
        {"identity": orders("Reserve"), "displayName": "Reserve",
         "kind": {"module": IDENTITY, "name": "population"},
         "members": [orders("Widget")], "extent": "closed",
         "origin": {"source": {"sourceIdentity": "ix://acme/orders/spec",
             "path": "spec/populations.md", "startLine": 2, "startColumn": 1}}},
    ]);
    let (mut wire, _, _) = self::package(&selected, &[("Gadget", OBJECT), ("Widget", OBJECT)], &[]);
    select_model(&mut wire, EMPTY_MODEL);
    settle(&mut wire);
    // The second graph root sorts first by selected identity. Compare the
    // catalog with the indexed wire's actual node IDs, not just package names.
    let mut indexed = wire.clone();
    index_fixture_node_ids(&mut indexed);
    let graph_roots = indexed["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .filter(|node| node["semantic_form"] == "model_root")
        .map(|node| {
            (
                node["owner"]["identity"]
                    .as_str()
                    .expect("root identity")
                    .to_owned(),
                serde_json::from_value::<CheckedNodeId>(node["node_id"].clone())
                    .expect("indexed root ID"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(graph_roots[0].0, IDENTITY);
    assert_eq!(graph_roots[1].0, EMPTY_MODEL);
    let evidence = evidence_with_document(&wire, IDENTITY, &selected);
    let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("expected admitted selected populations, got {other:?}"),
    };
    let generous = CheckedModelPopulationCatalogLimits {
        work: 1_000_000,
        entries: 100,
    };
    let catalog = admitted
        .selected_model_population_catalog(generous)
        .expect("complete selected catalog");
    assert_eq!(catalog.roots().len(), 2);
    assert_eq!(catalog.roots()[0].model_identity(), EMPTY_MODEL);
    assert_eq!(catalog.roots()[0].root(), &graph_roots[1].1);
    assert!(catalog.roots()[0].populations().is_empty());
    assert_eq!(catalog.roots()[1].model_identity(), IDENTITY);
    assert_eq!(catalog.roots()[1].root(), &graph_roots[0].1);
    let populations = catalog.roots()[1].populations();
    assert_eq!(populations.len(), 2);
    assert_eq!(populations[0].original_identity(), orders("Active"));
    assert_eq!(populations[1].original_identity(), orders("Reserve"));
    assert_eq!(populations[0].members().len(), 2);
    assert_eq!(
        populations[0].members()[0].original_identity(),
        orders("Gadget")
    );
    let gadget = EffectiveId::new(
        None,
        OriginalDeclarationKey::new(IDENTITY, orders("Gadget")),
    );
    assert_eq!(
        populations[0].members()[0].normalized_type_identity(),
        &gadget
    );
    let active = &populations[0].members()[1];
    assert_eq!(active.original_identity(), orders("WidgetAlias"));
    let expected = EffectiveId::new(
        None,
        OriginalDeclarationKey::new(IDENTITY, orders("Widget")),
    );
    assert_eq!(active.normalized_type_identity(), &expected);
    assert_eq!(
        populations[1].members()[0].normalized_type_identity(),
        &expected
    );
    assert_eq!(
        populations[1].members()[0].original_identity(),
        orders("Widget")
    );
    // Independently count 2 roots, 2 populations, 3 members, 4 alias lookups
    // and three owned FR-150 type identities (Gadget, Widget, Widget).
    let expected_work = 11
        + 3 * IDENTITY.len()
        + orders("Gadget").len()
        + 2 * orders("Widget").len()
        + 3 * std::mem::size_of::<OriginalDeclarationKey>();
    let expected_work = u64::try_from(expected_work).expect("fixture work fits u64");
    assert_eq!(catalog.work(), expected_work);
    assert_eq!(catalog.entries(), 7);
    let exact = CheckedModelPopulationCatalogLimits {
        work: expected_work,
        entries: 7,
    };
    assert_eq!(
        admitted.selected_model_population_catalog(exact),
        Ok(catalog.clone())
    );
    let cloned = admitted.clone();
    assert_eq!(
        cloned.selected_model_population_catalog(exact),
        Ok(catalog.clone())
    );
    let one_less_work = CheckedModelPopulationCatalogLimits {
        work: exact.work - 1,
        ..exact
    };
    assert!(matches!(
        admitted.selected_model_population_catalog(one_less_work),
        Err(CheckedModelPopulationCatalogError::Limit {
            kind: CheckedModelPopulationCatalogLimitKind::Work,
            limit,
            consumed,
            locus,
        }) if limit == exact.work - 1 && consumed == exact.work
            && locus.model_identity == IDENTITY
            && locus.root == &graph_roots[0].1
            && locus.population == Some(orders("Reserve").as_str())
            && locus.member == Some(orders("Widget").as_str())
    ));
    let one_less_entry = CheckedModelPopulationCatalogLimits {
        entries: exact.entries - 1,
        ..exact
    };
    assert!(matches!(
        admitted.selected_model_population_catalog(one_less_entry),
        Err(CheckedModelPopulationCatalogError::Limit {
            kind: CheckedModelPopulationCatalogLimitKind::Entries,
            limit,
            consumed,
            locus,
        }) if limit == exact.entries - 1 && consumed == exact.entries
            && locus.model_identity == IDENTITY
            && locus.root == &graph_roots[0].1
            && locus.population == Some(orders("Reserve").as_str())
            && locus.member == Some(orders("Widget").as_str())
    ));
}

/// Trace: FR-038-AC-475.
#[trace("FR-038-AC-475")]
#[test]
fn selected_population_catalog_refuses_wrong_kind_without_partial_result() {
    let mut selected = document(vec![value_type("Thousand", "integer", Some(("0", "1000")))]);
    selected["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": IDENTITY, "name": "population"},
            "construct": {"meaning": "quire.meaning.model.population/v1"},
        }));
    selected["populations"] = json!([{
        "identity": orders("Numbers"), "displayName": "Numbers",
        "kind": {"module": IDENTITY, "name": "population"},
        "members": [orders("Thousand")], "extent": "closed",
        "origin": {"source": {"sourceIdentity": "ix://acme/orders/spec",
            "path": "spec/populations.md", "startLine": 1, "startColumn": 1}},
    }]);
    let admitted = admit(&selected, &[], &[]);
    let limits = CheckedModelPopulationCatalogLimits {
        work: 1_000_000,
        entries: 100,
    };
    assert!(matches!(
        admitted.selected_model_population_catalog(limits),
        Err(CheckedModelPopulationCatalogError::UnresolvedMember {
            locus,
            reason: CheckedModelPopulationMemberUnresolved::WrongKind,
        }) if locus.model_identity == IDENTITY
            && locus.root.package.as_ref() == IDENTITY
            && locus.population == Some(orders("Numbers").as_str())
            && locus.member == Some(orders("Thousand").as_str())
    ));
}

/// Trace: FR-038-AC-475.
#[trace("FR-038-AC-475")]
#[test]
fn selected_population_catalog_classifies_retained_namespace_as_wrong_kind() {
    let mut namespace = typed_object("Area", "namespace", &[], &[]);
    namespace
        .as_object_mut()
        .expect("namespace")
        .remove("fields");
    namespace
        .as_object_mut()
        .expect("namespace")
        .remove("operations");
    namespace["members"] = json!([]);
    namespace["vocabulary"] = json!([]);
    let mut selected = document(vec![namespace]);
    selected["constructs"]
        .as_array_mut()
        .expect("constructs")
        .extend([
            json!({"kind": {"module": IDENTITY, "name": "namespace"},
                "construct": {"meaning": "quire.meaning.model.namespace/v1"}}),
            json!({"kind": {"module": IDENTITY, "name": "population"},
                "construct": {"meaning": "quire.meaning.model.population/v1"}}),
        ]);
    selected["populations"] = json!([{
        "identity": orders("Areas"), "displayName": "Areas",
        "kind": {"module": IDENTITY, "name": "population"},
        "members": [orders("Area")], "extent": "closed",
        "origin": {"source": {"sourceIdentity": "ix://acme/orders/spec",
            "path": "spec/populations.md", "startLine": 1, "startColumn": 1}},
    }]);
    let admitted = admit(&selected, &[], &[]);
    assert!(matches!(
        admitted.selected_model_population_catalog(CheckedModelPopulationCatalogLimits {
            work: 1_000_000,
            entries: 100,
        }),
        Err(CheckedModelPopulationCatalogError::UnresolvedMember {
            locus,
            reason: CheckedModelPopulationMemberUnresolved::WrongKind,
        }) if locus.model_identity == IDENTITY
            && locus.population == Some(orders("Areas").as_str())
            && locus.member == Some(orders("Area").as_str())
    ));
}

/// Trace: FR-038-AC-467, FR-038-AC-468.
#[trace("FR-038-AC-467", "FR-038-AC-468", "FR-358-AC-2")]
#[test]
fn selected_record_fields_expose_presence_and_shape_without_exact_member_projection() {
    let mut selected = document(vec![
        json!({"identity": orders("OptInt"), "kind": "option", "inner": "ix://quire/native/Integer"}),
        typed_object(
            "Receipt",
            "record",
            &[],
            &[
                field("link", &orders("Other")),
                field("note", "ix://quire/native/String").optional(),
                field("maybe_count", &orders("OptInt")),
                field("nested", &orders("OptInt")).many((0, Some(3), true, false)),
                field("text", "ix://quire/native/Text"),
            ],
        ),
        object("Other", &[], &[]),
    ]);
    selected["constructs"].as_array_mut().unwrap().push(json!({
        "kind": {"module": IDENTITY, "name": "record"},
        "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
    }));
    let admitted = admit(&selected, &[("Receipt", "record_value_type")], &[]);
    let receipt: CheckedNodeId =
        serde_json::from_value(node_id(&declaration_key("Receipt", "record_value_type")))
            .expect("record node id");
    let fields = admitted
        .model_record_fields(&receipt)
        .expect("selected record fields");
    assert_eq!(
        fields
            .fields()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        ["link", "note", "maybe_count", "nested", "text"]
    );
    let link = fields.field("link").expect("link field");
    assert_eq!(link.identity(), format!("{}/link", orders("Receipt")));
    assert!(!link.optional());
    assert_eq!(
        link.member_type(),
        Some(&CheckedMemberType::Reference(declaration_id("Other")))
    );
    let note = fields.field("note").expect("optional note");
    assert!(note.optional());
    assert_eq!(
        note.member_type(),
        Some(&CheckedMemberType::Option(Box::new(
            CheckedMemberType::String
        )))
    );
    let maybe_count = fields.field("maybe_count").expect("required option");
    assert!(!maybe_count.optional());
    assert_eq!(
        maybe_count.member_type(),
        Some(&CheckedMemberType::Option(Box::new(
            CheckedMemberType::Integer
        )))
    );
    let nested = fields.field("nested").expect("nested collection");
    assert!(!nested.optional());
    assert_eq!(
        nested.member_type(),
        Some(&CheckedMemberType::Collection {
            kind: CheckedCollectionKind::Sequence,
            element: Box::new(CheckedMemberType::Option(Box::new(
                CheckedMemberType::Integer
            ))),
            bounds: Some((0, 3)),
        })
    );
    assert_eq!(fields.field("absent"), None);
    assert_eq!(
        fields
            .field("text")
            .expect("unmapped text field")
            .member_type(),
        None
    );
    assert_eq!(admitted.clone().model_record_fields(&receipt), Ok(fields));

    let key = NodeKey::new("checked", vec![PathSegment::Text("Receipt".into())], 0).unwrap();
    let names = ["link", "note", "maybe_count", "nested", "text"];
    let declaration = SelectedExactDeclarationRow::new(
        receipt.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::RecordValueType,
        orders("Receipt"),
        SelectedExactIdentity::Record(key.clone()),
    );
    let field_rows = names.map(|name| {
        SelectedExactFieldRow::new(
            receipt.clone(),
            IDENTITY,
            orders("Receipt"),
            format!("{}/{name}", orders("Receipt")),
            MemberId::new(MemberOwner::Node(key.clone()), name).unwrap(),
        )
    });
    let (budgeted, _) =
        crate::native_join_budget::assert_exact_join(&receipt, &receipt, |budget| {
            SelectedExactIdentities::from_stage_rows(
                &admitted,
                [declaration.clone()],
                field_rows.clone(),
                budget,
            )
        });
    let identities = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [declaration],
        field_rows,
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("record identity rows");
    assert_eq!(identities, budgeted);
    let refusal = admitted
        .project_exact_type(&receipt, &identities, 100)
        .expect_err("unselected object member cannot complete Exact projection");
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration
    );
}

/// Trace: FR-038-AC-469.
#[trace("FR-038-AC-469")]
#[test]
fn selected_record_fields_distinguish_missing_wrong_kind_and_ambiguous_declarations() {
    let mut selected = document(vec![
        typed_object(
            "Receipt",
            "record",
            &[],
            &[field("id", "ix://quire/native/Integer")],
        ),
        object("Other", &[], &[]),
        typed_object(
            "Base",
            "record",
            &[],
            &[field("a", "ix://quire/native/Integer")],
        ),
        typed_object(
            "Ambiguous",
            "record",
            &["Base"],
            &[field("a", "ix://quire/native/Integer")],
        ),
    ]);
    selected["constructs"].as_array_mut().unwrap().push(json!({
        "kind": {"module": IDENTITY, "name": "record"},
        "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
    }));
    let admitted = admit(
        &selected,
        &[
            ("Receipt", "record_value_type"),
            ("Other", OBJECT),
            ("Ambiguous", "record_value_type"),
        ],
        &[],
    );
    let record_id = |short: &str| -> CheckedNodeId {
        serde_json::from_value(node_id(&declaration_key(short, "record_value_type")))
            .expect("record node id")
    };
    assert_eq!(
        admitted.model_record_fields(&record_id("Missing")),
        Err(CheckedModelRecordFieldsError::UnknownNode)
    );
    assert_eq!(
        admitted.model_record_fields(&declaration_id("Other")),
        Err(CheckedModelRecordFieldsError::NotModelRecordType)
    );
    let expected = Err(CheckedModelRecordFieldsError::AmbiguousField("a".into()));
    assert_eq!(
        admitted.model_record_fields(&record_id("Ambiguous")),
        expected
    );
}

#[trace("FR-352-AC-12", "FR-352-AC-13", "FR-352-AC-15")]
#[test]
fn selected_record_projects_nested_collection_and_option_members_with_bounded_work() {
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(|| {
            let mut selected = document(vec![
                json!({"identity": orders("OptInt"), "kind": "option", "inner": "ix://quire/native/Integer"}),
                typed_object("Wrapped", "record", &[], &[
                    field("sequence", "ix://quire/native/Integer").many((0, None, true, false)),
                    field("set", "ix://quire/native/Integer").many((0, None, false, true)),
                    field("bag", "ix://quire/native/Integer").many((0, None, false, false)),
                    field("ordered", "ix://quire/native/Integer").many((0, Some(3), true, true)),
                    field("nested", &orders("OptInt")).many((0, Some(3), true, false)),
                    field("option", &orders("OptInt")),
                    field("optional_collection", "ix://quire/native/Integer").optional().many((0, Some(3), false, true)),
                    field("optional_string", "ix://quire/native/String").optional(),
                ]),
            ]);
            selected["constructs"].as_array_mut().unwrap().push(json!({
                "kind": {"module": IDENTITY, "name": "record"},
                "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
            }));
            let mut item = "ix://quire/native/Integer".to_owned();
            for depth in 0..64 {
                let identity = orders(&format!("Deep{depth}"));
                selected["types"].as_array_mut().unwrap().push(if depth % 2 == 0 {
                    json!({"identity": identity, "kind": "sequence", "items": item})
                } else {
                    json!({"identity": identity, "kind": "option", "inner": item})
                });
                item = identity;
            }
            selected["types"][1]["fields"].as_array_mut().unwrap().push(json!({
                "identity": format!("{}/deep", orders("Wrapped")), "name": "deep",
                "typeRef": item, "presence": "required", "nullable": false,
                "defaultKind": "none", "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
            }));
            let admitted = admit(&selected, &[("Wrapped", "record_value_type")], &[]);
            let structural: CheckedNodeId = serde_json::from_value(node_id(&declaration_key("Wrapped", "record_value_type"))).unwrap();
            let key = NodeKey::new("checked", vec![PathSegment::Text("Wrapped".into())], 0).unwrap();
            let names = ["sequence", "set", "bag", "ordered", "nested", "option", "optional_collection", "optional_string", "deep"];
            let declaration = SelectedExactDeclarationRow::new(
                structural.clone(), IDENTITY, SelectedExactDeclarationForm::RecordValueType,
                orders("Wrapped"), SelectedExactIdentity::Record(key.clone()),
            );
            let fields = names.map(|name| SelectedExactFieldRow::new(
                structural.clone(), IDENTITY, orders("Wrapped"),
                format!("{}/{name}", orders("Wrapped")),
                MemberId::new(MemberOwner::Node(key.clone()), name).unwrap(),
            ));
            let identities = SelectedExactIdentities::from_stage_rows(&admitted, [declaration], fields, &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)).unwrap();
            let clone = admitted.clone();
            let projection = admitted.project_exact_type(&structural, &identities, 200).unwrap();
            let work = projection.work();
            assert_eq!(admitted.project_exact_type(&structural, &identities, work), Ok(projection.clone()));
            assert_eq!(clone.project_exact_type(&structural, &identities, work), Ok(projection.clone()));
            let view = projection.record_view().unwrap();
            assert_eq!(view.record_key(), &key);
            let bounded = Some(CardinalityBound::new(0, 3).unwrap());
            let collection = |kind, element, bound| ValueType::collection(CollectionType::new(kind, element, bound));
            let mut expected = vec![
                collection(CollectionKind::Sequence, ValueType::Integer, None),
                collection(CollectionKind::Set, ValueType::Integer, None),
                collection(CollectionKind::Bag, ValueType::Integer, None),
                collection(CollectionKind::OrderedSet, ValueType::Integer, bounded),
                collection(CollectionKind::Sequence, ValueType::option(ValueType::Integer), bounded),
                ValueType::option(ValueType::Integer),
                collection(CollectionKind::Set, ValueType::Integer, bounded),
                ValueType::String,
            ];
            let mut deep = ValueType::Integer;
            for depth in 0..64 {
                deep = if depth % 2 == 0 {
                    collection(CollectionKind::Sequence, deep, None)
                } else {
                    ValueType::option(deep)
                };
            }
            expected.push(deep);
            assert_eq!(view.fields().len(), names.len());
            for (index, field) in view.fields().iter().enumerate() {
                let name = names[index];
                assert_eq!(field.ir_identity(), format!("{}/{name}", orders("Wrapped")));
                assert_eq!(field.name(), name);
                assert_eq!(field.member_id(), &MemberId::new(MemberOwner::Node(key.clone()), name).unwrap());
                assert_eq!(field.value_type(), &expected[index]);
                assert_eq!(field.presence(), if (6..8).contains(&index) { Presence::Optional } else { Presence::Required });
            }
            let exhausted = admitted.project_exact_type(&structural, &identities, work - 1).unwrap_err();
            assert_eq!(clone.project_exact_type(&structural, &identities, work - 1), Err(exhausted.clone()));
            assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);
            assert_eq!(exhausted.requested.as_ref(), &structural);
            assert_eq!(exhausted.at.as_deref(), Some(&structural));
            assert_eq!(exhausted.form.as_ref(), "record_field_collection");
            assert_eq!(exhausted.work, work - 1);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[trace("FR-352-AC-14", "FR-358-AC-5")]
#[test]
fn nested_collection_of_nominal_record_keeps_target_identity_and_refuses_missing_witness() {
    let mut selected = document(vec![
        typed_object(
            "A",
            "record",
            &[],
            &[field("next", &orders("B")).many((0, None, true, false))],
        ),
        typed_object("B", "record", &[], &[field("next", &orders("A"))]),
        typed_object(
            "Bad",
            "record",
            &[],
            &[field("float", "ix://quire/native/Float32").many((0, None, false, true))],
        ),
    ]);
    selected["constructs"].as_array_mut().unwrap().push(json!({
        "kind": {"module": IDENTITY, "name": "record"},
        "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
    }));
    let admitted = admit(
        &selected,
        &[
            ("A", "record_value_type"),
            ("B", "record_value_type"),
            ("Bad", "record_value_type"),
        ],
        &[],
    );
    let structural = |name| -> CheckedNodeId {
        serde_json::from_value(node_id(&declaration_key(name, "record_value_type"))).unwrap()
    };
    let key =
        |name: &str| NodeKey::new("checked", vec![PathSegment::Text(name.into())], 0).unwrap();
    let rows = ["A", "B", "Bad"].map(|name| {
        SelectedExactDeclarationRow::new(
            structural(name),
            IDENTITY,
            SelectedExactDeclarationForm::RecordValueType,
            orders(name),
            SelectedExactIdentity::Record(key(name)),
        )
    });
    let fields = [("A", "next"), ("B", "next"), ("Bad", "float")].map(|(owner, name)| {
        SelectedExactFieldRow::new(
            structural(owner),
            IDENTITY,
            orders(owner),
            format!("{}/{name}", orders(owner)),
            MemberId::new(MemberOwner::Node(key(owner)), name).unwrap(),
        )
    });
    let witnesses = SelectedExactIdentities::from_stage_rows(
        &admitted,
        rows.clone(),
        fields.clone(),
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    let projection = admitted
        .project_exact_type(&structural("A"), &witnesses, 100)
        .unwrap();
    assert_eq!(
        projection.record_view().unwrap().fields()[0].value_type(),
        &ValueType::collection(CollectionType::new(
            CollectionKind::Sequence,
            ValueType::Composite(key("B")),
            None
        ))
    );
    let missing = SelectedExactIdentities::from_stage_rows(
        &admitted,
        rows.clone()
            .into_iter()
            .filter(|row| row.structural_node != structural("B")),
        fields
            .clone()
            .into_iter()
            .filter(|row| row.record_node != structural("B")),
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    let refused = admitted
        .project_exact_type(&structural("A"), &missing, 100)
        .unwrap_err();
    assert_eq!(
        refused.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    assert_eq!(refused.at.as_deref(), Some(&structural("B")));
    assert_eq!(refused.requested.as_ref(), &structural("A"));
    assert_eq!(refused.form.as_ref(), "model/record_value_type");

    let assert_stage_refusal = |declarations: Vec<SelectedExactDeclarationRow>,
                                expected_code,
                                expected_node: &CheckedNodeId| {
        let mut budget = quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX);
        let error = SelectedExactIdentities::from_stage_rows(
            &admitted,
            declarations,
            Vec::<SelectedExactFieldRow>::new(),
            &mut budget,
        )
        .expect_err("invalid target identity cannot produce projection witnesses");
        assert_eq!(error.code, expected_code);
        assert_eq!(error.requested.as_ref(), expected_node);
        assert_eq!(error.at.as_deref(), Some(expected_node));
        assert_eq!(error.form.as_ref(), "selected_identity");
        assert_eq!(error.work, budget.consumed().work);
        assert!(error.work > 0);
    };
    let mut foreign = rows[1].clone();
    foreign.structural_node.package = "foreign/package".into();
    let foreign_node = foreign.structural_node.clone();
    assert_stage_refusal(
        vec![rows[0].clone(), foreign],
        ExactTypeProjectionErrorCode::ForeignNode,
        &foreign_node,
    );
    assert_stage_refusal(
        vec![rows[0].clone(), rows[1].clone(), rows[1].clone()],
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
        &structural("B"),
    );
    let mut wrong_form = rows[1].clone();
    wrong_form.form = SelectedExactDeclarationForm::ObjectType;
    assert_stage_refusal(
        vec![rows[0].clone(), wrong_form],
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
        &structural("B"),
    );
    let unsupported = admitted
        .project_exact_type(&structural("Bad"), &witnesses, 100)
        .unwrap_err();
    assert_eq!(
        unsupported.code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
    assert_eq!(unsupported.requested.as_ref(), &structural("Bad"));
}

fn orders(short: &str) -> String {
    format!("ix://acme/orders/{short}")
}

fn node_id(key: &str) -> Value {
    if let Some(parts) = key.strip_prefix("carried_quantity:") {
        let (unit, carrier) = parts.split_once(';').expect("carried Rational operands");
        return structural_id(
            "test/families",
            vec![
                json!("scalar_type"),
                json!("carried_quantity"),
                node_id(unit),
                node_id(carrier),
                json!("rounding"),
                node_id("scalar_type/text"),
                json!("exact"),
            ],
            0,
        );
    }
    if let Some(model) = key.strip_prefix("model:") {
        let mut parts = model.splitn(3, ':');
        let tag = parts.next().expect("model tag");
        let form = parts.next().expect("model form");
        let declaration = parts.next().expect("model declaration");
        return structural_id(
            IDENTITY,
            vec![json!(tag), json!(form), json!(declaration)],
            0,
        );
    }
    if let Some(form) = key.strip_prefix("scalar_type/") {
        return structural_id("test/families", vec![json!("scalar_type"), json!(form)], 0);
    }
    for form in [
        "reference",
        "option",
        "set",
        "bag",
        "sequence",
        "ordered_set",
    ] {
        if let Some(operand) = key.strip_prefix(&format!("composite_type/{form}/")) {
            return structural_id(
                "test/families",
                vec![json!("composite_type"), json!(form), node_id(operand)],
                0,
            );
        }
    }
    if let Some(bounds) = key.strip_prefix("bounded_domain/integer_range/") {
        let (lower, upper) = bounds.split_once('/').expect("integer range bounds");
        return structural_id(
            "test/families",
            vec![
                json!("bounded_domain"),
                json!("integer_range"),
                json!("min"),
                node_id("scalar_type/integer"),
                json!(lower),
                json!("max"),
                json!(upper),
            ],
            0,
        );
    }
    if let Some(bounds) = key.strip_prefix("bounded_domain/rational_range/") {
        let parts = bounds.split('/').collect::<Vec<_>>();
        let [nmin, nmax, dmin, dmax] = parts.as_slice() else {
            panic!("rational range bounds");
        };
        return structural_id(
            "test/families",
            vec![
                json!("bounded_domain"),
                json!("rational_range"),
                json!("numerator_min"),
                node_id("scalar_type/integer"),
                json!(nmin),
                json!("numerator_max"),
                json!(nmax),
                json!("denominator_min"),
                json!(dmin),
                json!("denominator_max"),
                json!(dmax),
            ],
            0,
        );
    }
    if let Some(bounds) = key.strip_prefix("bounded_domain/text_bounds/") {
        let (lower, rest) = bounds.split_once('/').expect("text lower");
        let (upper, profile) = rest.split_once('/').expect("text upper and profile");
        return structural_id(
            "test/families",
            vec![
                json!("bounded_domain"),
                json!("text_bounds"),
                json!("min"),
                node_id("scalar_type/integer"),
                json!(lower),
                json!("max"),
                json!(upper),
                json!("text_profile"),
                node_id("scalar_type/text"),
                json!(profile),
            ],
            0,
        );
    }
    if let Some(bounds) = key.strip_prefix("bounded_domain/collection_bounds/") {
        let (collection_and_lower, upper) = bounds.rsplit_once('/').expect("collection upper");
        let (collection, lower) = collection_and_lower
            .rsplit_once('/')
            .expect("collection lower");
        return structural_id(
            "test/families",
            vec![
                json!("bounded_domain"),
                json!("collection_bounds"),
                json!("min"),
                node_id("scalar_type/integer"),
                json!(lower),
                json!("max"),
                json!(upper),
                node_id(collection),
            ],
            0,
        );
    }
    fixture_node_id(key)
}

fn path_label(id: &CheckedNodeId) -> String {
    match id.path.as_slice() {
        [CheckedNodePathSegment::Name(name)] => name.to_string(),
        [CheckedNodePathSegment::Name(tag), CheckedNodePathSegment::Name(form), CheckedNodePathSegment::Name(declaration)]
            if id.package.as_ref() == IDENTITY =>
        {
            format!("model:{tag}:{form}:{declaration}")
        }
        [CheckedNodePathSegment::Name(first), CheckedNodePathSegment::Name(second)] => {
            format!("qualified:{first}/{second}")
        }
        other => panic!("unexpected fixture identity path {other:?}"),
    }
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn wire_node(
    key: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(semantic_type),
        "dependencies": dependencies.iter().map(|key| node_id(key)).collect::<Vec<_>>(),
        "occurrences": [{"role": "type", "ordinal": 0}],
        "body": body,
    })
}

/// The model declaration node key of `short`, a declaration of the selected
/// domain package, in `form`.
fn declaration_key(short: &str, form: &str) -> String {
    let tag = if form == "relationship" {
        "relation"
    } else {
        "model"
    };
    format!("model:{tag}:{form}:{}", orders(short))
}

/// A member type, as a test states it: a node key is derived from it here.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Ty {
    Boolean,
    Integer,
    Float32,
    Float64,
    String,
    Uuid,
    Timestamp,
    Bytes,
    JsonObject,
    Clock,
    Range(i128, i128),
    RationalRange(i128, i128, i128, i128),
    TextBounds(u64, u64, String),
    /// `Reference<O>`, by the key of `O`'s model declaration node.
    Reference(String),
    Record(String),
    Tuple(String),
    Variant(String),
    Quantity(String),
    CarriedRational(String, Box<Ty>),
    Option(Box<Ty>),
    Collection(Kind, Box<Ty>, Option<(u64, u64)>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Set,
    Bag,
    Sequence,
    OrderedSet,
}

impl Kind {
    fn form(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Bag => "bag",
            Self::Sequence => "sequence",
            Self::OrderedSet => "ordered_set",
        }
    }
}

fn ty_of(member: &CheckedMemberType) -> Ty {
    match member {
        CheckedMemberType::Boolean => Ty::Boolean,
        CheckedMemberType::Integer => Ty::Integer,
        CheckedMemberType::Float32 => Ty::Float32,
        CheckedMemberType::Float64 => Ty::Float64,
        CheckedMemberType::String => Ty::String,
        CheckedMemberType::Uuid => Ty::Uuid,
        CheckedMemberType::Timestamp => Ty::Timestamp,
        CheckedMemberType::Bytes => Ty::Bytes,
        CheckedMemberType::JsonObject => Ty::JsonObject,
        CheckedMemberType::Clock => Ty::Clock,
        CheckedMemberType::IntRange { lower, upper } => Ty::Range(*lower, *upper),
        CheckedMemberType::RationalRange {
            numerator_min,
            numerator_max,
            denominator_min,
            denominator_max,
        } => Ty::RationalRange(
            *numerator_min,
            *numerator_max,
            *denominator_min,
            *denominator_max,
        ),
        CheckedMemberType::TextBounds {
            lower,
            upper,
            profile,
        } => Ty::TextBounds(*lower, *upper, profile.to_string()),
        CheckedMemberType::Reference(target) => Ty::Reference(path_label(target)),
        CheckedMemberType::Record(target) => Ty::Record(path_label(target)),
        CheckedMemberType::Tuple(target) => Ty::Tuple(path_label(target)),
        CheckedMemberType::Variant(target) => Ty::Variant(path_label(target)),
        CheckedMemberType::Quantity(target) => Ty::Quantity(path_label(target)),
        CheckedMemberType::CarriedRational { unit, carrier } => {
            Ty::CarriedRational(path_label(unit), Box::new(ty_of(carrier)))
        }
        CheckedMemberType::Option(element) => Ty::Option(Box::new(ty_of(element))),
        CheckedMemberType::Collection {
            kind,
            element,
            bounds,
        } => Ty::Collection(
            match kind {
                CheckedCollectionKind::Set => Kind::Set,
                CheckedCollectionKind::Bag => Kind::Bag,
                CheckedCollectionKind::Sequence => Kind::Sequence,
                CheckedCollectionKind::OrderedSet => Kind::OrderedSet,
            },
            Box::new(ty_of(element)),
            *bounds,
        ),
    }
}

fn bounds_body(integer: &str, lower: &str, upper: &str) -> Value {
    let binding = |name: &str, value: &str| {
        json!({"term": "binding", "name": name, "value": {
            "term": "literal", "type": node_id(integer), "value_kind": "integer", "value": value,
        }})
    };
    json!({"term": "aggregate", "members": [binding("min", lower), binding("max", upper)]})
}

/// Adds the nodes of `ty` under assigned structural paths and returns its path.
fn type_nodes(ty: &Ty, nodes: &mut BTreeMap<String, Value>) -> String {
    let mut add = |key: String, node: Value| {
        nodes.entry(key.clone()).or_insert(node);
        key
    };
    match ty {
        Ty::Boolean
        | Ty::Integer
        | Ty::Float32
        | Ty::Float64
        | Ty::String
        | Ty::Uuid
        | Ty::Timestamp
        | Ty::Bytes
        | Ty::JsonObject
        | Ty::Clock => {
            let form = match ty {
                Ty::Boolean => "boolean",
                Ty::Integer => "integer",
                Ty::Float32 => "float32",
                Ty::Float64 => "float64",
                Ty::String => "string",
                Ty::Uuid => "uuid",
                Ty::Timestamp => "timestamp",
                Ty::Bytes => "bytes",
                Ty::JsonObject => "json_object",
                Ty::Clock => "clock",
                _ => unreachable!("matched scalar types"),
            };
            let key = format!("scalar_type/{form}");
            let node = wire_node(&key, "scalar_type", form, &key, &[], empty());
            add(key, node)
        }
        Ty::Range(lower, upper) => {
            let integer = type_nodes(&Ty::Integer, nodes);
            let body = bounds_body(&integer, &lower.to_string(), &upper.to_string());
            let key = format!("bounded_domain/integer_range/{lower}/{upper}");
            let node = wire_node(
                &key,
                "bounded_domain",
                "integer_range",
                &integer,
                &[&integer],
                body,
            );
            nodes.entry(key.clone()).or_insert(node);
            key
        }
        Ty::RationalRange(numerator_min, numerator_max, denominator_min, denominator_max) => {
            let integer = type_nodes(&Ty::Integer, nodes);
            let rational = "scalar_type/rational".to_owned();
            nodes.entry(rational.clone()).or_insert_with(|| {
                wire_node(
                    &rational,
                    "scalar_type",
                    "rational",
                    &rational,
                    &[],
                    empty(),
                )
            });
            let binding = |name: &str, value: i128| {
                json!({"term":"binding","name":name,
                "value":{"term":"literal","type":node_id(&integer),
                "value":value.to_string(),"value_kind":"integer"}})
            };
            let body = json!({"term":"aggregate","members":[
                binding("numerator_min", *numerator_min),
                binding("numerator_max", *numerator_max),
                binding("denominator_min", *denominator_min),
                binding("denominator_max", *denominator_max)]});
            let key = format!("bounded_domain/rational_range/{numerator_min}/{numerator_max}/{denominator_min}/{denominator_max}");
            nodes.entry(key.clone()).or_insert_with(|| {
                wire_node(
                    &key,
                    "bounded_domain",
                    "rational_range",
                    &rational,
                    &[],
                    body,
                )
            });
            key
        }
        Ty::CarriedRational(unit, carrier) => {
            let carrier = type_nodes(carrier, nodes);
            let text = "scalar_type/text".to_owned();
            nodes
                .entry(text.clone())
                .or_insert_with(|| wire_node(&text, "scalar_type", "text", &text, &[], empty()));
            let key = format!("carried_quantity:{unit};{carrier}");
            let body = json!({"term":"aggregate","members":[
                {"term":"binding","name":"unit","value":{"term":"reference","target":node_id(unit)}},
                {"term":"binding","name":"carrier","value":{"term":"reference","target":node_id(&carrier)}},
                {"term":"binding","name":"rounding","value":{"term":"literal","type":node_id(&text),"value":"exact","value_kind":"text"}}
            ]});
            nodes.entry(key.clone()).or_insert_with(|| {
                wire_node(
                    &key,
                    "scalar_type",
                    "carried_quantity",
                    &key,
                    &[unit, &carrier],
                    body,
                )
            });
            key
        }
        Ty::TextBounds(lower, upper, profile) => {
            let integer = type_nodes(&Ty::Integer, nodes);
            let text = "scalar_type/text".to_owned();
            nodes
                .entry(text.clone())
                .or_insert_with(|| wire_node(&text, "scalar_type", "text", &text, &[], empty()));
            let binding = |name: &str, ty: &str, value: String, value_kind: &str| {
                json!({
                    "term":"binding", "name":name, "value":{
                        "term":"literal", "type":node_id(ty), "value":value,
                        "value_kind":value_kind
                    }
                })
            };
            let body = json!({"term":"aggregate", "members":[
                binding("min", &integer, lower.to_string(), "integer"),
                binding("max", &integer, upper.to_string(), "integer"),
                binding("text_profile", &text, profile.clone(), "text")
            ]});
            let key = format!("bounded_domain/text_bounds/{lower}/{upper}/{profile}");
            nodes.entry(key.clone()).or_insert_with(|| {
                wire_node(&key, "bounded_domain", "text_bounds", &text, &[&text], body)
            });
            key
        }
        Ty::Reference(target) => {
            let body = json!({"term": "aggregate", "members": [reference(target)]});
            let key = format!("composite_type/reference/{target}");
            let node = wire_node(&key, "composite_type", "reference", &key, &[target], body);
            add(key, node)
        }
        Ty::Record(target) | Ty::Tuple(target) | Ty::Variant(target) | Ty::Quantity(target) => {
            target.clone()
        }
        Ty::Option(element) => {
            let element = type_nodes(element, nodes);
            let body = json!({"term": "aggregate", "members": [reference(&element)]});
            let key = format!("composite_type/option/{element}");
            let node = wire_node(&key, "composite_type", "option", &key, &[&element], body);
            nodes.entry(key.clone()).or_insert(node);
            key
        }
        Ty::Collection(kind, element, bounds) => {
            let element = type_nodes(element, nodes);
            let body = json!({"term": "aggregate", "members": [reference(&element)]});
            let collection = format!("composite_type/{}/{element}", kind.form());
            let node = wire_node(
                &collection,
                "composite_type",
                kind.form(),
                &collection,
                &[&element],
                body,
            );
            nodes.entry(collection.clone()).or_insert(node);
            let Some((lower, upper)) = bounds else {
                return collection;
            };
            let integer = type_nodes(&Ty::Integer, nodes);
            let body = bounds_body(&integer, &lower.to_string(), &upper.to_string());
            let key = format!("bounded_domain/collection_bounds/{collection}/{lower}/{upper}");
            let node = wire_node(
                &key,
                "bounded_domain",
                "collection_bounds",
                &collection,
                &[&collection],
                body,
            );
            nodes.entry(key.clone()).or_insert(node);
            key
        }
    }
}

// --- the domain package document -------------------------------------------------

/// `(lower, upper, ordered, unique)` of a declared multiplicity; `upper`
/// `None` is unbounded.
type Multiplicity = (u64, Option<u64>, bool, bool);

const ONE: Multiplicity = (1, Some(1), false, true);

struct Field {
    name: &'static str,
    type_ref: String,
    optional: bool,
    multiplicity: Multiplicity,
    redefines: Option<String>,
}

fn field(name: &'static str, type_ref: &str) -> Field {
    Field {
        name,
        type_ref: type_ref.to_owned(),
        optional: false,
        multiplicity: ONE,
        redefines: None,
    }
}

impl Field {
    fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    fn many(mut self, multiplicity: Multiplicity) -> Self {
        self.multiplicity = multiplicity;
        self
    }

    fn redefining(mut self, member: &str) -> Self {
        self.redefines = Some(member.to_owned());
        self
    }
}

const NATIVE: &str = "ix://quire/native/";

fn object(short: &str, supertypes: &[&str], fields: &[Field]) -> Value {
    typed_object(short, "entity", supertypes, fields)
}

fn typed_object(short: &str, kind: &str, supertypes: &[&str], fields: &[Field]) -> Value {
    let node = orders(short);
    let fields: Vec<Value> = fields
        .iter()
        .map(|field| {
            let (lower, upper, ordered, unique) = field.multiplicity;
            let mut multiplicity = json!({"lower": lower, "ordered": ordered, "unique": unique});
            if let Some(upper) = upper {
                multiplicity["upper"] = json!(upper);
            }
            let mut declared = json!({
                "identity": format!("{node}/{}", field.name), "name": field.name,
                "typeRef": field.type_ref,
                "presence": if field.optional { "optional" } else { "required" },
                "nullable": false, "defaultKind": "none", "multiplicity": multiplicity,
            });
            if let Some(target) = &field.redefines {
                declared["redefines"] = json!(target);
            }
            declared
        })
        .collect();
    json!({
        "identity": node, "displayName": short,
        "kind": {"module": "acme/orders", "name": kind},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes.iter().map(|short| orders(short)).collect::<Vec<_>>(),
        "fields": fields, "operations": [],
    })
}

fn process(short: &str, supertypes: &[&str], fields: &[Field]) -> Value {
    let identity = if fields.is_empty() {
        "run"
    } else {
        fields[0].name
    };
    let default_fields = [field("run", "ix://quire/native/UUID")];
    let mut declared = typed_object(
        short,
        "process",
        supertypes,
        if fields.is_empty() {
            &default_fields
        } else {
            fields
        },
    );
    declared["identityFields"] = json!([format!("{}/{identity}", orders(short))]);
    declared["steps"] = json!([{
        "identity":format!("{}/advance", orders(short)),
        "name":"advance", "stepKind":"command", "consumes":[], "emits":[]
    }]);
    declared
}

/// A value type of `scalar` with `min` and `max` constraints, written as the
/// decimal text a bound past 2^53 needs.
fn value_type(short: &str, scalar: &str, bounds: Option<(&str, &str)>) -> Value {
    let constraints: Vec<Value> = bounds
        .into_iter()
        .flat_map(|(min, max)| {
            [
                json!({"keyword": "min", "operands": {"value": min}}),
                json!({"keyword": "max", "operands": {"value": max}}),
            ]
        })
        .collect();
    json!({
        "identity": orders(short), "displayName": short,
        "kind": {"module": "acme/orders", "name": "digit"}, "roles": [], "extensions": [],
        "unknownPolicy": "reject", "scalar": scalar, "constraints": constraints,
    })
}

fn document(types: Vec<Value>) -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": IDENTITY, "version": "1.0.0"},
        "constructs": [
            {"kind": {"module": "acme/orders", "name": "entity"},
             "construct": {"meaning": "quire.meaning.model.object-type/v1"}},
            {"kind": {"module": "acme/orders", "name": "digit"},
             "construct": {"meaning": "quire.meaning.model.value-type/v1"}},
            {"kind": {"module": "acme/orders", "name": "interface"},
             "construct": {"meaning": "quire.meaning.systems.interface/v1"}},
            {"kind": {"module": "acme/orders", "name": "event"},
             "construct": {"meaning": "quire.meaning.model.event-type/v1"}},
        ],
        "types": types,
    })
}

fn thousand() -> Value {
    value_type("Thousand", "integer", Some(("0", "1000")))
}

// --- the package ------------------------------------------------------------------

/// A read of `field` on `owner`, typed `ty`: an `expression`/`query`
/// application of `quire.op.record.project` over the owner's declaration node.
struct Read {
    owner: &'static str,
    field: &'static str,
    ty: Ty,
}

/// A package selecting `document`, with a model declaration node for each of
/// `declared` (`(short name, form)`) and one read node per entry of `reads`;
/// returns the package, its evidence and the position of each read.
fn package(
    document: &Value,
    declared: &[(&str, &str)],
    reads: &[Read],
) -> (Value, CheckedPackageEvidence, Vec<usize>) {
    let mut package = nominal_package(&[]);
    let mut nodes: BTreeMap<String, Value> = BTreeMap::new();
    let mut ordered: Vec<Value> = Vec::new();
    for (short, form) in declared {
        let key = declaration_key(short, form);
        let tag = if *form == "relationship" {
            "relation"
        } else {
            "model"
        };
        let mut node = wire_node(&key, tag, form, &key, &[], empty());
        if matches!(
            *form,
            OBJECT
                | INTERFACE
                | "relationship"
                | "record_value_type"
                | "event_type"
                | "tuple_value_type"
                | "process"
                | "state_machine"
                | "persistence_interface"
        ) {
            node["owner"] = json!({
                "kind": "model", "identity": IDENTITY, "node": orders(short),
            });
        }
        ordered.push(node);
    }
    let mut read_nodes = Vec::new();
    let mut read_offsets = Vec::new();
    for (read_index, read) in reads.iter().enumerate() {
        let ty = type_nodes(&read.ty, &mut nodes);
        let form = declared
            .iter()
            .find_map(|(short, form)| (*short == read.owner).then_some(*form))
            .expect("the read owner has a declaration");
        let owner = declaration_key(read.owner, form);
        let (argument, dependencies, member) = if form == "record_value_type" {
            (
                reference(&owner),
                vec![owner.clone()],
                json!({"kind": "field", "declaration": node_id(&owner), "name": read.field}),
            )
        } else {
            let reference_type = type_nodes(&Ty::Reference(owner.clone()), &mut nodes);
            let deref_key = format!("expression/deref/{}/{read_index}", read.owner);
            let deref_body = json!({
                "term": "application", "operator": "deref",
                "operation": {"identity": "quire.op.model.deref", "laws": [],
                    "mode": null, "member": null, "leaves": []},
                "result_type": node_id(&owner),
                "arguments": [reference(&reference_type)]
            });
            read_nodes.push(wire_node(
                &deref_key,
                "expression",
                "deref",
                &owner,
                &[&reference_type],
                deref_body,
            ));
            (
                reference(&deref_key),
                vec![owner.clone(), deref_key],
                json!({"kind": "field", "declaration": node_id(&owner),
                    "name": read.field, "read_form": "bare"}),
            )
        };
        let body = json!({
            "term": "application",
            "operator": "query",
            "operation": {
                "identity": "quire.op.record.project", "laws": [], "mode": null,
                "member": member,
                "leaves": [],
            },
            "result_type": node_id(&ty),
            "arguments": [argument],
        });
        let key = format!(
            "expression/query/{}/{}/{}",
            read.owner, read.field, read_index
        );
        read_offsets.push(read_nodes.len());
        let dependencies = dependencies.iter().map(String::as_str).collect::<Vec<_>>();
        read_nodes.push(wire_node(
            &key,
            "expression",
            "query",
            &ty,
            &dependencies,
            body,
        ));
    }
    let mut graph: Vec<Value> = nodes.into_values().collect();
    graph.extend(ordered);
    let first_read = graph.len();
    graph.extend(read_nodes);
    let base = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("base nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(graph);
    select_model(&mut package, IDENTITY);
    settle(&mut package);
    let evidence = evidence_with_document(&package, IDENTITY, document);
    let positions = read_offsets
        .into_iter()
        .map(|offset| base + first_read + offset)
        .collect();
    (package, evidence, positions)
}

/// The pointer of the owner of the `model_root` selecting `identity`, where the
/// reader locates a fault of the selected document.
fn root_owner(package: &Value, identity: &str) -> String {
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "model_root" && node["owner"]["identity"] == identity
        })
        .expect("selected root");
    format!("/semantic_graph/nodes/{position}/owner")
}

fn read_with(
    package: &Value,
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
) -> CheckedPackageV2ReadResult {
    let mut wire = package.clone();
    index_fixture_node_ids(&mut wire);
    CheckedPackageV2::read(&canonical(&wire), limits, evidence)
}

fn admit(document: &Value, declared: &[(&str, &str)], reads: &[Read]) -> CheckedPackageV2 {
    let (package, evidence, _) = self::package(document, declared, reads);
    match read_with(&package, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, got {other:?}"),
    }
}

/// Trace: FR-038-AC-420, QSpec FR-322-AC-68, TC-281 EV-01/EV-02.
#[trace("TC-048", "TC-281", "FR-038-AC-420")]
#[test]
fn tc_281_event_type_model_owner_joins_only_selected_event_declarations() {
    let mut event = typed_object(
        "OrderPlaced",
        "event",
        &[],
        &[field("occurred_at", "ix://quire/native/Timestamp")],
    );
    event["occurrenceField"] = json!(format!("{}/occurred_at", orders("OrderPlaced")));
    let mut selected = document(vec![
        event,
        typed_object("Money", "record", &[], &[]),
        object("Order", &[], &[]),
    ]);
    selected["constructs"]
        .as_array_mut()
        .expect("construct catalog")
        .push(json!({
            "kind": {"module": IDENTITY, "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
        }));
    let (base, evidence, _) = package(&selected, &[("OrderPlaced", "event_type")], &[]);
    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "event_type")
        .expect("event node");
    assert_eq!(
        base["semantic_graph"]["nodes"][position]["owner"]["kind"],
        "model"
    );
    assert!(matches!(
        read_with(&base, &evidence, CheckedPackageReadLimits::bounded()),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    for wrong in ["Money", "Order", "Absent"] {
        let mut package = base.clone();
        package["semantic_graph"]["nodes"][position]["owner"]["node"] = json!(orders(wrong));
        settle(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_with(&package, &evidence, CheckedPackageReadLimits::bounded())
        else {
            panic!("EV-02 owner naming {wrong} must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MissingDeclaration,
            "{wrong}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MissingSelection),
            "{wrong}"
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some(format!("/semantic_graph/nodes/{position}/node_id").as_str()),
            "{wrong}"
        );
    }

    let pointer = format!("/semantic_graph/nodes/{position}");
    let mut absent = base.clone();
    absent["semantic_graph"]["nodes"][position]
        .as_object_mut()
        .expect("event node")
        .remove("owner");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with(&absent, &evidence, CheckedPackageReadLimits::bounded())
    else {
        panic!("ownerless event must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(pointer.as_str())
    );

    let mut source = base.clone();
    let selected_source = &base["lock"]["sources"][0];
    source["semantic_graph"]["nodes"][position]["owner"] = json!({
        "kind": "source", "authority": selected_source["authority"],
        "identity": selected_source["identity"],
    });
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with(&source, &evidence, CheckedPackageReadLimits::bounded())
    else {
        panic!("source-owned undeclared event must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(format!("{pointer}/owner").as_str())
    );
}

/// The public runtime projection returns selected record and object facts.
#[test]
fn selected_runtime_types_keep_record_and_object_fields_and_parents() {
    let mut selected = document(vec![
        typed_object(
            "Receipt",
            "record",
            &[],
            &[field("total", "ix://quire/native/Integer")],
        ),
        object(
            "Base",
            &[],
            &[field("enabled", "ix://quire/native/Boolean")],
        ),
        object(
            "Child",
            &["Base"],
            &[field("count", "ix://quire/native/Integer")],
        ),
    ]);
    selected["constructs"]
        .as_array_mut()
        .expect("construct catalog")
        .push(json!({
            "kind": {"module": IDENTITY, "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
        }));
    let admitted = admit(
        &selected,
        &[
            ("Receipt", "record_value_type"),
            ("Base", OBJECT),
            ("Child", OBJECT),
        ],
        &[],
    );
    let structural = |short, form| -> CheckedNodeId {
        serde_json::from_value(node_id(&declaration_key(short, form)))
            .expect("structural declaration")
    };
    let record = structural("Receipt", "record_value_type");
    let base = structural("Base", OBJECT);
    let child = structural("Child", OBJECT);
    let record_key =
        NodeKey::new("checked", vec![PathSegment::Text("Receipt".into())], 0).expect("record key");
    let base_key = EffectiveId::new(None, OriginalDeclarationKey::new(IDENTITY, orders("Base")));
    let child_key = EffectiveId::new(None, OriginalDeclarationKey::new(IDENTITY, orders("Child")));
    let record_row = SelectedExactDeclarationRow::new(
        record.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::RecordValueType,
        orders("Receipt"),
        SelectedExactIdentity::Record(record_key.clone()),
    );
    let base_row = SelectedExactDeclarationRow::new(
        base.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::ObjectType,
        orders("Base"),
        SelectedExactIdentity::Object(base_key.clone()),
    );
    let child_row = SelectedExactDeclarationRow::new(
        child.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::ObjectType,
        orders("Child"),
        SelectedExactIdentity::Object(child_key.clone()),
    );
    let record_field = SelectedExactFieldRow::new(
        record.clone(),
        IDENTITY,
        orders("Receipt"),
        format!("{}/total", orders("Receipt")),
        MemberId::new(MemberOwner::Node(record_key.clone()), "total").expect("member id"),
    );
    let witnessed = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [record_row.clone(), base_row.clone(), child_row.clone()],
        [record_field.clone()],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("selected rows join");
    let complete = admitted
        .project_selected_type_declarations(
            &witnessed,
            &[],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect("selected record/object view");
    assert_eq!(complete.records().len(), 1);
    assert_eq!(complete.records()[0].view().record_key(), &record_key);
    assert_eq!(complete.records()[0].view().fields()[0].name(), "total");
    assert_eq!(
        complete.records()[0].view().fields()[0].value_type(),
        &ValueType::Integer
    );
    assert_eq!(complete.objects().len(), 2);
    let projected_child = complete
        .objects()
        .iter()
        .find(|object| object.effective_id() == &child_key)
        .expect("selected child");
    assert_eq!(projected_child.supertypes(), &[base_key]);
    assert_eq!(
        projected_child
            .declared_fields()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        ["count"]
    );
    assert_eq!(
        projected_child
            .fields()
            .iter()
            .map(|field| (field.name(), field.value_type()))
            .collect::<Vec<_>>(),
        [
            ("enabled", &ValueType::Boolean),
            ("count", &ValueType::Integer)
        ]
    );
    assert_eq!(
        admitted.clone().project_selected_type_declarations(
            &witnessed,
            &[],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        ),
        Ok(complete.clone())
    );
    assert!(complete.work() > 0);

    let mut extra_selected = selected.clone();
    extra_selected["types"]
        .as_array_mut()
        .expect("selected types")
        .push(object("Unrepresented", &[], &[]));
    let incomplete_graph = admit(
        &extra_selected,
        &[
            ("Receipt", "record_value_type"),
            ("Base", OBJECT),
            ("Child", OBJECT),
        ],
        &[],
    );
    let same_rows = SelectedExactIdentities::from_stage_rows(
        &incomplete_graph,
        [record_row.clone(), base_row.clone(), child_row.clone()],
        [record_field.clone()],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("represented rows still join");
    let unrepresented = incomplete_graph
        .project_selected_type_declarations(
            &same_rows,
            &[],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect_err("an unrepresented selected object refuses the whole view");
    assert_eq!(
        unrepresented.code,
        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration
    );

    let missing_child = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [record_row, base_row],
        [record_field],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("other selected rows join");
    let refusal = admitted
        .project_selected_type_declarations(
            &missing_child,
            &[],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect_err("missing object witness refuses the whole view");
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    assert_eq!(refusal.at.as_ref(), &child);
    let exhausted = admitted
        .project_selected_type_declarations(
            &witnessed,
            &[],
            &[],
            1,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect_err("finite work limit is charged before a view exists");
    assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    assert_eq!(exhausted.work, 1);
}

/// Trace: FR-357-AC-5, FR-357-AC-6, FR-357-AC-7.
#[test]
fn selected_facts_join_only_explicit_model_field_and_charge_full_emission_rows() {
    let mut selected = document(vec![typed_object(
        "Receipt",
        "record",
        &[],
        &[
            field("total", "ix://quire/native/Integer"),
            field("note", "ix://quire/native/String").optional(),
            field("raw", "ix://quire/native/Float32"),
        ],
    )]);
    selected["constructs"].as_array_mut().unwrap().push(json!({
        "kind": {"module": IDENTITY, "name": "record"},
        "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
    }));
    let (mut wire, evidence, _) =
        self::package(&selected, &[("Receipt", "record_value_type")], &[]);
    let boolean = "scalar_type/boolean";
    let equality = "selected/equality";
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(wire_node(
            boolean,
            "scalar_type",
            "boolean",
            boolean,
            &[],
            empty(),
        ));
    let mut equality_node = wire_node(
        equality,
        "expression",
        "binary",
        boolean,
        &[boolean],
        json!({"term":"application", "operator":"binary",
            "operation":{"identity":"quire.op.boolean.eq", "laws":[], "mode":null,
                "member":null, "leaves":[]},
            "result_type":node_id(boolean),
            "arguments":[reference(boolean), reference(boolean)]}),
    );
    equality_node["occurrences"] = json!([{"role":"expression", "ordinal":0}]);
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(equality_node);
    wire["checked_term_data"] = json!({
        "equality_decisions":[{
            "application":node_id(equality),
            "occurrence":{"role":"expression","ordinal":0},
            "term_path":[], "operator":"equal", "schedule":"plan",
            "left":{"operand":0,"source_type":node_id(boolean),"conversion_target":null},
            "right":{"operand":1,"source_type":node_id(boolean),"conversion_target":null},
            "units":[],"enum_members":[]
        }],
        "protocol_locations":[],"dispatch_tables":[],"probabilistic_claims":[]
    });
    settle(&mut wire);
    let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("selected model and equality fixture must admit: {other:?}"),
    };
    let record: CheckedNodeId =
        serde_json::from_value(node_id(&declaration_key("Receipt", "record_value_type"))).unwrap();
    let record_key = NodeKey::new("checked", vec![PathSegment::Text("Receipt".into())], 0).unwrap();
    let declaration = SelectedExactDeclarationRow::new(
        record.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::RecordValueType,
        orders("Receipt"),
        SelectedExactIdentity::Record(record_key.clone()),
    );
    let field_row = SelectedExactFieldRow::new(
        record.clone(),
        IDENTITY,
        orders("Receipt"),
        format!("{}/total", orders("Receipt")),
        MemberId::new(MemberOwner::Node(record_key.clone()), "total").unwrap(),
    );
    let optional_row = SelectedExactFieldRow::new(
        record.clone(),
        IDENTITY,
        orders("Receipt"),
        format!("{}/note", orders("Receipt")),
        MemberId::new(MemberOwner::Node(record_key.clone()), "note").unwrap(),
    );
    let raw_row = SelectedExactFieldRow::new(
        record.clone(),
        IDENTITY,
        orders("Receipt"),
        format!("{}/raw", orders("Receipt")),
        MemberId::new(MemberOwner::Node(record_key), "raw").unwrap(),
    );
    let selected_field = SelectedFactFieldId {
        record: record.clone(),
        original_identity: format!("{}/total", orders("Receipt")).into(),
    };
    let optional_field = SelectedFactFieldId {
        record: record.clone(),
        original_identity: format!("{}/note", orders("Receipt")).into(),
    };
    let raw_field = SelectedFactFieldId {
        record: record.clone(),
        original_identity: format!("{}/raw", orders("Receipt")).into(),
    };
    let ids = SelectedFactIds {
        declarations: std::slice::from_ref(&record),
        fields: std::slice::from_ref(&selected_field),
        equalities: &[],
    };
    let extra = SelectedExactDeclarationRow::new(
        serde_json::from_value(node_id("scalar_type/integer")).unwrap(),
        "unrelated/document",
        SelectedExactDeclarationForm::RecordValueType,
        "unrelated/declaration",
        SelectedExactIdentity::Record(
            NodeKey::new("other", vec![PathSegment::Text("unrelated".into())], 0).unwrap(),
        ),
    );
    let declarations = [declaration.clone()];
    let fields = [field_row.clone(), optional_row.clone(), raw_row];
    let rows = SelectedFactRows {
        declarations: &declarations,
        fields: &fields,
        source_units: &[],
        equality_applications: &[],
        equality_facts: &[],
    };
    let complete = admitted
        .project_selected_facts(
            ids,
            rows,
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap();
    assert_eq!(complete.declarations, vec![declaration.clone()]);
    assert_eq!(complete.fields.len(), 1);
    assert_eq!(complete.fields[0].row, field_row);
    assert_eq!(complete.fields[0].name.as_ref(), "total");
    assert_eq!(complete.fields[0].order, 0);
    assert_eq!(complete.fields[0].presence, Presence::Required);
    assert_eq!(complete.fields[0].value_type, ValueType::Integer);
    assert!(complete.equalities.is_empty());
    let application: CheckedNodeId = serde_json::from_value(node_id(equality)).unwrap();
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let original_application = NodeKey::new(
        "selected",
        vec![PathSegment::Text("boolean-equality".into())],
        0,
    )
    .unwrap();
    let source_map_entry = admitted
        .source_map()
        .iter()
        .find(|row| row.node_id == application && row.role == CheckedOccurrenceRole::Expression)
        .expect("admitted equality source map")
        .clone();
    let application_row = SelectedEqualityApplicationRow::new(
        application.clone(),
        occurrence.clone(),
        source_map_entry,
        original_application.clone(),
        quire_exact::Location::new(
            original_application.clone(),
            quire_exact::Origin::new(quire_exact::Role::from("expression"), 0),
        ),
    );
    let selected_equality = SelectedFactEqualityId {
        application: application.clone(),
        occurrence: occurrence.clone(),
    };
    let application_rows = [application_row];
    let combined_fields = [selected_field.clone(), optional_field.clone()];
    let combined_ids = || SelectedFactIds {
        declarations: std::slice::from_ref(&record),
        fields: &combined_fields,
        equalities: std::slice::from_ref(&selected_equality),
    };
    let combined_rows = || SelectedFactRows {
        declarations: &declarations,
        fields: &fields,
        source_units: &[],
        equality_applications: &application_rows,
        equality_facts: &[],
    };
    let combined = admitted
        .project_selected_facts(
            combined_ids(),
            combined_rows(),
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect("one admitted read retains model, field and equality facts");
    assert_eq!(combined.declarations, complete.declarations);
    assert_eq!(combined.fields.len(), 2);
    assert_eq!(combined.fields[0], complete.fields[0]);
    assert_eq!(combined.fields[1].row, optional_row);
    assert_eq!(combined.fields[1].order, 1);
    assert_eq!(combined.fields[1].presence, Presence::Optional);
    assert_eq!(combined.fields[1].value_type, ValueType::String);
    assert_eq!(combined.equalities.len(), 1);
    assert_eq!(combined.equalities[0].id, selected_equality);
    assert_eq!(
        combined.equalities[0].decision.application(),
        &original_application
    );
    assert_eq!(
        admitted.clone().project_selected_facts(
            combined_ids(),
            combined_rows(),
            combined.work,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        ),
        Ok(combined.clone()),
    );
    let combined_short = admitted
        .project_selected_facts(
            combined_ids(),
            combined_rows(),
            combined.work - 1,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect_err("one less than complete selected-fact work refuses");
    assert_eq!(combined_short.code, SelectedFactsErrorCode::Equality);
    assert_eq!(
        combined_short.equality.as_ref().unwrap().code,
        quire_contract_model::AdmittedEqualityReadErrorCode::WorkLimit
    );
    let exact = admitted
        .project_selected_facts(
            SelectedFactIds {
                declarations: &[record.clone(), record.clone()],
                fields: &[selected_field.clone(), selected_field.clone()],
                equalities: &[],
            },
            SelectedFactRows {
                declarations: &declarations,
                fields: &fields,
                source_units: &[],
                equality_applications: &[],
                equality_facts: &[],
            },
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap();
    assert_eq!(exact.declarations.len(), 1);
    assert_eq!(exact.fields.len(), 1);
    let replay = admitted
        .clone()
        .project_selected_facts(
            SelectedFactIds {
                declarations: std::slice::from_ref(&record),
                fields: std::slice::from_ref(&selected_field),
                equalities: &[],
            },
            SelectedFactRows {
                declarations: &declarations,
                fields: &fields,
                source_units: &[],
                equality_applications: &[],
                equality_facts: &[],
            },
            complete.work,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap();
    assert_eq!(replay, complete);
    let exhausted = admitted
        .project_selected_facts(
            SelectedFactIds {
                declarations: std::slice::from_ref(&record),
                fields: std::slice::from_ref(&selected_field),
                equalities: &[],
            },
            SelectedFactRows {
                declarations: &declarations,
                fields: &fields,
                source_units: &[],
                equality_applications: &[],
                equality_facts: &[],
            },
            complete.work - 1,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(exhausted.code, SelectedFactsErrorCode::ExhaustedWork);
    assert!(exhausted.work < complete.work);
    assert_eq!(exhausted.work + exhausted.attempted_work, complete.work);
    assert_eq!(exhausted.at.as_ref(), &record);
    assert!(exhausted.attempted_work > 0);
    let longer_rows = [extra, declaration];
    let longer = admitted.project_selected_facts(
        SelectedFactIds {
            declarations: std::slice::from_ref(&record),
            fields: std::slice::from_ref(&selected_field),
            equalities: &[],
        },
        SelectedFactRows {
            declarations: &longer_rows,
            fields: &fields,
            source_units: &[],
            equality_applications: &[],
            equality_facts: &[],
        },
        100_000,
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    );
    let longer = longer.unwrap();
    assert_eq!(longer.declarations, complete.declarations);
    assert_eq!(longer.fields, complete.fields);
    assert!(longer.work > complete.work);

    let selected_once = || SelectedFactIds {
        declarations: std::slice::from_ref(&record),
        fields: std::slice::from_ref(&selected_field),
        equalities: &[],
    };
    let with_rows = |declarations, fields| SelectedFactRows {
        declarations,
        fields,
        source_units: &[],
        equality_applications: &[],
        equality_facts: &[],
    };
    assert_eq!(
        admitted
            .project_selected_facts(
                selected_once(),
                with_rows(&[], &fields),
                100_000,
                &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
            )
            .unwrap_err()
            .code,
        SelectedFactsErrorCode::MissingRow,
    );
    assert_eq!(
        admitted
            .project_selected_facts(
                selected_once(),
                with_rows(&declarations, &[]),
                100_000,
                &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
            )
            .unwrap_err()
            .code,
        SelectedFactsErrorCode::MissingRow,
    );
    let duplicate_declarations = [declarations[0].clone(), declarations[0].clone()];
    assert_eq!(
        admitted
            .project_selected_facts(
                selected_once(),
                with_rows(&duplicate_declarations, &fields),
                100_000,
                &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
            )
            .unwrap_err()
            .code,
        SelectedFactsErrorCode::DuplicateRow,
    );
    let mut wrong_document = declarations[0].clone();
    wrong_document.selected_document = "wrong/document".into();
    let wrong_document_rows = [wrong_document];
    let wrong = admitted
        .project_selected_facts(
            selected_once(),
            with_rows(&wrong_document_rows, &fields),
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(wrong.code, SelectedFactsErrorCode::Exact);
    assert_eq!(
        wrong.exact.as_ref().unwrap().code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    assert_eq!(wrong.at.as_ref(), &record);
    let mut stale = declarations[0].clone();
    stale.structural_node.package = "foreign/document".into();
    let stale_rows = [stale];
    let stale_error = admitted
        .project_selected_facts(
            selected_once(),
            with_rows(&stale_rows, &fields),
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(stale_error.code, SelectedFactsErrorCode::Exact);
    assert_eq!(
        stale_error.exact.as_ref().unwrap().code,
        ExactTypeProjectionErrorCode::ForeignNode
    );
    let mut wrong_field = fields[0].clone();
    wrong_field.member_id = MemberId::new(
        MemberOwner::Node(
            NodeKey::new("other", vec![PathSegment::Text("owner".into())], 0).unwrap(),
        ),
        "total",
    )
    .unwrap();
    let wrong_field_rows = [wrong_field];
    let wrong_field_error = admitted
        .project_selected_facts(
            selected_once(),
            with_rows(&declarations, &wrong_field_rows),
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(wrong_field_error.code, SelectedFactsErrorCode::Exact);
    assert_eq!(
        wrong_field_error.exact.as_ref().unwrap().code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let foreign: CheckedNodeId = serde_json::from_value(structural_id(
        "foreign/document",
        vec![json!("model"), json!("record_value_type"), json!("Receipt")],
        0,
    ))
    .unwrap();
    let foreign_result = admitted
        .project_selected_facts(
            SelectedFactIds {
                declarations: std::slice::from_ref(&foreign),
                fields: &[],
                equalities: &[],
            },
            with_rows(&declarations, &fields),
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(foreign_result.code, SelectedFactsErrorCode::ForeignNode);
    assert_eq!(foreign_result.at.as_ref(), &foreign);

    let mixed_declarations = [declarations[0].clone(), stale_rows[0].clone()];
    let mixed_declaration = admitted
        .project_selected_facts(
            selected_once(),
            with_rows(&mixed_declarations, &fields),
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(mixed_declaration.code, SelectedFactsErrorCode::DuplicateRow);
    assert_eq!(
        mixed_declaration.at.as_ref(),
        &stale_rows[0].structural_node
    );
    let mut stale_field = fields[0].clone();
    stale_field.record_node.package = "foreign/document".into();
    let mixed_fields = [fields[0].clone(), stale_field.clone()];
    let mixed_field = admitted
        .project_selected_facts(
            selected_once(),
            with_rows(&declarations, &mixed_fields),
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(mixed_field.code, SelectedFactsErrorCode::DuplicateRow);
    assert_eq!(mixed_field.at.as_ref(), &stale_field.record_node);
    assert_eq!(
        mixed_field.field.as_deref(),
        Some(selected_field.original_identity.as_ref())
    );

    let mut oversized = declarations[0].clone();
    oversized.original_identity = "x".repeat(1_000_000).into();
    let oversized_rows = [oversized];
    let denied = admitted
        .project_selected_facts(
            selected_once(),
            with_rows(&oversized_rows, &fields),
            10,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(denied.code, SelectedFactsErrorCode::ExhaustedWork);
    assert_eq!(denied.at.as_ref(), &record);
    assert!(denied.attempted_work > 0);

    let unsupported: CheckedNodeId = serde_json::from_value(node_id(boolean)).unwrap();
    let unsupported_error = admitted
        .project_selected_facts(
            SelectedFactIds {
                declarations: std::slice::from_ref(&unsupported),
                fields: &[],
                equalities: &[],
            },
            with_rows(&declarations, &fields),
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        unsupported_error.code,
        SelectedFactsErrorCode::UnsupportedForm
    );
    assert_eq!(unsupported_error.at.as_ref(), &unsupported);
    let unrepresentable = admitted
        .project_selected_facts(
            SelectedFactIds {
                declarations: std::slice::from_ref(&record),
                fields: std::slice::from_ref(&raw_field),
                equalities: &[],
            },
            with_rows(&declarations, &fields),
            100_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(unrepresentable.code, SelectedFactsErrorCode::Exact);
    assert_eq!(
        unrepresentable.exact_code,
        Some(ExactTypeProjectionErrorCode::UnsupportedExactType)
    );
    assert_eq!(unrepresentable.at.as_ref(), &record);
    assert_eq!(
        unrepresentable.field.as_deref(),
        Some(raw_field.original_identity.as_ref())
    );
    assert_eq!(
        admitted.project_selected_facts(
            selected_once(),
            with_rows(&declarations, &fields),
            complete.work,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        ),
        Ok(complete),
        "unselected unsupported forms and fields must not change the selected result",
    );
}

/// A flattened field set cannot stand in for an authored redefinition link.
#[test]
fn selected_runtime_types_refuse_unprojected_object_redefinition() {
    let selected = document(vec![
        object(
            "Base",
            &[],
            &[field("enabled", "ix://quire/native/Boolean")],
        ),
        object(
            "Child",
            &["Base"],
            &[field("active", "ix://quire/native/Boolean")
                .redefining(&format!("{}/enabled", orders("Base")))],
        ),
    ]);
    let admitted = admit(&selected, &[("Base", OBJECT), ("Child", OBJECT)], &[]);
    let rows = ["Base", "Child"].map(|short| {
        SelectedExactDeclarationRow::new(
            serde_json::from_value(node_id(&declaration_key(short, OBJECT)))
                .expect("structural declaration"),
            IDENTITY,
            SelectedExactDeclarationForm::ObjectType,
            orders(short),
            SelectedExactIdentity::Object(EffectiveId::new(
                None,
                OriginalDeclarationKey::new(IDENTITY, orders(short)),
            )),
        )
    });
    let witnessed = SelectedExactIdentities::from_stage_rows(
        &admitted,
        rows,
        [],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("selected object rows join");
    let error = admitted
        .project_selected_type_declarations(
            &witnessed,
            &[],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect_err("redefinition needs its own admitted join");
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
    assert_eq!(error.form.as_ref(), "object_field_redefinition");
}

/// An object's own operations need admitted effect facts before a type view can succeed.
#[test]
fn selected_runtime_types_refuse_object_operations_without_callable_facts() {
    let mut operated = object("Runner", &[], &[]);
    operated["operations"] = json!([{
        "identity": format!("{}/run", orders("Runner")),
        "name": "run",
        "params": []
    }]);
    let admitted = admit(&document(vec![operated]), &[("Runner", OBJECT)], &[]);
    let witnessed = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [SelectedExactDeclarationRow::new(
            serde_json::from_value(node_id(&declaration_key("Runner", OBJECT)))
                .expect("structural declaration"),
            IDENTITY,
            SelectedExactDeclarationForm::ObjectType,
            orders("Runner"),
            SelectedExactIdentity::Object(EffectiveId::new(
                None,
                OriginalDeclarationKey::new(IDENTITY, orders("Runner")),
            )),
        )],
        [],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("selected object row joins");
    let error = admitted
        .project_selected_type_declarations(
            &witnessed,
            &[],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect_err("operation-bearing object cannot yield a partial type view");
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    assert_eq!(error.form.as_ref(), "object_operation_effect");
}

/// A selected tuple needs its producer's tuple identity row.
#[test]
fn selected_runtime_types_refuse_tuple_without_stage_identity() {
    let tuple = orders("Pair");
    let mut selected = document(vec![json!({
        "identity": tuple,
        "displayName": "Pair",
        "kind": {"module": IDENTITY, "name": "tuple"},
        "positions": [{
            "identity": format!("{tuple}/p0"),
            "typeRef": "ix://quire/native/Boolean"
        }]
    })]);
    selected["constructs"]
        .as_array_mut()
        .expect("construct catalog")
        .push(json!({
            "kind": {"module": IDENTITY, "name": "tuple"},
            "construct": {"meaning": "quire.meaning.model.tuple-value-type/v1"}
        }));
    let admitted = admit(&selected, &[("Pair", "tuple_value_type")], &[]);
    let witnessed = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [],
        [],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("no supplied row can witness a tuple yet");
    let error = admitted
        .project_selected_type_declarations(
            &witnessed,
            &[],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect_err("a tuple cannot produce a partial environment view");
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
}

/// A tuple row joins the retained declaration and exposes every admitted position.
#[test]
#[trace("FR-354-AC-1", "FR-354-AC-2")]
fn selected_tuple_positions_require_matching_unique_stage_rows() {
    let tuple = orders("Pair");
    let mut selected = document(vec![json!({
        "identity": tuple, "displayName": "Pair",
        "kind": {"module": IDENTITY, "name": "tuple"},
        "positions": [
            {"identity": format!("{tuple}/p0"), "typeRef": "ix://quire/native/Boolean"},
            {"identity": format!("{tuple}/p1"), "typeRef": "ix://quire/native/Integer"}
        ]
    })]);
    selected["constructs"].as_array_mut().unwrap().push(json!({
        "kind": {"module": IDENTITY, "name": "tuple"},
        "construct": {"meaning": "quire.meaning.model.tuple-value-type/v1"}
    }));
    let admitted = admit(&selected, &[("Pair", "tuple_value_type")], &[]);
    let node: CheckedNodeId =
        serde_json::from_value(node_id(&declaration_key("Pair", "tuple_value_type"))).unwrap();
    let key = NodeKey::new("tuple-stage", vec![PathSegment::Text("Pair".into())], 0).unwrap();
    let row = SelectedTupleDeclarationRow::new(node.clone(), IDENTITY, tuple.clone(), key.clone());
    let identities = SelectedExactIdentities::empty();
    let view = admitted
        .project_selected_type_declarations(
            &identities,
            std::slice::from_ref(&row),
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap();
    assert_eq!(view.tuples().len(), 1);
    assert_eq!(view.tuples()[0].node_key(), &key);
    assert_eq!(
        view.tuples()[0].positions(),
        &[ValueType::Boolean, ValueType::Integer]
    );
    assert_eq!(view.tuples()[0].original_identity(), tuple);
    assert_eq!(
        admitted.clone().project_selected_type_declarations(
            &identities,
            std::slice::from_ref(&row),
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        ),
        Ok(view.clone())
    );
    let missing = admitted
        .project_selected_type_declarations(
            &identities,
            &[],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        missing.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    let conflicting = admitted
        .project_selected_type_declarations(
            &identities,
            &[row.clone(), row.clone()],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        conflicting.code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
    );
    let foreign =
        SelectedTupleDeclarationRow::new(node.clone(), "foreign", tuple.clone(), key.clone());
    let mismatch = admitted
        .project_selected_type_declarations(
            &identities,
            &[foreign],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        mismatch.code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let mut foreign_node = node;
    foreign_node.package = "different-package".into();
    let foreign = SelectedTupleDeclarationRow::new(foreign_node, IDENTITY, tuple, key);
    let refusal = admitted
        .project_selected_type_declarations(
            &identities,
            &[foreign],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(refusal.code, ExactTypeProjectionErrorCode::ForeignNode);
    let exhausted = admitted
        .project_selected_type_declarations(
            &identities,
            &[row],
            &[],
            view.work() - 1,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);

    let long_key =
        NodeKey::new("tuple-stage", vec![PathSegment::Text("x".repeat(4096))], 0).unwrap();
    let long_row = SelectedTupleDeclarationRow::new(
        serde_json::from_value(node_id(&declaration_key("Pair", "tuple_value_type"))).unwrap(),
        IDENTITY,
        orders("Pair"),
        long_key,
    );
    let long_view = admitted
        .project_selected_type_declarations(
            &identities,
            std::slice::from_ref(&long_row),
            &[],
            20_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap();
    assert!(long_view.work() > view.work() + 4000);
    assert!(admitted
        .project_selected_type_declarations(
            &identities,
            std::slice::from_ref(&long_row),
            &[],
            long_view.work(),
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        )
        .is_ok());
    let one_less = admitted
        .project_selected_type_declarations(
            &identities,
            std::slice::from_ref(&long_row),
            &[],
            long_view.work() - 1,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(one_less.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    let tight = admitted
        .project_selected_type_declarations(
            &identities,
            std::slice::from_ref(&long_row),
            &[],
            view.work() + 100,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(tight.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    assert_eq!(tight.at.as_ref(), &long_row.structural_node);
    assert_eq!(tight.form.as_ref(), "selected_tuple_row");
}

#[test]
#[trace("FR-354-AC-1", "FR-354-AC-2")]
fn selected_record_field_uses_witnessed_tuple_key() {
    let tuple = orders("Pair");
    let record = orders("Receipt");
    let mut selected = document(vec![
        json!({
            "identity": tuple, "displayName": "Pair",
            "kind": {"module": IDENTITY, "name": "tuple"},
            "positions": [{"identity": format!("{tuple}/p0"), "typeRef": "ix://quire/native/Boolean"}]
        }),
        typed_object("Receipt", "record", &[], &[field("pair", &tuple)]),
    ]);
    for (kind, meaning) in [
        ("tuple", "quire.meaning.model.tuple-value-type/v1"),
        ("record", "quire.meaning.model.record-value-type/v1"),
    ] {
        selected["constructs"].as_array_mut().unwrap().push(json!({
            "kind": {"module": IDENTITY, "name": kind},
            "construct": {"meaning": meaning}
        }));
    }
    let admitted = admit(
        &selected,
        &[
            ("Pair", "tuple_value_type"),
            ("Receipt", "record_value_type"),
        ],
        &[],
    );
    let tuple_node: CheckedNodeId =
        serde_json::from_value(node_id(&declaration_key("Pair", "tuple_value_type"))).unwrap();
    let record_node: CheckedNodeId =
        serde_json::from_value(node_id(&declaration_key("Receipt", "record_value_type"))).unwrap();
    let tuple_key = NodeKey::new("tuple-stage", vec![PathSegment::Text("Pair".into())], 0).unwrap();
    let record_key =
        NodeKey::new("record-stage", vec![PathSegment::Text("Receipt".into())], 0).unwrap();
    let identities = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [SelectedExactDeclarationRow::new(
            record_node.clone(),
            IDENTITY,
            SelectedExactDeclarationForm::RecordValueType,
            record.clone(),
            SelectedExactIdentity::Record(record_key.clone()),
        )],
        [SelectedExactFieldRow::new(
            record_node.clone(),
            IDENTITY,
            record.clone(),
            format!("{record}/pair"),
            MemberId::new(MemberOwner::Node(record_key.clone()), "pair").unwrap(),
        )],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    let tuple_row =
        SelectedTupleDeclarationRow::new(tuple_node, IDENTITY, tuple, tuple_key.clone());
    let view = admitted
        .project_selected_type_declarations(
            &identities,
            std::slice::from_ref(&tuple_row),
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap();
    assert_eq!(view.tuples()[0].positions(), &[ValueType::Boolean]);
    assert_eq!(
        view.records()[0].view().fields()[0].value_type(),
        &ValueType::Composite(tuple_key)
    );
    let competing = SelectedTupleDeclarationRow::new(
        tuple_row.structural_node.clone(),
        IDENTITY,
        tuple_row.original_identity.clone(),
        record_key,
    );
    let refusal = admitted
        .project_selected_type_declarations(
            &identities,
            &[competing],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
    );
}

#[test]
#[trace("FR-354-AC-7")]
fn selected_record_many_fields_respects_exact_work_boundary() {
    let record = orders("Receipt");
    let mut selected = document(vec![typed_object("Receipt", "record", &[], &[])]);
    selected["constructs"].as_array_mut().unwrap().push(json!({
        "kind": {"module": IDENTITY, "name": "record"},
        "construct": {"meaning": "quire.meaning.model.record-value-type/v1"}
    }));
    let names = (0..128)
        .map(|index| format!("field{index:03}{}", "x".repeat(96)))
        .collect::<Vec<_>>();
    for name in &names {
        selected["types"][0]["fields"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "identity": format!("{record}/{name}"), "name": name,
                "typeRef": "ix://quire/native/Boolean", "presence": "required",
                "nullable": false, "defaultKind": "none",
                "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}
            }));
    }
    let admitted = admit(&selected, &[("Receipt", "record_value_type")], &[]);
    let record_node: CheckedNodeId =
        serde_json::from_value(node_id(&declaration_key("Receipt", "record_value_type"))).unwrap();
    let record_key =
        NodeKey::new("record-stage", vec![PathSegment::Text("r".repeat(4096))], 0).unwrap();
    let declaration = SelectedExactDeclarationRow::new(
        record_node.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::RecordValueType,
        record.clone(),
        SelectedExactIdentity::Record(record_key.clone()),
    );
    let fields = names
        .iter()
        .map(|name| {
            SelectedExactFieldRow::new(
                record_node.clone(),
                IDENTITY,
                record.clone(),
                format!("{record}/{name}"),
                MemberId::new(MemberOwner::Node(record_key.clone()), name.as_str()).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let identities = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [declaration],
        fields,
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    assert_eq!(identities.field_rows().len(), names.len());
    let view = admitted
        .project_selected_type_declarations(
            &identities,
            &[],
            &[],
            30_000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap();
    assert_eq!(view.records()[0].view().fields().len(), names.len());
    assert_eq!(view.records()[0].view().fields()[0].name(), names[0]);
    assert_eq!(view.records()[0].view().fields()[127].name(), names[127]);
    assert!(view.work() < 30_000);
    assert!(admitted
        .project_selected_type_declarations(
            &identities,
            &[],
            &[],
            view.work(),
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        )
        .is_ok());
    let one_less = admitted
        .project_selected_type_declarations(
            &identities,
            &[],
            &[],
            view.work() - 1,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(one_less.code, ExactTypeProjectionErrorCode::ExhaustedWork);
}

#[trace("FR-352-AC-5", "FR-358-AC-3")]
#[test]
fn admitted_selected_declared_unit_projects_only_with_matching_witness() {
    let mut selected = document(vec![typed_object(
        "Receipt",
        "record",
        &[],
        &[field("duration", "ix://quire/unit/acme/orders/second")],
    )]);
    selected["constructs"].as_array_mut().unwrap().push(json!({
        "kind": {"module": IDENTITY, "name": "record"},
        "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
    }));
    selected["dimensions"] = json!([{
        "identity": "ix://acme/orders/dimension/Time", "name": "Time", "base": true, "terms": []
    }]);
    selected["units"] = json!([{
        "identity": "ix://acme/orders/unit/second", "name": "second",
        "dimension": "ix://acme/orders/dimension/Time", "scale": "1", "offset": "0"
    }]);
    let (mut wire, _, _) = package(&selected, &[("Receipt", "record_value_type")], &[]);
    let dimension = structural_id(
        IDENTITY,
        vec![
            json!("scalar_type"),
            json!("dimension"),
            json!("ix://acme/orders/dimension/Time"),
        ],
        0,
    );
    let unit = structural_id(
        IDENTITY,
        vec![
            json!("scalar_type"),
            json!("unit"),
            json!("ix://acme/orders/unit/second"),
        ],
        0,
    );
    let nodes = wire["semantic_graph"]["nodes"].as_array_mut().unwrap();
    nodes.push(json!({
        "node_id": dimension, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "dimension", "semantic_type": dimension,
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "owner": {"kind": "model", "identity": IDENTITY, "node": "ix://acme/orders/dimension/Time"},
        "body": {"term": "aggregate", "members": []}
    }));
    nodes.push(json!({
        "node_id": unit, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "unit", "semantic_type": dimension,
        "dependencies": [dimension], "occurrences": [{"role": "generated", "ordinal": 0}],
        "owner": {"kind": "model", "identity": IDENTITY, "node": "ix://acme/orders/unit/second"},
        "body": {"term": "aggregate", "members": []}
    }));
    let text_type = node_id("scalar_type/text");
    let integer_type = node_id("scalar_type/integer");
    nodes.push(wire_node(
        "scalar_type/text",
        "scalar_type",
        "text",
        "scalar_type/text",
        &[],
        empty(),
    ));
    nodes.push(wire_node(
        "scalar_type/integer",
        "scalar_type",
        "integer",
        "scalar_type/integer",
        &[],
        empty(),
    ));
    let parameter = structural_id(
        "test/families",
        vec![
            json!("value"),
            json!("parameter"),
            json!("duration"),
            json!("0"),
        ],
        0,
    );
    nodes.push(json!({
        "node_id": parameter, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "value", "semantic_form": "parameter", "semantic_type": unit,
        "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}],
        "body": {"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value": {"term": "literal", "type": text_type, "value_kind": "text", "value": "duration"}},
            {"term": "binding", "name": "level", "value": {"term": "literal", "type": integer_type, "value_kind": "integer", "value": "0"}}
        ]}
    }));
    let long_model_identity = format!("aaa/{}", "x".repeat(4096));
    select_model(&mut wire, &long_model_identity);
    let model_root = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .pop()
        .expect("new selected model root");
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .insert(0, model_root);
    settle(&mut wire);
    let evidence = evidence_with_document(&wire, IDENTITY, &selected);
    let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
        other => panic!("selected declared unit fixture must admit: {other:?}"),
    };
    let runtime_refusal = admitted
        .project_selected_type_declarations(
            &SelectedExactIdentities::empty(),
            &[],
            &[],
            1000,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect_err("a unit-bearing package cannot claim a complete runtime type view yet");
    assert_eq!(
        runtime_refusal.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    let unit: CheckedNodeId = serde_json::from_value(unit).unwrap();
    let parameter: CheckedNodeId = serde_json::from_value(parameter).unwrap();
    let record: CheckedNodeId =
        serde_json::from_value(node_id(&declaration_key("Receipt", "record_value_type"))).unwrap();
    let record_key = NodeKey::new("checked", vec![PathSegment::Text("Receipt".into())], 0).unwrap();
    let unit_id = UnitId::declared(
        NodeKey::new("checked", vec![PathSegment::Text("second".into())], 0).unwrap(),
    );
    let record_row = SelectedExactDeclarationRow::new(
        record.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::RecordValueType,
        orders("Receipt"),
        SelectedExactIdentity::Record(record_key.clone()),
    );
    let unit_row = SelectedExactDeclarationRow::new(
        unit.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::DeclaredUnit,
        "ix://acme/orders/unit/second",
        SelectedExactIdentity::DeclaredUnit(unit_id.clone()),
    );
    let mut short_lookup = quire_contract_model::NativeJoinBudget::new(128, u64::MAX);
    let lookup_error = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [unit_row.clone()],
        [],
        &mut short_lookup,
    )
    .expect_err("selected-model identity is charged before the unit lookup scans it");
    assert_eq!(
        lookup_error.code,
        ExactTypeProjectionErrorCode::ExhaustedWork
    );
    assert_eq!(lookup_error.form.as_ref(), "stage_unit_model");
    assert_eq!(lookup_error.at.as_deref(), Some(&unit));
    assert!(short_lookup.first_unpaid().unwrap().amount >= 4096);
    let field_row = SelectedExactFieldRow::new(
        record.clone(),
        IDENTITY,
        orders("Receipt"),
        format!("{}/duration", orders("Receipt")),
        MemberId::new(MemberOwner::Node(record_key.clone()), "duration").unwrap(),
    );

    let missing = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [record_row.clone()],
        [field_row.clone()],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    let missing_error = admitted
        .project_exact_type(&record, &missing, 100)
        .unwrap_err();
    assert_eq!(
        missing_error.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    assert_eq!(missing_error.at.as_deref(), Some(&unit));

    let mut wrong_original = unit_row.clone();
    wrong_original.original_identity = "ix://acme/orders/unit/other".into();
    assert_eq!(
        SelectedExactIdentities::from_stage_rows(
            &admitted,
            [record_row.clone(), wrong_original],
            [field_row.clone()],
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        )
        .unwrap_err()
        .code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let mut wrong_document = unit_row.clone();
    wrong_document.selected_document = "another/document".into();
    assert_eq!(
        SelectedExactIdentities::from_stage_rows(
            &admitted,
            [record_row.clone(), wrong_document],
            [field_row.clone()],
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        )
        .unwrap_err()
        .code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let mut wrong_form = unit_row.clone();
    wrong_form.form = SelectedExactDeclarationForm::RecordValueType;
    assert_eq!(
        SelectedExactIdentities::from_stage_rows(
            &admitted,
            [record_row.clone(), wrong_form],
            [field_row.clone()],
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        )
        .unwrap_err()
        .code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let mut foreign = unit_row.clone();
    foreign.structural_node.package = "foreign".into();
    assert_eq!(
        SelectedExactIdentities::from_stage_rows(
            &admitted,
            [record_row.clone(), foreign],
            [field_row.clone()],
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        )
        .unwrap_err()
        .code,
        ExactTypeProjectionErrorCode::ForeignNode
    );

    let mut shared_key = unit_row.clone();
    shared_key.exact_identity = SelectedExactIdentity::DeclaredUnit(UnitId::declared(record_key));
    let collision = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [record_row.clone(), shared_key],
        [field_row.clone()],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap_err();
    assert_eq!(
        collision.code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
    );
    assert_eq!(collision.at.as_deref(), Some(&unit));

    let witnessed = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [record_row, unit_row],
        [field_row],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    let projected = admitted
        .project_exact_type(&record, &witnessed, 100)
        .unwrap();
    assert_eq!(
        admitted
            .project_exact_type(&parameter, &witnessed, 10)
            .unwrap()
            .value_type(),
        &ValueType::Quantity(unit_id.clone())
    );
    let view = projected.record_view().unwrap();
    assert_eq!(view.fields().len(), 1);
    assert_eq!(view.fields()[0].value_type(), &ValueType::Quantity(unit_id));
    let exhausted = admitted
        .project_exact_type(&record, &witnessed, 4)
        .unwrap_err();
    assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    assert_eq!(exhausted.at.as_deref(), Some(&unit));
    assert_eq!(exhausted.work, 4);
}

/// Complete record projections read every retained effective field, including
/// inherited fields and members with no expression read.
#[trace("FR-352-AC-2")]
#[test]
fn admitted_selected_records_project_complete_exact_views() {
    let mut selected = document(vec![
        value_type("Digit", "integer", Some(("0", "9"))),
        typed_object(
            "Receipt",
            "record",
            &[],
            &[field("total", "ix://quire/native/Integer")],
        ),
        typed_object(
            "Attempt",
            "record",
            &["Receipt"],
            &[field("note", "ix://quire/native/String").optional()],
        ),
        typed_object(
            "Flag",
            "record",
            &["Receipt"],
            &[
                field("total", "ix://quire/native/Boolean")
                    .redefining(&format!("{}/total", orders("Receipt"))),
                field("set", "ix://quire/native/Boolean"),
            ],
        ),
        typed_object("Tick", "record", &[], &[field("count", &orders("Digit"))]),
    ]);
    selected["constructs"]
        .as_array_mut()
        .expect("construct catalog")
        .push(json!({
            "kind": {"module": IDENTITY, "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
        }));
    let declared = [
        ("Receipt", "record_value_type"),
        ("Attempt", "record_value_type"),
        ("Flag", "record_value_type"),
        ("Tick", "record_value_type"),
    ];
    let admitted = admit(&selected, &declared, &[]);
    let mut declaration_rows = Vec::new();
    let mut field_rows = Vec::new();
    let mut records = Vec::new();
    for (short, fields) in [
        ("Receipt", vec![("Receipt", "total")]),
        ("Attempt", vec![("Receipt", "total"), ("Attempt", "note")]),
        ("Flag", vec![("Flag", "total"), ("Flag", "set")]),
        ("Tick", vec![("Tick", "count")]),
    ] {
        let structural: CheckedNodeId =
            serde_json::from_value(node_id(&declaration_key(short, "record_value_type")))
                .expect("structural record ID");
        let key = NodeKey::new("checked", vec![PathSegment::Text(short.into())], 0)
            .expect("Exact record key");
        declaration_rows.push(SelectedExactDeclarationRow::new(
            structural.clone(),
            IDENTITY,
            SelectedExactDeclarationForm::RecordValueType,
            orders(short),
            SelectedExactIdentity::Record(key.clone()),
        ));
        for (owner, name) in fields {
            field_rows.push(SelectedExactFieldRow::new(
                structural.clone(),
                IDENTITY,
                orders(short),
                format!("{}/{name}", orders(owner)),
                MemberId::new(MemberOwner::Node(key.clone()), format!("{owner}/{name}"))
                    .expect("Exact member ID"),
            ));
        }
        records.push((short, structural, key));
    }
    let missing_tick = SelectedExactIdentities::from_stage_rows(
        &admitted,
        declaration_rows.clone(),
        field_rows
            .iter()
            .filter(|row| {
                row.field_original_identity.as_ref() != format!("{}/count", orders("Tick"))
            })
            .cloned(),
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("the other rows join");
    let tick_node = &records[3].1;
    let missing = admitted
        .project_exact_type(tick_node, &missing_tick, 100)
        .expect_err("unwitnessed count refuses the whole view");
    assert_eq!(
        missing.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    assert_eq!(missing.requested.as_ref(), tick_node);
    assert_eq!(missing.at.as_deref(), Some(tick_node));
    assert_eq!(missing.form.as_ref(), "model/record_value_type");
    let mut conflicting_fields = field_rows.clone();
    let duplicate_member_id = conflicting_fields[1].member_id.clone();
    conflicting_fields[2].member_id = duplicate_member_id;
    assert_eq!(
        SelectedExactIdentities::from_stage_rows(
            &admitted,
            declaration_rows.clone(),
            conflicting_fields,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        )
        .expect_err("distinct effective fields cannot share one Exact member")
        .code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
    );

    let witnesses = SelectedExactIdentities::from_stage_rows(
        &admitted,
        declaration_rows,
        field_rows,
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("all selected rows join the admitted package");
    let projections = records
        .iter()
        .map(|(_, structural, key)| {
            let projection = admitted
                .project_exact_type(structural, &witnesses, 100)
                .expect("complete selected record");
            assert_eq!(projection.value_type(), &ValueType::Composite(key.clone()));
            assert_eq!(
                projection.record_view().expect("record view").record_key(),
                key
            );
            projection
        })
        .collect::<Vec<_>>();
    let digit = ValueType::Int(
        quire_exact::IntegerInterval::new(Integer::from(0_i64), Integer::from(9_i64)).unwrap(),
    );
    for (index, expected_fields) in [
        vec![("Receipt", "total", ValueType::Integer, Presence::Required)],
        vec![
            ("Receipt", "total", ValueType::Integer, Presence::Required),
            ("Attempt", "note", ValueType::String, Presence::Optional),
        ],
        vec![
            ("Flag", "total", ValueType::Boolean, Presence::Required),
            ("Flag", "set", ValueType::Boolean, Presence::Required),
        ],
        vec![("Tick", "count", digit, Presence::Required)],
    ]
    .into_iter()
    .enumerate()
    {
        let expected = expected_fields
            .into_iter()
            .map(|(owner, name, ty, presence)| {
                (
                    format!("{}/{name}", orders(owner)),
                    name.to_owned(),
                    MemberId::new(
                        MemberOwner::Node(records[index].2.clone()),
                        format!("{owner}/{name}"),
                    )
                    .unwrap(),
                    ty,
                    presence,
                )
            })
            .collect::<Vec<_>>();
        let actual = projections[index]
            .record_view()
            .unwrap()
            .fields()
            .iter()
            .map(|field| {
                (
                    field.ir_identity().to_owned(),
                    field.name().to_owned(),
                    field.member_id().clone(),
                    field.value_type().clone(),
                    field.presence(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual, expected,
            "complete ordered {} fields",
            records[index].0
        );
        for other in index + 1..records.len() {
            assert_ne!(
                projections[index].value_type(),
                projections[other].value_type()
            );
        }
    }
    assert_eq!(
        projections[0].record_view().expect("Receipt").fields()[0].value_type(),
        &ValueType::Integer,
    );
    assert_eq!(
        projections[2].record_view().expect("Flag").fields()[0].value_type(),
        &ValueType::Boolean,
    );
    assert_eq!(
        projections[2]
            .record_view()
            .expect("Flag")
            .fields()
            .iter()
            .map(|field| field.ir_identity().to_owned())
            .collect::<Vec<_>>(),
        vec![
            format!("{}/total", orders("Flag")),
            format!("{}/set", orders("Flag")),
        ],
    );
    let exhausted = admitted
        .project_exact_type(
            tick_node,
            &witnesses,
            witnesses.field_rows().len() as u64 + 2,
        )
        .expect_err("the field itself must be charged");
    assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    assert_eq!(exhausted.at.as_deref(), Some(tick_node));
    assert_eq!(
        projections[1]
            .record_view()
            .expect("Attempt fields")
            .fields()
            .iter()
            .map(|field| (
                field.ir_identity().to_owned(),
                field.name().to_owned(),
                field.presence()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                format!("{}/total", orders("Receipt")),
                "total".into(),
                Presence::Required
            ),
            (
                format!("{}/note", orders("Attempt")),
                "note".into(),
                Presence::Optional
            ),
        ],
    );
    let tick = projections[3].record_view().expect("Tick fields");
    let count = &tick.fields()[0];
    assert_eq!(count.ir_identity(), format!("{}/count", orders("Tick")));
    assert_eq!(count.name(), "count");
    assert_eq!(
        count.member_id().owner(),
        &MemberOwner::Node(records[3].2.clone())
    );
    assert_eq!(count.presence(), Presence::Required);
    assert_eq!(
        count.value_type(),
        &ValueType::Int(
            quire_exact::IntegerInterval::new(
                "0".parse().expect("lower"),
                "9".parse().expect("upper"),
            )
            .expect("inclusive interval")
        ),
    );
    let shape = tick.field_declarations();
    let missing =
        record(&shape, tick.record_key().clone(), vec![]).expect_err("required count is missing");
    assert_eq!(missing.cause, ConstructionCause::MissingField);
    let ten: Integer = "10".parse().expect("integer");
    let out_of_range = record(
        &shape,
        tick.record_key().clone(),
        vec![(
            count.member_id().clone(),
            FieldValue::Present(ExactValue::Integer(ten)),
        )],
    )
    .expect_err("count exceeds the selected range");
    assert_eq!(out_of_range.cause, ConstructionCause::TypeMismatch);

    let mut foreign = records[0].1.clone();
    foreign.package = "other/package".into();
    assert_eq!(
        admitted
            .project_exact_type(&foreign, &witnesses, 100)
            .expect_err("a similarly named foreign record is not selected")
            .code,
        ExactTypeProjectionErrorCode::ForeignNode,
    );
}

/// An admitted field with no complete Exact representation refuses the whole
/// record before a selected view is returned.
#[test]
fn admitted_record_with_unsupported_field_refuses_without_view() {
    let mut selected = document(vec![typed_object(
        "Unsupported",
        "record",
        &[],
        &[field("value", "ix://quire/native/Float32")],
    )]);
    selected["constructs"]
        .as_array_mut()
        .expect("construct catalog")
        .push(json!({
            "kind": {"module": IDENTITY, "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
        }));
    let admitted = admit(&selected, &[("Unsupported", "record_value_type")], &[]);
    let structural: CheckedNodeId = serde_json::from_value(node_id(&declaration_key(
        "Unsupported",
        "record_value_type",
    )))
    .expect("structural record ID");
    let key = NodeKey::new("checked", vec![PathSegment::Text("Unsupported".into())], 0)
        .expect("Exact record key");
    let witnesses = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [SelectedExactDeclarationRow::new(
            structural.clone(),
            IDENTITY,
            SelectedExactDeclarationForm::RecordValueType,
            orders("Unsupported"),
            SelectedExactIdentity::Record(key.clone()),
        )],
        [SelectedExactFieldRow::new(
            structural.clone(),
            IDENTITY,
            orders("Unsupported"),
            format!("{}/value", orders("Unsupported")),
            MemberId::new(MemberOwner::Node(key), "value").expect("Exact member ID"),
        )],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("selected rows");
    let refusal = admitted
        .project_exact_type(&structural, &witnesses, 100)
        .expect_err("Float32 mode is not represented by this field projection");
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
    assert_eq!(refusal.at.as_deref(), Some(&structural));
}

/// Trusted native rows exercise a late unsupported member and bounded whole-record reads.
#[test]
fn admitted_native_record_late_unsupported_field_respects_exact_work_boundary() {
    // Independently counted: requested node + semantic type + two witness rows + two fields.
    const COMPLETE_WORK: u64 = 6;
    const BEFORE_SECOND_FIELD: u64 = 5;
    for (name, second_type) in [
        ("LateUnsupported", "ix://quire/native/Float32"),
        ("CompleteInteger", "ix://quire/native/Integer"),
    ] {
        let mut selected = document(vec![typed_object(
            name,
            "record",
            &[],
            &[
                field("first", "ix://quire/native/Integer"),
                field("second", second_type),
            ],
        )]);
        selected["constructs"]
            .as_array_mut()
            .expect("construct catalog")
            .push(json!({
                "kind": {"module": IDENTITY, "name": "record"},
                "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
            }));
        let admitted = admit(&selected, &[(name, "record_value_type")], &[]);
        let structural: CheckedNodeId =
            serde_json::from_value(node_id(&declaration_key(name, "record_value_type")))
                .expect("structural record ID");
        let key = NodeKey::new("checked", vec![PathSegment::Text(name.into())], 0)
            .expect("Exact record key");
        let members = ["first", "second"].map(|field_name| {
            MemberId::new(MemberOwner::Node(key.clone()), field_name).expect("Exact field member")
        });
        let field_rows =
            ["first", "second"]
                .into_iter()
                .zip(&members)
                .map(|(field_name, member)| {
                    SelectedExactFieldRow::new(
                        structural.clone(),
                        IDENTITY,
                        orders(name),
                        format!("{}/{field_name}", orders(name)),
                        member.clone(),
                    )
                });
        let witnesses = SelectedExactIdentities::from_stage_rows(
            &admitted,
            [SelectedExactDeclarationRow::new(
                structural.clone(),
                IDENTITY,
                SelectedExactDeclarationForm::RecordValueType,
                orders(name),
                SelectedExactIdentity::Record(key.clone()),
            )],
            field_rows,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect("both native field witnesses join this independently admitted record");
        let cloned = admitted.clone();
        drop(selected);
        let exhausted = quire_contract_model::ExactTypeProjectionError {
            code: ExactTypeProjectionErrorCode::ExhaustedWork,
            requested: Box::new(structural.clone()),
            at: Some(Box::new(structural.clone())),
            form: "record_field".into(),
            work: BEFORE_SECOND_FIELD,
        };
        for package in [&admitted, &cloned] {
            for _ in 0..2 {
                assert_eq!(
                    package.project_exact_type(&structural, &witnesses, BEFORE_SECOND_FIELD),
                    Err(exhausted.clone()),
                    "the second field must consume work before its leaf is inspected",
                );
            }
        }
        if name == "LateUnsupported" {
            let unsupported = quire_contract_model::ExactTypeProjectionError {
                code: ExactTypeProjectionErrorCode::UnsupportedExactType,
                requested: Box::new(structural.clone()),
                at: Some(Box::new(structural.clone())),
                form: "model/record_value_type".into(),
                work: COMPLETE_WORK,
            };
            for limit in [COMPLETE_WORK, 100] {
                assert_eq!(
                    admitted.project_exact_type(&structural, &witnesses, limit),
                    Err(unsupported.clone()),
                );
            }
            drop(admitted);
            for limit in [COMPLETE_WORK, 100] {
                for _ in 0..2 {
                    assert_eq!(
                        cloned.project_exact_type(&structural, &witnesses, limit),
                        Err(unsupported.clone()),
                        "a late unsupported field refuses the entire result",
                    );
                }
            }
        } else {
            let projected = admitted
                .project_exact_type(&structural, &witnesses, COMPLETE_WORK)
                .expect("both Integer fields fit the independently counted allowance");
            assert_eq!(projected.node(), &structural);
            assert_eq!(projected.type_node(), &structural);
            assert_eq!(projected.work(), COMPLETE_WORK);
            assert_eq!(projected.value_type(), &ValueType::Composite(key.clone()));
            let view = projected.record_view().expect("complete record view");
            assert_eq!(view.record_key(), &key);
            let actual = view
                .fields()
                .iter()
                .map(|field| {
                    (
                        field.ir_identity().to_owned(),
                        field.name().to_owned(),
                        field.member_id().clone(),
                        field.presence(),
                        field.value_type().clone(),
                    )
                })
                .collect::<Vec<_>>();
            let expected = ["first", "second"]
                .into_iter()
                .zip(members)
                .map(|(field_name, member)| {
                    (
                        format!("{}/{field_name}", orders(name)),
                        field_name.to_owned(),
                        member,
                        Presence::Required,
                        ValueType::Integer,
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, expected);
            assert_eq!(
                admitted.project_exact_type(&structural, &witnesses, COMPLETE_WORK),
                Ok(projected.clone())
            );
            drop(admitted);
            for _ in 0..2 {
                assert_eq!(
                    cloned.project_exact_type(&structural, &witnesses, COMPLETE_WORK),
                    Ok(projected.clone())
                );
            }
        }
    }
}

/// A selected record's nested record and object-reference fields resolve by
/// admitted target declaration and consume work at each target.
#[test]
fn admitted_nested_record_and_reference_fields_require_target_witnesses_and_work() {
    let mut selected = document(vec![
        typed_object("Address", "record", &[], &[]),
        object("Person", &[], &[]),
        typed_object(
            "Envelope",
            "record",
            &[],
            &[
                field("address", &orders("Address")),
                field("person", &orders("Person")),
            ],
        ),
    ]);
    selected["constructs"]
        .as_array_mut()
        .expect("construct catalog")
        .push(json!({
            "kind": {"module": IDENTITY, "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
        }));
    let admitted = admit(
        &selected,
        &[
            ("Address", "record_value_type"),
            ("Person", OBJECT),
            ("Envelope", "record_value_type"),
        ],
        &[],
    );
    let structural = |short, form| -> CheckedNodeId {
        serde_json::from_value(node_id(&declaration_key(short, form)))
            .expect("admitted structural declaration")
    };
    let address = structural("Address", "record_value_type");
    let person = structural("Person", OBJECT);
    let envelope = structural("Envelope", "record_value_type");
    let key = |short: &str| {
        NodeKey::new("checked", vec![PathSegment::Text(short.into())], 0)
            .expect("Exact declaration key")
    };
    let address_key = key("Address");
    let envelope_key = key("Envelope");
    let person_id = EffectiveId::new(
        None,
        OriginalDeclarationKey::new(IDENTITY, orders("Person")),
    );
    let address_row = SelectedExactDeclarationRow::new(
        address.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::RecordValueType,
        orders("Address"),
        SelectedExactIdentity::Record(address_key.clone()),
    );
    let person_row = SelectedExactDeclarationRow::new(
        person.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::ObjectType,
        orders("Person"),
        SelectedExactIdentity::Object(person_id.clone()),
    );
    let envelope_row = SelectedExactDeclarationRow::new(
        envelope.clone(),
        IDENTITY,
        SelectedExactDeclarationForm::RecordValueType,
        orders("Envelope"),
        SelectedExactIdentity::Record(envelope_key.clone()),
    );
    let field_rows = ["address", "person"].map(|name| {
        SelectedExactFieldRow::new(
            envelope.clone(),
            IDENTITY,
            orders("Envelope"),
            format!("{}/{name}", orders("Envelope")),
            MemberId::new(MemberOwner::Node(envelope_key.clone()), name)
                .expect("Exact field member"),
        )
    });
    let complete = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [
            address_row.clone(),
            person_row.clone(),
            envelope_row.clone(),
        ],
        field_rows.clone(),
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .expect("selected target and field rows");
    let projection = admitted
        .project_exact_type(&envelope, &complete, 100)
        .expect("complete nested field types");
    let fields = projection.record_view().expect("record view").fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].value_type(), &ValueType::Composite(address_key));
    assert_eq!(fields[1].value_type(), &ValueType::Reference(person_id));

    for (declarations, failed_target) in [
        (vec![person_row.clone(), envelope_row.clone()], &address),
        (vec![address_row, envelope_row], &person),
    ] {
        let missing = SelectedExactIdentities::from_stage_rows(
            &admitted,
            declarations,
            field_rows.clone(),
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .expect("incomplete target rows retain valid field joins");
        let refusal = admitted
            .project_exact_type(&envelope, &missing, 100)
            .expect_err("missing target witness refuses the complete view");
        assert_eq!(
            refusal.code,
            ExactTypeProjectionErrorCode::UnavailableIdentityWitness
        );
        assert_eq!(refusal.at.as_deref(), Some(failed_target));
    }
    for (limit, failed_target) in [(5, &address), (7, &person)] {
        let refusal = admitted
            .project_exact_type(&envelope, &complete, limit)
            .expect_err("nested target read must consume the remaining work");
        assert_eq!(refusal.code, ExactTypeProjectionErrorCode::ExhaustedWork);
        assert_eq!(refusal.at.as_deref(), Some(failed_target));
        assert_eq!(refusal.work, limit);
    }
}

pub(crate) fn record_project_control() -> (Value, CheckedPackageEvidence) {
    let document = document(vec![object(
        ORDER,
        &[],
        &[field("string", "ix://quire/native/String")],
    )]);
    let read = Read {
        owner: ORDER,
        field: "string",
        ty: Ty::String,
    };
    let (package, evidence, _) = self::package(&document, &[(ORDER, OBJECT)], &[read]);
    (package, evidence)
}

/// Trace: FR-349-AC-8, QSpec FR-322-AC-34.
#[trace("FR-349-AC-8", "FR-322-AC-34")]
#[test]
fn model_field_reference_without_deref_refuses_by_read_form() {
    let (mut wire, evidence) = record_project_control();
    let projection = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.record.project")
        .expect("model projection");
    let declaration = projection["body"]["operation"]["member"]["declaration"].clone();
    projection["body"]["arguments"][0]["target"] = declaration;
    settle(&mut wire);
    let result = read_with(&wire, &evidence, CheckedPackageReadLimits::bounded());
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Refused(refusal)
        if refusal.code == CheckedPackageRefusalCode::MalformedWire)
    );

    let projection = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.record.project")
        .expect("model projection");
    projection["body"]["operation"]["member"]
        .as_object_mut()
        .expect("field member")
        .remove("read_form");
    settle(&mut wire);
    let result = read_with(&wire, &evidence, CheckedPackageReadLimits::bounded());
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Refused(refusal)
        if refusal.code == CheckedPackageRefusalCode::IllTyped
            && refusal.cause == Some(CheckedPackageRefusalCause::OperatorIneligible))
    );
}

/// An admitted package holding an `integer.add` application and a
/// `value`/`literal` node, for refusals that need neither a model member
/// nor a relationship.
pub(crate) fn integer_add_control() -> CheckedPackageV2 {
    let argument = |placeholder: char| json!({"term": "reference", "target": node_id(&format!("placeholder/{placeholder}"))});
    scalar_operand_package(vec![argument('a'), argument('c')]).0
}

fn declaration_id(short: &str) -> CheckedNodeId {
    serde_json::from_value(node_id(&declaration_key(short, OBJECT))).expect("a node id")
}

fn fields_of(package: &CheckedPackageV2, short: &str) -> CheckedModelObjectFields {
    package
        .model_object_fields(&declaration_id(short))
        .expect("the object's fields")
}

/// Repeated public reads over reader-admitted packages preserve the exact
/// field result and structural package name as the graph grows.
#[trace("TC-227", "FR-038-AC-139", "FR-038-AC-144")]
#[test]
fn admitted_model_field_queries_preserve_results_across_graph_sizes() {
    for models in [4, 8, 16] {
        let names: Vec<String> = (0..models).map(|number| format!("R{number}")).collect();
        let types = names.iter().map(|name| object(name, &[], &[])).collect();
        let declarations: Vec<_> = names.iter().map(|name| (name.as_str(), OBJECT)).collect();
        let admitted = admit(&document(types), &declarations, &[]);
        let selected = names.last().expect("selected model");
        let expected = fields_of(&admitted, selected);
        let identity = admitted.name().to_owned();
        let sources = admitted.source_map().to_vec();
        for requests in [2, 4, 8] {
            for _ in 0..requests {
                assert_eq!(fields_of(&admitted, selected), expected);
            }
        }
        assert_eq!(admitted.name(), identity);
        assert_eq!(admitted.source_map(), sources);
    }
}

const ORDER: &str = "Order";

fn range(lower: i128, upper: i128) -> CheckedMemberType {
    CheckedMemberType::IntRange { lower, upper }
}

/// A small admitted graph for the public scalar operand API. The integer
/// bounds and application key use the same independently built preimages as
/// this module's model-field cases.
fn scalar_operand_package(arguments: Vec<Value>) -> (CheckedPackageV2, CheckedNodeId) {
    scalar_operand_package_for("quire.op.integer.add", "binary", arguments)
}

fn scalar_operand_package_for(
    operation: &str,
    operator: &str,
    arguments: Vec<Value>,
) -> (CheckedPackageV2, CheckedNodeId) {
    scalar_operand_package_with_bounds(operation, operator, arguments, Some(("0", "1000")))
}

fn scalar_operand_package_with_bounds(
    operation: &str,
    operator: &str,
    arguments: Vec<Value>,
    bounds: Option<(&str, &str)>,
) -> (CheckedPackageV2, CheckedNodeId) {
    scalar_operand_package_with_literal(operation, operator, arguments, bounds, "7")
}

fn scalar_operand_package_with_literal(
    operation: &str,
    operator: &str,
    arguments: Vec<Value>,
    bounds: Option<(&str, &str)>,
    literal_value: &str,
) -> (CheckedPackageV2, CheckedNodeId) {
    scalar_operand_package_named(
        operation,
        operator,
        arguments,
        bounds,
        literal_value,
        ["left", "right"],
    )
}

/// As [`scalar_operand_package_with_literal`], naming the two parameters
/// that the `a` and `b` argument placeholders reference.
fn scalar_operand_package_named(
    operation: &str,
    operator: &str,
    mut arguments: Vec<Value>,
    bounds: Option<(&str, &str)>,
    literal_value: &str,
    names: [&str; 2],
) -> (CheckedPackageV2, CheckedNodeId) {
    let mut nodes = BTreeMap::new();
    let integer = type_nodes(&Ty::Integer, &mut nodes);
    let bounded = if let Some((lower, upper)) = bounds {
        let body = bounds_body(&integer, lower, upper);
        let key = format!("bounded_domain/integer_range/{lower}/{upper}");
        nodes.insert(
            key.clone(),
            wire_node(
                &key,
                "bounded_domain",
                "integer_range",
                &integer,
                &[&integer],
                body,
            ),
        );
        key
    } else {
        integer.clone()
    };
    let text = "scalar_type/text".to_owned();
    nodes.insert(
        text.clone(),
        wire_node(&text, "scalar_type", "text", &text, &[], empty()),
    );
    let mut parameter_keys = Vec::new();
    for name in names {
        let body = json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value":
                {"term": "literal", "type": node_id(&text), "value_kind": "text", "value": name}},
            {"term": "binding", "name": "level", "value":
                {"term": "literal", "type": node_id(&integer), "value_kind": "integer", "value": "0"}},
        ]});
        let key = format!("value/parameter/{name}/{bounded}");
        let mut node = wire_node(&key, "value", "parameter", &bounded, &[], body);
        node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
        nodes.insert(key.clone(), node);
        parameter_keys.push(key);
    }
    let literal_body = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": literal_value});
    let literal_key = format!("value/literal/{literal_value}/{bounded}");
    let mut literal_node = wire_node(
        &literal_key,
        "value",
        "literal",
        &bounded,
        &[],
        literal_body,
    );
    literal_node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    nodes.insert(literal_key.clone(), literal_node);
    for argument in &mut arguments {
        let Some(target) = argument.pointer_mut("/target/path/0") else {
            continue;
        };
        let replacement = match target.as_str() {
            Some("placeholder/a") => Some(&parameter_keys[0]),
            Some("placeholder/b") => Some(&parameter_keys[1]),
            Some("placeholder/c") => Some(&literal_key),
            _ => None,
        };
        if let Some(replacement) = replacement {
            *target = json!(replacement);
        }
    }
    if let Some(position) = arguments.iter().position(|argument| argument == "nested") {
        let inline = json!({"term": "literal", "type": node_id(&integer),
            "value_kind": "integer", "value": "7"});
        let nested_body = json!({
            "term": "application", "operator": "binary",
            "operation": {"identity": "quire.op.integer.add", "laws": [],
                "mode": null, "member": null, "leaves": []},
            "result_type": node_id(&bounded), "arguments": [inline.clone(), inline],
        });
        let nested_key = "expression/binary/nested".to_owned();
        let mut nested = wire_node(
            &nested_key,
            "expression",
            "binary",
            &bounded,
            &[],
            nested_body,
        );
        nested["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
        nodes.insert(nested_key.clone(), nested);
        arguments[position] = reference(&nested_key);
    }
    let body = json!({
        "term": "application", "operator": operator,
        "operation": {"identity": operation, "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": node_id(&bounded), "arguments": arguments,
    });
    let key = format!("expression/{operator}/root");
    let mut application = wire_node(&key, "expression", operator, &bounded, &[], body);
    application["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let mut package = nominal_package(&[]);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes.into_values());
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(application);
    crate::support::checked_package::settle(&mut package);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .last_mut()
        .expect("application")["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"role": "expression", "ordinal": 1}));
    let mut second_source = package["source_map"]
        .as_array()
        .expect("source map")
        .last()
        .expect("application source")
        .clone();
    second_source["ordinal"] = json!(1);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second_source);
    let application_id: CheckedNodeId = serde_json::from_value(
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .last()
            .expect("application")["node_id"]
            .clone(),
    )
    .expect("node id");
    let evidence = evidence_for(&package);
    let admitted = match read_with(&package, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected scalar fixture admission, got {other:?}"),
    };
    (admitted, application_id)
}

#[trace("TC-048", "FR-038-AC-261")]
#[test]
fn tc_048_parameter_declarations_follow_scalar_operand_order() {
    use quire_contract_model::{
        CheckedOccurrence, CheckedOccurrenceRole, CheckedParameterDeclarationError,
        CheckedScalarOperandChild,
    };
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let argument = |placeholder: char| json!({"term": "reference", "target": node_id(&format!("placeholder/{placeholder}"))});
    let declared = |arguments: Vec<Value>| {
        let (package, application) = scalar_operand_package_named(
            "quire.op.integer.add",
            "binary",
            arguments,
            Some(("0", "9")),
            "7",
            ["a", "b"],
        );
        package
            .scalar_application_operands(&application, &occurrence)
            .expect("Int[0, 9] operands are finite")
            .into_iter()
            .map(|operand| {
                assert_eq!((operand.range.lower, operand.range.upper), (0, 9));
                let CheckedScalarOperandChild::GraphChild(id) = operand.child else {
                    panic!("a parameter operand is a graph child");
                };
                let declaration = package.parameter_declaration(&id).expect("declaration");
                assert_eq!(declaration.node, id);
                assert_eq!(declaration.level.as_ref(), "0");
                declaration.name.to_string()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(declared(vec![argument('a'), argument('b')]), ["a", "b"]);
    assert_eq!(declared(vec![argument('b'), argument('a')]), ["b", "a"]);

    // An inline literal operand carries no operand node id: its only id is the
    // containing application's, which is not a parameter.
    let (package, application) = scalar_operand_package_named(
        "quire.op.integer.add",
        "binary",
        vec![
            argument('a'),
            json!({"term": "literal", "type": node_id("scalar_type/integer"),
                "value_kind": "integer", "value": "7"}),
        ],
        Some(("0", "9")),
        "7",
        ["a", "b"],
    );
    let operands = package
        .scalar_application_operands(&application, &occurrence)
        .expect("operands");
    let CheckedScalarOperandChild::InlineLiteral {
        application: carried,
        occurrence: at,
        ordinal,
    } = &operands[1].child
    else {
        panic!("the literal operand is inline");
    };
    assert_eq!((carried, at, *ordinal), (&application, &occurrence, 1));
    assert_eq!(
        package.parameter_declaration(carried),
        Err(CheckedParameterDeclarationError::NotParameter)
    );
}

/// The child's independently assigned structural path: the public accessor
/// test does not inspect the admitted graph body to find its identity.
fn scalar_fixture_child_id(form: &str, value: &str) -> Value {
    assert!(matches!(form, "parameter" | "literal"));
    node_id(&format!(
        "value/{form}/{value}/bounded_domain/integer_range/0/1000"
    ))
}

#[trace(
    "TC-048",
    "FR-038-AC-159",
    "FR-038-AC-160",
    "FR-038-AC-161",
    "FR-038-AC-164"
)]
#[test]
fn tc_048_scalar_operands_keep_argument_identity_and_exact_range() {
    use quire_contract_model::{
        CheckedOccurrence, CheckedOccurrenceRole, CheckedScalarOperand, CheckedScalarOperandChild,
        CheckedScalarOperandError,
    };
    fn consume_child(entry: &CheckedScalarOperand) -> (u64, i128, i128, CheckedNodeId) {
        let node = match &entry.child {
            CheckedScalarOperandChild::GraphChild(node) => node.clone(),
            CheckedScalarOperandChild::InlineLiteral {
                application,
                occurrence,
                ordinal,
            } => {
                assert_eq!(*ordinal, entry.ordinal);
                assert_eq!(occurrence.role, CheckedOccurrenceRole::Expression);
                application.clone()
            }
        };
        (entry.ordinal, entry.range.lower, entry.range.upper, node)
    }
    fn consume_error(error: CheckedScalarOperandError) -> &'static str {
        match error {
            CheckedScalarOperandError::UnknownNode => "unknown node",
            CheckedScalarOperandError::NotApplication => "not application",
            CheckedScalarOperandError::MissingOccurrence => "missing occurrence",
            CheckedScalarOperandError::UnknownOperator => "unknown operator",
            CheckedScalarOperandError::IneligibleOperator => "ineligible operator",
            CheckedScalarOperandError::MissingChild => "missing child",
            CheckedScalarOperandError::MissingRange => "missing range",
            CheckedScalarOperandError::UnboundedRange => "unbounded range",
            CheckedScalarOperandError::RangeOutOfI128 => "out of i128",
        }
    }
    let first = node_id("placeholder/a");
    let integer = "scalar_type/integer".to_owned();
    let inline = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "7"});
    let (package, application) =
        scalar_operand_package(vec![json!({"term": "reference", "target": first}), inline]);
    let first = scalar_fixture_child_id("parameter", "left");
    let second = scalar_fixture_child_id("parameter", "right");
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let before = package.clone();
    let entries = package
        .scalar_application_operands(&application, &occurrence)
        .expect("typed operands");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].ordinal, 0);
    assert_eq!(entries[0].range.lower, 0);
    assert_eq!(entries[0].range.upper, 1000);
    assert_eq!(entries[1].ordinal, 1);
    assert_eq!(consume_child(&entries[1]).0, 1);
    assert_eq!((entries[1].range.lower, entries[1].range.upper), (7, 7));
    assert!(matches!(
        &entries[0].child,
        CheckedScalarOperandChild::GraphChild(id) if *id == serde_json::from_value(first).expect("id")
    ));
    assert!(matches!(
        &entries[1].child,
        CheckedScalarOperandChild::InlineLiteral { application: id, occurrence: at, ordinal: 1 }
            if id == &application && at == &occurrence
    ));
    assert_eq!(
        entries,
        package
            .clone()
            .scalar_application_operands(&application, &occurrence)
            .expect("repeat")
    );
    for requests in [2, 4, 8] {
        for _ in 0..requests {
            assert_eq!(
                package.scalar_application_operands(&application, &occurrence),
                Ok(entries.clone())
            );
        }
    }
    assert_eq!(package, before);
    assert_eq!(
        consume_error(
            package
                .scalar_application_operands(
                    &application,
                    &CheckedOccurrence {
                        role: CheckedOccurrenceRole::Expression,
                        ordinal: 2,
                    },
                )
                .expect_err("no third occurrence"),
        ),
        "missing occurrence"
    );

    let second_occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 1,
    };
    let second_entries = package
        .scalar_application_operands(&application, &second_occurrence)
        .expect("second occurrence");
    assert!(matches!(&second_entries[1].child,
        CheckedScalarOperandChild::InlineLiteral { application: id, occurrence: at, ordinal: 1 }
        if id == &application && at == &second_occurrence));
    assert_ne!(second_entries[1].child, entries[1].child);

    let (swapped, swapped_id) = scalar_operand_package(vec![
        json!({"term": "reference", "target": second}),
        json!({"term": "reference", "target": node_id("placeholder/a")}),
    ]);
    let result = swapped
        .scalar_application_operands(&swapped_id, &occurrence)
        .expect("swapped");
    assert_eq!(result.len(), 2);
    assert!(
        matches!(&result[0].child, CheckedScalarOperandChild::GraphChild(id)
        if *id == serde_json::from_value(second).expect("id"))
    );
    assert_ne!(result[0].child, result[1].child);
    let (forward, forward_id) = scalar_operand_package(vec![
        json!({"term": "reference", "target": node_id("placeholder/a")}),
        json!({"term": "reference", "target": node_id("placeholder/b")}),
    ]);
    let forward = forward
        .scalar_application_operands(&forward_id, &occurrence)
        .expect("forward");
    assert_eq!(forward[0].child, result[1].child);
    assert_eq!(forward[1].child, result[0].child);

    let graph_literal = node_id("placeholder/c");
    let (package, id) = scalar_operand_package(vec![
        json!({"term": "reference", "target": graph_literal.clone()}),
        json!({"term": "reference", "target": node_id("placeholder/a")}),
    ]);
    let graph_literal = scalar_fixture_child_id("literal", "7");
    let entries = package
        .scalar_application_operands(&id, &occurrence)
        .expect("literal child");
    assert_eq!((entries[0].range.lower, entries[0].range.upper), (7, 7));
    assert!(
        matches!(&entries[0].child, CheckedScalarOperandChild::GraphChild(child)
        if *child == serde_json::from_value(graph_literal).expect("id"))
    );

    let (package, outer) = scalar_operand_package(vec![
        json!("nested"),
        json!({"term": "literal", "type": node_id(&integer),
            "value_kind": "integer", "value": "7"}),
    ]);
    let inner = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "binary" && node.node_id != outer)
        .expect("inner application");
    let entries = package
        .scalar_application_operands(&outer, &occurrence)
        .expect("nested child");
    assert!(
        matches!(&entries[0].child, CheckedScalarOperandChild::GraphChild(id)
        if *id == inner.node_id)
    );
    assert_eq!((entries[0].range.lower, entries[0].range.upper), (0, 1000));
}

#[trace("TC-048", "FR-038-AC-161", "FR-038-AC-162")]
#[test]
fn tc_048_scalar_operands_refuse_unknown_node_nonapplication_and_missing_occurrence() {
    use quire_contract_model::{
        CheckedOccurrence, CheckedOccurrenceRole, CheckedScalarOperandError,
    };
    let integer = "scalar_type/integer".to_owned();
    let inline = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "7"});
    let (package, application) = scalar_operand_package(vec![inline.clone(), inline]);
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let missing = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 2,
    };
    let operands = package
        .scalar_application_operands(&application, &occurrence)
        .expect("two inline terms");
    assert_ne!(operands[0].child, operands[1].child);
    assert_eq!(
        package.scalar_application_operands(&application, &missing),
        Err(CheckedScalarOperandError::MissingOccurrence)
    );
    assert_eq!(
        package.scalar_application_operands(
            &serde_json::from_value(node_id(&integer)).expect("id"),
            &occurrence
        ),
        Err(CheckedScalarOperandError::NotApplication)
    );
    assert_eq!(
        package.scalar_application_operands(
            &serde_json::from_value(node_id(&"f".repeat(64))).expect("id"),
            &occurrence
        ),
        Err(CheckedScalarOperandError::UnknownNode)
    );
}

#[trace("TC-048", "FR-038-AC-159", "FR-038-AC-163")]
#[test]
fn tc_048_scalar_operand_eligibility_uses_catalogued_integer_operations() {
    use quire_contract_model::{
        CheckedOccurrence, CheckedOccurrenceRole, CheckedScalarOperandError,
    };
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let integer = "scalar_type/integer".to_owned();
    let literal = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "7"});
    for (identity, operator, count) in [
        ("quire.op.integer.add", "binary", 2),
        ("quire.op.integer.sub", "binary", 2),
        ("quire.op.integer.mul", "binary", 2),
        ("quire.op.integer.negate", "unary", 1),
    ] {
        let (package, node) =
            scalar_operand_package_for(identity, operator, vec![literal.clone(); count]);
        let entries = package
            .scalar_application_operands(&node, &occurrence)
            .expect(identity);
        assert_eq!(entries.len(), count, "{identity}");
        assert!(entries.iter().enumerate().all(|(position, entry)| {
            entry.ordinal == u64::try_from(position).expect("small fixture")
                && entry.range.lower == 7
                && entry.range.upper == 7
        }));
    }
    let (package, node) = scalar_operand_package_for(
        "quire.op.integer.mod",
        "binary",
        vec![literal.clone(), literal],
    );
    assert_eq!(
        package.scalar_application_operands(&node, &occurrence),
        Err(CheckedScalarOperandError::IneligibleOperator)
    );
}

#[trace("TC-048", "FR-038-AC-162")]
#[test]
fn tc_048_scalar_operands_refuse_unbounded_and_out_of_i128_ranges() {
    use quire_contract_model::{
        CheckedOccurrence, CheckedOccurrenceRole, CheckedScalarOperandError,
    };
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let first = json!({"term": "reference", "target": node_id("placeholder/a")});
    let integer = "scalar_type/integer".to_owned();
    let inline = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "7"});
    let (unbounded, node) = scalar_operand_package_with_bounds(
        "quire.op.integer.add",
        "binary",
        vec![first.clone(), inline.clone()],
        None,
    );
    assert_eq!(
        unbounded.scalar_application_operands(&node, &occurrence),
        Err(CheckedScalarOperandError::UnboundedRange)
    );

    for (lower, upper) in [
        ("0", "170141183460469231731687303715884105728"),
        ("-170141183460469231731687303715884105729", "0"),
    ] {
        let (package, node) = scalar_operand_package_with_bounds(
            "quire.op.integer.add",
            "binary",
            vec![first.clone(), inline.clone()],
            Some((lower, upper)),
        );
        assert_eq!(
            package.scalar_application_operands(&node, &occurrence),
            Err(CheckedScalarOperandError::RangeOutOfI128),
            "{lower}..={upper}"
        );
    }

    let beyond = "170141183460469231731687303715884105728";
    let large_inline = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": beyond});
    let (package, node) = scalar_operand_package(vec![large_inline, inline]);
    assert_eq!(
        package.scalar_application_operands(&node, &occurrence),
        Err(CheckedScalarOperandError::RangeOutOfI128)
    );

    let (package, node) = scalar_operand_package_with_literal(
        "quire.op.integer.add",
        "binary",
        vec![
            json!({"term": "reference", "target": node_id("placeholder/c")}),
            json!({"term": "literal", "type": node_id(&integer),
                "value_kind": "integer", "value": "7"}),
        ],
        None,
        beyond,
    );
    assert_eq!(
        package.scalar_application_operands(&node, &occurrence),
        Err(CheckedScalarOperandError::RangeOutOfI128)
    );

    let minimum = i128::MIN.to_string();
    let maximum = i128::MAX.to_string();
    let (package, node) = scalar_operand_package_with_bounds(
        "quire.op.integer.add",
        "binary",
        vec![
            first,
            json!({"term": "literal", "type": node_id(&integer),
            "value_kind": "integer", "value": "7"}),
        ],
        Some((&minimum, &maximum)),
    );
    let entries = package
        .scalar_application_operands(&node, &occurrence)
        .expect("i128 edges");
    assert_eq!(
        (entries[0].range.lower, entries[0].range.upper),
        (i128::MIN, i128::MAX)
    );
}

fn names(fields: &CheckedModelObjectFields) -> Vec<&str> {
    fields.fields().iter().map(|field| field.name()).collect()
}

#[trace("TC-048", "FR-038-AC-242")]
#[test]
fn selected_native_fields_keep_distinct_member_types() {
    let document = document(vec![object(
        ORDER,
        &[],
        &[
            field("string", "ix://quire/native/String"),
            field("uuid", "ix://quire/native/UUID"),
            field("timestamp", "ix://quire/native/Timestamp"),
        ],
    )]);
    let reads = [
        Read {
            owner: ORDER,
            field: "string",
            ty: Ty::String,
        },
        Read {
            owner: ORDER,
            field: "uuid",
            ty: Ty::Uuid,
        },
        Read {
            owner: ORDER,
            field: "timestamp",
            ty: Ty::Timestamp,
        },
    ];
    let admitted = admit(&document, &[(ORDER, OBJECT)], &reads);
    let fields = fields_of(&admitted, ORDER);
    for (name, expected, form) in [
        ("string", CheckedMemberType::String, "string"),
        ("uuid", CheckedMemberType::Uuid, "uuid"),
        ("timestamp", CheckedMemberType::Timestamp, "timestamp"),
    ] {
        assert_eq!(
            fields.field(name).and_then(|field| field.member_type()),
            Some(&expected)
        );
        let scalar = admitted
            .graph()
            .nodes
            .iter()
            .find(|node| {
                node.node_tag.as_ref() == "scalar_type" && node.semantic_form.as_ref() == form
            })
            .expect("native scalar node");
        assert_eq!(scalar.node_id, scalar.semantic_type);
    }
}

// --- AC-136 -----------------------------------------------------------------------

/// A domain document declaring `balance` and `audit`, each `Int[0, 1000]`,
/// and a read of `balance` only: both fields come back, ascending by name,
/// with `i128` bounds, although no read names `audit`; an absent field is
/// `None`, not an error.
#[trace("TC-227", "FR-038-AC-136")]
#[test]
fn tc_227_the_fields_of_a_model_object_come_back_as_values() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object(
            ORDER,
            &[],
            &[field("balance", &thousand), field("audit", &thousand)],
        ),
    ]);
    let read = Read {
        owner: ORDER,
        field: "balance",
        ty: Ty::Range(0, 1000),
    };
    let package = admit(&document, &[(ORDER, OBJECT)], &[read]);
    let fields = fields_of(&package, ORDER);
    assert_eq!(names(&fields), ["balance", "audit"]);
    for field in fields.fields() {
        assert_eq!(field.member_type(), Some(&range(0, 1000)));
    }
    let audit = fields
        .field("audit")
        .expect("audit is a field no read names");
    assert_eq!(audit.name(), "audit");
    assert!(fields.field("missing").is_none());
    // Authored declaration order is retained: `zeta` is declared first.
    let document = self::document(vec![
        thousand_type(),
        object(
            ORDER,
            &[],
            &[field("zeta", &thousand), field("alpha", &thousand)],
        ),
    ]);
    let package = admit(&document, &[(ORDER, OBJECT)], &[]);
    assert_eq!(names(&fields_of(&package, ORDER)), ["zeta", "alpha"]);
}

fn thousand_type() -> Value {
    thousand()
}

// --- AC-137 and AC-141 -----------------------------------------------------------

/// The package of `document` with `Order`'s field `name` read at `ty`:
/// whether it admits, and when refused the refusal's code, cause and path.
fn read_of(
    document: &Value,
    declared: &[(&str, &str)],
    owner: &'static str,
    name: &'static str,
    ty: Ty,
) -> Result<
    (),
    (
        CheckedPackageRefusalCode,
        Option<CheckedPackageRefusalCause>,
        String,
    ),
> {
    let read = Read {
        owner,
        field: name,
        ty,
    };
    let (package, evidence, positions) = package(document, declared, &[read]);
    match read_with(&package, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(_) => Ok(()),
        CheckedPackageV2ReadResult::Refused(refusal) => {
            let at = positions.first().copied().expect("a read position");
            let result_type = format!("/semantic_graph/nodes/{at}/body/result_type");
            let path = refusal.loci[0].path.as_ref().map(ToString::to_string);
            assert_eq!(path.as_deref(), Some(result_type.as_str()), "{refusal:?}");
            Err((refusal.code, refusal.cause, result_type))
        }
        other => panic!("expected admission or a refusal, got {other:?}"),
    }
}

const INELIGIBLE: (
    CheckedPackageRefusalCode,
    Option<CheckedPackageRefusalCause>,
) = (
    CheckedPackageRefusalCode::IllTyped,
    Some(CheckedPackageRefusalCause::OperatorIneligible),
);

/// An unknown field on a selected model record retains record.project's
/// expression refusal, separate from frame name resolution.
#[trace("TC-056", "FR-040-AC-14")]
#[test]
fn tc_056_record_projection_unknown_field_remains_ineligible() {
    let thousand = orders("Thousand");
    let mut document = document(vec![
        thousand_type(),
        typed_object("Address", "record", &[], &[field("number", &thousand)]),
    ]);
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": IDENTITY, "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
        }));
    let declared = [("Address", "record_value_type")];
    let (package, evidence, positions) = package(
        &document,
        &declared,
        &[Read {
            owner: "Address",
            field: "ghost",
            ty: Ty::Range(0, 1000),
        }],
    );
    match read_with(&package, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.loci[0].path.as_ref().map(|path| path.as_str())
            ),
            (
                CheckedPackageRefusalCode::IllTyped,
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                Some(
                    format!(
                        "/semantic_graph/nodes/{}/body/operation/member/name",
                        positions[0]
                    )
                    .as_str()
                )
            )
        ),
        other => panic!("an unknown record projection must be ineligible: {other:?}"),
    }
}

/// Trace: FR-068-AC-2, FR-068-AC-4, QSpec FR-322-AC-63, FR-322-AC-65
#[trace("TC-463")]
#[test]
fn selected_record_address_street_has_profiled_text_and_account_keeps_record_type() {
    let mut code = value_type("Code", "string", None);
    code["constraints"] = json!([
        {"keyword":"minLength", "operands":{"value":0}},
        {"keyword":"maxLength", "operands":{"value":64}},
        {"keyword":"format", "operands":{"name":"text-profile:nfc"}}
    ]);
    let mut document = document(vec![
        code,
        typed_object(
            "Address",
            "record",
            &[],
            &[field("street", &orders("Code"))],
        ),
        object("Account", &[], &[field("address", &orders("Address"))]),
    ]);
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind":{"module":IDENTITY,"name":"record"},
            "construct":{"meaning":"quire.meaning.model.record-value-type/v1"}
        }));
    let address = declaration_key("Address", "record_value_type");
    let reads = [Read {
        owner: "Address",
        field: "street",
        ty: Ty::TextBounds(0, 64, "nfc".into()),
    }];
    let admitted = admit(
        &document,
        &[("Address", "record_value_type"), ("Account", OBJECT)],
        &reads,
    );
    let fields = fields_of(&admitted, "Account");
    assert_eq!(
        fields
            .field("address")
            .and_then(|field| field.member_type()),
        Some(&CheckedMemberType::Record(
            serde_json::from_value(node_id(&address)).expect("id")
        ))
    );
    let (unknown, evidence, positions) = package(
        &document,
        &[("Address", "record_value_type")],
        &[Read {
            owner: "Address",
            field: "missing",
            ty: Ty::TextBounds(0, 64, "nfc".into()),
        }],
    );
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with(&unknown, &evidence, CheckedPackageReadLimits::bounded())
    else {
        panic!("unknown record field must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(
            format!(
                "/semantic_graph/nodes/{}/body/operation/member/name",
                positions[0]
            )
            .as_str()
        )
    );
}

/// Trace: FR-068-AC-2, QSpec FR-322-AC-72
#[trace("TC-463")]
#[test]
fn selected_process_field_n_resolves_through_the_model_member_step() {
    let mut document = document(vec![process(
        "Flow",
        &[],
        &[field("n", "ix://quire/native/Integer")],
    )]);
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind":{"module":IDENTITY,"name":"process"},
            "construct":{"meaning":"quire.meaning.model.process/v1"}
        }));
    let (wire, evidence, positions) = package(
        &document,
        &[("Flow", "process")],
        &[Read {
            owner: "Flow",
            field: "n",
            ty: Ty::Integer,
        }],
    );
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read_with(&wire, &evidence, CheckedPackageReadLimits::bounded())
    else {
        panic!("Flow.n projection must admit");
    };
    assert_eq!(
        admitted.graph().nodes[positions[0]].semantic_form.as_ref(),
        "query"
    );
}

/// Trace: FR-068-AC-2, QSpec FR-322-AC-72
#[trace("TC-463")]
#[test]
fn selected_subprocess_inherits_only_process_members() {
    let mut document = document(vec![
        process("Flow", &[], &[field("n", "ix://quire/native/Integer")]),
        process("SubFlow", &["Flow"], &[]),
    ]);
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind":{"module":IDENTITY,"name":"process"},
            "construct":{"meaning":"quire.meaning.model.process/v1"}
        }));
    let reads = [Read {
        owner: "SubFlow",
        field: "n",
        ty: Ty::Integer,
    }];
    admit(
        &document,
        &[("Flow", "process"), ("SubFlow", "process")],
        &reads,
    );
    document["types"][0]["kind"]["name"] = json!("entity");
    let (wire, evidence, _) = package(&document, &[("SubFlow", "process")], &[]);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with(&wire, &evidence, CheckedPackageReadLimits::bounded())
    else {
        panic!("process inheriting an object type must refuse at selected intake");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
}

/// Trace: FR-068-AC-4, QSpec FR-322-AC-65
#[trace("TC-463")]
#[test]
fn selected_record_field_refuses_each_non_value_object_meaning_at_intake() {
    for (kind, meaning) in [
        ("event", "quire.meaning.model.event-type/v1"),
        ("process", "quire.meaning.model.process/v1"),
        ("machine", "quire.meaning.model.state-machine/v1"),
        (
            "persistence",
            "quire.meaning.model.persistence-interface/v1",
        ),
        ("namespace", "quire.meaning.model.namespace/v1"),
    ] {
        let mut other = match kind {
            "process" => process("Other", &[], &[]),
            "event" => typed_object(
                "Other",
                kind,
                &[],
                &[field("at", "ix://quire/native/Timestamp")],
            ),
            _ => typed_object("Other", kind, &[], &[]),
        };
        match kind {
            "event" => other["occurrenceField"] = json!(format!("{}/at", orders("Other"))),
            "machine" => {
                other["operations"] = json!([{"identity":format!("{}/toggle", orders("Other")),
                    "name":"toggle", "params":[]}]);
                other["states"] = json!([{"identity":format!("{}/open", orders("Other")),
                    "name":"open"}]);
                other["transitions"] = json!([{
                    "identity":format!("{}/transition", orders("Other")),
                    "from":format!("{}/open", orders("Other")),
                    "to":format!("{}/open", orders("Other")),
                    "trigger":format!("{}/toggle", orders("Other")), "emits":[]
                }]);
            }
            "persistence" => {
                other["operations"] = json!([{"identity":format!("{}/save", orders("Other")),
                    "name":"save", "params":[]}]);
                other["persists"] = json!([orders("Account")]);
            }
            "namespace" => {
                other.as_object_mut().expect("namespace").remove("fields");
                other
                    .as_object_mut()
                    .expect("namespace")
                    .remove("operations");
                other["members"] = json!([orders("Account")]);
                other["vocabulary"] = json!([{"term":"Account", "doc":"Account"}]);
            }
            _ => {}
        }
        let mut account = object("Account", &[], &[field("id", "ix://quire/native/UUID")]);
        account["identityFields"] = json!([format!("{}/id", orders("Account"))]);
        let mut document = document(vec![
            typed_object(
                "Address",
                "record",
                &[],
                &[field("street", &orders("Other"))],
            ),
            other,
            account,
        ]);
        document["constructs"]
            .as_array_mut()
            .expect("constructs")
            .extend([
                json!({"kind":{"module":IDENTITY,"name":"record"},
                "construct":{"meaning":"quire.meaning.model.record-value-type/v1"}}),
                json!({"kind":{"module":IDENTITY,"name":kind},"construct":{"meaning":meaning}}),
            ]);
        let (wire, evidence, _) = package(&document, &[("Address", "record_value_type")], &[]);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_with(&wire, &evidence, CheckedPackageReadLimits::bounded())
        else {
            panic!("record field of {kind} must refuse at selected intake");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidModelBinding,
            "{kind}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            "{kind}"
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some(root_owner(&wire, IDENTITY).as_str()),
            "{kind}"
        );
    }
}

/// Trace: FR-068-AC-4, QSpec FR-322-AC-65
#[trace("TC-463")]
#[test]
fn selected_record_node_cannot_be_an_object_reference_target() {
    let mut document = document(vec![typed_object("Address", "record", &[], &[])]);
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind":{"module":IDENTITY,"name":"record"},
            "construct":{"meaning":"quire.meaning.model.record-value-type/v1"}
        }));
    let (mut wire, evidence, _) = package(&document, &[("Address", "record_value_type")], &[]);
    let target = declaration_key("Address", "record_value_type");
    let mut nodes = BTreeMap::new();
    type_nodes(&Ty::Reference(target), &mut nodes);
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes.into_values());
    rebuild_source_map(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with(&wire, &evidence, CheckedPackageReadLimits::bounded())
    else {
        panic!("Reference<Address> must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidDerivedShape)
    );
}

/// A chain with an inherited field, a field redefined by a subtype with a
/// narrower range, a most-derived redefiner two owners deep and an own field:
/// each exposed name comes back once, with the most-derived redefiner's type
/// and no hidden field; a read keyed from the returned type admits and the
/// same read keyed from the hidden base field's type refuses.
#[trace("TC-227", "FR-038-AC-137")]
#[test]
fn tc_227_the_effective_set_is_the_one_admission_resolves() {
    let (hundred, ten) = (orders("Hundred"), orders("Ten"));
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        value_type("Hundred", "integer", Some(("0", "100"))),
        value_type("Ten", "integer", Some(("0", "10"))),
        object(
            "Base",
            &[],
            &[field("limit", &thousand), field("inherited", &thousand)],
        ),
        object(
            "Mid",
            &["Base"],
            &[field("limit", &hundred).redefining(&format!("{}/limit", orders("Base")))],
        ),
        object(
            ORDER,
            &["Mid"],
            &[
                field("limit", &ten).redefining(&format!("{}/limit", orders("Mid"))),
                field("own", &thousand),
            ],
        ),
    ]);
    let declared = [(ORDER, OBJECT), ("Base", OBJECT), ("Mid", OBJECT)];
    let package = admit(&document, &declared, &[]);
    let fields = fields_of(&package, ORDER);
    assert_eq!(names(&fields), ["limit", "inherited", "own"]);
    assert_eq!(
        fields.field("limit").and_then(|f| f.member_type()),
        Some(&range(0, 10))
    );
    assert_eq!(
        fields.field("limit").expect("limit").declaring_node_id(),
        &declaration_id(ORDER)
    );
    assert_eq!(
        fields
            .field("inherited")
            .expect("inherited")
            .declaring_node_id(),
        &declaration_id("Base")
    );
    assert_eq!(
        fields.field("inherited").and_then(|f| f.member_type()),
        Some(&range(0, 1000))
    );
    for field in fields.fields() {
        let name = match field.name() {
            "inherited" => "inherited",
            "limit" => "limit",
            _ => "own",
        };
        let returned = ty_of(field.member_type().expect("a typed field"));
        assert_eq!(
            read_of(&document, &declared, ORDER, name, returned),
            Ok(()),
            "a read of {name} keyed from the returned type admits"
        );
    }
    // The hidden base field's type is not the field's type.
    for hidden in [Ty::Range(0, 1000), Ty::Range(0, 100)] {
        let refused = read_of(&document, &declared, ORDER, "limit", hidden)
            .expect_err("a read keyed from a hidden field's type refuses");
        assert_eq!((refused.0, refused.1), INELIGIBLE);
    }
    // The less derived owners expose their own redefinition.
    assert_eq!(
        fields_of(&package, "Mid")
            .field("limit")
            .and_then(|f| f.member_type()),
        Some(&range(0, 100))
    );
    assert_eq!(
        fields_of(&package, "Base")
            .field("limit")
            .and_then(|f| f.member_type()),
        Some(&range(0, 1000))
    );
}

// --- AC-138 and AC-141 -----------------------------------------------------------

const MAX: &str = "170141183460469231731687303715884105727";
const MIN: &str = "-170141183460469231731687303715884105728";
const PAST_MAX: &str = "170141183460469231731687303715884105728";
const PAST_MIN: &str = "-170141183460469231731687303715884105729";

/// The document of AC-138: a field of every member kind the tables derive and
/// of every kind they do not.
fn every_kind() -> Value {
    let thousand = orders("Thousand");
    let many = |ordered, unique| (0, None, ordered, unique);
    document(vec![
        thousand_type(),
        value_type("Zero", "integer", Some(("0", "0"))),
        value_type("Signed", "integer", Some(("-5", "5"))),
        value_type("Wide", "integer", Some((MIN, MAX))),
        // 2^53 + 1: a bound an `f64` cannot hold.
        value_type("Odd", "integer", Some(("0", "9007199254740993"))),
        value_type("PastMax", "integer", Some(("0", PAST_MAX))),
        value_type("PastMin", "integer", Some((PAST_MIN, "0"))),
        value_type("Label", "text", None),
        typed_object("Pump", "interface", &[], &[]),
        typed_object("Alarm", "event", &[], &[]),
        object("Other", &[], &[]),
        object(
            ORDER,
            &[],
            &[
                field("flag", &format!("{NATIVE}Boolean")),
                field("number", &format!("{NATIVE}Integer")),
                field("zero", &orders("Zero")),
                field("signed", &orders("Signed")),
                field("wide", &orders("Wide")),
                field("odd", &orders("Odd")),
                field("link", &orders("Other")),
                field("maybe", &thousand).optional(),
                field("set", &thousand).many(many(false, true)),
                field("bag", &thousand).many(many(false, false)),
                field("sequence", &thousand).many(many(true, false)),
                field("ordered", &thousand).many(many(true, true)),
                field("window", &thousand).many((1, Some(4), false, true)),
                field("maybe_set", &thousand)
                    .optional()
                    .many(many(false, true)),
                field("text", &format!("{NATIVE}Text")),
                field("rational", &format!("{NATIVE}Rational")),
                field("decimal", &format!("{NATIVE}Decimal")),
                field("float", &format!("{NATIVE}Float32")),
                field("pump", &orders("Pump")),
                field("alarm", &orders("Alarm")),
                field("label", &orders("Label")),
                field("past_max", &orders("PastMax")),
                field("past_min", &orders("PastMin")),
                field("at_least_one", &thousand).many((1, None, false, true)),
            ],
        ),
    ])
}

/// What AC-138 says each field returns.
fn expected_kinds() -> Vec<(&'static str, Option<Ty>)> {
    let thousand = || Ty::Range(0, 1000);
    let collection = |kind, bounds| Some(Ty::Collection(kind, Box::new(thousand()), bounds));
    let other = declaration_key("Other", OBJECT);
    vec![
        ("flag", Some(Ty::Boolean)),
        ("number", Some(Ty::Integer)),
        ("zero", Some(Ty::Range(0, 0))),
        ("signed", Some(Ty::Range(-5, 5))),
        ("wide", Some(Ty::Range(i128::MIN, i128::MAX))),
        ("odd", Some(Ty::Range(0, 9_007_199_254_740_993))),
        ("link", Some(Ty::Reference(other))),
        ("maybe", Some(Ty::Option(Box::new(thousand())))),
        ("set", collection(Kind::Set, None)),
        ("bag", collection(Kind::Bag, None)),
        ("sequence", collection(Kind::Sequence, None)),
        ("ordered", collection(Kind::OrderedSet, None)),
        ("window", collection(Kind::Set, Some((1, 4)))),
        (
            "maybe_set",
            Some(Ty::Option(Box::new(
                collection(Kind::Set, None).expect("a collection"),
            ))),
        ),
        ("text", None),
        ("rational", None),
        ("decimal", None),
        ("float", Some(Ty::Float32)),
        ("pump", None),
        ("alarm", None),
        ("label", None),
        ("past_max", None),
        ("past_min", None),
        ("at_least_one", None),
    ]
}

fn kinds_package() -> CheckedPackageV2 {
    admit(
        &every_kind(),
        &[(ORDER, OBJECT), ("Other", OBJECT), ("Pump", INTERFACE)],
        &[],
    )
}

/// Each member kind returns the exact value the tables derive, the `i128`
/// extremes without loss and one past each as `None`, and a field with no type
/// is present beside the others and never refuses the call.
#[trace("TC-227", "FR-038-AC-138")]
#[test]
fn tc_227_every_member_kind_returns_its_derived_type_or_none() {
    let package = kinds_package();
    let fields = fields_of(&package, ORDER);
    let expected = expected_kinds();
    assert_eq!(fields.fields().len(), expected.len());
    for ((name, ty), field) in expected.iter().zip(fields.fields()) {
        assert_eq!(field.name(), *name);
        assert_eq!(
            field.member_type().map(ty_of),
            *ty,
            "the member type of {name}"
        );
    }
    // The extremes are values, not floats or 64-bit integers.
    assert_eq!(
        fields.field("wide").and_then(|f| f.member_type()),
        Some(&range(i128::MIN, i128::MAX))
    );
    assert!(matches!(
        fields.field("link").and_then(|f| f.member_type()),
        Some(CheckedMemberType::Reference(target)) if *target == declaration_id("Other")
    ));
}

/// The perturbation of AC-141: a type that stays well formed and is not the
/// field's.
fn perturbed(ty: &Ty, other: &str) -> Ty {
    match ty {
        Ty::Boolean => Ty::Integer,
        Ty::Integer => Ty::Boolean,
        Ty::Float32
        | Ty::Float64
        | Ty::String
        | Ty::Uuid
        | Ty::Timestamp
        | Ty::Bytes
        | Ty::JsonObject
        | Ty::Clock => Ty::Boolean,
        Ty::Range(lower, upper) if upper > lower => Ty::Range(*lower, upper - 1),
        Ty::Range(lower, upper) if *lower < i128::MAX => Ty::Range(*lower, upper + 1),
        Ty::Range(lower, upper) => Ty::Range(lower - 1, *upper),
        Ty::RationalRange(..) => Ty::Boolean,
        Ty::TextBounds(lower, upper, profile) if upper > lower => {
            Ty::TextBounds(*lower, upper - 1, profile.clone())
        }
        Ty::TextBounds(lower, upper, profile) if *upper < u64::MAX => {
            Ty::TextBounds(*lower, upper + 1, profile.clone())
        }
        Ty::TextBounds(lower, upper, profile) => Ty::TextBounds(lower - 1, *upper, profile.clone()),
        Ty::Reference(_) => Ty::Reference(other.to_owned()),
        Ty::Record(_) => Ty::Record(other.to_owned()),
        Ty::Tuple(_) => Ty::Tuple(other.to_owned()),
        Ty::Variant(_) => Ty::Variant(other.to_owned()),
        Ty::Quantity(_) => Ty::Quantity(other.to_owned()),
        Ty::CarriedRational(..) => Ty::Boolean,
        Ty::Option(element) => Ty::Option(Box::new(perturbed(element, other))),
        Ty::Collection(kind, element, Some((lower, upper))) => {
            let bounds = if upper > lower {
                (*lower, upper - 1)
            } else if *lower < u64::MAX {
                (*lower, upper + 1)
            } else {
                (lower - 1, *upper)
            };
            Ty::Collection(*kind, element.clone(), Some(bounds))
        }
        Ty::Collection(kind, element, None) => {
            Ty::Collection(*kind, Box::new(perturbed(element, other)), None)
        }
    }
}

/// For each field with a derived type, a read keyed by the key recomputed here
/// from the returned type admits, and the same read keyed from the perturbed
/// type refuses `ill_typed`/`operator-ineligible` at the read: the accessor
/// reports the type admission compares.
#[trace("TC-227", "FR-038-AC-141")]
#[test]
fn tc_227_the_returned_type_is_the_type_admission_compares() {
    let package = kinds_package();
    let fields = fields_of(&package, ORDER);
    let document = every_kind();
    let declared = [(ORDER, OBJECT), ("Other", OBJECT), ("Pump", INTERFACE)];
    let other = declaration_key(ORDER, OBJECT);
    let mut checked = 0;
    for (name, _) in expected_kinds() {
        let Some(returned) = fields
            .field(name)
            .and_then(|field| field.member_type())
            .map(ty_of)
        else {
            continue;
        };
        let name = expected_kinds()
            .iter()
            .map(|(candidate, _)| *candidate)
            .find(|candidate| *candidate == name)
            .expect("a declared field");
        assert_eq!(
            read_of(&document, &declared, ORDER, name, returned.clone()),
            Ok(()),
            "a read of {name} keyed from {returned:?}"
        );
        let perturbed = perturbed(&returned, &other);
        assert_ne!(perturbed, returned);
        let refused = read_of(&document, &declared, ORDER, name, perturbed.clone())
            .expect_err("a read keyed from the perturbed type refuses");
        assert_eq!((refused.0, refused.1), INELIGIBLE, "{name}: {perturbed:?}");
        checked += 1;
    }
    assert_eq!(checked, 15, "every field with a derived type was exercised");
}

// --- AC-139 ----------------------------------------------------------------------

/// The package of AC-139: `Order` and `Holder` (an object type with an
/// unread declaration node), a systems interface, and the scalar node a read
/// would name.
fn refusal_document() -> Value {
    let thousand = orders("Thousand");
    document(vec![
        thousand_type(),
        typed_object("Pump", "interface", &[], &[]),
        object(ORDER, &[], &[field("balance", &thousand)]),
    ])
}

/// `UnknownNode` and `NotModelObjectType` are two errors: an id no node has,
/// a scalar node and a systems interface's declaration node each name what is
/// wrong, and the call is `Ok` for an absent field name. An unselected model
/// owner is checked separately at admission.
#[trace("TC-227", "FR-038-AC-139", "FR-038-AC-155")]
#[test]
fn tc_227_the_accessor_tells_an_unknown_node_from_a_node_that_is_no_object_type() {
    let read = Read {
        owner: ORDER,
        field: "balance",
        ty: Ty::Range(0, 1000),
    };
    let package = admit(
        &refusal_document(),
        &[(ORDER, OBJECT), ("Pump", INTERFACE)],
        &[read],
    );
    let id = |path: &str| -> CheckedNodeId { structural_fixture_id(&node_id(path)) };
    assert_eq!(
        package.model_object_fields(&id("placeholder/9")),
        Err(CheckedModelFieldsError::UnknownNode)
    );
    // The scalar node of the read's type.
    let mut scalar = BTreeMap::new();
    let integer = type_nodes(&Ty::Integer, &mut scalar);
    let thousand = type_nodes(&Ty::Range(0, 1000), &mut scalar);
    for key in [&integer, &thousand] {
        let structural = id(key);
        let wire = package
            .graph()
            .nodes
            .iter()
            .enumerate()
            .find(|(position, _)| package.structural_key_at(*position) == Some(&structural))
            .map(|(_, node)| &node.node_id)
            .expect("authored type node");
        assert_eq!(
            package.model_object_fields(wire),
            Err(CheckedModelFieldsError::NotModelObjectType),
            "a type node is no model object"
        );
    }
    assert_eq!(
        package.model_object_fields(&id(&declaration_key("Pump", INTERFACE))),
        Err(CheckedModelFieldsError::NotModelObjectType),
        "a systems interface is no object type"
    );
    let fields = fields_of(&package, ORDER);
    assert!(fields.field("absent").is_none());
    assert_eq!(names(&fields), ["balance"]);
}

/// A model declaration's closed body is authenticated before the accessor is
/// available; an unselected model owner still refuses during owner joining.
#[trace("TC-227", "FR-038-AC-139", "FR-038-AC-150")]
#[test]
fn tc_227_a_changed_model_declaration_refuses_before_field_access() {
    let declared = [(ORDER, OBJECT), ("Pump", INTERFACE)];
    let (package, evidence, _) = package(&refusal_document(), &declared, &[]);
    let order = declaration_key(ORDER, OBJECT);
    let admitted = |package: &Value| match read_with(
        package,
        &evidence,
        CheckedPackageReadLimits::bounded(),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, got {other:?}"),
    };
    let intact = admitted(&package);
    assert!(intact.model_object_fields(&declaration_id(ORDER)).is_ok());
    // The same package, with the declaration node's body not empty.
    let mut tampered = package.clone();
    for node in tampered["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["node_id"] == node_id(&order) {
            node["body"] = json!({"term": "aggregate", "members": [
                reference(&declaration_key("Pump", INTERFACE)),
            ]});
            node["dependencies"] = json!([node_id(&declaration_key("Pump", INTERFACE))]);
        }
    }
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with(&tampered, &evidence, CheckedPackageReadLimits::bounded())
    else {
        panic!("a changed model declaration must refuse admission");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidDerivedShape)
    );
    let order_position = tampered["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == node_id(&order))
        .expect("Order declaration");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{order_position}/node_id").as_str())
    );
    assert_eq!(refusal.locus.as_ref(), Some(&declaration_id(ORDER)));
    // A node keyed as no selected declaration.
    let mut unselected = package;
    let stranger = "model:model:object_type:ix://acme/other/Stranger";
    let mut stranger_node = wire_node(stranger, "model", OBJECT, stranger, &[], empty());
    let stranger_id = structural_id(
        "acme/other",
        vec![
            json!("model"),
            json!(OBJECT),
            json!("ix://acme/other/Stranger"),
        ],
        0,
    );
    stranger_node["node_id"] = stranger_id.clone();
    stranger_node["semantic_type"] = stranger_id;
    stranger_node["owner"] = json!({
        "kind": "model", "identity": "acme/other", "node": orders("Stranger"),
    });
    unselected["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(stranger_node);
    rebuild_source_map(&mut unselected);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with(&unselected, &evidence, CheckedPackageReadLimits::bounded())
    else {
        panic!("an unselected model owner must refuse admission");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
}

/// An object type whose document gives two unhidden effective fields of one
/// name admits when no read names it and returns the error naming the name,
/// for the whole call although the object also declares a valid field; the
/// same document with a read of the name is refused at admission.
#[trace("TC-227", "FR-038-AC-139")]
#[test]
fn tc_227_an_ambiguous_name_refuses_the_call_not_the_admission() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object("Left", &[], &[field("x", &thousand)]),
        object(
            ORDER,
            &["Left"],
            &[field("x", &thousand), field("y", &thousand)],
        ),
    ]);
    let declared = [(ORDER, OBJECT), ("Left", OBJECT)];
    let package = admit(&document, &declared, &[]);
    assert_eq!(
        package.model_object_fields(&declaration_id(ORDER)),
        Err(CheckedModelFieldsError::AmbiguousField("x".into()))
    );
    // The supertype alone is not ambiguous.
    assert_eq!(names(&fields_of(&package, "Left")), ["x"]);
    // Package B: the same document with a read of `x` is refused.
    let read = Read {
        owner: ORDER,
        field: "x",
        ty: Ty::Range(0, 1000),
    };
    let (package_b, evidence, _) = self::package(&document, &declared, &[read]);
    match read_with(&package_b, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::AmbiguousDeclaration
            );
            assert_eq!(
                refusal.cause,
                Some(CheckedPackageRefusalCause::AmbiguousName)
            );
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

// --- AC-142 ----------------------------------------------------------------------

/// The accessor takes `&self` and one node id and no evidence; the three
/// enums match exhaustively with no wildcard arm from outside the crate,
/// which compiles only while they are closed.
#[trace("TC-227", "FR-038-AC-142")]
#[test]
fn tc_227_the_public_shape_is_closed_and_takes_one_node_id() {
    let accessor: fn(
        &CheckedPackageV2,
        &CheckedNodeId,
    ) -> Result<CheckedModelObjectFields, CheckedModelFieldsError> =
        CheckedPackageV2::model_object_fields;
    let _ = accessor;
    fn member(ty: &CheckedMemberType) -> u8 {
        match ty {
            CheckedMemberType::Boolean => 0,
            CheckedMemberType::Integer => 1,
            CheckedMemberType::Float32 => 15,
            CheckedMemberType::Float64 => 16,
            CheckedMemberType::String => 8,
            CheckedMemberType::Uuid => 9,
            CheckedMemberType::Timestamp => 10,
            CheckedMemberType::Bytes => 17,
            CheckedMemberType::JsonObject => 18,
            CheckedMemberType::Clock => 12,
            CheckedMemberType::IntRange { .. } => 2,
            CheckedMemberType::RationalRange { .. } => 19,
            CheckedMemberType::TextBounds { .. } => 13,
            CheckedMemberType::Reference(_) => 3,
            CheckedMemberType::Record(_) => 6,
            CheckedMemberType::Tuple(_) => 14,
            CheckedMemberType::Variant(_) => 7,
            CheckedMemberType::Quantity(_) => 11,
            CheckedMemberType::CarriedRational { .. } => 20,
            CheckedMemberType::Option(_) => 4,
            CheckedMemberType::Collection { .. } => 5,
        }
    }
    fn collection(kind: CheckedCollectionKind) -> u8 {
        match kind {
            CheckedCollectionKind::Set => 0,
            CheckedCollectionKind::Bag => 1,
            CheckedCollectionKind::Sequence => 2,
            CheckedCollectionKind::OrderedSet => 3,
        }
    }
    fn error(error: &CheckedModelFieldsError) -> u8 {
        match error {
            CheckedModelFieldsError::UnknownNode => 0,
            CheckedModelFieldsError::NotModelObjectType => 1,
            CheckedModelFieldsError::AmbiguousField(_) => 2,
            CheckedModelFieldsError::MissingDeclaringNode { .. } => 3,
            CheckedModelFieldsError::AmbiguousDeclaringNode { .. } => 4,
        }
    }
    assert_eq!(member(&CheckedMemberType::Boolean), 0);
    assert_eq!(member(&CheckedMemberType::Float32), 15);
    assert_eq!(member(&CheckedMemberType::Float64), 16);
    assert_eq!(member(&CheckedMemberType::Bytes), 17);
    assert_eq!(member(&CheckedMemberType::JsonObject), 18);
    assert_eq!(collection(CheckedCollectionKind::OrderedSet), 3);
    assert_eq!(error(&CheckedModelFieldsError::UnknownNode), 0);
}

// --- AC-143 ----------------------------------------------------------------------

/// Two calls, a clone and two admissions of one package under different byte
/// limits agree, and the retained tables take no part in the package's
/// equality; a chain of 200 object types and 100 redefinitions of one field
/// is read without panic or stack overflow.
#[trace("TC-227", "FR-038-AC-143", "FR-038-AC-373")]
#[test]
fn tc_227_the_accessor_is_pure_and_total() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object(ORDER, &[], &[field("balance", &thousand)]),
    ]);
    let (package, evidence, _) = package(&document, &[(ORDER, OBJECT)], &[]);
    let admit_under = |bytes: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.bytes = bytes;
        match read_with(&package, &evidence, limits) {
            CheckedPackageV2ReadResult::Admitted(package) => *package,
            other => panic!("expected admission under {bytes} bytes, got {other:?}"),
        }
    };
    let roomy = CheckedPackageReadLimits::bounded().bytes;
    let (a, b) = (admit_under(roomy), admit_under(roomy / 2));
    assert_eq!(a, b, "packages admitted from one set of bytes are equal");
    let first = fields_of(&a, ORDER);
    assert_eq!(first, fields_of(&a, ORDER), "two calls agree");
    assert_eq!(first, fields_of(&a.clone(), ORDER), "a clone agrees");
    assert_eq!(
        first,
        fields_of(&b, ORDER),
        "a package read under other limits agrees"
    );

    // 200 types: `R1` declares `f`, `R2` to `R101` redefine their predecessor's
    // (100 redefinitions), `R102` to `R200` each declare a field of their own.
    let names: Vec<String> = (1..=200).map(|k| format!("R{k}")).collect();
    let own: Vec<String> = (102..=200).map(|k| format!("g{k}")).collect();
    let types: Vec<Value> = std::iter::once(thousand_type())
        .chain((1..=200).map(|k| {
            let supertypes: Vec<&str> = if k == 1 {
                vec![]
            } else {
                vec![names[k - 2].as_str()]
            };
            let declared = if k <= 101 {
                let mut declared = field("f", &thousand);
                if k > 1 {
                    declared = declared.redefining(&format!("{}/f", orders(&names[k - 2])));
                }
                declared
            } else {
                field(leak(&own[k - 102]), &thousand)
            };
            object(&names[k - 1], &supertypes, &[declared])
        }))
        .collect();
    let declarations: Vec<_> = names.iter().map(|name| (name.as_str(), OBJECT)).collect();
    let deep = admit(&document_of(types), &declarations, &[]);
    let fields = deep
        .model_object_fields(&declaration_id("R200"))
        .expect("the chain's fields");
    assert_eq!(fields.fields().len(), 100);
    assert_eq!(
        fields.field("f").and_then(|f| f.member_type()),
        Some(&range(0, 1000))
    );
}

fn document_of(types: Vec<Value>) -> Value {
    document(types)
}

/// A field name the document builder keeps for the program's life.
fn leak(text: &str) -> &'static str {
    Box::leak(text.to_owned().into_boxed_str())
}

// --- AC-144 ----------------------------------------------------------------------

fn chain(count: usize) -> Value {
    let thousand = orders("Thousand");
    let names: Vec<String> = (1..=count).map(|k| format!("C{k}")).collect();
    let fields: Vec<String> = (1..=count).map(|k| format!("f{k}")).collect();
    let types: Vec<Value> = std::iter::once(thousand_type())
        .chain((1..=count).map(|k| {
            let supertypes: Vec<&str> = if k == 1 {
                vec![]
            } else {
                vec![names[k - 2].as_str()]
            };
            object(
                &names[k - 1],
                &supertypes,
                &[field(leak(&fields[k - 1]), &thousand)],
            )
        }))
        .collect();
    document(types)
}

fn last(count: usize) -> String {
    format!("C{count}")
}

fn chain_package(count: usize) -> (Value, CheckedPackageEvidence) {
    let declared = last(count);
    let (package, evidence, _) = package(&chain(count), &[(declared.as_str(), OBJECT)], &[]);
    (package, evidence)
}

fn admits_under(package: &Value, evidence: &CheckedPackageEvidence, work: u64) -> bool {
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work;
    match read_with(package, evidence, limits) {
        CheckedPackageV2ReadResult::Admitted(_) => true,
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            false
        }
        other => panic!("expected admission or `incomplete`, got {other:?}"),
    }
}

/// The smallest `work` limit under which `package` admits.
fn smallest_admitting_work(package: &Value, evidence: &CheckedPackageEvidence) -> u64 {
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(admits_under(package, evidence, high));
    while low < high {
        let middle = low + (high - low) / 2;
        if admits_under(package, evidence, middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

/// The last table charge precedes the graph stage. The selected-owner charge
/// follows graph validation, so one below the whole-read minimum stops there.
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_chain_admits_at_its_smallest_limit_and_is_incomplete_one_below() {
    for count in [4, 8] {
        let (package, evidence) = chain_package(count);
        let smallest = smallest_admitting_work(&package, &evidence);
        assert!(smallest > 0);
        assert!(admits_under(&package, &evidence, smallest));
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = smallest - 1;
        match read_with(&package, &evidence, limits) {
            CheckedPackageV2ReadResult::Incomplete(incomplete) => {
                assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
                assert_eq!(
                    incomplete.path.as_ref().map(ToString::to_string).as_deref(),
                    Some(root_owner(&package, IDENTITY).as_str())
                );
            }
            other => panic!("expected one-below incomplete, got {other:?}"),
        }
    }
}

/// Under `CheckedPackageReadLimits::bounded()` a chain of 1000 types admits
/// and a chain of 1500 is `incomplete` at the selection's row: the owner
/// decision's numbers.
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_the_long_chains_are_decided_by_the_bounded_work_limit() {
    let bounded = CheckedPackageReadLimits::bounded().work;
    assert_eq!(bounded, 1_000_000);
    let (package, evidence) = chain_package(1000);
    assert!(admits_under(&package, &evidence, bounded));
    let (package, evidence) = chain_package(1500);
    assert!(!admits_under(&package, &evidence, bounded));
    match read_with(&package, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(
                incomplete.path.as_ref().map(ToString::to_string).as_deref(),
                Some(root_owner(&package, IDENTITY).as_str())
            );
        }
        other => panic!("expected table-charge incomplete, got {other:?}"),
    }
}

/// The package of a chain of 8 types with a duplicate node id: a graph-stage
/// `invalid_semantic_graph` refusal.
fn duplicate_chain_package() -> (Value, Value, CheckedPackageEvidence, usize) {
    let read = || Read {
        owner: "C8",
        field: "f8",
        ty: Ty::Range(0, 1000),
    };
    let declared = [("C8", OBJECT)];
    let (valid, _, _) = package(&chain(8), &declared, &[read()]);
    let (mut duplicate, evidence, positions) = package(&chain(8), &declared, &[read()]);
    let at = positions.first().copied().expect("the read");
    let package_name = duplicate["name"].clone();
    let copied = duplicate["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .take(at)
        .find(|node| node["node_id"]["package"] == package_name && node.get("owner").is_none())
        .expect("earlier local node")["node_id"]
        .clone();
    duplicate["semantic_graph"]["nodes"][at]["node_id"] = copied;
    rebuild_source_map(&mut duplicate);
    (valid, duplicate, evidence, at)
}

/// Tables are built after every row's step 1 and before the graph stage: a
/// table charge that exhausts the limit together with a graph-stage refusal
/// (a duplicate node id) is `incomplete`, and a step 1 refusal at row 1 together
/// with a table charge that exhausts the limit at row 0 is the step 1 refusal.
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_the_tables_follow_every_step_one_and_precede_the_graph_stage() {
    let (valid, duplicate, evidence, duplicate_at) = duplicate_chain_package();
    // The smallest limit under which the tables are built: one below it, the
    // build is what exhausts the limit.
    let exhausts_tables = |package: &Value, work: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        matches!(
            read_with(package, &evidence, limits),
            CheckedPackageV2ReadResult::Incomplete(incomplete)
                if incomplete.path.as_ref().map(ToString::to_string).as_deref()
                    == Some(root_owner(package, IDENTITY).as_str())
        )
    };
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(!exhausts_tables(&duplicate, high));
    while low < high {
        let middle = low + (high - low) / 2;
        if exhausts_tables(&duplicate, middle) {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    let smallest = low;
    assert!(smallest > 43, "the tables of 8 types are charged 43");
    // With room, the duplicate id is the refusal the graph stage draws.
    match read_with(&duplicate, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::InvalidSemanticGraph
            );
            assert_eq!(refusal.cause, None);
        }
        other => panic!("expected the duplicate-id refusal, got {other:?}"),
    }
    // One under, the same package is `incomplete` at the tables' row, not the
    // graph stage's duplicate id.
    assert!(exhausts_tables(&duplicate, smallest - 1));

    // The duplicate read node supplies a separate graph-charge boundary. The
    // first sufficient budget reaches its refusal; one below stops at its id.
    let duplicate_path = format!("/semantic_graph/nodes/{duplicate_at}/node_id");
    let graph_refuses = |work: u64| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        matches!(
            read_with(&duplicate, &evidence, limits),
            CheckedPackageV2ReadResult::Refused(refusal)
                if refusal.code == CheckedPackageRefusalCode::InvalidSemanticGraph
                    && refusal.cause.is_none()
                    && refusal.loci[0].path.as_ref().map(ToString::to_string).as_deref()
                        == Some(duplicate_path.as_str())
        )
    };
    let (mut low, mut high) = (smallest, CheckedPackageReadLimits::bounded().work);
    assert!(graph_refuses(high));
    while low < high {
        let middle = low + (high - low) / 2;
        if graph_refuses(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = low - 1;
    match read_with(&duplicate, &evidence, limits) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(
                incomplete.path.as_ref().map(ToString::to_string).as_deref(),
                Some(duplicate_path.as_str())
            );
        }
        other => panic!("expected duplicate-id graph charge denial, got {other:?}"),
    }

    // Root 0 is the chain, root 1 a selection no evidence supplies.
    let mut two = valid;
    select_model(&mut two, "acme/other");
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = smallest - 1;
    match read_with(&two, &evidence, limits) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingImport);
            assert_eq!(
                refusal.loci[0]
                    .path
                    .clone()
                    .map(|path| path.to_string())
                    .as_deref(),
                Some(format!("{}/identity", root_owner(&two, "acme/other")).as_str())
            );
        }
        other => panic!("expected the row 1 step 1 refusal, got {other:?}"),
    }
}

/// Inherited authored slots precede own slots even when name order differs.
/// Name lookup selects the same entries without assuming sorted names.
#[trace("TC-227", "FR-038-AC-136")]
#[test]
fn tc_227_the_order_is_authored_where_name_order_differs() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object(
            "Base",
            &[],
            &[field("zeta", &thousand), field("mu", &thousand)],
        ),
        object(ORDER, &["Base"], &[field("alpha", &thousand)]),
    ]);
    let package = admit(&document, &[(ORDER, OBJECT), ("Base", OBJECT)], &[]);
    let fields = fields_of(&package, ORDER);
    assert_eq!(names(&fields), ["zeta", "mu", "alpha"]);
    for name in ["alpha", "mu", "zeta"] {
        assert_eq!(fields.field(name).map(|field| field.name()), Some(name));
    }
}

/// With two ambiguous names, the error names the smallest, so the answer does
/// not depend on the order of the declarations.
#[trace("TC-227", "FR-038-AC-139")]
#[test]
fn tc_227_the_ambiguous_name_the_error_carries_is_the_smallest() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object("Left", &[], &[field("b", &thousand), field("a", &thousand)]),
        object(
            ORDER,
            &["Left"],
            &[field("b", &thousand), field("a", &thousand)],
        ),
    ]);
    let package = admit(&document, &[(ORDER, OBJECT)], &[]);
    assert_eq!(
        package.model_object_fields(&declaration_id(ORDER)),
        Err(CheckedModelFieldsError::AmbiguousField("a".into()))
    );
}

/// The members of the returned structs are reached through their accessors
/// alone: `name()` and `member_type()` of a field, `fields()` and `field()` of
/// the list. That no member is public is decided by the `compile_fail`
/// doctests of `CheckedModelField` and `CheckedModelObjectFields`, one probe
/// per member, which `make test` runs in its `--doc` lane.
#[trace("TC-227", "FR-038-AC-142")]
#[test]
fn tc_227_the_struct_members_are_reached_through_accessors() {
    let thousand = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object(ORDER, &[], &[field("balance", &thousand)]),
    ]);
    let package = admit(&document, &[(ORDER, OBJECT)], &[]);
    let fields = fields_of(&package, ORDER);
    let [balance] = fields.fields() else {
        panic!("one field");
    };
    assert_eq!(balance.name(), "balance");
    assert_eq!(balance.member_type(), Some(&range(0, 1000)));
    assert_eq!(fields.field("balance"), Some(balance));
}

/// The model object fields, authored source map and exact admitting work boundary.
#[trace("TC-227", "FR-038-AC-139", "FR-038-AC-144")]
#[test]
fn model_fields_source_map_and_work_boundary_hold() {
    let (wire, evidence) = record_project_control();
    let mut indexed_wire = wire.clone();
    index_fixture_node_ids(&mut indexed_wire);
    let bytes = canonical(&indexed_wire);
    let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("baseline admission: {other:?}"),
    };
    let result = admitted
        .model_object_fields(&declaration_id(ORDER))
        .expect("model fields");
    let [string] = result.fields() else {
        panic!("one model field")
    };
    assert_eq!(string.name(), "string");
    assert_eq!(string.member_type(), Some(&CheckedMemberType::String));
    let missing = admitted.model_object_fields(&declaration_id("Absent"));
    assert_eq!(missing, Err(CheckedModelFieldsError::UnknownNode));
    let authored_sources: Vec<quire_contract_model::CheckedSourceMapEntry> =
        serde_json::from_value(indexed_wire["source_map"].clone()).expect("authored sources");
    assert_eq!(admitted.source_map(), authored_sources);
    let mut low = 0;
    let mut high = CheckedPackageReadLimits::bounded().work;
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = middle;
        match CheckedPackageV2::read(&bytes, limits, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle,
            other => panic!("baseline work: {other:?}"),
        }
    }
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = high - 1;
    let CheckedPackageV2ReadResult::Incomplete(incomplete) =
        CheckedPackageV2::read(&bytes, limits, &evidence)
    else {
        panic!("work boundary");
    };
    assert_eq!(incomplete.consumed, high);
}

#[trace("TC-227", "FR-038-AC-370")]
#[test]
fn tc_227_fields_keep_authored_order_through_the_public_reader() {
    let ty = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object(ORDER, &[], &[field("zeta", &ty), field("alpha", &ty)]),
    ]);
    let package = admit(&document, &[(ORDER, OBJECT)], &[]);
    let fields = fields_of(&package, ORDER);
    assert_eq!(names(&fields), ["zeta", "alpha"]);
    assert_eq!(fields.field("alpha"), Some(&fields.fields()[1]));
    assert!(fields.field("absent").is_none());
}

#[trace("TC-227", "FR-038-AC-370", "FR-038-AC-373")]
#[test]
fn tc_227_inherited_fields_retain_actual_declaring_nodes() {
    let ty = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object("Base", &[], &[field("b", &ty)]),
        object("Mid", &["Base"], &[field("m", &ty)]),
        object("Leaf", &["Mid"], &[field("l", &ty)]),
    ]);
    let package = admit(
        &document,
        &[("Leaf", OBJECT), ("Base", OBJECT), ("Mid", OBJECT)],
        &[],
    );
    let fields = fields_of(&package, "Leaf");
    assert_eq!(names(&fields), ["b", "m", "l"]);
    for (field, owner) in fields.fields().iter().zip(["Base", "Mid", "Leaf"]) {
        assert_eq!(field.declaring_node_id(), &declaration_id(owner));
        assert_eq!(field.member_type(), Some(&range(0, 1000)));
    }
    assert_eq!(fields, fields_of(&package.clone(), "Leaf"));
}

#[trace("TC-227", "FR-038-AC-371")]
#[test]
fn tc_227_diamond_and_redefinition_keep_first_inherited_slot_and_real_owner() {
    let ty = orders("Thousand");
    for redefine in [false, true] {
        for (supers, expected) in [
            (["Left", "Right"], ["z", "left", "right", "own"]),
            (["Right", "Left"], ["z", "right", "left", "own"]),
        ] {
            let document = document(vec![
                thousand_type(),
                value_type("Ten", "integer", Some(("0", "10"))),
                object("Base", &[], &[field("z", &ty)]),
                object("Left", &["Base"], &[field("left", &ty)]),
                object("Right", &["Base"], &[field("right", &ty)]),
                object(
                    "Leaf",
                    &supers,
                    &if redefine {
                        vec![
                            field("z", &orders("Ten")).redefining(&format!("{}/z", orders("Base"))),
                            field("own", &ty),
                        ]
                    } else {
                        vec![field("own", &ty)]
                    },
                ),
            ]);
            let declared = [
                ("Base", OBJECT),
                ("Left", OBJECT),
                ("Right", OBJECT),
                ("Leaf", OBJECT),
            ];
            let package = admit(&document, &declared, &[]);
            let base = fields_of(&package, "Left");
            assert_eq!(
                base.fields()[0].declaring_node_id(),
                &declaration_id("Base")
            );
            let fields = fields_of(&package, "Leaf");
            assert_eq!(names(&fields), expected);
            for (name, owner) in [
                ("z", if redefine { "Leaf" } else { "Base" }),
                ("left", "Left"),
                ("right", "Right"),
                ("own", "Leaf"),
            ] {
                assert_eq!(
                    fields
                        .field(name)
                        .expect("effective field")
                        .declaring_node_id(),
                    &declaration_id(owner)
                );
            }
            assert_eq!(
                fields.field("z").and_then(|field| field.member_type()),
                Some(&range(0, if redefine { 10 } else { 1000 }))
            );
        }
    }
}

#[trace("TC-227", "FR-038-AC-372")]
#[test]
fn tc_227_missing_and_ambiguous_declaring_nodes_refuse_without_partial_fields() {
    let ty = orders("Thousand");
    let document = document(vec![
        thousand_type(),
        object("Base", &[], &[field("z", &ty)]),
        object("Other", &[], &[field("a", &ty)]),
        object("Leaf", &["Base", "Other"], &[]),
    ]);
    let only_leaf = admit(&document, &[("Leaf", OBJECT)], &[]);
    assert_eq!(
        only_leaf.model_object_fields(&declaration_id("Leaf")),
        Err(CheckedModelFieldsError::MissingDeclaringNode {
            field_identity: format!("{}/z", orders("Base")).into()
        })
    );
    let all = [("Leaf", OBJECT), ("Base", OBJECT), ("Other", OBJECT)];
    let complete = admit(&document, &all, &[]);
    assert_eq!(
        fields_of(&complete, "Leaf").fields()[0].declaring_node_id(),
        &declaration_id("Base")
    );
    let (mut wire, evidence, _) = package(&document, &all, &[]);
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let mut duplicate = nodes
        .iter()
        .find(|node| node["owner"]["node"] == orders("Base"))
        .expect("Base")
        .clone();
    duplicate["node_id"]["ordinal"] = json!(1);
    duplicate["semantic_type"] = duplicate["node_id"].clone();
    nodes.push(duplicate);
    settle(&mut wire);
    let ambiguous = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("distinct structural ordinals admit: {other:?}"),
    };
    assert_eq!(
        ambiguous.model_object_fields(&declaration_id("Leaf")),
        Err(CheckedModelFieldsError::AmbiguousDeclaringNode {
            field_identity: format!("{}/z", orders("Base")).into()
        })
    );
}

#[trace("TC-227", "FR-038-AC-372")]
#[test]
fn tc_227_declaring_node_errors_follow_field_order_before_name_order() {
    let ty = orders("Thousand");
    for supers in [["Base", "Other"], ["Other", "Base"]] {
        let document = document(vec![
            thousand_type(),
            object("Base", &[], &[field("z", &ty)]),
            object("Other", &[], &[field("a", &ty)]),
            object("Leaf", &supers, &[]),
        ]);
        let (mut wire, evidence, _) =
            package(&document, &[("Leaf", OBJECT), ("Base", OBJECT)], &[]);
        let nodes = wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        let mut duplicate = nodes
            .iter()
            .find(|node| node["owner"]["node"] == orders("Base"))
            .expect("Base")
            .clone();
        duplicate["node_id"]["ordinal"] = json!(1);
        duplicate["semantic_type"] = duplicate["node_id"].clone();
        nodes.push(duplicate);
        settle(&mut wire);
        let package = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("mixed absent and repeated owners admit: {other:?}"),
        };
        let expected = if supers[0] == "Base" {
            CheckedModelFieldsError::AmbiguousDeclaringNode {
                field_identity: format!("{}/z", orders("Base")).into(),
            }
        } else {
            CheckedModelFieldsError::MissingDeclaringNode {
                field_identity: format!("{}/a", orders("Other")).into(),
            }
        };
        assert_eq!(
            package.model_object_fields(&declaration_id("Leaf")),
            Err(expected)
        );
        assert_eq!(
            package.model_object_fields(&declaration_id("Unknown")),
            Err(CheckedModelFieldsError::UnknownNode)
        );
    }
}

mod operation_census;

mod option_types;

/// Selected identity rows must join the complete admitted declaration key.
#[trace("FR-352-AC-4", "FR-358-AC-3", "FR-358-AC-5")]
#[test]
fn exact_projection_identity_joins_refuse_changed_and_competing_owners() {
    let mut selected = document(vec![
        typed_object("Receipt", "record", &[], &[]),
        typed_object("OtherReceipt", "record", &[], &[]),
    ]);
    selected["constructs"].as_array_mut().unwrap().push(json!({
        "kind": {"module": IDENTITY, "name": "record"},
        "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
    }));
    selected["types"][1]["displayName"] = json!("Receipt");
    let (mut wire, evidence, _) = package(
        &selected,
        &[
            ("Receipt", "record_value_type"),
            ("OtherReceipt", "record_value_type"),
        ],
        &[],
    );
    let original = node_id(&declaration_key("Receipt", "record_value_type"));
    let mut alias = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["node_id"] == original)
        .unwrap()
        .clone();
    alias["node_id"]["ordinal"] = json!(7);
    alias["semantic_type"] = alias["node_id"].clone();
    let alias_id: CheckedNodeId = serde_json::from_value(alias["node_id"].clone()).unwrap();
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(alias);
    settle(&mut wire);
    let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
        other => panic!("independently keyed selected declarations must admit: {other:?}"),
    };
    let row = |name: &str| {
        SelectedExactDeclarationRow::new(
            serde_json::from_value(node_id(&declaration_key(name, "record_value_type"))).unwrap(),
            IDENTITY,
            SelectedExactDeclarationForm::RecordValueType,
            orders(name),
            SelectedExactIdentity::Record(
                NodeKey::new("selected", vec![PathSegment::Text(name.into())], 0).unwrap(),
            ),
        )
    };
    let receipt = row("Receipt");
    let other = row("OtherReceipt");
    let mut long_original = receipt.clone();
    long_original.original_identity = "x".repeat(4096).into();
    let mut short = quire_contract_model::NativeJoinBudget::new(128, u64::MAX);
    let error =
        SelectedExactIdentities::from_stage_rows(&admitted, [long_original], [], &mut short)
            .expect_err("stage text must be charged before comparison");
    assert_eq!(error.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    assert_eq!(error.at.as_deref(), Some(&receipt.structural_node));
    assert_eq!(
        short.first_unpaid().unwrap().kind,
        quire_contract_model::NativeJoinChargeKind::Work
    );
    assert!(short.first_unpaid().unwrap().amount >= 4096);
    let identities = SelectedExactIdentities::from_stage_rows(
        &admitted,
        [receipt.clone(), other.clone()],
        [],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    for expected in [&receipt, &other] {
        let SelectedExactIdentity::Record(key) = &expected.exact_identity else {
            panic!("record row")
        };
        assert_eq!(
            admitted
                .project_exact_type(&expected.structural_node, &identities, 100)
                .unwrap()
                .value_type(),
            &ValueType::Composite(key.clone())
        );
    }
    let check = |rows: Vec<SelectedExactDeclarationRow>, locus: &CheckedNodeId, code| {
        let mut budget = quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX);
        let error = SelectedExactIdentities::from_stage_rows(&admitted, rows, [], &mut budget)
            .expect_err("invalid identity join");
        assert_eq!(error.code, code);
        assert_eq!(error.requested.as_ref(), locus);
        assert_eq!(error.at.as_deref(), Some(locus));
        assert_eq!(error.form.as_ref(), "selected_identity");
        assert_eq!(error.work, budget.consumed().work);
        assert!(error.work > 0);
    };
    for changed in [
        {
            let mut row = receipt.clone();
            row.structural_node.path[2] =
                CheckedNodePathSegment::Name("different-declaration".into());
            row
        },
        {
            let mut row = receipt.clone();
            row.structural_node.ordinal += 1;
            row
        },
        {
            let mut row = receipt.clone();
            row.structural_node.package = "foreign/package".into();
            row
        },
    ] {
        check(
            vec![changed.clone()],
            &changed.structural_node,
            ExactTypeProjectionErrorCode::ForeignNode,
        );
        let error = admitted
            .project_exact_type(&changed.structural_node, &identities, 100)
            .expect_err("structural key must resolve exactly");
        assert_eq!(error.code, ExactTypeProjectionErrorCode::ForeignNode);
        assert_eq!(error.requested.as_ref(), &changed.structural_node);
        assert_eq!(error.at, None);
        assert_eq!(error.form.as_ref(), "unknown_node");
        assert_eq!(error.work, 1);
    }
    for changed in [
        {
            let mut row = receipt.clone();
            row.selected_document = "alias/orders".into();
            row
        },
        {
            let mut row = receipt.clone();
            row.original_identity = "ix://another/orders/Receipt".into();
            row
        },
        {
            let mut row = receipt.clone();
            row.form = SelectedExactDeclarationForm::ObjectType;
            row
        },
        {
            let mut row = other.clone();
            row.original_identity = receipt.original_identity.clone();
            row
        },
    ] {
        check(
            vec![changed.clone()],
            &changed.structural_node,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
        );
    }
    check(
        vec![receipt.clone(), receipt.clone()],
        &receipt.structural_node,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
    );
    let mut conflicting = receipt.clone();
    conflicting.exact_identity = other.exact_identity.clone();
    check(
        vec![receipt.clone(), conflicting],
        &receipt.structural_node,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
    );
    let mut ambiguous = receipt.clone();
    ambiguous.structural_node = alias_id.clone();
    ambiguous.exact_identity = SelectedExactIdentity::Record(
        NodeKey::new("selected", vec![PathSegment::Text("alias".into())], 0).unwrap(),
    );
    check(
        vec![receipt.clone(), ambiguous],
        &alias_id,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
    );
    let mut shared_key = other.clone();
    shared_key.exact_identity = receipt.exact_identity.clone();
    check(
        vec![receipt, shared_key],
        &other.structural_node,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
    );
}

/// A reference exposes only its selected object's witnessed EffectiveId.
#[trace("FR-352-AC-3")]
#[test]
fn exact_reference_projection_preserves_each_selected_object_identity() {
    let selected = document(vec![
        object("Node", &[], &[]),
        object("OtherNode", &[], &[]),
    ]);
    let (mut wire, evidence, _) =
        package(&selected, &[("Node", OBJECT), ("OtherNode", OBJECT)], &[]);
    for name in ["Node", "OtherNode"] {
        let target = declaration_key(name, OBJECT);
        let key = format!("composite_type/reference/{target}");
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(wire_node(
                &key,
                "composite_type",
                "reference",
                &key,
                &[&target],
                json!({"term": "aggregate", "members": [reference(&target)]}),
            ));
    }
    settle(&mut wire);
    index_fixture_node_ids(&mut wire);
    let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
        other => panic!("reference package must admit: {other:?}"),
    };
    let rows = ["Node", "OtherNode"].map(|name| {
        SelectedExactDeclarationRow::new(
            serde_json::from_value(node_id(&declaration_key(name, OBJECT))).unwrap(),
            IDENTITY,
            SelectedExactDeclarationForm::ObjectType,
            orders(name),
            SelectedExactIdentity::Object(EffectiveId::new(
                None,
                OriginalDeclarationKey::new(IDENTITY, orders(name)),
            )),
        )
    });
    let identities = SelectedExactIdentities::from_stage_rows(
        &admitted,
        rows.clone(),
        [],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    let references = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|node| node["semantic_form"] == "reference")
        .map(|node| serde_json::from_value::<CheckedNodeId>(node["node_id"].clone()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(references.len(), 2);
    for (index, row) in rows.iter().enumerate() {
        let SelectedExactIdentity::Object(identity) = &row.exact_identity else {
            panic!("object row")
        };
        let projected = admitted
            .project_exact_type(&references[index], &identities, 100)
            .unwrap();
        assert_eq!(
            projected.value_type(),
            &ValueType::Reference(identity.clone())
        );
        assert_eq!(projected.node(), &references[index]);
        let own = admitted
            .project_exact_type(&row.structural_node, &identities, 100)
            .expect_err("an object is not a standalone Exact ValueType");
        assert_eq!(own.code, ExactTypeProjectionErrorCode::UnsupportedExactType);
        assert_eq!(own.at.as_deref(), Some(&row.structural_node));
        assert_eq!(own.form.as_ref(), "model/object_type");
        assert_eq!(own.requested.as_ref(), &row.structural_node);
        assert_eq!(own.work, 2);
        let missing = SelectedExactIdentities::from_stage_rows(
            &admitted,
            [rows[1 - index].clone()],
            [],
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap();
        let error = admitted
            .project_exact_type(&references[index], &missing, 100)
            .expect_err("another object cannot witness this reference");
        assert_eq!(
            error.code,
            ExactTypeProjectionErrorCode::UnavailableIdentityWitness
        );
        assert_eq!(error.at.as_deref(), Some(&row.structural_node));
        assert_eq!(error.requested.as_ref(), &references[index]);
        assert_eq!(error.form.as_ref(), "model/object_type");
        assert_eq!(error.work, 3);
    }
    assert_ne!(
        admitted
            .project_exact_type(&references[0], &identities, 100)
            .unwrap()
            .value_type(),
        admitted
            .project_exact_type(&references[1], &identities, 100)
            .unwrap()
            .value_type()
    );
    let mut foreign = references[0].clone();
    foreign.package = "foreign/package".into();
    let error = admitted
        .project_exact_type(&foreign, &identities, 100)
        .expect_err("foreign reference");
    assert_eq!(error.code, ExactTypeProjectionErrorCode::ForeignNode);
    assert_eq!(error.requested.as_ref(), &foreign);
    assert_eq!(error.at, None);
    assert_eq!(error.form.as_ref(), "unknown_node");
    assert_eq!(error.work, 1);

    // Foreign and dangling targets never reach the admitted projection boundary.
    let position = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["semantic_form"] == "reference")
        .unwrap();
    let mut foreign_target = node_id(&declaration_key("Node", OBJECT));
    foreign_target["package"] = json!("foreign/package");
    for target in [node_id("missing-object"), foreign_target] {
        let mut adverse = wire.clone();
        adverse["semantic_graph"]["nodes"][position]["body"]["members"][0]["target"] = target;
        settle(&mut adverse);
        match read_with(&adverse, &evidence, CheckedPackageReadLimits::bounded()) {
            CheckedPackageV2ReadResult::Refused(error) => {
                assert_eq!(error.code, CheckedPackageRefusalCode::InvalidPackage);
                assert_eq!(
                    error.cause,
                    Some(CheckedPackageRefusalCause::InvalidDerivedShape)
                );
                assert_eq!(error.locus.as_ref(), Some(&references[0]));
                assert_eq!(error.loci.len(), 1);
                assert_eq!(
                    error.loci[0].path,
                    Some(crate::support::checked_package::pointer(&format!(
                        "/semantic_graph/nodes/{position}/node_id"
                    )))
                );
            }
            other => panic!("invalid reference target must refuse before projection: {other:?}"),
        }
    }
}

fn source_collection_fixture() -> (
    CheckedPackageV2,
    Vec<(SelectedSourceCollectionRow, CheckedNodeId)>,
) {
    let shapes = [
        ("sequence", Kind::Sequence, None, true, false),
        ("set", Kind::Set, None, false, true),
        ("bag", Kind::Bag, None, false, false),
        ("ordered", Kind::OrderedSet, None, true, true),
        ("bounded", Kind::Sequence, Some((0, 3)), true, false),
    ];
    let mut fields = shapes
        .map(|(name, _, bound, ordered, unique)| {
            field(name, "ix://quire/native/Integer").many((
                0,
                bound.map(|(_, upper)| upper),
                ordered,
                unique,
            ))
        })
        .into_iter()
        .collect::<Vec<_>>();
    fields.push(field("unrelated", "ix://quire/native/Float32"));
    let selected = document(vec![object("Collect", &[], &fields)]);
    let mut reads = shapes
        .map(|(name, kind, bound, _, _)| Read {
            owner: "Collect",
            field: name,
            ty: Ty::Collection(kind, Box::new(Ty::Integer), bound),
        })
        .into_iter()
        .collect::<Vec<_>>();
    reads.push(Read {
        owner: "Collect",
        field: "unrelated",
        ty: Ty::Float32,
    });
    let package = admit(&selected, &[("Collect", OBJECT)], &reads);
    let graph = &package.graph().nodes;
    let element_node = graph
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "scalar_type" && node.semantic_form.as_ref() == "integer"
        })
        .expect("final emitted Integer node")
        .node_id
        .clone();
    let rows: Vec<_> = shapes
        .into_iter()
        .map(|(_, kind, bound, _, _)| {
            let base_node = graph
                .iter()
                .find(|node| {
                    node.node_tag.as_ref() == "composite_type"
                        && node.semantic_form.as_ref() == kind.form()
                })
                .expect("final emitted collection base")
                .node_id
                .clone();
            let structural_node = if bound.is_some() {
                graph
                    .iter()
                    .find(|node| {
                        node.semantic_form.as_ref() == "collection_bounds"
                            && node.semantic_type == base_node
                    })
                    .expect("final emitted collection bound")
                    .node_id
                    .clone()
            } else {
                base_node.clone()
            };
            let value_type = ValueType::collection(CollectionType::new(
                match kind {
                    Kind::Sequence => CollectionKind::Sequence,
                    Kind::Set => CollectionKind::Set,
                    Kind::Bag => CollectionKind::Bag,
                    Kind::OrderedSet => CollectionKind::OrderedSet,
                },
                ValueType::Integer,
                bound.map(|(lower, upper)| CardinalityBound::new(lower, upper).unwrap()),
            ));
            let row = SelectedSourceCollectionRow {
                structural_node: structural_node.clone(),
                base_node,
                element_node: element_node.clone(),
                value_type,
            };
            let requested = graph
                .iter()
                .find(|node| {
                    node.node_tag.as_ref() == "expression"
                        && node.semantic_form.as_ref() == "query"
                        && node.semantic_type == structural_node
                })
                .expect("final emitted collection read")
                .node_id
                .clone();
            (row, requested)
        })
        .collect();
    (package, rows)
}

/// Trace: FR-352-AC-20.
#[trace("FR-352-AC-20", "FR-358-AC-2")]
#[test]
fn source_collection_rows_project_all_kinds_and_bounds() {
    let (package, rows) = source_collection_fixture();
    let first_row = rows[0].0.clone();
    let (budgeted, _) = crate::native_join_budget::assert_exact_join(
        &first_row.structural_node,
        &first_row.structural_node,
        |budget| {
            SelectedExactIdentities::empty().with_source_collection_rows(
                &package,
                [first_row.clone()],
                budget,
            )
        },
    );
    assert_eq!(budgeted.source_collection_rows().next(), Some(&first_row));
    let join_exact = (1..=500)
        .find(|&limit| {
            SelectedExactIdentities::empty()
                .with_source_collection_rows(
                    &package,
                    [first_row.clone()],
                    &mut quire_contract_model::NativeJoinBudget::new(limit, u64::MAX),
                )
                .is_ok()
        })
        .expect("finite exact join allowance");
    let mut join_budget = quire_contract_model::NativeJoinBudget::new(join_exact - 1, u64::MAX);
    let join_exhausted = SelectedExactIdentities::empty()
        .with_source_collection_rows(&package, [first_row], &mut join_budget)
        .expect_err("one below join allowance");
    assert_eq!(
        join_exhausted.code,
        ExactTypeProjectionErrorCode::ExhaustedWork
    );
    assert_eq!(join_exhausted.work, join_budget.consumed().work);
    assert!(join_exhausted.work < join_exact);
    assert_eq!(
        join_exhausted.at.as_deref(),
        Some(&rows[0].0.structural_node)
    );
    assert_eq!(
        join_budget.first_unpaid().unwrap().kind,
        quire_contract_model::NativeJoinChargeKind::Work
    );
    let selected = SelectedExactIdentities::empty()
        .with_source_collection_rows(
            &package,
            rows.iter().map(|(row, _)| row.clone()),
            &mut quire_contract_model::NativeJoinBudget::new(500, u64::MAX),
        )
        .expect("complete collection rows join");
    assert_eq!(selected.source_collection_rows().count(), rows.len());
    for (row, requested) in &rows {
        let projection = package
            .project_exact_type(requested, &selected, 500)
            .expect("collection projection");
        assert_eq!(projection.type_node(), &row.structural_node);
        assert_eq!(projection.value_type(), &row.value_type);
        let exact = projection.work();
        assert_eq!(
            package.project_exact_type(requested, &selected, exact),
            Ok(projection.clone())
        );
        let exhausted = package
            .project_exact_type(requested, &selected, exact - 1)
            .expect_err("one below exact allowance");
        assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);
        assert_eq!(exhausted.work, exact - 1);
        assert_eq!(exhausted.at.as_deref(), Some(&row.structural_node));
        assert_eq!(
            package
                .clone()
                .project_exact_type(requested, &selected, exact - 1),
            Err(exhausted)
        );
        assert_eq!(
            package
                .clone()
                .project_exact_type(requested, &selected, exact),
            Ok(projection)
        );
    }
    let bounded = &rows[4].0;
    assert_ne!(bounded.structural_node, bounded.base_node);
    let unrelated = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "float32")
        .expect("unrelated admitted float type");
    assert_eq!(
        package
            .project_exact_type(&unrelated.node_id, &selected, 500)
            .unwrap_err()
            .code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
}

#[trace("FR-358-AC-1", "FR-358-AC-3", "FR-358-AC-4")]
#[test]
fn native_collection_joins_share_budget_and_precharge_nonempty_prior_map_copy() {
    use quire_contract_model::{NativeJoinBudget, NativeJoinCharge, NativeJoinChargeKind};

    let (package, rows) = source_collection_fixture();
    let first = rows[0].0.clone();
    let second = rows[1].0.clone();
    let mut ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let selected = SelectedExactIdentities::empty()
        .with_source_collection_rows(&package, [first.clone()], &mut ample)
        .unwrap();
    let first_delta = ample.last_success_delta().unwrap();
    assert_eq!(ample.consumed(), first_delta);
    let complete = selected
        .with_source_collection_rows(&package, [second.clone()], &mut ample)
        .unwrap();
    let second_delta = ample.last_success_delta().unwrap();
    let total = NativeJoinCharge {
        work: first_delta.work + second_delta.work,
        storage: first_delta.storage + second_delta.storage,
    };
    assert_eq!(ample.consumed(), total);
    assert_eq!(complete.source_collection_rows().count(), 2);
    assert_eq!(selected.source_collection_rows().next(), Some(&first));
    assert_eq!(selected.source_collection_rows().count(), 1);
    assert_eq!(complete.source_collection_rows().next(), Some(&first));
    assert_eq!(
        ample.consumed(),
        total,
        "borrowed reads do not consume storage"
    );

    let cloned_package = package.clone();
    let mut replay = NativeJoinBudget::new(total.work, total.storage);
    let replay_first = SelectedExactIdentities::empty()
        .with_source_collection_rows(&cloned_package, [first.clone()], &mut replay)
        .unwrap();
    assert_eq!(replay.last_success_delta(), Some(first_delta));
    let replay_complete = replay_first
        .with_source_collection_rows(&cloned_package, [second.clone()], &mut replay)
        .unwrap();
    assert_eq!(replay_complete, complete);
    assert_eq!(replay.last_success_delta(), Some(second_delta));
    assert_eq!(replay.consumed(), total);

    for kind in [NativeJoinChargeKind::Work, NativeJoinChargeKind::Storage] {
        let (work, storage) = match kind {
            NativeJoinChargeKind::Work => (total.work - 1, u64::MAX),
            NativeJoinChargeKind::Storage => (u64::MAX, total.storage - 1),
        };
        let mut short = NativeJoinBudget::new(work, storage);
        let prior = SelectedExactIdentities::empty()
            .with_source_collection_rows(&package, [first.clone()], &mut short)
            .expect("first operation is individually affordable");
        assert_eq!(short.last_success_delta(), Some(first_delta));
        let denied = prior
            .with_source_collection_rows(&package, [second.clone()], &mut short)
            .expect_err("combined allowance stops in second operation");
        assert_eq!(denied.requested.as_ref(), &second.structural_node);
        assert_eq!(
            denied.code,
            match kind {
                NativeJoinChargeKind::Work => ExactTypeProjectionErrorCode::ExhaustedWork,
                NativeJoinChargeKind::Storage => ExactTypeProjectionErrorCode::ExhaustedStorage,
            }
        );
        assert_eq!(short.first_unpaid().unwrap().kind, kind);
        assert!(short.first_unpaid().unwrap().amount > 0);
        assert_eq!(short.last_success_delta(), None);
        assert!(short.consumed().work >= first_delta.work);
        assert!(short.consumed().storage >= first_delta.storage);
        assert_eq!(prior.source_collection_rows().count(), 1);
        assert_eq!(prior.source_collection_rows().next(), Some(&first));
        if kind == NativeJoinChargeKind::Storage {
            assert!(short.first_unpaid().unwrap().amount > 1);
            assert!(short.consumed().storage < storage);
            assert_eq!(
                u128::from(short.consumed().storage) + short.first_unpaid().unwrap().amount,
                u128::from(total.storage),
                "the denied final charge is the nonempty prior map copy"
            );
        }
    }
}

#[trace("FR-358-AC-4")]
#[test]
fn native_budget_counts_stage_and_each_join_on_one_admitted_package() {
    use quire_contract_model::{
        NativeJoinBudget, NativeJoinCharge, SelectedSourceEnumRow, SelectedSourceRecordRow,
        SelectedSourceTupleRow, SelectedSourceUnionRow,
    };

    let (package, rows) = source_collection_fixture();
    let collection = rows[0].0.clone();
    let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let mut deltas = Vec::new();
    let stage = SelectedExactIdentities::from_stage_rows(&package, [], [], &mut budget).unwrap();
    deltas.push(budget.last_success_delta().unwrap());
    let union = stage
        .with_source_union_rows(&package, Vec::<SelectedSourceUnionRow>::new(), &mut budget)
        .unwrap();
    deltas.push(budget.last_success_delta().unwrap());
    let record = union
        .with_source_record_rows(&package, Vec::<SelectedSourceRecordRow>::new(), &mut budget)
        .unwrap();
    deltas.push(budget.last_success_delta().unwrap());
    let tuple = record
        .with_source_tuple_rows(&package, Vec::<SelectedSourceTupleRow>::new(), &mut budget)
        .unwrap();
    deltas.push(budget.last_success_delta().unwrap());
    let source_enum = tuple
        .with_source_enum_rows(&package, Vec::<SelectedSourceEnumRow>::new(), &mut budget)
        .unwrap();
    deltas.push(budget.last_success_delta().unwrap());
    let completed = source_enum
        .with_source_collection_rows(&package, [collection.clone()], &mut budget)
        .unwrap();
    deltas.push(budget.last_success_delta().unwrap());
    assert_eq!(completed.source_collection_rows().next(), Some(&collection));
    assert_eq!(deltas.len(), 6);
    assert!(deltas[0].storage > 0);
    assert!(deltas[1..5].iter().all(|delta| delta.storage > 0));
    let sum = deltas
        .into_iter()
        .fold(NativeJoinCharge::default(), |sum, delta| NativeJoinCharge {
            work: sum.work + delta.work,
            storage: sum.storage + delta.storage,
        });
    assert_eq!(budget.consumed(), sum);
    assert_eq!(budget.first_unpaid(), None);
}

/// Trace: FR-352-AC-20, FR-352-AC-21.
#[trace("FR-352-AC-20", "FR-352-AC-21")]
#[test]
fn source_collection_incremental_join_charges_prior_rows_before_copy() {
    let (package, rows) = source_collection_fixture();
    let prior = rows[..2].iter().map(|(row, _)| row.clone());
    let populated = SelectedExactIdentities::empty()
        .with_source_collection_rows(
            &package,
            prior,
            &mut quire_contract_model::NativeJoinBudget::new(500, u64::MAX),
        )
        .expect("two prior collection rows");
    let next = rows[2].0.clone();
    let empty_exact = (1..=500)
        .find(|&limit| {
            SelectedExactIdentities::empty()
                .with_source_collection_rows(
                    &package,
                    [next.clone()],
                    &mut quire_contract_model::NativeJoinBudget::new(limit, u64::MAX),
                )
                .is_ok()
        })
        .expect("empty-table exact allowance");
    let populated_exact = (1..=500)
        .find(|&limit| {
            populated
                .with_source_collection_rows(
                    &package,
                    [next.clone()],
                    &mut quire_contract_model::NativeJoinBudget::new(limit, u64::MAX),
                )
                .is_ok()
        })
        .expect("populated-table exact allowance");
    assert_eq!(populated_exact, empty_exact + 2);
    let joined = populated
        .with_source_collection_rows(
            &package,
            [next.clone()],
            &mut quire_contract_model::NativeJoinBudget::new(populated_exact, u64::MAX),
        )
        .expect("exact allowance charges both prior entries");
    assert_eq!(joined.source_collection_rows().count(), 3);
    let first_unpaid = rows[..2]
        .iter()
        .map(|(row, _)| &row.structural_node)
        .max()
        .expect("two prior entries");
    let refusal = populated
        .with_source_collection_rows(
            &package,
            [next.clone()],
            &mut quire_contract_model::NativeJoinBudget::new(populated_exact - 1, u64::MAX),
        )
        .expect_err("one below the populated-table allowance");
    assert_eq!(refusal.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    assert_eq!(refusal.work, populated_exact - 1);
    assert_eq!(refusal.at.as_deref(), Some(first_unpaid));
    assert_eq!(
        populated.clone().with_source_collection_rows(
            &package,
            [next],
            &mut quire_contract_model::NativeJoinBudget::new(populated_exact - 1, u64::MAX)
        ),
        Err(refusal)
    );
}

/// Trace: FR-352-AC-21.
#[trace("FR-352-AC-21")]
#[test]
fn source_collection_rows_refuse_missing_and_changed_structural_facts() {
    let (package, rows) = source_collection_fixture();
    let (row, requested) = &rows[4];
    assert_eq!(
        package
            .project_exact_type(requested, &SelectedExactIdentities::empty(), 500)
            .unwrap_err()
            .code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    let join = |candidate: SelectedSourceCollectionRow| {
        SelectedExactIdentities::empty()
            .with_source_collection_rows(
                &package,
                [candidate],
                &mut quire_contract_model::NativeJoinBudget::new(500, u64::MAX),
            )
            .expect_err("changed collection row")
    };
    let mut foreign = row.clone();
    foreign.structural_node.package = "foreign".into();
    assert_eq!(
        join(foreign).code,
        ExactTypeProjectionErrorCode::ForeignNode
    );
    let mut wrong_form = row.clone();
    wrong_form.structural_node = row.element_node.clone();
    assert_eq!(
        join(wrong_form).code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let mut changed_base = row.clone();
    changed_base.base_node = rows[1].0.base_node.clone();
    assert_eq!(
        join(changed_base).code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let mut changed_element = row.clone();
    changed_element.element_node = row.base_node.clone();
    assert_eq!(
        join(changed_element).code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let mut changed_kind = row.clone();
    changed_kind.value_type = ValueType::collection(CollectionType::new(
        CollectionKind::Set,
        ValueType::Integer,
        Some(CardinalityBound::new(0, 3).unwrap()),
    ));
    assert_eq!(
        join(changed_kind).code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let mut changed_bound = row.clone();
    changed_bound.value_type = ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Integer,
        Some(CardinalityBound::new(1, 3).unwrap()),
    ));
    assert_eq!(
        join(changed_bound).code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    assert_eq!(
        SelectedExactIdentities::empty()
            .with_source_collection_rows(
                &package,
                [row.clone(), row.clone()],
                &mut quire_contract_model::NativeJoinBudget::new(500, u64::MAX)
            )
            .unwrap_err()
            .code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
    );
    let mut changed_opaque_element = row.clone();
    changed_opaque_element.value_type = ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Boolean,
        Some(CardinalityBound::new(0, 3).unwrap()),
    ));
    let selected = SelectedExactIdentities::empty()
        .with_source_collection_rows(
            &package,
            [changed_opaque_element],
            &mut quire_contract_model::NativeJoinBudget::new(500, u64::MAX),
        )
        .expect("structural row joins before element projection");
    let refusal = package
        .project_exact_type(requested, &selected, 500)
        .expect_err("opaque element mismatch");
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    assert_eq!(refusal.at.as_deref(), Some(&row.element_node));
}

/// Trace: FR-352-AC-21.
#[trace("FR-352-AC-21")]
#[test]
fn source_collection_with_unprojectable_option_element_keeps_native_refusal() {
    let selected = document(vec![
        json!({"identity": orders("OptInt"), "kind": "option", "inner": "ix://quire/native/Integer"}),
        object(
            "Collect",
            &[],
            &[field("xs", &orders("OptInt")).many((0, None, true, false))],
        ),
    ]);
    let package = admit(
        &selected,
        &[("Collect", OBJECT)],
        &[Read {
            owner: "Collect",
            field: "xs",
            ty: Ty::Collection(
                Kind::Sequence,
                Box::new(Ty::Option(Box::new(Ty::Integer))),
                None,
            ),
        }],
    );
    let graph = &package.graph().nodes;
    let option = graph
        .iter()
        .find(|node| node.semantic_form.as_ref() == "option")
        .expect("emitted option element")
        .node_id
        .clone();
    let base = graph
        .iter()
        .find(|node| node.semantic_form.as_ref() == "sequence")
        .expect("emitted sequence base")
        .node_id
        .clone();
    let requested = graph
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "expression"
                && node.semantic_form.as_ref() == "query"
                && node.semantic_type == base
        })
        .expect("emitted query")
        .node_id
        .clone();
    let row = SelectedSourceCollectionRow {
        structural_node: base.clone(),
        base_node: base,
        element_node: option.clone(),
        value_type: ValueType::collection(CollectionType::new(
            CollectionKind::Sequence,
            ValueType::option(ValueType::Integer),
            None,
        )),
    };
    let selected = SelectedExactIdentities::empty()
        .with_source_collection_rows(
            &package,
            [row],
            &mut quire_contract_model::NativeJoinBudget::new(500, u64::MAX),
        )
        .expect("structural collection row joins");
    let refusal = package
        .project_exact_type(&requested, &selected, 500)
        .expect_err("option element has no general Exact projection yet");
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
    assert_eq!(refusal.at.as_deref(), Some(&option));
}

/// Trace: FR-352-AC-20, FR-352-AC-21.
#[trace("FR-352-AC-20", "FR-352-AC-21")]
#[test]
fn nested_collection_rows_follow_final_element_nodes_without_recursion() {
    let selected = document(vec![object("Collect", &[], &[])]);
    let (mut wire, evidence, _) = package(&selected, &[("Collect", OBJECT)], &[]);
    let mut nodes = BTreeMap::new();
    type_nodes(
        &Ty::Collection(
            Kind::Sequence,
            Box::new(Ty::Collection(Kind::Set, Box::new(Ty::Integer), None)),
            None,
        ),
        &mut nodes,
    );
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph")
        .extend(nodes.into_values());
    settle(&mut wire);
    let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("nested collection fixture must admit: {other:?}"),
    };
    let graph = &admitted.graph().nodes;
    let integer = graph
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "scalar_type" && node.semantic_form.as_ref() == "integer"
        })
        .expect("element")
        .node_id
        .clone();
    let inner = graph
        .iter()
        .find(|node| node.semantic_form.as_ref() == "set" && node.dependencies.contains(&integer))
        .expect("inner set")
        .node_id
        .clone();
    let outer = graph
        .iter()
        .find(|node| {
            node.semantic_form.as_ref() == "sequence" && node.dependencies.contains(&inner)
        })
        .expect("outer sequence")
        .node_id
        .clone();
    let inner_type = ValueType::collection(CollectionType::new(
        CollectionKind::Set,
        ValueType::Integer,
        None,
    ));
    let outer_type = ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        inner_type.clone(),
        None,
    ));
    let inner_row = SelectedSourceCollectionRow {
        structural_node: inner.clone(),
        base_node: inner.clone(),
        element_node: integer,
        value_type: inner_type,
    };
    let outer_row = SelectedSourceCollectionRow {
        structural_node: outer.clone(),
        base_node: outer.clone(),
        element_node: inner.clone(),
        value_type: outer_type.clone(),
    };
    let identities = SelectedExactIdentities::empty()
        .with_source_collection_rows(
            &admitted,
            [outer_row, inner_row],
            &mut quire_contract_model::NativeJoinBudget::new(500, u64::MAX),
        )
        .expect("both nested rows join");
    let projected = admitted
        .project_exact_type(&outer, &identities, 500)
        .expect("iterative nested projection");
    assert_eq!(projected.value_type(), &outer_type);
    let without_inner = SelectedExactIdentities::empty()
        .with_source_collection_rows(
            &admitted,
            [SelectedSourceCollectionRow {
                structural_node: outer.clone(),
                base_node: outer.clone(),
                element_node: inner.clone(),
                value_type: outer_type,
            }],
            &mut quire_contract_model::NativeJoinBudget::new(500, u64::MAX),
        )
        .expect("outer structural row joins alone");
    let refusal = admitted
        .project_exact_type(&outer, &without_inner, 500)
        .expect_err("reached inner row required");
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    assert_eq!(refusal.at.as_deref(), Some(&inner));
}

/// A selected record with an unread field, object, and tuple for complete joins.
pub(crate) fn complete_witness_model_wire() -> (Value, Value) {
    let tuple = orders("Pair");
    let mut selected = document(vec![
        typed_object(
            "Receipt",
            "record",
            &[],
            &[
                field("count", "ix://quire/native/Boolean"),
                field("pair", &orders("Pair")),
                field("distance", "ix://quire/unit/Example/Metre"),
            ],
        ),
        object("Node", &[], &[]),
        json!({"identity":tuple,"displayName":"Pair", "kind":{"module":IDENTITY,"name":"tuple"},
            "positions":[{"identity":format!("{tuple}/p0"),"typeRef":"ix://quire/native/Boolean"}]}),
    ]);
    for (name, meaning) in [
        ("record", "quire.meaning.model.record-value-type/v1"),
        ("tuple", "quire.meaning.model.tuple-value-type/v1"),
    ] {
        selected["constructs"].as_array_mut().unwrap().push(json!({
            "kind":{"module":IDENTITY,"name":name},"construct":{"meaning":meaning}
        }));
    }
    let (wire, _, _) = package(
        &selected,
        &[
            ("Receipt", "record_value_type"),
            ("Node", OBJECT),
            ("Pair", "tuple_value_type"),
        ],
        &[],
    );
    (wire, selected)
}

pub(crate) fn complete_witness_model_rows(
    package: &CheckedPackageV2,
) -> (
    Vec<SelectedExactDeclarationRow>,
    Vec<SelectedExactFieldRow>,
    Vec<SelectedTupleDeclarationRow>,
) {
    let id = |original: &str| {
        package.graph().nodes.iter().find(|node| matches!(&node.owner, Some(quire_contract_model::CheckedNodeOwner::Model { identity, node }) if identity.as_ref() == IDENTITY && node.as_ref() == original)).unwrap().node_id.clone()
    };
    let record_key = NodeKey::new(
        "complete-stage",
        vec![PathSegment::Text("Receipt".into())],
        0,
    )
    .unwrap();
    let record_id = id(&orders("Receipt"));
    let declarations = vec![
        SelectedExactDeclarationRow::new(
            record_id.clone(),
            IDENTITY,
            SelectedExactDeclarationForm::RecordValueType,
            orders("Receipt"),
            SelectedExactIdentity::Record(record_key.clone()),
        ),
        SelectedExactDeclarationRow::new(
            id(&orders("Node")),
            IDENTITY,
            SelectedExactDeclarationForm::ObjectType,
            orders("Node"),
            SelectedExactIdentity::Object(EffectiveId::new(
                None,
                OriginalDeclarationKey::new(IDENTITY, orders("Node")),
            )),
        ),
    ];
    let fields = ["count", "pair", "distance"]
        .into_iter()
        .map(|name| {
            SelectedExactFieldRow::new(
                record_id.clone(),
                IDENTITY,
                orders("Receipt"),
                format!("{}/{name}", orders("Receipt")),
                MemberId::new(MemberOwner::Node(record_key.clone()), name).unwrap(),
            )
        })
        .collect();
    let tuples = vec![SelectedTupleDeclarationRow::new(
        id(&orders("Pair")),
        IDENTITY,
        orders("Pair"),
        NodeKey::new("complete-stage", vec![PathSegment::Text("Pair".into())], 0).unwrap(),
    )];
    (declarations, fields, tuples)
}

#[trace("TC-467", "FR-352-AC-22", "FR-352-AC-28", "FR-352-AC-29")]
#[test]
fn complete_identity_witness_reuses_checked_private_channels_for_runtime_projection() {
    use quire_contract_model::{CheckedNodeOwner, SelectedSourceUnitRow};
    let (mut wire, document) = complete_witness_model_wire();
    let unit_wire = nominal_package(
        &crate::support::checked_package::nominal_fixture_members()
            .into_iter()
            .skip(2)
            .collect::<Vec<_>>(),
    );
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .extend(
            unit_wire["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .cloned(),
        );
    settle(&mut wire);
    let evidence = evidence_with_document(&wire, IDENTITY, &document);
    let package = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("complete private-channel fixture admits: {other:?}"),
    };
    let (declarations, fields, tuples) = complete_witness_model_rows(&package);
    let units: Vec<_> = package
        .graph()
        .nodes
        .iter()
        .filter(|node| node.semantic_form.as_ref() == "unit")
        .map(|node| {
            let CheckedNodeOwner::Source {
                authority,
                identity,
            } = node.owner.as_ref().unwrap()
            else {
                panic!("source unit");
            };
            SelectedSourceUnitRow::new(
                node.node_id.clone(),
                authority.clone(),
                identity.clone(),
                node.declaration.as_ref().unwrap().qualified_name.clone(),
                UnitId::Declared(
                    NodeKey::new("complete-stage", vec![PathSegment::Text("Metre".into())], 0)
                        .unwrap(),
                ),
            )
        })
        .collect();
    let mut budget = quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX);
    let witness =
        SelectedExactIdentities::from_stage_rows(&package, declarations, fields, &mut budget)
            .unwrap()
            .with_admitted_type_rows(&package, &tuples, &units, &mut budget)
            .unwrap()
            .validate_and_rebind(&package, &mut budget)
            .unwrap();
    let view = package
        .project_selected_type_declarations(&witness, &tuples, &units, 100_000, &mut budget)
        .unwrap();
    assert_eq!(view.records().len(), 1);
    assert_eq!(view.tuples().len(), 1);
    assert_eq!(view.objects().len(), 1);
    assert_eq!(view.units().len(), 1);
    assert_eq!(view.tuples()[0].node_key(), &tuples[0].node_key);
    assert_eq!(view.units()[0].unit_id, units[0].unit_id);
    assert_eq!(
        witness.validate_and_rebind(&package.clone(), &mut budget),
        Ok(witness.clone())
    );
}

#[trace("TC-467", "FR-352-AC-23", "FR-352-AC-26")]
#[test]
fn complete_identity_witness_collection_helper_base_is_not_an_independent_obligation() {
    let build = |independent: bool, metadata_only: bool| {
        let mut fields =
            vec![field("bounded", "ix://quire/native/Integer").many((0, Some(3), true, false))];
        let mut reads = vec![Read {
            owner: "Collect",
            field: "bounded",
            ty: Ty::Collection(Kind::Sequence, Box::new(Ty::Integer), Some((0, 3))),
        }];
        if independent {
            fields.push(field("plain", "ix://quire/native/Integer").many((0, None, true, false)));
            reads.push(Read {
                owner: "Collect",
                field: "plain",
                ty: Ty::Collection(Kind::Sequence, Box::new(Ty::Integer), None),
            });
        }
        let selected = document(vec![object("Collect", &[], &fields)]);
        let (mut wire, evidence, _) = package(&selected, &[("Collect", OBJECT)], &reads);
        if metadata_only {
            let base = wire["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|node| node["semantic_form"] == "sequence")
                .unwrap()["node_id"]
                .clone();
            let object = wire["semantic_graph"]["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|node| node["semantic_form"] == "object_type")
                .unwrap();
            object["dependencies"].as_array_mut().unwrap().push(base);
            settle(&mut wire);
        }
        let admitted = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
            CheckedPackageV2ReadResult::Admitted(package) => *package,
            other => panic!("helper-base control admits: {other:?}"),
        };
        let base = admitted
            .graph()
            .nodes
            .iter()
            .find(|node| node.semantic_form.as_ref() == "sequence")
            .unwrap()
            .node_id
            .clone();
        let bounded = admitted
            .graph()
            .nodes
            .iter()
            .find(|node| node.semantic_form.as_ref() == "collection_bounds")
            .unwrap()
            .node_id
            .clone();
        let element = admitted
            .graph()
            .nodes
            .iter()
            .find(|node| node.semantic_form.as_ref() == "integer")
            .unwrap()
            .node_id
            .clone();
        let row = SelectedSourceCollectionRow {
            structural_node: bounded,
            base_node: base,
            element_node: element,
            value_type: ValueType::collection(CollectionType::new(
                CollectionKind::Sequence,
                ValueType::Integer,
                Some(CardinalityBound::new(0, 3).unwrap()),
            )),
        };
        let declaration = SelectedExactDeclarationRow::new(
            admitted
                .graph()
                .nodes
                .iter()
                .find(|node| node.semantic_form.as_ref() == "object_type")
                .unwrap()
                .node_id
                .clone(),
            IDENTITY,
            SelectedExactDeclarationForm::ObjectType,
            orders("Collect"),
            SelectedExactIdentity::Object(EffectiveId::new(
                None,
                OriginalDeclarationKey::new(IDENTITY, orders("Collect")),
            )),
        );
        let mut budget = quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX);
        let witness =
            SelectedExactIdentities::from_stage_rows(&admitted, [declaration], [], &mut budget)
                .unwrap()
                .with_source_collection_rows(&admitted, [row.clone()], &mut budget)
                .unwrap();
        (admitted, witness, row)
    };
    for metadata_only in [false, true] {
        let (package, witness, _) = build(false, metadata_only);
        assert_eq!(
            witness.validate_and_rebind(
                &package,
                &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
            ),
            Ok(witness.clone()),
            "a helper or dependency-only edge is not an independent type use"
        );
    }
    let (package, witness, bounded) = build(true, false);
    let refusal = witness
        .validate_and_rebind(
            &package,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::IncompleteIdentityWitness
    );
    assert_eq!(refusal.at.as_deref(), Some(&bounded.base_node));
    let plain = SelectedSourceCollectionRow {
        structural_node: bounded.base_node.clone(),
        base_node: bounded.base_node,
        element_node: bounded.element_node,
        value_type: ValueType::collection(CollectionType::new(
            CollectionKind::Sequence,
            ValueType::Integer,
            None,
        )),
    };
    let complete = witness
        .with_source_collection_rows(
            &package,
            [plain],
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap();
    assert_eq!(
        complete.validate_and_rebind(
            &package,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)
        ),
        Ok(complete.clone())
    );
}

#[trace("TC-467", "FR-352-AC-23", "FR-352-AC-28")]
#[test]
fn complete_identity_witness_refuses_unemitted_and_unsupported_selected_identity_types() {
    let selected = document(vec![
        object("Present", &[], &[]),
        object("Unread", &[], &[]),
    ]);
    let package = admit(&selected, &[("Present", OBJECT)], &[]);
    let row = SelectedExactDeclarationRow::new(
        declaration_id("Present"),
        IDENTITY,
        SelectedExactDeclarationForm::ObjectType,
        orders("Present"),
        SelectedExactIdentity::Object(EffectiveId::new(
            None,
            OriginalDeclarationKey::new(IDENTITY, orders("Present")),
        )),
    );
    let witness = SelectedExactIdentities::from_stage_rows(
        &package,
        [row],
        [],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    let error = witness
        .validate_and_rebind(
            &package,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration
    );
    let root = &package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "model_root")
        .unwrap()
        .node_id;
    assert_eq!(error.at.as_deref(), Some(root));
    let mut selected = document(vec![process(
        "Flow",
        &[],
        &[field("n", "ix://quire/native/Integer")],
    )]);
    selected["constructs"].as_array_mut().unwrap().push(json!({"kind":{"module":IDENTITY,"name":"process"},"construct":{"meaning":"quire.meaning.model.process/v1"}}));
    let package = admit(&selected, &[("Flow", "process")], &[]);
    let id = &package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "process")
        .unwrap()
        .node_id;
    let empty = SelectedExactIdentities::empty();
    assert_eq!(
        package
            .project_exact_type(id, &empty, 10_000)
            .unwrap_err()
            .code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
    let error = empty
        .validate_and_rebind(
            &package,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
    assert_eq!(error.at.as_deref(), Some(id));
}

/// A strict reader-admitted record with an unread field and a model unit.
fn complete_review_model_unit(original: &str, extra_nodes: usize) -> CheckedPackageV2 {
    let mut selected = document(vec![typed_object(
        "Receipt",
        "record",
        &[],
        &[field("count", "ix://quire/native/Boolean")],
    )]);
    selected["constructs"].as_array_mut().unwrap().push(json!({
        "kind":{"module":IDENTITY,"name":"record"},
        "construct":{"meaning":"quire.meaning.model.record-value-type/v1"}
    }));
    let dimension_original = "ix://acme/orders/dimension/Time";
    selected["dimensions"] =
        json!([{"identity":dimension_original,"name":"Time","base":true,"terms":[]}]);
    selected["units"] = json!([{"identity":original,"name":"second","dimension":dimension_original,"scale":"1","offset":"0"}]);
    let (mut wire, _, _) = package(&selected, &[("Receipt", "record_value_type")], &[]);
    let dimension = structural_id(
        IDENTITY,
        vec![
            json!("scalar_type"),
            json!("dimension"),
            json!(dimension_original),
        ],
        0,
    );
    let unit = structural_id(
        IDENTITY,
        vec![json!("scalar_type"), json!("unit"), json!(original)],
        0,
    );
    let nodes = wire["semantic_graph"]["nodes"].as_array_mut().unwrap();
    for (id, form, original, ty) in [
        (&dimension, "dimension", dimension_original, &dimension),
        (&unit, "unit", original, &dimension),
    ] {
        nodes.push(
            json!({"node_id":id,"schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"scalar_type","semantic_form":form,"semantic_type":ty,"dependencies":if form == "unit" { vec![dimension.clone()] } else { vec![] },
            "occurrences":[{"role":"generated","ordinal":0}],
            "owner":{"kind":"model","identity":IDENTITY,"node":original},"body":empty()}),
        );
    }
    for i in 0..extra_nodes {
        let id = format!("review-unrelated-{i}");
        nodes.push(wire_node(&id, "scalar_type", "boolean", &id, &[], empty()));
    }
    settle(&mut wire);
    let evidence = evidence_with_document(&wire, IDENTITY, &selected);
    match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("complete review model unit fixture must admit: {other:?}"),
    }
}

#[trace("TC-467", "FR-352-AC-26")]
#[test]
fn complete_identity_witness_missing_model_unit_precedes_unread_field() {
    let unit_original = "ix://acme/orders/unit/second";
    let package = complete_review_model_unit(unit_original, 0);
    let record = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "record_value_type")
        .unwrap()
        .node_id
        .clone();
    let unit = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "unit")
        .unwrap()
        .node_id
        .clone();
    let witness = SelectedExactIdentities::from_stage_rows(
        &package,
        [SelectedExactDeclarationRow::new(
            record,
            IDENTITY,
            SelectedExactDeclarationForm::RecordValueType,
            orders("Receipt"),
            SelectedExactIdentity::Record(
                NodeKey::new(
                    "complete-review",
                    vec![PathSegment::Text("Receipt".into())],
                    0,
                )
                .unwrap(),
            ),
        )],
        [],
        &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
    )
    .unwrap();
    let error = witness
        .validate_and_rebind(
            &package,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::IncompleteIdentityWitness
    );
    assert_eq!(error.form.as_ref(), "selected_model_unit");
    assert_eq!(error.at.as_deref(), Some(&unit));
}

#[trace("TC-467", "FR-352-AC-25")]
#[test]
fn complete_identity_witness_precharges_long_model_unit_owner_text() {
    let original = format!("ix://acme/orders/unit/{}", "x".repeat(4096));
    let package = complete_review_model_unit(&original, 0);
    let record_original = orders("Receipt");
    // One selected model; record and unit document obligations; the sole
    // selected declaration is inspected once for each obligation. This is a
    // fixed logical ledger, independent of an ample implementation run.
    let mut prefix = 16 + 5 + 3 * record_original.len() + original.len();
    let mut nodes: Vec<_> = package.graph().nodes.iter().collect();
    nodes.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    let mut expected_amount = None;
    for node in nodes {
        prefix += 1;
        if let Some(quire_contract_model::CheckedNodeOwner::Model {
            identity,
            node: owner,
        }) = &node.owner
        {
            prefix += identity.len() + IDENTITY.len();
            expected_amount = Some(owner.len() + original.len());
            break;
        }
    }
    let mut budget = quire_contract_model::NativeJoinBudget::new(prefix as u64, u64::MAX);
    let error = SelectedExactIdentities::empty()
        .validate_and_rebind(&package, &mut budget)
        .unwrap_err();
    assert_eq!(error.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    assert_eq!(error.form.as_ref(), "complete_model_unit_original");
    assert_eq!(budget.consumed().work, prefix as u64);
    assert_eq!(
        budget.first_unpaid().unwrap().amount,
        expected_amount.unwrap() as u128
    );
}

#[trace("TC-467", "FR-352-AC-25")]
#[test]
fn complete_identity_witness_precharges_every_unrelated_census_candidate() {
    for extra in [0, 64, 256] {
        let selected = document(Vec::new());
        let (mut wire, _, _) = package(&selected, &[], &[]);
        for i in 0..extra {
            let id = format!("unrelated-census-{i}");
            wire["semantic_graph"]["nodes"]
                .as_array_mut()
                .unwrap()
                .push(wire_node(&id, "scalar_type", "boolean", &id, &[], empty()));
        }
        settle(&mut wire);
        let evidence = evidence_with_document(&wire, IDENTITY, &selected);
        let package = match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
            CheckedPackageV2ReadResult::Admitted(package) => *package,
            other => panic!("unrelated census fixture admits: {other:?}"),
        };
        // Outer binding and content-stage binding each consume eight;
        // even an empty selected model costs one.
        let mut empty_model = quire_contract_model::NativeJoinBudget::new(16, u64::MAX);
        let error = SelectedExactIdentities::empty()
            .validate_and_rebind(&package, &mut empty_model)
            .unwrap_err();
        assert_eq!(error.form.as_ref(), "complete_selected_model_candidate");
        assert_eq!(empty_model.first_unpaid().unwrap().amount, 1);
        // The next unit-declaration candidate must be paid before node_kind,
        // even when all candidates happen to be unrelated non-units.
        let mut candidate = quire_contract_model::NativeJoinBudget::new(17, u64::MAX);
        let error = SelectedExactIdentities::empty()
            .validate_and_rebind(&package, &mut candidate)
            .unwrap_err();
        assert_eq!(error.code, ExactTypeProjectionErrorCode::ExhaustedWork);
        assert_eq!(error.form.as_ref(), "complete_model_unit_candidate");
        assert_eq!(candidate.consumed().work, 17);
        assert_eq!(candidate.first_unpaid().unwrap().amount, 1);
    }
}

#[trace("TC-467", "FR-352-AC-28", "FR-352-AC-29")]
#[test]
fn complete_identity_witness_tuple_validation_matches_projection() {
    let package = integer_add_control();
    let primitive = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_tag.as_ref() == "scalar_type")
        .unwrap()
        .node_id
        .clone();
    let row = SelectedTupleDeclarationRow::new(
        primitive.clone(),
        "nonexistent-document",
        "nonexistent-tuple",
        NodeKey::new("review", vec![PathSegment::Text("Tuple".into())], 0).unwrap(),
    );
    let empty = SelectedExactIdentities::empty();
    let direct = empty
        .with_admitted_type_rows(
            &package,
            std::slice::from_ref(&row),
            &[],
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    let projection = package
        .project_selected_type_declarations(
            &empty,
            &[row],
            &[],
            u64::MAX,
            &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        direct.code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    assert_eq!(projection.code, direct.code);
    assert_eq!(projection.at.as_ref(), &primitive);
}
