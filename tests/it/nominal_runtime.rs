// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-354 nominal joins over reader-admitted canonical nominal fixtures.
//! Opaque stage keys below are explicit row fixtures, not QSL provenance evidence.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_fixture_members, nominal_package,
};
use ix_trace_rs::trace;
use quire_contract_model::nominal_runtime::{
    EnumMemberRow, NominalDeclarationRow, NominalIdentity, NominalReadCode,
};
use quire_contract_model::{
    CheckedNodeOwner, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedSemanticNodeV2,
};
use quire_exact::{NodeKey, PathSegment, UnitId};
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
#[trace("FR-354-AC-5", "FR-354-AC-7")]
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
    let view = package
        .read_unit_resolver(&requested, &rows, 10_000)
        .unwrap();
    assert_eq!(view.units.len(), 2);
    assert_eq!(view.dimensions.len(), 1);
    assert!(std::ptr::eq(
        view.dimensions[0].node,
        view.units[0].dimension
    ));
    assert!(view.dimensions[0].terms.is_empty());
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
            .read_unit_resolver(&requested, &rows, view.work)
            .unwrap(),
        view
    );
    let error = package
        .read_unit_resolver(&requested, &rows, view.work - 1)
        .unwrap_err();
    assert_eq!(error.code, NominalReadCode::LimitReached);
    assert_eq!(error.work, view.work);
    assert_eq!(error.limit, view.work - 1);
    assert_eq!(
        package
            .clone()
            .read_unit_resolver(&requested, &rows, 10_000)
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
            .read_unit_resolver(&requested, &[], 10_000)
            .unwrap_err()
            .code,
        NominalReadCode::MissingWitness
    );
    assert_eq!(
        package
            .read_unit_resolver(&requested[1..], &[converted_row.clone()], 10_000)
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
            .read_unit_resolver(&requested, &[wrong, converted_row.clone()], 10_000)
            .unwrap_err()
            .code,
        NominalReadCode::ForeignWitness
    );
    assert_eq!(
        package
            .read_unit_resolver(&requested, &[root_row.clone(), root_row.clone()], 10_000)
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
            .read_unit_resolver(&requested, &[root_row, competing], 10_000)
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
fn dangling_unit_targets_cannot_enter_an_admitted_runtime_view() {
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
            .read_unit_resolver(std::slice::from_ref(&missing), &[wrong], 10_000)
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
        .read_unit_resolver(&requested, &rows, 500)
        .unwrap_err();
    assert_eq!(error.code, NominalReadCode::LimitReached);
    assert_eq!(error.work, 501);
    assert!(std::ptr::eq(error.node, &root.node_id));
    let error = package
        .read_unit_resolver(&requested, &rows, 0)
        .unwrap_err();
    assert_eq!(error.code, NominalReadCode::LimitReached);
    assert!(std::ptr::eq(error.node, &root.node_id));
}
