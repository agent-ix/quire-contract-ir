// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Production admission and public reads of the optional operation-effect carrier.

use crate::support::checked_package::{
    canonical, evidence_for, evidence_with_document, family_model_document, node_id,
    v2_all_families,
};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    encode_checked_package_v2, CheckedOperationEffectRow, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageSidecarsRef,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn empty_row(operation: &str) -> Value {
    json!({
        "document": "test/families",
        "declaring": node_id("fixture/1515"),
        "operation": operation,
        "effect": {"modifies": [], "creates": [], "deletes": [], "deletes_scoped": []},
        "provenance": {
            "origin": "model_document_operation_frame",
            "member": format!("ix://test/families/Account/{operation}")
        }
    })
}

fn read(wire: &Value) -> CheckedPackageV2ReadResult {
    read_at(wire, CheckedPackageReadLimits::bounded())
}

fn read_at(wire: &Value, limits: CheckedPackageReadLimits) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(&canonical(wire), limits, &evidence_for(wire))
}

#[trace("TC-977", "FR-038-AC-408")]
#[test]
fn tight_work_limit_precedes_duplicate_effect_inspection() {
    let mut wire = present();
    let entry = json!({"kind":"field","declaration":node_id("fixture/1515"),
        "name":"balance","scope":"every-object"});
    wire["operation_effects"][0]["effect"]["modifies"] = json!([entry.clone(), entry]);
    refusal(
        &wire,
        Code::MalformedWire,
        None,
        "/operation_effects/0/effect/modifies",
    );

    let mut low = 0;
    let mut high = CheckedPackageReadLimits::bounded().work;
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = middle;
        match read_at(&wire, limits) {
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle,
            CheckedPackageV2ReadResult::Refused(_) => high = middle,
            other => panic!("duplicate must never admit: {other:?}"),
        }
    }
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = high;
    assert!(matches!(
        read_at(&wire, limits),
        CheckedPackageV2ReadResult::Refused(_)
    ));
    limits.work = high - 1;
    let CheckedPackageV2ReadResult::Incomplete(incomplete) = read_at(&wire, limits) else {
        panic!("work one below duplicate inspection must be incomplete");
    };
    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
    assert_eq!(
        incomplete.path.as_ref().map(|path| path.as_str()),
        Some("/operation_effects/0/effect/modifies/1")
    );
}

fn present() -> Value {
    let mut wire = v2_all_families();
    wire["operation_effects"] = json!([empty_row("deposit"), empty_row("withdraw")]);
    wire
}

fn refusal(wire: &Value, code: Code, cause: Option<Cause>, path: &str) {
    let read = read(wire);
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("expected {code:?} at {path}, got {read:?}");
    };
    assert_eq!(refusal.code, code);
    assert_eq!(refusal.cause, cause);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|at| at.as_str()),
        Some(path)
    );
}

#[trace(
    "TC-977",
    "TC-058",
    "FR-322-AC-303",
    "FR-038-AC-406",
    "FR-019-AC-21",
    "FR-019-AC-24"
)]
#[test]
fn absent_and_intentionally_empty_effect_are_distinct() {
    let CheckedPackageV2ReadResult::Admitted(absent) = read(&v2_all_families()) else {
        panic!("baseline package must admit");
    };
    assert!(absent.operation_effects().is_none());
    assert!(absent.operation_effect_rows().is_none());

    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&present()) else {
        panic!("present empty effect carrier must admit");
    };
    let rows = admitted
        .operation_effects()
        .expect("present carrier")
        .collect::<Vec<_>>();
    assert_eq!(admitted.operation_effect_rows().map(<[_]>::len), Some(2));
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].document(), "test/families");
    assert_eq!(
        rows[0].declaring(),
        &serde_json::from_value(node_id("fixture/1515")).unwrap()
    );
    assert_eq!(rows[0].operation(), "deposit");
    assert!(rows[0].effect().modifies.is_empty());
    assert!(rows[0].effect().creates.is_empty());
    assert!(rows[0].effect().deletes.is_empty());
    assert!(rows[0].effect().deletes_scoped.is_empty());
    assert_eq!(
        rows[0].provenance().member.as_ref(),
        "ix://test/families/Account/deposit"
    );
    assert_eq!(rows[1].operation(), "withdraw");
    assert_eq!(admitted.clone().operation_effects().unwrap().count(), 2);
}

