// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Production admission and public reads of the optional operation-effect carrier.

use crate::support::checked_package::{canonical, evidence_for, node_id, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
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
    CheckedPackageV2::read(
        &canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(wire),
    )
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

#[trace("TC-977", "FR-322-AC-303", "FR-038-AC-406", "FR-019-AC-21")]
#[test]
fn absent_and_intentionally_empty_effect_are_distinct() {
    let CheckedPackageV2ReadResult::Admitted(absent) = read(&v2_all_families()) else {
        panic!("baseline package must admit");
    };
    assert!(absent.operation_effects().is_none());

    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&present()) else {
        panic!("present empty effect carrier must admit");
    };
    let rows = admitted
        .operation_effects()
        .expect("present carrier")
        .collect::<Vec<_>>();
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
