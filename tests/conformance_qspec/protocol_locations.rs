// SPDX-License-Identifier: AGPL-3.0-or-later
//! Instantiate QSpec's symbolic protocol-location vectors in an admitted graph.
//! Vector origin/path/term facts remain authoritative owner data, read in place.

use crate::fixture_support::{
    canonical, evidence_for, positive_operation_identities, refresh_identity,
};
use ix_trace_rs::trace;
use quire_contract_model::CheckedProtocolLocationReadError;
use quire_contract_model::{
    CheckedOccurrence, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

fn vectors() -> Value {
    let root = PathBuf::from(
        std::env::var_os("QUIRE_SPECIFICATION_DIR")
            .expect("QUIRE_SPECIFICATION_DIR must select the owning QSpec vector checkout"),
    );
    let path = root.join("proposals/checked-package-v2/checked-term-data-vectors.json");
    serde_json::from_slice(&fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
        .unwrap()
}
fn positive(id: &str) -> Value {
    let mut row = vectors()["positive"]["protocol_locations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .expect("published QSpec vector")
        .clone();
    row.as_object_mut().unwrap().remove("id");
    row
}
fn fixture(mut rows: Vec<Value>) -> (Value, usize) {
    let mut wire = positive_operation_identities();
    let nodes = wire["semantic_graph"]["nodes"].as_array().unwrap();
    let position = nodes
        .iter()
        .position(|node| node["node_tag"] == "expression" && node["body"]["term"] == "application")
        .expect("the admitted fixture has a local application");
    let application = nodes[position]["node_id"].clone();
    let occurrence = nodes[position]["occurrences"][0].clone();
    for row in &mut rows {
        row["application"] = application.clone();
        row["occurrence"] = occurrence.clone();
    }
    wire["checked_term_data"] =
        json!({"equality_decisions":[], "protocol_locations":rows, "dispatch_tables":[]});
    refresh_identity(&mut wire);
    (wire, position)
}
fn read(wire: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(wire),
    )
}
fn admitted(wire: &Value) -> Box<CheckedPackageV2> {
    match read(wire) {
        CheckedPackageV2ReadResult::Admitted(p) => p,
        other => panic!("protocol fixture must admit: {other:?}"),
    }
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn published_locations_preserve_distinct_semantic_terms_on_one_application() {
    let originals = [positive("PL-01-root"), positive("PL-02-child")];
    let (wire, position) = fixture(originals.to_vec());
    let package = admitted(&wire);
    let node = &package.graph().nodes[position];
    let occurrence = &node.occurrences[0];
    let first = package
        .protocol_location(&node.node_id, occurrence, 0, 100)
        .unwrap();
    let second = package
        .protocol_location(&node.node_id, occurrence, 1, 100)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&first.location).unwrap(),
        originals[0]["location"]
    );
    assert_eq!(
        serde_json::to_value(&second.location).unwrap(),
        originals[1]["location"]
    );
    assert_ne!(first.location, second.location);
    assert_eq!(first.application, second.application);
    assert_eq!(first.occurrence, second.occurrence);
    assert_eq!(second.term_ordinal, 1);
    assert!(std::ptr::eq(
        first,
        package
            .protocol_location(&node.node_id, occurrence, 0, 100)
            .unwrap()
    ));
    assert_eq!(
        package.protocol_location(&node.node_id, occurrence, 2, 100),
        Err(CheckedProtocolLocationReadError::Missing)
    );
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn published_named_type_origin_is_returned_without_source_region_inference() {
    let original = positive("PL-03-declaration");
    let (wire, position) = fixture(vec![original.clone()]);
    let package = admitted(&wire);
    let node = &package.graph().nodes[position];
    let row = package
        .protocol_location(&node.node_id, &node.occurrences[0], 0, 100)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&row.location).unwrap(),
        original["location"]
    );
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn published_duplicate_term_key_refuses_at_the_second_captured_row() {
    let all = vectors();
    let vector = all["reader_refusals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "PL-R1-duplicate-term")
        .unwrap();
    let (wire, _) = fixture(vector["value"].as_array().unwrap().clone());
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wire) else {
        panic!("duplicate term must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::ConflictingDefinition)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().to_string(),
        vector["locus"].as_str().unwrap()
    );
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn missing_selected_mapping_is_not_substituted_by_a_sibling_term() {
    let all = vectors();
    let vector = all["reader_refusals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "PL-R2-missing-term-mapping")
        .unwrap();
    let (wire, _) = fixture(vector["value"].as_array().unwrap().clone());
    let result = read(&wire);
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("QSpec missing-term vector requires its recorded refusal, got {result:?}");
    };
    assert_eq!(
        serde_json::to_value(refusal.code).unwrap(),
        vector["expected_code"]
    );
    assert_eq!(
        serde_json::to_value(refusal.cause).unwrap(),
        vector["expected_cause"]
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        vector["locus"].as_str().unwrap()
    );
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn foreign_application_missing_occurrence_and_non_application_each_refuse() {
    let (wire, position) = fixture(vec![positive("PL-01-root")]);
    let mut foreign = wire.clone();
    foreign["checked_term_data"]["protocol_locations"][0]["application"]["package"] =
        json!("foreign");
    assert!(matches!(
        read(&foreign),
        CheckedPackageV2ReadResult::Refused(_)
    ));
    let mut wrong_occurrence = wire.clone();
    wrong_occurrence["checked_term_data"]["protocol_locations"][0]["occurrence"]["ordinal"] =
        json!(999);
    assert!(matches!(
        read(&wrong_occurrence),
        CheckedPackageV2ReadResult::Refused(_)
    ));
    let mut non_application = wire.clone();
    let other = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .find(|(i, n)| *i != position && n["body"]["term"] != "application")
        .unwrap()
        .1;
    non_application["checked_term_data"]["protocol_locations"][0]["application"] =
        other["node_id"].clone();
    assert!(matches!(
        read(&non_application),
        CheckedPackageV2ReadResult::Refused(_)
    ));
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn unknown_origin_and_noncanonical_child_ordinals_never_admit() {
    let all = vectors();
    let vector = all["schema_invalid"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "PL-S1-unknown-origin")
        .unwrap();
    let (wire, _) = fixture(vec![vector["value"].clone()]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wire) else {
        panic!("unknown origin must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    for ordinal in [
        json!(9007199254740992_u64),
        json!("01"),
        json!("9007199254740991"),
        json!("-1"),
    ] {
        let (mut wire, _) = fixture(vec![positive("PL-02-child")]);
        wire["checked_term_data"]["protocol_locations"][0]["location"]["path"][0] = ordinal;
        assert!(matches!(
            read(&wire),
            CheckedPackageV2ReadResult::Refused(_)
        ));
    }
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn bounded_read_returns_complete_borrowed_path_only_at_the_exact_work_limit() {
    let (wire, position) = fixture(vec![positive("PL-01-root"), positive("PL-02-child")]);
    let package = admitted(&wire);
    let node = &package.graph().nodes[position];
    // Two inspected rows plus the selected row's two child indices.
    assert!(package
        .protocol_location(&node.node_id, &node.occurrences[0], 1, 4)
        .is_ok());
    assert_eq!(
        package.protocol_location(&node.node_id, &node.occurrences[0], 1, 3),
        Err(CheckedProtocolLocationReadError::WorkExhausted {
            limit: 3,
            consumed: 4,
            row: 1,
            child: Some(1)
        })
    );
    let absent = CheckedOccurrence {
        ordinal: 999,
        ..node.occurrences[0].clone()
    };
    assert_eq!(
        package.protocol_location(&node.node_id, &absent, 0, 100),
        Err(CheckedProtocolLocationReadError::Missing)
    );
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn absent_capture_and_present_empty_capture_have_distinct_typed_reads() {
    let (mut wire, position) = fixture(Vec::new());
    let empty = admitted(&wire);
    let node = &empty.graph().nodes[position];
    assert_eq!(
        empty.protocol_location(&node.node_id, &node.occurrences[0], 0, 100),
        Err(CheckedProtocolLocationReadError::Missing)
    );
    wire.as_object_mut().unwrap().remove("checked_term_data");
    let absent = admitted(&wire);
    let node = &absent.graph().nodes[position];
    assert_eq!(
        absent.protocol_location(&node.node_id, &node.occurrences[0], 0, 100),
        Err(CheckedProtocolLocationReadError::Unavailable)
    );
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn valid_large_child_indices_remain_exact_canonical_owner_values() {
    let (mut wire, position) = fixture(vec![positive("PL-02-child")]);
    wire["checked_term_data"]["protocol_locations"][0]["location"]["path"] = json!([
        9007199254740991_u64,
        "9007199254740992",
        "18446744073709551616"
    ]);
    let package = admitted(&wire);
    let node = &package.graph().nodes[position];
    let row = package
        .protocol_location(&node.node_id, &node.occurrences[0], 1, 100)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&row.location).unwrap()["path"],
        json!([
            9007199254740991_u64,
            "9007199254740992",
            "18446744073709551616"
        ])
    );
}

#[test]
#[trace("TC-058", "FR-354-AC-9")]
fn reversed_distinct_term_keys_refuse_at_the_first_out_of_order_row() {
    let (wire, _) = fixture(vec![positive("PL-02-child"), positive("PL-01-root")]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wire) else {
        panic!("reversed distinct keys must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/checked_term_data/protocol_locations/1"
    );
}
