// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public admitted type reads with finite work and deep or cyclic type graphs.

use crate::support::checked_package::{
    canonical, evidence_for, evidence_with_document, node_id, nominal_package, select_model,
    settle, structural_id,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult, ExactTypeProjectionErrorCode, NativeJoinBudget,
    NativeJoinChargeKind, SelectedExactDeclarationForm, SelectedExactDeclarationRow,
    SelectedExactFieldRow, SelectedExactIdentities, SelectedExactIdentity,
};
use quire_exact::{
    Integer, IntegerInterval, MemberId, MemberOwner, NodeKey, PathSegment, Presence, ValueType,
};
use serde_json::{json, Value};

fn node(name: &str, tag: &str, form: &str, ty: &str, dependencies: &[&str], body: Value) -> Value {
    json!({"node_id": node_id(name), "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag, "semantic_form": form, "semantic_type": node_id(ty),
        "dependencies": dependencies.iter().map(|name| node_id(name)).collect::<Vec<_>>(),
        "occurrences": [{"role": "type", "ordinal": 0}], "body": body})
}

fn package(nodes: Vec<Value>) -> Value {
    let mut wire = nominal_package(&[]);
    wire["semantic_graph"]["nodes"] = Value::Array(nodes);
    settle(&mut wire);
    wire
}

fn read(wire: &Value) -> CheckedPackageV2ReadResult {
    let bytes = canonical(wire);
    CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits {
            bytes: u64::try_from(bytes.len()).expect("fixture length fits the reader budget"),
            ..CheckedPackageReadLimits::bounded()
        },
        &evidence_for(wire),
    )
}

fn admitted(wire: &Value) -> Box<CheckedPackageV2> {
    match read(wire) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("independent type graph must admit: {other:?}"),
    }
}

