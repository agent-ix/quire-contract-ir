// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public reader checks for the first protocol validation passes.

use crate::support::checked_package::{
    canonical, evidence_for, protocol_named_node, rebuild_source_map, refresh_identity, refusal,
    refusal_at, rekey_application_node, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn clause_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol" && node["semantic_form"] == "protocol_clause"
        })
        .expect("protocol clause")
}

fn append_duplicate_clause(package: &mut Value, body: Value) -> (usize, String) {
    let original = clause_position(package);
    let dependencies = package["semantic_graph"]["nodes"][original]["dependencies"]
        .as_array()
        .expect("dependencies")
        .iter()
        .map(|id| id["digest"].as_str().expect("digest").to_owned())
        .collect::<Vec<_>>();
    let duplicate = protocol_named_node(
        "protocol_clause",
        &["Example", "OrderFlow"],
        body,
        &dependencies,
    );
    let key = duplicate["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(duplicate);
    rebuild_source_map(package);
    refresh_identity(package);
    (position, key)
}

#[trace("TC-048")]
#[test]
fn protocol_clause_requires_the_selected_profile_law_in_the_public_reader() {
    let mut package = v2_all_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["body"]["operation"]["laws"] = json!([]);
    rekey_application_node(&mut package, position);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{position}/body/operation/laws"),
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn malformed_protocol_enum_names_the_offending_term() {
    let mut package = v2_all_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"]["members"][0]
        ["value"]["value"] = json!("unknown-activation");
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{position}/body/arguments/1/value/members/0/value"),
        )
    );

    let mut package = v2_all_families();
    let semantic_type = package["semantic_graph"]["nodes"][position]["semantic_type"].clone();
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][10]["value"]["members"] = json!([{
        "term": "literal", "type": semantic_type,
        "value_kind": "text", "value": "unknown-scheduling"
    }]);
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{position}/body/arguments/10/value/members/0"),
        )
    );
}

#[trace("TC-048")]
#[test]
fn protocol_operator_refusal_precedes_duplicate_declaration() {
    let mut package = v2_all_families();
    let original = clause_position(&package);
    let mut body = package["semantic_graph"]["nodes"][original]["body"].clone();
    body["operator"] = json!("call");
    let (position, key) = append_duplicate_clause(&mut package, body);
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{position}/body"),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            &key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn valid_protocol_operators_still_refuse_duplicate_declarations() {
    let mut package = v2_all_families();
    let original = clause_position(&package);
    let original_key = package["semantic_graph"]["nodes"][original]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let mut body = package["semantic_graph"]["nodes"][original]["body"].clone();
    let semantic_type = package["semantic_graph"]["nodes"][original]["semantic_type"].clone();
    body["arguments"][10]["value"]["members"] = json!([{
        "term": "literal", "type": semantic_type,
        "value_kind": "text", "value": "adversarial"
    }]);
    let (duplicate_position, duplicate_key) = append_duplicate_clause(&mut package, body);
    let (position, key) = if original_key < duplicate_key {
        (duplicate_position, duplicate_key)
    } else {
        (original, original_key)
    };
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &format!("/semantic_graph/nodes/{position}/declaration/qualified_name"),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            &key,
        )
    );
}
