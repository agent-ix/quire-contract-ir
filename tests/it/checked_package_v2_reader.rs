// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Strict V2 admission and structural identity controls. Published positive
//! and adverse corpus readbacks run in `conformance_qspec`.

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{Value, json};

fn local_package() -> Value {
    let mut package = v2_all_families();
    let node = package["semantic_graph"]["nodes"][0].clone();
    let id = node["node_id"].clone();
    package["semantic_graph"]["nodes"] = json!([node]);
    package["source_map"] = json!(
        package["source_map"]
            .as_array()
            .expect("source map")
            .iter()
            .filter(|entry| entry["node_id"] == id)
            .cloned()
            .collect::<Vec<_>>()
    );
    package
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn refuse(package: &Value) -> quire_contract_model::CheckedPackageRefusal {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!("expected V2 refusal, got {:?}", read(package));
    };
    refusal
}

#[trace("TC-048")]
#[test]
fn structural_node_identity_and_source_region_survive_reading() {
    let first = local_package();
    let first_id = first["semantic_graph"]["nodes"][0]["node_id"].clone();
    let mut changed = first.clone();
    changed["source_map"][0]["regions"][0]["start"] = json!(100);
    changed["source_map"][0]["regions"][0]["end"] = json!(101);
    let CheckedPackageV2ReadResult::Admitted(first_read) = read(&first) else {
        panic!("first package must admit: {:?}", read(&first));
    };
    let CheckedPackageV2ReadResult::Admitted(changed_read) = read(&changed) else {
        panic!("changed source region must admit: {:?}", read(&changed));
    };
    assert_eq!(first_read.name(), changed_read.name());
    assert_eq!(first_read.graph().nodes, changed_read.graph().nodes);
    assert_eq!(
        serde_json::to_value(&changed_read.graph().nodes[0].node_id).expect("node id"),
        first_id
    );
    assert_ne!(first_read.source_map(), changed_read.source_map());
    assert_eq!(changed_read.source_map()[0].regions[0].start, 100);
}

#[trace("TC-048")]
#[test]
fn source_map_must_bind_the_recorded_occurrence() {
    let mut package = local_package();
    package["source_map"][0]["ordinal"] = json!(1);
    let refusal = refuse(&package);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidSourceMap);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/source_map/0".to_owned())
    );
}

#[trace("TC-048")]
#[test]
fn unknown_contract_version_refuses_before_graph_read() {
    let mut package = local_package();
    package["contract_version"] = json!("quire.checked-package/v99");
    package["semantic_graph"]["nodes"][0]["node_tag"] = json!("future_tag");
    let refusal = refuse(&package);
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnknownContractVersion
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/contract_version".to_owned())
    );
}

#[trace("TC-048")]
#[test]
fn duplicate_and_unknown_members_refuse_at_their_pointers() {
    let package = local_package();
    let bytes = canonical(&package);
    let text = std::str::from_utf8(&bytes).expect("canonical UTF-8");
    let duplicate = format!("{{\"name\":\"other\",{}", &text[1..]);
    let outcome = CheckedPackageV2::read(
        duplicate.as_bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = outcome else {
        panic!("duplicate member must refuse: {outcome:?}");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::DuplicateMember);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/name".to_owned())
    );

    let mut unknown = package;
    unknown["extra"] = json!(true);
    let refusal = refuse(&unknown);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::UnknownMember);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/extra".to_owned())
    );
}