#[trace("TC-468", "FR-359-AC-1", "FR-359-AC-2", "FR-359-AC-3", "FR-359-AC-5")]
#[test]
fn exact_projection_charges_complete_values_and_semantic_diagnostics() {
    let empty = json!({"term":"aggregate","members":[]});
    let wire = package(vec![
        node("integer", "scalar_type", "integer", "integer", &[], empty),
        node(
            "option",
            "composite_type",
            "option",
            "option",
            &["integer"],
            json!({"term":"aggregate","members":[{"term":"reference","target":node_id("integer")}] }),
        ),
    ]);
    let admitted = admitted(&wire);
    let cloned = admitted.clone();
    let identities = SelectedExactIdentities::empty();
    let integer: CheckedNodeId = serde_json::from_value(node_id("integer")).unwrap();
    let option: CheckedNodeId = serde_json::from_value(node_id("option")).unwrap();

    let mut ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let expected = admitted
        .project_exact_type(&integer, &identities, &mut ample)
        .expect("scalar projects");
    let required = ample.consumed();
    assert!(required.work > 0 && required.storage > 0);
    let mut exact = NativeJoinBudget::new(required.work, required.storage);
    assert_eq!(
        admitted.project_exact_type(&integer, &identities, &mut exact),
        Ok(expected.clone())
    );
    assert_eq!(exact.last_success_delta(), Some(required));
    for kind in [NativeJoinChargeKind::Work, NativeJoinChargeKind::Storage] {
        let mut short = match kind {
            NativeJoinChargeKind::Work => {
                NativeJoinBudget::new(required.work - 1, required.storage)
            }
            NativeJoinChargeKind::Storage => {
                NativeJoinBudget::new(required.work, required.storage - 1)
            }
        };
        let error = admitted
            .project_exact_type(&integer, &identities, &mut short)
            .unwrap_err();
        assert_eq!(error.denied.unwrap().kind, kind);
        assert_eq!(error.requested, None);
        assert_eq!(error.at, None);
        assert_eq!(error.form, None);
        assert_eq!(short.last_success_delta(), None);
        assert_eq!(short.first_unpaid().unwrap().kind, kind);
    }
    let mut stage_ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let staged = SelectedExactIdentities::from_stage_rows(&admitted, [], [], &mut stage_ample)
        .expect("empty selected stage admits");
    let stage_storage = stage_ample.consumed().storage;
    assert!(stage_storage > 0);
    let mut shared = NativeJoinBudget::new(u64::MAX, stage_storage + required.storage - 1);
    SelectedExactIdentities::from_stage_rows(&admitted, [], [], &mut shared)
        .expect("prior stage is affordable");
    let shared_error = admitted
        .project_exact_type(&integer, &staged, &mut shared)
        .unwrap_err();
    assert_eq!(
        shared_error.code,
        ExactTypeProjectionErrorCode::ExhaustedStorage
    );
    assert_eq!(
        shared.first_unpaid().unwrap().kind,
        NativeJoinChargeKind::Storage
    );
    assert_eq!(shared.last_success_delta(), None);

    let mut semantic_budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
    let semantic = admitted
        .project_exact_type(&option, &identities, &mut semantic_budget)
        .unwrap_err();
    assert_eq!(
        semantic.code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
    assert_eq!(semantic.at.as_deref(), Some(&option));
    let diagnostic_storage = semantic_budget.consumed().storage;
    let mut exact_diagnostic = NativeJoinBudget::new(u64::MAX, diagnostic_storage);
    assert_eq!(
        admitted.project_exact_type(&option, &identities, &mut exact_diagnostic),
        Err(semantic.clone())
    );
    assert_eq!(exact_diagnostic.last_success_delta(), None);
    for storage in [0, diagnostic_storage - 1] {
        let mut short = NativeJoinBudget::new(u64::MAX, storage);
        let error = cloned
            .project_exact_type(&option, &identities, &mut short)
            .unwrap_err();
        assert_eq!(error.code, ExactTypeProjectionErrorCode::ExhaustedStorage);
        assert_eq!(error.requested, None);
        assert_eq!(error.at, None);
        assert_eq!(error.form, None);
        assert_eq!(short.last_success_delta(), None);
        assert_eq!(
            short.first_unpaid().unwrap().kind,
            NativeJoinChargeKind::Storage
        );
    }
}

#[trace("FR-352-AC-7")]
#[test]
fn admitted_projection_repeats_scalar_and_unsupported_outcomes_on_a_small_stack() {
    std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {
        let empty = || json!({"term": "aggregate", "members": []});
        let mut nodes = vec![node("integer", "scalar_type", "integer", "integer", &[], empty()), node("text", "scalar_type", "text", "text", &[], empty())];
        nodes.push(node("digit", "bounded_domain", "integer_range", "integer", &["integer"], json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "min", "value": {"term": "literal", "type": node_id("integer"), "value_kind": "integer", "value": "0"}},
            {"term": "binding", "name": "max", "value": {"term": "literal", "type": node_id("integer"), "value_kind": "integer", "value": "9"}}
        ]})));
        let mut parameter = node("parameter", "value", "parameter", "digit", &[], json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value": {"term": "literal", "type": node_id("text"), "value_kind": "text", "value": "digit_parameter"}},
            {"term": "binding", "name": "level", "value": {"term": "literal", "type": node_id("integer"), "value_kind": "integer", "value": "0"}}
        ]}));
        parameter["occurrences"][0]["role"] = json!("expression");
        nodes.push(parameter);
        let mut target = "integer".to_owned();
        for index in 0..4096 {
            let name = format!("deep-{index}");
            nodes.push(node(&name, "composite_type", "option", &name, &[&target], json!({"term": "aggregate", "members": [{"term": "reference", "target": node_id(&target)}]})));
            target = name;
        }
        let mut cyclic = node("cyclic", "composite_type", "option", "cyclic", &["cyclic"], json!({"term": "aggregate", "members": [{"term": "reference", "target": node_id("cyclic") }]}));
        cyclic["recursion_group"] = json!("recursive-type");
        nodes.push(cyclic);
        let wire = package(nodes);
        let mut invalid = wire.clone();
        invalid["semantic_graph"]["nodes"][0]["semantic_type"] = node_id("missing-type");
        match read(&invalid) {
            CheckedPackageV2ReadResult::Refused(error) => {
                assert_eq!(error.code, CheckedPackageRefusalCode::InvalidPackage);
                assert_eq!(error.cause, Some(CheckedPackageRefusalCause::InvalidDerivedShape));
                assert_eq!(error.locus, Some(serde_json::from_value(node_id("integer")).unwrap()));
            }
            other => panic!("an invalid type fixture cannot grant admission: {other:?}"),
        }
        let admitted = admitted(&wire);
        drop(invalid);
        drop(wire);
        let clone = admitted.clone();
        let identities = SelectedExactIdentities::empty();
        for (name, work, expected) in [
            ("integer", 2, Some(ValueType::Integer)),
            ("parameter", 4, Some(ValueType::Int(IntegerInterval::new(Integer::from(0_i64), Integer::from(9_i64)).unwrap()))),
            (target.as_str(), 2, None), ("cyclic", 2, None),
        ] {
            let id: CheckedNodeId = serde_json::from_value(node_id(name)).unwrap();
            let result = admitted.project_exact_type(&id, &identities, &mut quire_contract_model::NativeJoinBudget::new(work, u64::MAX));
            assert_eq!(result, clone.project_exact_type(&id, &identities, &mut quire_contract_model::NativeJoinBudget::new(work, u64::MAX)));
            assert_eq!(result, admitted.project_exact_type(&id, &identities, &mut quire_contract_model::NativeJoinBudget::new(work, u64::MAX)));
            match expected {
                Some(expected) => {
                    let projection = result.expect("complete Exact type at its exact work limit");
                    assert_eq!(projection.value_type(), &expected);
                    assert_eq!(projection.work(), work);
                    assert_eq!(projection.node(), &id);
                    assert_eq!(projection.record_view(), None);
                }
                None => {
                    let error = result.expect_err("unsupported nested type has no partial view");
                    assert_eq!(error.code, ExactTypeProjectionErrorCode::UnsupportedExactType);
                    assert_eq!(error.at.as_deref(), Some(&id));
                    assert_eq!(error.form.as_deref(), Some("composite_type/option"));
                    assert_eq!(error.work, work);
                }
            }
            let error = admitted.project_exact_type(&id, &identities, &mut quire_contract_model::NativeJoinBudget::new(work-1, u64::MAX)).expect_err("one below required work must exhaust");
            assert_eq!(error, clone.project_exact_type(&id, &identities, &mut quire_contract_model::NativeJoinBudget::new(work-1, u64::MAX)).unwrap_err());
            assert_eq!(error.code, ExactTypeProjectionErrorCode::ExhaustedWork);
            assert_eq!(error.requested, None);
            assert_eq!(error.at, None);
            assert_eq!(error.work, if name == "parameter" { 2 } else { 1 });
            assert_eq!(error.form, None);
            assert_eq!(error.denied.unwrap().kind, quire_contract_model::NativeJoinChargeKind::Work);
        }
    }).expect("spawn small-stack reader").join().expect("bounded public reads must not grow the stack with type depth");
}