#[trace("TC-977", "FR-038-AC-407")]
#[test]
fn explicit_null_and_empty_carrier_refuse_at_member() {
    for value in [Value::Null, json!([]), json!(false)] {
        let mut wire = v2_all_families();
        wire["operation_effects"] = value;
        refusal(&wire, Code::MalformedWire, None, "/operation_effects");
    }
}

#[trace("TC-977", "FR-038-AC-407")]
#[test]
fn similarly_named_unknown_top_level_member_stays_unknown() {
    let mut wire = v2_all_families();
    wire["operation_effects_extra"] = json!([]);
    refusal(&wire, Code::UnknownMember, None, "/operation_effects_extra");
}

#[trace("TC-977", "FR-038-AC-407")]
#[test]
fn invented_effect_and_missing_coverage_refuse_without_a_view() {
    let mut invented = present();
    invented["operation_effects"][0]["effect"]["creates"] = json!([node_id("fixture/1515")]);
    refusal(
        &invented,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/operation_effects/0/effect/creates",
    );

    let mut missing = present();
    missing["operation_effects"].as_array_mut().unwrap().pop();
    refusal(
        &missing,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        "/operation_effects",
    );
}

#[trace("TC-977", "TC-058", "FR-038-AC-406", "FR-019-AC-21", "FR-019-AC-24")]
#[test]
fn admitted_effect_rows_round_trip_through_typed_writer() {
    let mut wire = present();
    wire["operation_effects"][0]["effect"]["modifies"] = json!([{
        "kind": "field", "declaration": node_id("fixture/1515"),
        "name": "balance", "scope": "every-object"
    }]);
    let mut document = family_model_document();
    document["types"][0]["operations"][0]["frame"] =
        json!({"modifies": ["ix://test/families/Account/balance"]});
    let evidence = evidence_with_document(&wire, "test/families", &document);
    let CheckedPackageV2ReadResult::Admitted(admitted) = CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("present carrier must admit before typed encoding");
    };
    let rows: &[CheckedOperationEffectRow] = admitted
        .operation_effect_rows()
        .expect("present admitted producer rows");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].document.as_ref(), "test/families");
    assert_eq!(rows[0].operation.as_ref(), "deposit");
    assert_eq!(rows[1].operation.as_ref(), "withdraw");
    assert_eq!(
        rows[0].provenance.member.as_ref(),
        "ix://test/families/Account/deposit"
    );
    assert_eq!(rows[0].effect.modifies.len(), 1);
    assert!(rows[0].effect.creates.is_empty());
    assert!(rows[0].effect.deletes.is_empty());
    assert!(rows[0].effect.deletes_scoped.is_empty());
    assert!(rows[1].effect.modifies.is_empty());
    assert!(rows[1].effect.creates.is_empty());
    assert!(rows[1].effect.deletes.is_empty());
    assert!(rows[1].effect.deletes_scoped.is_empty());
    let encoded = encode_checked_package_v2(
        wire["name"].as_str().expect("package name"),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            schedulability_claims: None,
            operation_effects: Some(rows),
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        Limits::new(1 << 24),
    )
    .expect("typed carrier encodes");
    let reread = CheckedPackageV2::read(&encoded, CheckedPackageReadLimits::bounded(), &evidence);
    let CheckedPackageV2ReadResult::Admitted(reread) = reread else {
        panic!("encoded typed carrier must admit: {reread:?}");
    };
    assert_eq!(reread.operation_effect_rows(), Some(rows));
    assert_eq!(reread.operation_effects().unwrap().count(), rows.len());
}
