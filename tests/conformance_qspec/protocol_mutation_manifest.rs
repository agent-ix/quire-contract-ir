// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! By-reference reader reproduction for the QSpec STD-609 protocol mutations.

use super::{
    fixtures_dir, positive_document, proposal_dir_of, read_json, read_package_with_document,
    read_positive, vector_selected_documents,
};
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

#[test]
#[trace("TC-048", "FR-038-AC-454")]
fn current_qspec_protocol_positive_packages_remain_admitted() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let documents = vector_selected_documents(&proposal).expect("QSpec document selections");
    let vectors =
        read_json(&fixtures.join("protocol-node-vectors.json")).expect("QSpec protocol vectors");
    let positives = vectors["positive"].as_array().expect("published positives");
    assert!(
        !positives.is_empty(),
        "QSpec protocol positives must be present"
    );
    for row in positives {
        let id = row["id"].as_str().expect("positive id");
        assert_eq!(row["expected"], "admitted", "{id}");
        let name = row["fixture"].as_str().expect("positive fixture");
        let document = proposal.join(positive_document(name, &documents));
        read_positive(&fixtures, name, &document).unwrap_or_else(|error| panic!("{id}: {error}"));
    }
}

fn protocol_enum_values<'a>(schema: &'a Value, operator: &str, field: &str) -> &'a [Value] {
    schema["$defs"]["ProtocolControlApplication"]["allOf"]
        .as_array()
        .expect("protocol operator branches")
        .iter()
        .find(|branch| {
            branch
                .pointer("/if/properties/operation/properties/member/properties/operator/const")
                .and_then(Value::as_str)
                == Some(operator)
        })
        .and_then(|branch| branch["then"].pointer(field))
        .and_then(|field| field["enum"].as_array())
        .unwrap_or_else(|| panic!("published {operator} enum at {field}"))
}

fn protocol_text_term(type_id: &Value, spelling: &str) -> Value {
    serde_json::json!({
        "term": "literal", "type": type_id, "value_kind": "text", "value": spelling
    })
}

#[test]
#[trace("TC-048", "FR-038-AC-454")]
fn current_qspec_protocol_enumerations_remain_admitted() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let schema = read_json(&proposal.join("schema.json")).expect("QSpec checked-package schema");
    let base = read_json(&fixtures.join("protocol-node-base.json")).expect("QSpec protocol base");
    let manifest = read_json(&fixtures.join("protocol-node-mutation-manifest.json"))
        .expect("QSpec protocol mutation manifest");
    let document = fixtures.join(
        manifest["base"]["domain_document"]
            .as_str()
            .expect("selected domain document"),
    );
    let text_type = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("base nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "text")
        .expect("text type")["node_id"]
        .clone();
    let fields = [
        ("channel", "/properties/arguments/prefixItems/4/properties/value/properties/value", "/semantic_graph/nodes/62/body/arguments/4/value/value", false),
        ("queue", "/properties/arguments/prefixItems/2/properties/value/properties/value", "/semantic_graph/nodes/63/body/arguments/2/value/value", false),
        ("attempt", "/properties/arguments/prefixItems/2/properties/value/properties/members/items/properties/value", "/semantic_graph/nodes/66/body/arguments/2/value/members", true),
        ("parallel", "/properties/arguments/prefixItems/0/properties/value/properties/members/items/properties/value", "/semantic_graph/nodes/70/body/arguments/0/value/members", true),
        ("parallel", "/properties/arguments/prefixItems/4/properties/value/properties/members/items/properties/value", "/semantic_graph/nodes/70/body/arguments/4/value/members", true),
        ("protocol", "/properties/arguments/prefixItems/10/properties/value/properties/members/items/properties/value", "/semantic_graph/nodes/82/body/arguments/10/value/members", true),
    ];
    for (operator, schema_field, package_field, list) in fields {
        for value in protocol_enum_values(&schema, operator, schema_field) {
            let spelling = value.as_str().expect("text enum spelling");
            let mut changed = base.clone();
            *changed
                .pointer_mut(package_field)
                .expect("published base field") = if list {
                Value::Array(vec![protocol_text_term(&text_type, spelling)])
            } else {
                value.clone()
            };
            if operator == "queue" && spelling == "loss" {
                let loss = protocol_enum_values(
                    &schema,
                    "queue",
                    "/properties/arguments/prefixItems/3/properties/value/properties/members/items/properties/value",
                )[0]
                    .as_str()
                    .expect("loss spelling");
                changed["semantic_graph"]["nodes"][63]["body"]["arguments"][3]["value"]
                    ["members"] = Value::Array(vec![protocol_text_term(&text_type, loss)]);
            }
            let result = read(&changed, &document);
            assert!(
                matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                "{operator} {spelling} must admit: {result:?}"
            );
        }
    }

    for value in protocol_enum_values(
        &schema,
        "queue",
        "/properties/arguments/prefixItems/3/properties/value/properties/members/items/properties/value",
    ) {
        let spelling = value.as_str().expect("loss spelling");
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][63]["body"]["arguments"][2]["value"]["value"] =
            Value::from("loss");
        changed["semantic_graph"]["nodes"][63]["body"]["arguments"][3]["value"]["members"] =
            Value::Array(vec![protocol_text_term(&text_type, spelling)]);
        let result = read(&changed, &document);
        assert!(
            matches!(&result, CheckedPackageV2ReadResult::Admitted(_)),
            "queue loss {spelling} must admit: {result:?}"
        );
    }
    let modes = schema["$defs"]["ProtocolTerminalMode"]["properties"]["value"]["enum"]
        .as_array()
        .expect("published terminal modes");
    for mode in modes {
        let spelling = mode.as_str().expect("terminal mode");
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][80]["body"]["arguments"][0]["value"]["value"] =
            mode.clone();
        if spelling == "when" {
            changed["semantic_graph"]["nodes"][80]["body"]["arguments"][1]["value"]["members"] =
                Value::Array(vec![protocol_text_term(&text_type, "condition")]);
        }
        let result = read(&changed, &document);
        assert!(
            matches!(&result, CheckedPackageV2ReadResult::Admitted(_)),
            "terminal {spelling} must admit: {result:?}"
        );
    }
    let join_modes = schema["$defs"]["ProtocolJoin"]["properties"]["members"]["prefixItems"][0]
        ["properties"]["value"]["properties"]["value"]["enum"]
        .as_array()
        .expect("published join modes");
    for spelling in ["all", "any"] {
        assert!(
            join_modes.contains(&Value::from(spelling)),
            "published {spelling} mode"
        );
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][70]["body"]["arguments"][2]["value"]["members"][0]
            ["value"]["value"] = Value::from(spelling);
        let result = read(&changed, &document);
        assert!(
            matches!(&result, CheckedPackageV2ReadResult::Admitted(_)),
            "parallel {spelling} must admit: {result:?}"
        );
    }
}

