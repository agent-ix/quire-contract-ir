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

fn assert_manifest_refusal(id: &str, expected_locus: &str, node_index: usize) {
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
    assert_eq!(
        refusal.locus,
        Some(
            serde_json::from_value(
                mutated["semantic_graph"]["nodes"][node_index]["node_id"].clone()
            )
            .expect("published protocol node identity")
        ),
        "{id}: structural node locus"
    );
}

#[test]
#[trace("TC-946", "FR-038-AC-435")]
fn qspec_protocol_mutation_v13_refuses_wrong_result_type() {
    assert_manifest_refusal("V13", "/semantic_graph/nodes/64", 64);
}

#[test]
#[trace("TC-946", "FR-038-AC-436")]
fn qspec_protocol_mutation_v14_refuses_state_clause_protocol_member() {
    assert_manifest_refusal("V14", "/semantic_graph/nodes/54", 54);
}

fn apply_layout_edit(package: &mut Value, edit: &Value) {
    let path = edit["path"].as_str().expect("published edit path");
    let (parent, member) = path.rsplit_once('/').expect("absolute edit path");
    let target = package
        .pointer_mut(parent)
        .unwrap_or_else(|| panic!("published edit parent exists: {parent}"));
    let operation = edit["op"].as_str().expect("published edit operation");
    match (operation, target) {
        ("add", Value::Array(values)) => {
            let index = if member == "-" {
                values.len()
            } else {
                member.parse().expect("array index")
            };
            values.insert(index, edit["value"].clone());
        }
        ("add", Value::Object(fields)) | ("replace", Value::Object(fields)) => {
            if operation == "replace" {
                assert!(
                    fields.contains_key(member),
                    "{path}: replaced member exists"
                );
            }
            fields.insert(member.to_owned(), edit["value"].clone());
        }
        ("replace", Value::Array(values)) => {
            let index: usize = member.parse().expect("array index");
            values[index] = edit["value"].clone();
        }
        ("remove", Value::Array(values)) => {
            let index: usize = member.parse().expect("array index");
            values.remove(index);
        }
        ("remove", Value::Object(fields)) => {
            assert!(
                fields.remove(member).is_some(),
                "{path}: removed member exists"
            );
        }
        _ => panic!("{path}: unsupported published edit"),
    }
}

#[test]
#[trace("TC-048", "FR-038-AC-453")]
fn current_qspec_protocol_layout_rows_refuse_at_exact_node_pointer() {
    let fixtures = fixtures_dir();
    let layout = read_json(&fixtures.join("protocol-node-layout-rows.json"))
        .expect("current QSpec protocol layout rows");
    let manifest = read_json(&fixtures.join("protocol-node-mutation-manifest.json"))
        .expect("QSpec protocol mutation manifest");
    let base_name = manifest["base"]["fixture"].as_str().expect("base fixture");
    let base = read_json(&fixtures.join(base_name)).expect("QSpec protocol base");
    let document = fixtures.join(
        manifest["base"]["domain_document"]
            .as_str()
            .expect("selected domain document"),
    );
    let base_result = read(&base, &document);
    assert!(
        matches!(base_result, CheckedPackageV2ReadResult::Admitted(_)),
        "QSpec protocol base must admit: {base_result:?}"
    );
    let rows = layout["rows"].as_array().expect("published layout rows");
    assert!(!rows.is_empty(), "QSpec layout rows must be present");
    for row in rows {
        let id = row["id"].as_str().expect("row id");
        assert_eq!(row["base"], base_name, "{id}: selected base");
        assert_eq!(row["class"], "refuse", "{id}: refusal row");
        assert_eq!(row["expect"]["stage"], "schema", "{id}: schema stage");
        let mut edited = base.clone();
        for edit in row["edit"].as_array().expect("published edits") {
            apply_layout_edit(&mut edited, edit);
        }
        let result = read(&edited, &document);
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{id}: published layout row must refuse: {result:?}");
        };
        assert_eq!(
            serde_json::to_value(refusal.code).unwrap(),
            row["expect"]["code"],
            "{id}: code"
        );
        assert_eq!(
            serde_json::to_value(refusal.cause).unwrap(),
            row["expect"]["cause"],
            "{id}: cause"
        );
        assert_eq!(
            refusal.locus, None,
            "{id}: schema refusal has no structural locus"
        );
        assert_eq!(refusal.loci.len(), 1, "{id}: one primary pointer");
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .expect("primary path")
                .as_str(),
            row["expect"]["locus"].as_str().expect("published locus"),
            "{id}: primary pointer"
        );
    }
}
