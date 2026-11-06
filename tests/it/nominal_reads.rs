// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-354 nominal joins over reader-admitted canonical nominal fixtures.
//! Opaque stage keys below are explicit row fixtures, not QSL provenance evidence.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_fixture_members, nominal_package, refresh_identity,
};
use ix_trace_rs::trace;
use quire_contract_model::nominal_reads::{
    EnumMemberRow, NominalDeclarationRow, NominalIdentity, NominalReadCode,
};
use quire_contract_model::{
    CheckedNodeOwner, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedSemanticNodeV2, ExactTypeProjectionErrorCode, SelectedEnumDeclarationRow,
    SelectedEnumMemberRow, SelectedExactIdentities, SelectedSourceEnumRow, SelectedSourceUnitRow,
};
use quire_exact::{EnumShape, NodeKey, PathSegment, UnitId, ValueType, VariantId};
use serde_json::{json, Value};

fn fixture(ordered: bool) -> Box<CheckedPackageV2> {
    let mut members = nominal_fixture_members();
    members[1].0["ordered"] = json!(ordered);
    let mut done = members[0].0.clone();
    done["case"] = json!("DONE");
    members.push((done, "value/enum_value/DONE".to_owned()));
    let mut converted = members[2].0.clone();
    converted["qualified_declaration"] = json!(["Example", "Centimetre"]);
    converted["target_unit_node_id"] = node_id(&members[2].1);
    converted["scale"] = json!({"numerator":"1", "denominator":"100"});
    converted["offset"] = json!({"numerator":"3", "denominator":"2"});
    members.push((converted, "qualified:Example/Centimetre".to_owned()));
    let mut other_enum = members[1].0.clone();
    other_enum["qualified_declaration"] = json!(["Example", "OtherStatus"]);
    members.push((other_enum, "qualified:Example/OtherStatus".to_owned()));
    for case in ["ACTIVE", "DONE"] {
        let mut member = members[0].0.clone();
        member["declaration_node_id"] = node_id("qualified:Example/OtherStatus");
        member["case"] = json!(case);
        members.push((member, format!("value/enum_value/OtherStatus/{case}")));
    }
    let mut area = members[3].0.clone();
    area["qualified_declaration"] = json!(["Example", "Area"]);
    area["terms"] = json!([{"dimension_node_id": node_id(&members[3].1), "exponent": "2"}]);
    members.push((area, "qualified:Example/Area".to_owned()));
    let mut square = members[2].0.clone();
    square["qualified_declaration"] = json!(["Example", "SquareMetre"]);
    square["dimension_node_id"] = node_id("qualified:Example/Area");
    members.push((square, "qualified:Example/SquareMetre".to_owned()));
    let wire = nominal_package(&members);
    match CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("nominal fixture must admit: {other:?}"),
    }
}
fn minimal_enum_fixture(ordered: bool) -> Box<CheckedPackageV2> {
    let mut members = nominal_fixture_members();
    members[1].0["ordered"] = json!(ordered);
    let mut done = members[0].0.clone();
    done["case"] = json!("DONE");
    members.truncate(2);
    members.push((done, "value/enum_value/DONE".to_owned()));
    let wire = nominal_package(&members);
    match CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("minimal enum fixture must admit: {other:?}"),
    }
}
fn node<'a>(package: &'a CheckedPackageV2, form: &str, name: &str) -> &'a CheckedSemanticNodeV2 {
    package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.semantic_form.as_ref() == form
                && node
                    .declaration
                    .as_ref()
                    .is_some_and(|d| d.qualified_name.last().is_some_and(|n| n.as_ref() == name))
        })
        .expect("one named declaration")
}
fn key(name: &str) -> NodeKey {
    NodeKey::new("stage-row-fixture", vec![PathSegment::Text(name.into())], 0).unwrap()
}

fn source_enum_row(package: &CheckedPackageV2, ordered: bool) -> SelectedSourceEnumRow {
    let declaration = node(package, "enum", "Status");
    let source = &package.lock().sources[0];
    let declaration_key = key("Status-opaque");
    let variants = ["opaque-A", "opaque-B"]
        .map(|name| VariantId::new(key("variant-owner"), name).expect("fixture variant"));
    let shape = EnumShape::new(ordered, variants.clone());
    let members = [
        ("ACTIVE", "value/enum_value/ACTIVE", "member-A"),
        ("DONE", "value/enum_value/DONE", "member-B"),
    ]
    .into_iter()
    .zip(variants)
    .map(|((case, id, member_key), variant)| SelectedEnumMemberRow {
        structural_node: serde_json::from_value(node_id(id)).expect("member ID"),
        source_authority: source.authority.clone(),
        source_identity: source.identity.clone(),
        declaration_node: declaration.node_id.clone(),
        declaration_qualified_name: vec!["Example".into(), "Status".into()],
        case: case.into(),
        member_key: key(member_key),
        variant,
    })
    .collect();
    SelectedSourceEnumRow {
        declaration: SelectedEnumDeclarationRow {
            structural_node: declaration.node_id.clone(),
            source_authority: source.authority.clone(),
            source_identity: source.identity.clone(),
            qualified_declaration: vec!["Example".into(), "Status".into()],
            node_key: declaration_key,
            shape,
        },
        members,
    }
}