fn assert_appended_protocol_node_admits(
    base: &Value,
    node: Value,
    document: &std::path::Path,
    label: &str,
) {
    let mut package = base.clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("base nodes")
        .len();
    let occurrence = &node["occurrences"][0];
    let source = package["source_map"][0]["regions"][0]["source"].clone();
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(serde_json::json!({
            "node_id":node["node_id"], "role":occurrence["role"],
            "ordinal":occurrence["ordinal"],
            "regions":[{"source":source,"start":position,"end":position+1}]
        }));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .push(node);
    let result = read(&package, document);
    assert!(
        matches!(&result, CheckedPackageV2ReadResult::Admitted(_)),
        "published {label} layout must admit: {result:?}"
    );
}

#[test]
#[trace("TC-048", "FR-038-AC-454")]
fn current_qspec_appended_protocol_layouts_remain_admitted() {
    let fixtures = fixtures_dir();
    let base = read_json(&fixtures.join("protocol-node-base.json")).expect("QSpec protocol base");
    let layout = read_json(&fixtures.join("protocol-node-layout-rows.json"))
        .expect("QSpec protocol layout rows");
    let manifest = read_json(&fixtures.join("protocol-node-mutation-manifest.json"))
        .expect("QSpec protocol mutation manifest");
    let document = fixtures.join(
        manifest["base"]["domain_document"]
            .as_str()
            .expect("selected domain document"),
    );
    let fence = layout["rows"]
        .as_array()
        .expect("layout rows")
        .iter()
        .find(|row| row["id"] == "L12")
        .expect("published fence row")["edit"][0]["value"]
        .clone();
    let schema = read_json(&proposal_dir_of(&fixtures).join("schema.json"))
        .expect("QSpec checked-package schema");
    let orderings = protocol_enum_values(
        &schema,
        "fence",
        "/properties/arguments/prefixItems/0/properties/value/properties/value",
    );
    for value in orderings {
        let spelling = value.as_str().expect("fence ordering");
        let mut node = fence.clone();
        node["body"]["arguments"][0]["value"]["value"] = value.clone();
        assert_appended_protocol_node_admits(&base, node, &document, spelling);
    }
    let effect = &base["semantic_graph"]["nodes"][67]["body"]["arguments"];
    let binding =
        |name: &str, value: Value| serde_json::json!({"term":"binding","name":name,"value":value});
    let mut event = fence.clone();
    event["body"]["operation"]["member"]["operator"] = Value::from("event");
    event["body"]["arguments"] = Value::Array(vec![
        binding("role", effect[0]["value"].clone()),
        binding("compensation", effect[0]["value"].clone()),
        binding("binder", effect[0]["value"].clone()),
        binding("related", effect[1]["value"].clone()),
        binding("constraint", effect[2]["value"].clone()),
    ]);
    assert_appended_protocol_node_admits(&base, event, &document, "event");
    let mut awaited = fence;
    awaited["body"]["operation"]["member"]["operator"] = Value::from("await");
    awaited["body"]["arguments"] = Value::Array(
        ["after", "clock", "within", "event", "then", "timeout"]
            .into_iter()
            .map(|name| binding(name, effect[0]["value"].clone()))
            .collect(),
    );
    assert_appended_protocol_node_admits(&base, awaited, &document, "await");
}
