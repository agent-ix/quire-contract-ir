// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! By-reference reader reproduction for the QSpec STD-609 protocol mutations.

use super::{fixtures_dir, read_json, read_package_with_document};
use ix_trace_rs::trace;
use quire_contract_model::CheckedPackageV2ReadResult;
use serde_json::Value;

fn read(wire: &Value, document: &std::path::Path) -> CheckedPackageV2ReadResult {
    read_package_with_document(wire, "STD-609 protocol mutation", document)
        .expect("QSpec protocol package read")
}

fn assert_manifest_refusal(id: &str, expected_locus: &str) {
    let fixtures = fixtures_dir();
    let manifest = read_json(&fixtures.join("protocol-node-mutation-manifest.json"))
        .expect("QSpec STD-609 manifest");
    let base_name = manifest["base"]["fixture"]
        .as_str()
        .expect("manifest base fixture");
    assert_eq!(base_name, "protocol-node-base.json");
    let base = read_json(&fixtures.join(base_name)).expect("QSpec protocol base fixture");
    let document = fixtures.join(
        manifest["base"]["domain_document"]
            .as_str()
            .expect("manifest selected domain document"),
    );
    let base_result = read(&base, &document);
    assert!(
        matches!(&base_result, CheckedPackageV2ReadResult::Admitted(_)),
        "QSpec protocol base must admit before applying mutations: {base_result:?}"
    );

    let rows = manifest["rows"].as_array().expect("manifest rows");
    let selected = rows
        .iter()
        .filter(|row| row["id"] == id)
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1, "{id}: one QSpec manifest row");
    let row = selected[0];
    assert_eq!(row["expect"]["decision"], "refuse", "{id}");
    assert_eq!(row["expect"]["stage"], "reader", "{id}");
    assert_eq!(row["expect"]["code"], "ill_typed", "{id}");
    assert_eq!(row["expect"]["cause"], "operator-ineligible", "{id}");
    assert_eq!(row["expect"]["locus"], expected_locus, "{id}");

    let mut mutated = base.clone();
    let edits = row["edit"].as_array().expect("manifest edits");
    assert_eq!(edits.len(), 2, "{id}: exact two-edit mutation");
    for edit in edits {
        assert_eq!(edit["op"], "replace", "{id}: mutation operation");
        let path = edit["path"].as_str().expect("mutation pointer");
        let target = mutated
            .pointer_mut(path)
            .unwrap_or_else(|| panic!("{id}: mutation target exists: {path}"));
        *target = edit["value"].clone();
    }

    let refusal = match read(&mutated, &document) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        CheckedPackageV2ReadResult::Admitted(_) => {
            panic!("{id}: reader admitted; QSpec FR-464 requires refusal")
        }
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            panic!("{id}: reader incomplete ({incomplete:?}); QSpec FR-464 requires refusal")
        }
    };
    assert_eq!(
        serde_json::to_value(refusal.code).unwrap(),
        "ill_typed",
        "{id}"
    );
    assert_eq!(
        serde_json::to_value(refusal.cause).unwrap(),
        "operator-ineligible",
        "{id}"
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .expect("refusal locus")
            .as_str(),
        expected_locus,
        "{id}"
    );
}

#[test]
#[trace("TC-946", "FR-038-AC-435")]
fn qspec_protocol_mutation_v13_refuses_wrong_result_type() {
    assert_manifest_refusal("V13", "/semantic_graph/nodes/64");
}

#[test]
#[trace("TC-946", "FR-038-AC-436")]
fn qspec_protocol_mutation_v14_refuses_state_clause_protocol_member() {
    assert_manifest_refusal("V14", "/semantic_graph/nodes/54");
}
