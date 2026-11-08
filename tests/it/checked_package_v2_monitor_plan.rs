// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Default reader controls for the distinct closed monitor-plan form.

use crate::support::checked_package::{
    canonical, evidence_for, rebuild_complete_source_map, refresh_identity, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeKind, CheckedPackageReadLimits, CheckedPackageRefusalCode,
    CheckedPackageV2ReadResult, TemporalForm,
};
use serde_json::{json, Value};

fn read(wire: &Value) -> CheckedPackageV2ReadResult {
    crate::read_and_account(
        &canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(wire),
    )
}
#[trace("TC-048", "FR-038-AC-426", "FR-019-AC-23")]
#[test]
fn monitor_plan_absence_is_an_empty_exact_size_iterator() {
    let wire = v2_all_families();
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire) else {
        panic!("ordinary supporting package admits")
    };
    assert_eq!(package.monitor_plans().len(), 0);
    assert!(package.monitor_plans().next().is_none());
    assert_eq!(
        CheckedNodeKind::decode(
            quire_contract_model::CheckedNodeTag::Temporal,
            "monitor_plan"
        ),
        Some(CheckedNodeKind::Temporal(TemporalForm::MonitorPlan))
    );
}
#[trace("TC-048", "FR-038-AC-422")]
#[test]
fn monitor_plan_empty_body_refuses_at_first_required_binding() {
    let mut wire = v2_all_families();
    let boolean = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["semantic_form"] == "boolean")
        .unwrap()["node_id"]
        .clone();
    let root = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    let id = json!({"package":wire["name"],"path":["temporal","monitor_plan","empty"],"ordinal":0});
    wire["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "schema_version":"quire.checked-semantic-graph/v2","node_id":id,"node_tag":"temporal","semantic_form":"monitor_plan",
        "semantic_type":boolean,"body":{"term":"aggregate","members":[]},"dependencies":[boolean],"occurrences":[{"role":"generated","ordinal":0}]
    }));
    rebuild_complete_source_map(&mut wire);
    refresh_identity(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refused) = read(&wire) else {
        panic!("empty plan must refuse")
    };
    assert_eq!(
        refused.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(refused.cause, None);
    assert_eq!(
        refused.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{root}/body/members/0")
    );
    assert_eq!(
        refused.locus,
        Some(
            serde_json::from_value(wire["semantic_graph"]["nodes"][root]["node_id"].clone())
                .unwrap()
        )
    );
}

#[trace("TC-048", "FR-038-AC-425")]
#[test]
fn anonymous_entry_support_keeps_model_and_source_owner_requirements() {
    for source in [false, true] {
        let mut wire = v2_all_families();
        let node = wire["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .position(|n| n["node_tag"] == "function" && n["owner"]["kind"] == "source")
            .expect("existing declared function");
        if !source {
            wire["semantic_graph"]["nodes"][node]
                .as_object_mut()
                .unwrap()
                .remove("declaration");
            wire["semantic_graph"]["nodes"][node]["occurrences"][0]["role"] = json!("generated");
        }
        wire["semantic_graph"]["nodes"][node]
            .as_object_mut()
            .unwrap()
            .remove("owner");
        let CheckedPackageV2ReadResult::Refused(refused) = read(&wire) else {
            panic!("ordinary owner-bearing function cannot become an anonymous entry")
        };
        assert_eq!(refused.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(
            refused.loci[0].path.as_ref().unwrap().as_str(),
            format!("/semantic_graph/nodes/{node}")
        );
    }
}