/// Nominal record handles preserve a recursive type without unfolding it.
/// Every read still visits, witnesses and charges its own nested target.
#[trace("FR-352-AC-7", "TC-468", "FR-359-AC-1")]
#[test]
fn admitted_selected_record_cycle_projects_complete_views_and_charges_inner_targets() {
    std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {
        const MODEL: &str = "limits/selected";
        const COUNT: usize = 128;
        let original = |index| format!("ix://limits/selected/Record{index}");
        let id = |index| structural_id(MODEL, vec![json!("model"), json!("record_value_type"), json!(original(index))], 0);
        let key = |index| NodeKey::new("selected-stage", vec![PathSegment::Text(format!("Record{index}"))], 0).unwrap();
        let member = |index| MemberId::new(MemberOwner::Node(key(index)), "next").unwrap();
        let mut declarations = Vec::new();
        let mut nodes = Vec::new();
        for index in 0..COUNT {
            let next = (index + 1) % COUNT;
            declarations.push(json!({
                "identity": original(index), "displayName": format!("Record{index}"),
                "kind": {"module": MODEL, "name": "record"}, "roles": [], "constraints": [],
                "extensions": [], "unknownPolicy": "reject", "supertypes": [], "operations": [],
                "fields": [{"identity": format!("{}/next", original(index)), "name": "next",
                    "typeRef": original(next), "presence": "required", "nullable": false,
                    "defaultKind": "none", "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}}]
            }));
            nodes.push(json!({"node_id": id(index), "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": "model", "semantic_form": "record_value_type", "semantic_type": id(index),
                "dependencies": [], "occurrences": [{"role": "type", "ordinal": 0}],
                "owner": {"kind": "model", "identity": MODEL, "node": original(index)},
                "body": {"term": "aggregate", "members": []}}));
        }
        let document = json!({"contractVersion": "2.0.0", "package": {"identity": MODEL, "version": "1.0.0"},
            "constructs": [{"kind": {"module": MODEL, "name": "record"}, "construct": {"meaning": "quire.meaning.model.record-value-type/v1"}}],
            "types": declarations});
        let mut wire = package(nodes);
        select_model(&mut wire, MODEL);
        settle(&mut wire);
        let evidence = evidence_with_document(&wire, MODEL, &document);
        let bytes = canonical(&wire);
        let admitted = match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits {bytes: u64::try_from(bytes.len()).unwrap(), ..CheckedPackageReadLimits::bounded()}, &evidence) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("selected recursive record declarations must admit: {other:?}"),
        };
        let declaration_rows = (0..COUNT).map(|index| SelectedExactDeclarationRow::new(
            serde_json::from_value(id(index)).unwrap(), MODEL, SelectedExactDeclarationForm::RecordValueType,
            original(index), SelectedExactIdentity::Record(key(index)),
        )).collect::<Vec<_>>();
        let field_rows = (0..COUNT).map(|index| SelectedExactFieldRow::new(
            serde_json::from_value(id(index)).unwrap(), MODEL, original(index), format!("{}/next", original(index)), member(index),
        )).collect::<Vec<_>>();
        let identities = SelectedExactIdentities::from_stage_rows(&admitted, declaration_rows.clone(), field_rows.clone(), &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)).expect("stage rows join the admitted selected model");
        drop(bytes);
        drop(evidence);
        drop(wire);
        drop(document);
        let clone = admitted.clone();
        let work = u64::try_from(COUNT).unwrap() + 4;
        for index in 0..COUNT {
            let requested: CheckedNodeId = serde_json::from_value(id(index)).unwrap();
            let next = (index + 1) % COUNT;
            let projected = admitted.project_exact_type(&requested, &identities, &mut quire_contract_model::NativeJoinBudget::new(work, u64::MAX)).expect("complete nominal recursive record view");
            assert_eq!(clone.project_exact_type(&requested, &identities, &mut quire_contract_model::NativeJoinBudget::new(work, u64::MAX)), Ok(projected.clone()));
            assert_eq!(admitted.project_exact_type(&requested, &identities, &mut quire_contract_model::NativeJoinBudget::new(work, u64::MAX)), Ok(projected.clone()));
            assert_eq!(projected.node(), &requested);
            assert_eq!(projected.type_node(), &requested);
            assert_eq!(projected.value_type(), &ValueType::Composite(key(index)));
            assert_eq!(projected.work(), work);
            let view = projected.record_view().expect("complete selected record fields");
            assert_eq!(view.record_key(), &key(index));
            let fields = view.fields();
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].ir_identity(), format!("{}/next", original(index)));
            assert_eq!(fields[0].name(), "next");
            assert_eq!(fields[0].member_id(), &member(index));
            assert_eq!(fields[0].presence(), Presence::Required);
            assert_eq!(fields[0].value_type(), &ValueType::Composite(key(next)));
            let exhausted = admitted.project_exact_type(&requested, &identities, &mut quire_contract_model::NativeJoinBudget::new(work - 1, u64::MAX)).expect_err("inner target inspection must be charged before the complete view");
            assert_eq!(clone.project_exact_type(&requested, &identities, &mut quire_contract_model::NativeJoinBudget::new(work - 1, u64::MAX)), Err(exhausted.clone()));
            assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);
            assert_eq!(exhausted.requested, None);
            assert_eq!(exhausted.at, None);
            assert_eq!(exhausted.form, None);
            assert_eq!(exhausted.denied.unwrap().kind, quire_contract_model::NativeJoinChargeKind::Work);
            assert_eq!(exhausted.work, work - 1);
        }
        let requested: CheckedNodeId = serde_json::from_value(id(0)).unwrap();
        let mut stage_ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let staged = SelectedExactIdentities::from_stage_rows(
            &admitted,
            declaration_rows.clone(),
            field_rows.clone(),
            &mut stage_ample,
        )
        .expect("selected stage joins");
        let stage_work = stage_ample.consumed().work;
        assert!(stage_work > 0);
        let mut exact_ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
        admitted
            .project_exact_type(&requested, &staged, &mut exact_ample)
            .expect("record projects after stage");
        let exact_work = exact_ample.consumed().work;
        let mut shared = NativeJoinBudget::new(stage_work + exact_work - 1, u64::MAX);
        SelectedExactIdentities::from_stage_rows(
            &admitted,
            declaration_rows.clone(),
            field_rows.clone(),
            &mut shared,
        )
        .expect("prior stage remains affordable");
        let denied = admitted
            .project_exact_type(&requested, &staged, &mut shared)
            .unwrap_err();
        assert_eq!(denied.code, ExactTypeProjectionErrorCode::ExhaustedWork);
        assert_eq!(shared.first_unpaid().unwrap().kind, NativeJoinChargeKind::Work);
        assert_eq!(shared.last_success_delta(), None);
        let target: CheckedNodeId = serde_json::from_value(id(1)).unwrap();
        let missing = SelectedExactIdentities::from_stage_rows(&admitted, declaration_rows.into_iter().filter(|row| row.structural_node != target), field_rows.into_iter().filter(|row| row.record_node != target), &mut quire_contract_model::NativeJoinBudget::new(u64::MAX, u64::MAX)).unwrap();
        let error = admitted.project_exact_type(&requested, &missing, &mut quire_contract_model::NativeJoinBudget::new(work, u64::MAX)).expect_err("a nested record cannot borrow another declaration's identity");
        assert_eq!(error.code, ExactTypeProjectionErrorCode::UnavailableIdentityWitness);
        assert_eq!(error.requested.as_deref(), Some(&requested));
        assert_eq!(error.at.as_deref(), Some(&target));
        assert_eq!(error.form.as_deref(), Some("model/record_value_type"));
    }).expect("spawn supported recursive projection consumer").join().expect("supported nominal targets must remain bounded on a small stack");
}