/// Trace: FR-352-AC-19.
#[trace("FR-352-AC-19", "FR-358-AC-2")]
#[test]
fn source_enum_exact_projection_requires_complete_canonical_rows() {
    for ordered in [false, true] {
        let package = fixture(ordered);
        let row = source_enum_row(&package, ordered);
        let id = row.declaration.structural_node.clone();
        let (budgeted, _) = crate::native_join_budget::assert_exact_join(&id, &id, |budget| {
            SelectedExactIdentities::empty().with_source_enum_rows(&package, [row.clone()], budget)
        });
        let selected = SelectedExactIdentities::empty()
            .with_source_enum_rows(
                &package,
                [row.clone()],
                &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX),
            )
            .expect("complete source enum join");
        assert_eq!(selected, budgeted);
        let join_exact = (0..=1000)
            .find(|&limit| {
                SelectedExactIdentities::empty()
                    .with_source_enum_rows(
                        &package,
                        [row.clone()],
                        &mut quire_contract_model::NativeJoinBudget::new(limit, u64::MAX),
                    )
                    .is_ok()
            })
            .unwrap();
        assert!(join_exact > 0);
        assert_eq!(
            SelectedExactIdentities::empty()
                .with_source_enum_rows(
                    &package,
                    [row.clone()],
                    &mut quire_contract_model::NativeJoinBudget::new(join_exact - 1, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::ExhaustedWork
        );
        assert_eq!(selected.source_enum_rows().next(), Some(&row));
        let projection = package
            .project_exact_type(
                &id,
                &selected,
                &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX),
            )
            .unwrap();
        assert_eq!(
            projection.value_type(),
            &ValueType::Enum(row.declaration.shape.clone())
        );
        assert_eq!(projection.source_enum_view(), Some(&row));
        assert_eq!(
            projection.source_enum_view().unwrap().members[0].member_key,
            key("member-A")
        );
        let exact = projection.work();
        assert_eq!(
            package
                .project_exact_type(
                    &id,
                    &selected,
                    &mut quire_contract_model::NativeJoinBudget::new(exact, u64::MAX)
                )
                .unwrap(),
            projection
        );
        assert_eq!(
            package
                .project_exact_type(
                    &id,
                    &selected,
                    &mut quire_contract_model::NativeJoinBudget::new(exact - 1, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::ExhaustedWork
        );
        assert_eq!(
            package
                .clone()
                .project_exact_type(
                    &id,
                    &selected,
                    &mut quire_contract_model::NativeJoinBudget::new(exact, u64::MAX)
                )
                .unwrap(),
            projection
        );
        let minimal = minimal_enum_fixture(ordered);
        let minimal_row = source_enum_row(&minimal, ordered);
        let minimal_selected = SelectedExactIdentities::empty()
            .with_source_enum_rows(
                &minimal,
                [minimal_row],
                &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX),
            )
            .unwrap();
        assert_eq!(
            minimal
                .project_exact_type(
                    &id,
                    &minimal_selected,
                    &mut quire_contract_model::NativeJoinBudget::new(exact, u64::MAX)
                )
                .unwrap(),
            projection
        );
        assert_eq!(
            package
                .project_exact_type(
                    &id,
                    &SelectedExactIdentities::empty(),
                    &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::UnavailableIdentityWitness
        );

        let mut changed_case = row.clone();
        changed_case.members[1].case = "OTHER".into();
        let refusal = SelectedExactIdentities::empty()
            .with_source_enum_rows(
                &package,
                [changed_case],
                &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(
            refusal.code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        assert_eq!(refusal.at.as_deref(), Some(&row.members[1].structural_node));

        let mut changed_variant = row.clone();
        changed_variant.members[1].variant = changed_variant.members[0].variant.clone();
        assert_eq!(
            SelectedExactIdentities::empty()
                .with_source_enum_rows(
                    &package,
                    [changed_variant],
                    &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        let mut reordered = row.clone();
        reordered.members.swap(0, 1);
        assert_eq!(
            SelectedExactIdentities::empty()
                .with_source_enum_rows(
                    &package,
                    [reordered],
                    &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        let mut incomplete = row.clone();
        incomplete.members.pop();
        assert_eq!(
            SelectedExactIdentities::empty()
                .with_source_enum_rows(
                    &package,
                    [incomplete],
                    &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        let mut extra = row.clone();
        extra.members.push(extra.members[0].clone());
        assert_eq!(
            SelectedExactIdentities::empty()
                .with_source_enum_rows(
                    &package,
                    [extra],
                    &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        let mut duplicate_key = row.clone();
        duplicate_key.members[1].member_key = duplicate_key.members[0].member_key.clone();
        let refusal = SelectedExactIdentities::empty()
            .with_source_enum_rows(
                &package,
                [duplicate_key],
                &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(
            refusal.code,
            ExactTypeProjectionErrorCode::ConflictingIdentityWitness
        );
        assert_eq!(refusal.at.as_deref(), Some(&row.members[1].structural_node));
        let mut wrong_source = row.clone();
        wrong_source.declaration.source_identity = "other-source".into();
        assert_eq!(
            SelectedExactIdentities::empty()
                .with_source_enum_rows(
                    &package,
                    [wrong_source],
                    &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        let mut wrong_form = row.clone();
        wrong_form.declaration.structural_node = row.members[0].structural_node.clone();
        assert_eq!(
            SelectedExactIdentities::empty()
                .with_source_enum_rows(
                    &package,
                    [wrong_form],
                    &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        let mut wrong_member_node = row.clone();
        wrong_member_node.members[0].structural_node =
            serde_json::from_value(node_id("value/enum_value/OtherStatus/ACTIVE")).unwrap();
        let refusal = SelectedExactIdentities::empty()
            .with_source_enum_rows(
                &package,
                [wrong_member_node],
                &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(
            refusal.code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        assert_eq!(
            refusal.at.as_deref(),
            Some(&serde_json::from_value(node_id("value/enum_value/OtherStatus/ACTIVE")).unwrap())
        );
        let mut wrong_ordered = row.clone();
        wrong_ordered.declaration.shape =
            EnumShape::new(!ordered, row.declaration.shape.variants().cloned());
        assert_eq!(
            SelectedExactIdentities::empty()
                .with_source_enum_rows(
                    &package,
                    [wrong_ordered],
                    &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        let mut foreign = row.clone();
        foreign.declaration.structural_node.package = "other-package".into();
        let foreign_id = foreign.declaration.structural_node.clone();
        let refusal = SelectedExactIdentities::empty()
            .with_source_enum_rows(
                &package,
                [foreign],
                &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(refusal.code, ExactTypeProjectionErrorCode::ForeignNode);
        assert_eq!(refusal.at.as_deref(), Some(&foreign_id));
        assert_eq!(
            SelectedExactIdentities::empty()
                .with_source_enum_rows(
                    &package,
                    [row.clone(), row],
                    &mut quire_contract_model::NativeJoinBudget::new(2000, u64::MAX)
                )
                .unwrap_err()
                .code,
            ExactTypeProjectionErrorCode::ConflictingIdentityWitness
        );
    }
}

/// Trace: FR-352-AC-19.
#[trace("FR-352-AC-19")]
#[test]
fn source_enum_member_from_another_selected_source_refuses() {
    let mut members = nominal_fixture_members();
    let mut done = members[0].0.clone();
    done["case"] = json!("DONE");
    members.truncate(2);
    members.push((done, "value/enum_value/DONE".to_owned()));
    let mut wire = nominal_package(&members);
    let second_source = wire["lock"]["sources"][1].clone();
    wire["source_map"][0]["regions"][0]["source"] = second_source;
    refresh_identity(&mut wire);
    let package = match CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("two selected source fixture must admit: {other:?}"),
    };
    let row = source_enum_row(&package, false);
    let member_id = row.members[0].structural_node.clone();
    let refusal = SelectedExactIdentities::empty()
        .with_source_enum_rows(
            &package,
            [row],
            &mut quire_contract_model::NativeJoinBudget::new(1000, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    assert_eq!(refusal.at.as_deref(), Some(&member_id));
}

/// Trace: FR-353-AC-7, FR-352-AC-11.
/// Provenance: IR-1209 scoped source-unit join before unavailable dimension identity.
#[test]
fn equality_source_unit_row_joins_before_missing_dimension_key() {
    use quire_contract_model::{
        AdmittedEqualityReadErrorCode, CheckedOccurrence, CheckedOccurrenceRole,
        EqualityFactWitness, EqualityUnitRole, EqualityWitnessKind, SelectedEqualityApplicationRow,
        SelectedEqualityFactRow,
    };
    use quire_exact::{Location, Origin, Role};

    let members = nominal_fixture_members()
        .into_iter()
        .skip(2)
        .collect::<Vec<_>>();
    let mut wire = nominal_package(&members);
    let boolean = json!({"package":wire["name"],"path":["scalar_type","boolean"],"ordinal":0});
    let application = node_id("equality-application");
    let unit = node_id("qualified:Example/Metre");
    let dimension = node_id("qualified:Example/Length");
    let source = wire["lock"]["sources"][0].clone();
    let graph_nodes = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    graph_nodes.push(json!({
        "node_id":boolean,"schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"scalar_type","semantic_form":"boolean","semantic_type":boolean,
        "dependencies":[],"occurrences":[{"role":"type","ordinal":0}],
        "body":{"term":"aggregate","members":[]}
    }));
    graph_nodes.push(json!({
        "node_id":application,"schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"expression","semantic_form":"binary","semantic_type":boolean,
        "dependencies":[unit],"occurrences":[{"role":"expression","ordinal":0}],
        "body":{"term":"application","operator":"binary",
            "operation":{"identity":"quire.op.quantity.eq","laws":[],"mode":null,
                "member":null,"leaves":[]},
            "result_type":boolean,
            "arguments":[{"term":"reference","target":unit},{"term":"reference","target":unit}]}
    }));
    let source_map = wire["source_map"].as_array_mut().expect("source map");
    source_map.push(json!({"node_id":boolean,"role":"type","ordinal":0,
        "regions":[{"source":source,"start":0,"end":1}]}));
    source_map.push(
        json!({"node_id":application,"role":"expression","ordinal":0,
        "regions":[{"source":source,"start":1,"end":2}]}),
    );
    wire["checked_term_data"] = json!({"equality_decisions":[{
        "application":application,"occurrence":{"role":"expression","ordinal":0},
        "term_path":[],"operator":"equal","schedule":"quantity",
        "left":{"operand":0,"source_type":unit,"conversion_target":null},
        "right":{"operand":1,"source_type":unit,"conversion_target":null},
        "units":[{"quantity_type":unit,"unit":unit,"form":{
            "kind":"declared","dimension_node":dimension,"dimension_terms":[],
            "root_unit":unit,"path":[]}}],"enum_members":[]
    }],"protocol_locations":[],"dispatch_tables":[],"probabilistic_claims":[]});
    let package = match CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("quantity equality fixture admits: {other:?}"),
    };
    let application = serde_json::from_value(application).expect("application node");
    let occurrence = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let original = key("quantity-equality");
    let app_row = SelectedEqualityApplicationRow::new(
        application,
        occurrence.clone(),
        serde_json::from_value(
            wire["source_map"]
                .as_array()
                .expect("source map")
                .last()
                .expect("app source")
                .clone(),
        )
        .expect("app source row"),
        original.clone(),
        Location::new(original, Origin::new(Role::from("expression"), 0)),
    );
    let metre = node(&package, "unit", "Metre");
    let CheckedNodeOwner::Source {
        authority,
        identity,
    } = metre.owner.as_ref().expect("source owner")
    else {
        panic!("Metre has source owner")
    };
    let unit_row = SelectedSourceUnitRow::new(
        metre.node_id.clone(),
        authority.clone(),
        identity.clone(),
        metre
            .declaration
            .as_ref()
            .expect("unit declaration")
            .qualified_name
            .clone(),
        UnitId::Declared(key("Metre")),
    );
    let fact_row = SelectedEqualityFactRow {
        application: app_row.node_key.clone(),
        checked_application: app_row.application.clone(),
        occurrence: occurrence.clone(),
        source_map_entry: app_row.source_map_entry.clone(),
        structural_key: key("Metre"),
        structural_node: metre.node_id.clone(),
        owner: metre.owner.clone(),
        value_type: None,
        fact: EqualityFactWitness::Unit {
            unit_id: UnitId::Declared(key("Metre")),
            quantity_type: metre.node_id.clone(),
            operand_position: 0,
            role: EqualityUnitRole::Source,
        },
    };
    let missing = package
        .admitted_equality(
            &app_row.application,
            &occurrence,
            std::slice::from_ref(&app_row),
            std::slice::from_ref(&fact_row),
            &[],
            10_000,
        )
        .expect_err("source UnitId is required");
    assert_eq!(
        missing.code,
        AdmittedEqualityReadErrorCode::UnavailableEqualityWitness
    );
    assert_eq!(missing.witness, Some(EqualityWitnessKind::SourceUnit));
    assert_eq!(*missing.at, metre.node_id);

    let missing = package
        .admitted_equality(
            &app_row.application,
            &occurrence,
            std::slice::from_ref(&app_row),
            std::slice::from_ref(&fact_row),
            std::slice::from_ref(&unit_row),
            10_000,
        )
        .expect_err("dimension key is unavailable");
    assert_eq!(
        missing.code,
        AdmittedEqualityReadErrorCode::UnavailableEqualityWitness
    );
    assert_eq!(
        missing.witness,
        Some(EqualityWitnessKind::DimensionDeclaration)
    );
    assert_eq!(
        *missing.at,
        serde_json::from_value(dimension).expect("dimension node")
    );
}

#[test]
#[trace("FR-354-AC-1", "FR-354-AC-2", "FR-354-AC-5", "FR-354-AC-7")]
fn selected_source_units_join_owner_declaration_and_complete_conversion() {
    let mut members = nominal_fixture_members()
        .into_iter()
        .skip(2)
        .collect::<Vec<_>>();
    let mut converted = members[0].0.clone();
    converted["qualified_declaration"] = json!(["Example", "Centimetre"]);
    converted["target_unit_node_id"] = node_id("qualified:Example/Metre");
    converted["scale"] = json!({"numerator": "1", "denominator": "100"});
    members.push((converted, "qualified:Example/Centimetre".into()));
    let wire = nominal_package(&members);
    let package = match CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("unit package must admit: {other:?}"),
    };
    let root = node(&package, "unit", "Metre");
    let converted = node(&package, "unit", "Centimetre");
    let stage_row = |node: &CheckedSemanticNodeV2, name: &str| {
        let CheckedNodeOwner::Source {
            authority,
            identity,
        } = node.owner.as_ref().unwrap()
        else {
            panic!("source owner")
        };
        SelectedSourceUnitRow::new(
            node.node_id.clone(),
            authority.clone(),
            identity.clone(),
            node.declaration.as_ref().unwrap().qualified_name.clone(),
            UnitId::Declared(key(name)),
        )
    };
    let root_row = stage_row(root, "Metre");
    let converted_row = stage_row(converted, "Centimetre");
    let rows = [root_row.clone(), converted_row.clone()];
    let identities = SelectedExactIdentities::empty();
    let mut measured_budget = quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX);
    let view = package
        .project_selected_type_declarations(&identities, &[], &rows, &mut measured_budget)
        .unwrap();
    assert_eq!(view.units().len(), 2);
    let measured = measured_budget.consumed();
    let mut saw_first_insertion = false;
    for storage in 0..measured.storage {
        let mut short = quire_contract_model::NativeJoinBudget::new(measured.work, storage);
        let error = package
            .project_selected_type_declarations(&identities, &[], &rows, &mut short)
            .unwrap_err();
        if error.denied_locus
            == Some(quire_contract_model::NativeJoinLocus {
                source: quire_contract_model::NativeJoinLocusSource::SourceUnitRow(0),
                form: "selected_source_unit_index",
            })
        {
            assert_eq!(
                error.denied_node(&package, &[], &rows),
                Some(&rows[0].structural_node)
            );
            assert_eq!(error.denied_locus, short.first_unpaid_locus());
            assert_eq!(error.denied, short.first_unpaid());
            saw_first_insertion = true;
            break;
        }
    }
    assert!(
        saw_first_insertion,
        "first source-unit insertion must retain row zero after the multirow join"
    );

    assert_eq!(view.dimensions().len(), 1);
    let converted_view = view
        .units()
        .iter()
        .find(|unit| unit.structural_node == converted.node_id)
        .unwrap();
    assert_eq!(converted_view.unit_id, converted_row.unit_id);
    assert_eq!(converted_view.target_node.as_ref(), Some(&root.node_id));
    assert_eq!(converted_view.scale.numerator.as_ref(), "1");
    assert_eq!(converted_view.scale.denominator.as_ref(), "100");
    assert_eq!(converted_view.offset.numerator.as_ref(), "0");
    assert_eq!(
        view.dimensions()[0].structural_node,
        converted_view.dimension_node
    );
    assert_eq!(
        package.clone().project_selected_type_declarations(
            &identities,
            &[],
            &rows,
            &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX)
        ),
        Ok(view.clone())
    );
    let mut last_storage =
        quire_contract_model::NativeJoinBudget::new(measured.work, measured.storage - 1);
    let late = package
        .project_selected_type_declarations(&identities, &[], &rows, &mut last_storage)
        .unwrap_err();
    let dimension = &view.dimensions()[0].structural_node;
    let position = package
        .graph()
        .nodes
        .iter()
        .position(|node| &node.node_id == dimension)
        .unwrap();
    assert_eq!(
        late.denied_locus,
        Some(quire_contract_model::NativeJoinLocus {
            source: quire_contract_model::NativeJoinLocusSource::Package,
            form: "runtime_model_units"
        })
    );
    assert_eq!(late.denied_node(&package, &[], &rows), None);
    assert_eq!(late.denied, last_storage.first_unpaid());
    // The last output-container header follows the paid dimension copy.
    let mut dimension_storage =
        quire_contract_model::NativeJoinBudget::new(measured.work, measured.storage - 2);
    let dimension_denied = package
        .project_selected_type_declarations(&identities, &[], &rows, &mut dimension_storage)
        .unwrap_err();
    assert_eq!(
        dimension_denied.denied_locus,
        Some(quire_contract_model::NativeJoinLocus {
            source: quire_contract_model::NativeJoinLocusSource::GraphNode(position),
            form: "selected_dimension",
        })
    );
    assert_eq!(
        dimension_denied.denied_node(&package, &[], &rows),
        Some(dimension)
    );
    assert_eq!(dimension_denied.denied, dimension_storage.first_unpaid());
    let missing = package
        .project_selected_type_declarations(
            &identities,
            &[],
            std::slice::from_ref(&root_row),
            &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        missing.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    let conflict = package
        .project_selected_type_declarations(
            &identities,
            &[],
            &[root_row.clone(), root_row.clone(), converted_row.clone()],
            &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        conflict.code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
    );
    let foreign = SelectedSourceUnitRow::new(
        converted.node_id.clone(),
        "other",
        converted_row.source_identity.clone(),
        converted_row.qualified_declaration.clone(),
        converted_row.unit_id.clone(),
    );
    let mismatch = package
        .project_selected_type_declarations(
            &identities,
            &[],
            &[root_row.clone(), foreign],
            &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        mismatch.code,
        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
    );
    let mut foreign_node = converted.node_id.clone();
    foreign_node.package = "different-package".into();
    let foreign = SelectedSourceUnitRow::new(
        foreign_node,
        converted_row.source_authority.clone(),
        converted_row.source_identity.clone(),
        converted_row.qualified_declaration.clone(),
        converted_row.unit_id.clone(),
    );
    let refusal = package
        .project_selected_type_declarations(
            &identities,
            &[],
            &[root_row.clone(), foreign],
            &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(refusal.code, ExactTypeProjectionErrorCode::ForeignNode);
    let duplicate_key = SelectedSourceUnitRow::new(
        converted.node_id.clone(),
        converted_row.source_authority.clone(),
        converted_row.source_identity.clone(),
        converted_row.qualified_declaration.clone(),
        root_row.unit_id.clone(),
    );
    let conflict = package
        .project_selected_type_declarations(
            &identities,
            &[],
            &[root_row, duplicate_key],
            &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(
        conflict.code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
    );
    let exhausted = package
        .project_selected_type_declarations(
            &identities,
            &[],
            &rows,
            &mut quire_contract_model::NativeJoinBudget::new(view.work() - 1, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);

    let mut long_rows = rows.clone();
    long_rows[1].unit_id = UnitId::Declared(
        NodeKey::new("unit-stage", vec![PathSegment::Text("x".repeat(4096))], 0).unwrap(),
    );
    let long_view = package
        .project_selected_type_declarations(
            &identities,
            &[],
            &long_rows,
            &mut quire_contract_model::NativeJoinBudget::new(20_000, u64::MAX),
        )
        .unwrap();
    assert!(long_view.work() > view.work() + 4000);
    assert!(package
        .project_selected_type_declarations(
            &identities,
            &[],
            &long_rows,
            &mut quire_contract_model::NativeJoinBudget::new(long_view.work(), u64::MAX)
        )
        .is_ok());
    let one_less = package
        .project_selected_type_declarations(
            &identities,
            &[],
            &long_rows,
            &mut quire_contract_model::NativeJoinBudget::new(long_view.work() - 1, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(one_less.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    let tight = package
        .project_selected_type_declarations(
            &identities,
            &[],
            &long_rows,
            &mut quire_contract_model::NativeJoinBudget::new(view.work() + 100, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(tight.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    assert_eq!(tight.at, None);
    assert_eq!(tight.form, None);
    assert_eq!(
        tight.denied_node(&package, &[], &long_rows),
        Some(&converted.node_id)
    );
    assert_eq!(
        tight.denied_locus.unwrap().source,
        quire_contract_model::NativeJoinLocusSource::SourceUnitRow(1)
    );
    assert_eq!(tight.denied_locus.unwrap().form, "selected_source_unit_row");

    assert_eq!(
        tight.denied.unwrap().kind,
        quire_contract_model::NativeJoinChargeKind::Work
    );
    let mut long_declaration = converted_row;
    long_declaration.qualified_declaration = vec!["x".repeat(4096).into()];
    let refusal = package
        .project_selected_type_declarations(
            &identities,
            &[],
            &[rows[0].clone(), long_declaration],
            &mut quire_contract_model::NativeJoinBudget::new(view.work() + 100, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(refusal.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    assert_eq!(refusal.at, None);
    assert_eq!(refusal.form, None);
    assert_eq!(
        refusal.denied.unwrap().kind,
        quire_contract_model::NativeJoinChargeKind::Work
    );
}
fn row<'a>(
    node: &'a CheckedSemanticNodeV2,
    identity: NominalIdentity<'a>,
) -> NominalDeclarationRow<'a> {
    NominalDeclarationRow {
        node: &node.node_id,
        owner: node.owner.as_ref().unwrap(),
        declaration: node.declaration.as_ref().unwrap(),
        identity,
    }
}
fn member_rows<'a>(
    package: &'a CheckedPackageV2,
    declaration: &'a CheckedSemanticNodeV2,
    keys: &'a [NodeKey; 2],
) -> Vec<EnumMemberRow<'a>> {
    ["ACTIVE", "DONE"]
        .into_iter()
        .zip(keys)
        .map(|(case, identity)| {
            let member = package
                .graph()
                .nodes
                .iter()
                .find(|node| {
                    node.body.literal_read().is_some_and(|read| {
                        read.type_id == &declaration.node_id
                            && read.value == quire_contract_model::CheckedLiteralValue::Enum(case)
                    })
                })
                .unwrap();
            EnumMemberRow {
                node: &member.node_id,
                declaration: &declaration.node_id,
                case,
                identity,
            }
        })
        .collect()
}

#[test]
#[trace(
    "FR-354-AC-5",
    "FR-354-AC-7",
    "TC-470",
    "FR-360-AC-1",
    "FR-360-AC-2",
    "FR-360-AC-3",
    "FR-360-AC-4"
)]
fn unit_read_borrows_root_target_exact_scale_offset_and_stage_identity() {
    let package = fixture(false);
    let root = node(&package, "unit", "Metre");
    let converted = node(&package, "unit", "Centimetre");
    let identities = [
        UnitId::Declared(key("Metre")),
        UnitId::Declared(key("Centimetre")),
    ];
    let rows = [
        row(root, NominalIdentity::Unit(&identities[0])),
        row(converted, NominalIdentity::Unit(&identities[1])),
    ];
    let requested = [root.node_id.clone(), converted.node_id.clone()];
    let mut measured_budget = quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX);
    let view = package
        .read_unit_definition(&requested, &rows, &mut measured_budget)
        .unwrap();
    let measured = measured_budget.last_success_delta().unwrap();
    let mut exact = quire_contract_model::NativeJoinBudget::new(measured.work, measured.storage);
    assert_eq!(
        package.read_unit_definition(&requested, &rows, &mut exact),
        Ok(view.clone())
    );
    assert_eq!(exact.last_success_delta(), Some(measured));
    for work in 0..measured.work {
        let mut short = quire_contract_model::NativeJoinBudget::new(work, measured.storage);
        let denied = package
            .read_unit_definition(&requested, &rows, &mut short)
            .unwrap_err();
        assert_eq!(denied.code, NominalReadCode::LimitReached);
        assert_eq!(denied.work, short.consumed().work);
        assert_eq!(
            short.first_unpaid().unwrap().kind,
            quire_contract_model::NativeJoinChargeKind::Work
        );
        assert!(short.consumed().work <= work);
        assert_eq!(short.last_success_delta(), None);
    }
    for storage in 0..measured.storage {
        let mut short = quire_contract_model::NativeJoinBudget::new(measured.work, storage);
        let denied = package
            .read_unit_definition(&requested, &rows, &mut short)
            .unwrap_err();
        assert_eq!(denied.work, short.consumed().work);
        assert_eq!(
            short.first_unpaid().unwrap().kind,
            quire_contract_model::NativeJoinChargeKind::Storage
        );
        assert!(short.consumed().storage <= storage);
        assert_eq!(short.last_success_delta(), None);
        assert!(
            requested.iter().any(|id| id == denied.node)
                || package
                    .graph()
                    .nodes
                    .iter()
                    .any(|node| &node.node_id == denied.node)
        );
    }
    let mut shared =
        quire_contract_model::NativeJoinBudget::new(measured.work * 2, measured.storage * 2);
    assert_eq!(
        package.read_unit_definition(&requested, &rows, &mut shared),
        Ok(view.clone())
    );
    assert_eq!(
        package.read_unit_definition(&requested, &rows, &mut shared),
        Ok(view.clone())
    );
    assert_eq!(shared.consumed().work, measured.work * 2);
    assert_eq!(shared.consumed().storage, measured.storage * 2);
    assert_eq!(view.units.len(), 2);
    assert_eq!(view.dimensions.len(), 1);
    assert!(std::ptr::eq(
        view.dimensions[0].node,
        view.units[0].dimension
    ));
    assert!(view.dimensions[0].terms.is_empty());
    assert!(view.dimensions[0].base);
    assert!(view.units[0].dimension_base);
    assert!(view.units[0].target.is_none());
    assert!(std::ptr::eq(view.units[1].target.unwrap(), root));
    assert!(std::ptr::eq(view.units[1].node, converted));
    assert_eq!(view.units[1].identity, &identities[1]);
    assert_eq!(view.units[1].dimension, view.units[0].dimension);
    assert_eq!(view.units[1].scale.numerator.as_ref(), "1");
    assert_eq!(view.units[1].scale.denominator.as_ref(), "100");
    assert_eq!(view.units[1].offset.numerator.as_ref(), "3");
    assert_eq!(view.units[1].offset.denominator.as_ref(), "2");
    assert_eq!(
        package
            .read_unit_definition(
                &requested,
                &rows,
                &mut quire_contract_model::NativeJoinBudget::new(view.work, u64::MAX)
            )
            .unwrap(),
        view
    );
    let error = package
        .read_unit_definition(
            &requested,
            &rows,
            &mut quire_contract_model::NativeJoinBudget::new(view.work - 1, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(error.code, NominalReadCode::LimitReached);
    assert!(error.work < view.work);
    assert_eq!(error.limit, view.work - 1);
    assert_eq!(
        package
            .clone()
            .read_unit_definition(
                &requested,
                &rows,
                &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX)
            )
            .unwrap(),
        view
    );
}

#[test]
#[trace("FR-354-AC-5", "FR-354-AC-7")]
fn unit_read_refuses_missing_foreign_and_competing_selected_rows() {
    let package = fixture(false);
    let root = node(&package, "unit", "Metre");
    let converted = node(&package, "unit", "Centimetre");
    let identities = [
        UnitId::Declared(key("Metre")),
        UnitId::Declared(key("Centimetre")),
    ];
    let root_row = row(root, NominalIdentity::Unit(&identities[0]));
    let converted_row = row(converted, NominalIdentity::Unit(&identities[1]));
    let requested = [root.node_id.clone(), converted.node_id.clone()];
    assert_eq!(
        package
            .read_unit_definition(
                &requested,
                &[],
                &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX)
            )
            .unwrap_err()
            .code,
        NominalReadCode::MissingWitness
    );
    assert_eq!(
        package
            .read_unit_definition(
                &requested[1..],
                std::slice::from_ref(&converted_row),
                &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX)
            )
            .unwrap_err()
            .code,
        NominalReadCode::MissingWitness
    );
    let foreign = CheckedNodeOwner::Source {
        authority: "foreign".into(),
        identity: "other".into(),
    };
    let wrong = NominalDeclarationRow {
        owner: &foreign,
        ..root_row.clone()
    };
    assert_eq!(
        package
            .read_unit_definition(
                &requested,
                &[wrong, converted_row.clone()],
                &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX)
            )
            .unwrap_err()
            .code,
        NominalReadCode::ForeignWitness
    );
    assert_eq!(
        package
            .read_unit_definition(
                &requested,
                &[root_row.clone(), root_row.clone()],
                &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX)
            )
            .unwrap_err()
            .code,
        NominalReadCode::ConflictingWitness
    );
    let competing = NominalDeclarationRow {
        identity: NominalIdentity::Unit(&identities[0]),
        ..converted_row
    };
    assert_eq!(
        package
            .read_unit_definition(
                &requested,
                &[root_row, competing],
                &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX)
            )
            .unwrap_err()
            .code,
        NominalReadCode::ConflictingWitness
    );
}

#[test]
#[trace("FR-354-AC-6", "FR-354-AC-7")]
fn enum_read_retains_authored_order_and_members_even_when_rows_are_reversed() {
    for ordered in [false, true] {
        let package = fixture(ordered);
        let declaration = node(&package, "enum", "Status");
        let declaration_key = key("Status");
        let selected = row(declaration, NominalIdentity::Enum(&declaration_key));
        let keys = [key("ACTIVE"), key("DONE")];
        let mut rows = member_rows(&package, declaration, &keys);
        rows.reverse();
        let view = package.read_enum_index(&selected, &rows, 10_000).unwrap();
        assert_eq!(view.ordered, ordered);
        assert!(std::ptr::eq(view.node, declaration));
        assert_eq!(
            view.members
                .iter()
                .map(|m| (m.case, m.position, m.identity))
                .collect::<Vec<_>>(),
            [("ACTIVE", 0, &keys[0]), ("DONE", 1, &keys[1])]
        );
        assert_eq!(
            package
                .read_enum_index(&selected, &rows, view.work)
                .unwrap(),
            view
        );
        let error = package
            .read_enum_index(&selected, &rows, view.work - 1)
            .unwrap_err();
        assert_eq!(error.code, NominalReadCode::LimitReached);
        assert_eq!(error.work, view.work);
        assert_eq!(
            package
                .clone()
                .read_enum_index(&selected, &rows, 10_000)
                .unwrap(),
            view
        );
    }
}

#[test]
#[trace("FR-354-AC-6")]
fn enum_read_refuses_missing_duplicate_and_foreign_member_joins() {
    let package = fixture(false);
    let declaration = node(&package, "enum", "Status");
    let declaration_key = key("Status");
    let selected = row(declaration, NominalIdentity::Enum(&declaration_key));
    let keys = [key("ACTIVE"), key("DONE")];
    let rows = member_rows(&package, declaration, &keys);
    assert_eq!(
        package
            .read_enum_index(&selected, &rows[..1], 10_000)
            .unwrap_err()
            .code,
        NominalReadCode::MissingWitness
    );
    assert_eq!(
        package
            .read_enum_index(&selected, &[rows[0].clone(), rows[0].clone()], 10_000)
            .unwrap_err()
            .code,
        NominalReadCode::ConflictingWitness
    );
    let mut wrong = rows.clone();
    wrong[0].case = "DONE";
    assert_eq!(
        package
            .read_enum_index(&selected, &wrong, 10_000)
            .unwrap_err()
            .code,
        NominalReadCode::ForeignWitness
    );
    let foreign = &node(&package, "unit", "Metre").node_id;
    wrong = rows.clone();
    wrong[0].declaration = foreign;
    assert_eq!(
        package
            .read_enum_index(&selected, &wrong, 10_000)
            .unwrap_err()
            .code,
        NominalReadCode::ForeignWitness
    );
    wrong = rows.clone();
    wrong[1].identity = &keys[0];
    assert_eq!(
        package
            .read_enum_index(&selected, &wrong, 10_000)
            .unwrap_err()
            .code,
        NominalReadCode::ConflictingWitness
    );
}

#[test]
#[trace("FR-354-AC-5")]
fn dangling_unit_targets_cannot_enter_an_admitted_view() {
    let mut members = nominal_fixture_members();
    members[2].0["target_unit_node_id"] = node_id("qualified:Example/Missing");
    let wire: Value = nominal_package(&members);
    assert!(matches!(
        CheckedPackageV2::read(
            &canonical(&wire),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&wire)
        ),
        CheckedPackageV2ReadResult::Refused(_)
    ));
}

#[test]
#[trace("FR-354-AC-5", "FR-354-AC-6")]
fn names_do_not_substitute_for_selected_node_or_member_identity() {
    let package = fixture(false);
    let declaration = node(&package, "enum", "Status");
    let other = node(&package, "enum", "OtherStatus");
    let declaration_key = key("Status");
    let selected = row(declaration, NominalIdentity::Enum(&declaration_key));
    let keys = [key("ACTIVE"), key("DONE")];
    let mut foreign = member_rows(&package, other, &keys);
    for member in &mut foreign {
        member.declaration = &declaration.node_id;
    }
    // The row has the selected name and case, but the canonical member node
    // still belongs to the other admitted enum.
    assert_eq!(
        package
            .read_enum_index(&selected, &foreign, 10_000)
            .unwrap_err()
            .code,
        NominalReadCode::ForeignWitness
    );
    let root = node(&package, "unit", "Metre");
    let unit = UnitId::Declared(key("Metre"));
    let mut missing = root.node_id.clone();
    missing.package = "foreign-package".into();
    let wrong = NominalDeclarationRow {
        node: &missing,
        ..row(root, NominalIdentity::Unit(&unit))
    };
    assert_eq!(
        package
            .read_unit_definition(
                std::slice::from_ref(&missing),
                &[wrong],
                &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX)
            )
            .unwrap_err()
            .code,
        NominalReadCode::MissingNode
    );
}

#[test]
#[trace("FR-354-AC-5", "FR-354-AC-7")]
fn malformed_rows_cannot_walk_empty_segments_or_copy_a_locus_after_exhaustion() {
    let package = fixture(false);
    let root = node(&package, "unit", "Metre");
    let identity = UnitId::Declared(key("Metre"));
    let requested = [root.node_id.clone()];
    let malformed = quire_contract_model::CheckedDeclaration {
        qualified_name: vec!["".into(); 1_000],
    };
    let rows = [NominalDeclarationRow {
        declaration: &malformed,
        ..row(root, NominalIdentity::Unit(&identity))
    }];
    let error = package
        .read_unit_definition(
            &requested,
            &rows,
            &mut quire_contract_model::NativeJoinBudget::new(500, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(error.code, NominalReadCode::LimitReached);
    assert!(error.work <= 500);
    assert!(std::ptr::eq(error.node, &root.node_id));
    let error = package
        .read_unit_definition(
            &requested,
            &rows,
            &mut quire_contract_model::NativeJoinBudget::new(0, u64::MAX),
        )
        .unwrap_err();
    assert_eq!(error.code, NominalReadCode::LimitReached);
    assert!(std::ptr::eq(error.node, &root.node_id));
}

#[test]
#[trace("FR-354-AC-5", "FR-354-AC-7")]
fn derived_dimension_dependencies_are_returned_as_borrowed_canonical_terms() {
    let package = fixture(false);
    let square = node(&package, "unit", "SquareMetre");
    let area = node(&package, "dimension", "Area");
    let length = node(&package, "dimension", "Length");
    let identity = UnitId::Declared(key("SquareMetre"));
    let requested = [square.node_id.clone()];
    let rows = [row(square, NominalIdentity::Unit(&identity))];
    let view = package
        .read_unit_definition(
            &requested,
            &rows,
            &mut quire_contract_model::NativeJoinBudget::new(10_000, u64::MAX),
        )
        .unwrap();
    assert_eq!(view.dimensions.len(), 2);
    assert!(std::ptr::eq(view.dimensions[0].node, area));
    assert!(std::ptr::eq(view.dimensions[1].node, length));
    assert!(!view.dimensions[0].base);
    assert!(view.dimensions[1].base);
    assert_eq!(
        view.dimensions[0].terms[0].dimension_node_id,
        length.node_id
    );
    assert_eq!(view.dimensions[0].terms[0].exponent.as_ref(), "2");
    assert!(view.dimensions[1].terms.is_empty());
}

#[test]
#[trace("TC-470", "FR-360-AC-2", "FR-360-AC-3")]
fn dimension_sibling_term_denial_restores_the_paid_parent_coordinate() {
    use quire_contract_model::{NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocusSource};
    let mut members = nominal_fixture_members()
        .into_iter()
        .skip(2)
        .collect::<Vec<_>>();
    let mut time = members[1].0.clone();
    time["qualified_declaration"] = json!(["Example", "Time"]);
    members.push((time, "qualified:Example/Time".into()));
    let mut parent = members[1].0.clone();
    parent["qualified_declaration"] = json!(["Example", "CompositeDimension"]);
    parent["terms"] = json!([
        {"dimension_node_id":node_id("qualified:Example/Length"),"exponent":"1"},
        {"dimension_node_id":node_id("qualified:Example/Time"),"exponent":"123456789012345"}
    ]);
    members.push((parent, "qualified:Example/CompositeDimension".into()));
    let mut composite = members[0].0.clone();
    composite["qualified_declaration"] = json!(["Example", "CompositeUnit"]);
    composite["dimension_node_id"] = node_id("qualified:Example/CompositeDimension");
    members.push((composite, "qualified:Example/CompositeUnit".into()));
    let wire = nominal_package(&members);
    let package = match CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("multi-term source dimension must admit: {other:?}"),
    };
    let rows: Vec<_> = ["Metre", "CompositeUnit"]
        .into_iter()
        .map(|name| {
            let selected = node(&package, "unit", name);
            let CheckedNodeOwner::Source {
                authority,
                identity,
            } = selected.owner.as_ref().unwrap()
            else {
                panic!("source unit")
            };
            SelectedSourceUnitRow::new(
                selected.node_id.clone(),
                authority.clone(),
                identity.clone(),
                selected
                    .declaration
                    .as_ref()
                    .unwrap()
                    .qualified_name
                    .clone(),
                UnitId::Declared(key(name)),
            )
        })
        .collect();
    let identities = SelectedExactIdentities::empty();
    let parent = &node(&package, "dimension", "CompositeDimension").node_id;
    let child = &node(&package, "dimension", "Length").node_id;
    let mut ample = NativeJoinBudget::new(10_000, u64::MAX);
    let view = package
        .project_selected_type_declarations(&identities, &[], &rows, &mut ample)
        .unwrap();
    let dimension = view
        .dimensions()
        .iter()
        .find(|dimension| &dimension.structural_node == parent)
        .unwrap();
    assert_eq!(dimension.terms.len(), 2);
    assert_eq!(dimension.terms[1].exponent.as_ref(), "123456789012345");
    let mut matched_second_term = 0;
    for work in 0..ample.consumed().work {
        let mut short = NativeJoinBudget::new(work, u64::MAX);
        let error = package
            .project_selected_type_declarations(&identities, &[], &rows, &mut short)
            .unwrap_err();
        let Some(locus) = error.denied_locus else {
            panic!("every runtime refusal has a fixed coordinate")
        };
        let resolved = error.denied_node(&package, &[], &rows);
        // This sixteen-unit charge is the long second exponent, following
        // the paid lookup and pending insertion of the first child.
        if locus.form == "selected_unit_definition"
            && error.denied.unwrap().kind == NativeJoinChargeKind::Work
            && error.denied.unwrap().amount == 16
            && (resolved == Some(parent) || resolved == Some(child))
        {
            assert_eq!(
                resolved,
                Some(parent),
                "a sibling term belongs to its parent, not the prior child"
            );
            assert!(matches!(locus.source, NativeJoinLocusSource::GraphNode(_)));
            assert_eq!(short.first_unpaid_locus(), Some(locus));
            assert_eq!(short.last_success_delta(), None);
            matched_second_term += 1;
        }
    }
    assert!(
        matched_second_term >= 16,
        "every work boundary inside the second exponent charge must retain its parent"
    );
}
