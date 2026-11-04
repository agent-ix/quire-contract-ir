// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! By-reference reader conformance for QSpec protocol mutations and layouts.

use super::{
    fixture_support, fixtures_dir, positive_document, proposal_dir_of, read_json,
    read_package_with_document, read_positive, vector_selected_documents,
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

#[test]
#[trace(
    "TC-048",
    "FR-038-AC-455",
    "FR-038-AC-456",
    "FR-038-AC-457",
    "FR-038-AC-458",
    "FR-038-AC-459"
)]
fn current_qspec_protocol_reader_refusals_preserve_published_provenance() {
    let fixtures = fixtures_dir();
    let manifest = read_json(&fixtures.join("protocol-node-mutation-manifest.json"))
        .expect("QSpec protocol mutation manifest");
    let base_name = manifest["base"]["fixture"].as_str().expect("base fixture");
    let base = read_json(&fixtures.join(base_name)).expect("QSpec protocol base");
    let document = fixtures.join(
        manifest["base"]["domain_document"]
            .as_str()
            .expect("selected domain document"),
    );
    assert!(
        matches!(
            read(&base, &document),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "published protocol base must admit"
    );

    let rows = manifest["rows"]
        .as_array()
        .expect("published manifest rows");
    for id in ["V2", "V8", "V9", "V10", "V11", "RP6", "V16", "CP28", "C11"] {
        let selected = rows
            .iter()
            .filter(|row| row["id"] == id)
            .collect::<Vec<_>>();
        assert_eq!(selected.len(), 1, "{id}: one published row");
        let row = selected[0];
        assert_eq!(row["expect"]["decision"], "refuse", "{id}");
        let mut edited = base.clone();
        for edit in row["edit"].as_array().expect("published edits") {
            apply_layout_edit(&mut edited, edit);
        }
        let refusal = match read(&edited, &document) {
            CheckedPackageV2ReadResult::Refused(refusal) => refusal,
            other => panic!("{id}: expected published refusal, got {other:?}"),
        };
        assert_eq!(
            serde_json::to_value(refusal.code).expect("refusal code"),
            row["expect"]["code"],
            "{id}: code"
        );
        assert_eq!(
            serde_json::to_value(refusal.cause).expect("refusal cause"),
            row["expect"]["cause"],
            "{id}: cause"
        );
        assert_eq!(
            refusal
                .loci
                .first()
                .and_then(|at| at.path.as_ref())
                .map(|path| path.as_str()),
            row["expect"]["locus"].as_str(),
            "{id}: primary pointer"
        );
        assert_eq!(
            refusal.document_pointer.as_ref().map(|path| path.as_str()),
            row["expect"]["document_pointer"].as_str(),
            "{id}: document pointer"
        );
        if row["expect"]["stage"] == "schema" {
            assert!(
                refusal.locus.is_none(),
                "{id}: schema refusal has no graph locus"
            );
        } else {
            assert!(
                refusal.locus.is_some(),
                "{id}: reader refusal has a graph locus"
            );
        }
        if id == "C11" {
            let target = &edited["semantic_graph"]["nodes"][82]["body"]["arguments"][4]["value"]
                ["members"][0]["target"];
            assert_eq!(
                serde_json::to_value(refusal.locus).expect("retained graph target"),
                *target,
                "{id}: retained offending role target"
            );
            let role = edited["semantic_graph"]["nodes"]
                .as_array()
                .expect("graph nodes")
                .iter()
                .find(|node| node["node_id"] == *target)
                .expect("published target node");
            assert_eq!(
                role["semantic_form"],
                row["expect"]["retains"]["actual_kind"]
            );
        }
    }
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
    let mut failures = Vec::new();
    for row in rows {
        let id = row["id"].as_str().expect("row id");
        assert_eq!(row["base"], base_name, "{id}: selected base");
        assert_eq!(row["class"], "refuse", "{id}: refusal row");
        assert_eq!(row["expect"]["stage"], "schema", "{id}: schema stage");
        assert_eq!(row["expect"]["code"], "malformed_wire", "{id}: schema code");
        assert!(row["expect"]["cause"].is_null(), "{id}: no cause");
        let mut edited = base.clone();
        for edit in row["edit"].as_array().expect("published edits") {
            apply_layout_edit(&mut edited, edit);
        }
        let result = read(&edited, &document);
        match result {
            CheckedPackageV2ReadResult::Refused(refusal) => {
                let code = serde_json::to_value(refusal.code).expect("code");
                let cause = serde_json::to_value(refusal.cause).expect("cause");
                let path = refusal
                    .loci
                    .first()
                    .and_then(|locus| locus.path.as_ref())
                    .map(|path| path.as_str());
                if code != row["expect"]["code"]
                    || cause != row["expect"]["cause"]
                    || path != row["expect"]["locus"].as_str()
                    || refusal.locus.is_some()
                    || refusal.loci.len() != 1
                {
                    failures.push(format!(
                        "{id}: expected {:?}, got code={code} cause={cause} path={path:?} structural_locus={:?}",
                        row["expect"], refusal.locus
                    ));
                }
            }
            other => failures.push(format!("{id}: expected schema refusal, got {other:?}")),
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
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

fn protocol_node_index(package: &Value, operator: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol"
                && node["body"]["operation"]["member"]["operator"] == operator
        })
        .unwrap_or_else(|| panic!("published protocol {operator} node"))
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
    let boolean_type = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("base nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean type")["node_id"]
        .clone();
    let fields = [
        ("channel", "/properties/arguments/prefixItems/4/properties/value/properties/value/anyOf/0", 4, false),
        ("queue", "/properties/arguments/prefixItems/2/properties/value/properties/value", 2, false),
        ("attempt", "/properties/arguments/prefixItems/2/properties/value/properties/members/items/properties/value", 2, true),
        ("parallel", "/properties/arguments/prefixItems/0/properties/value/properties/members/items/properties/value", 0, true),
        ("parallel", "/properties/arguments/prefixItems/4/properties/value/properties/members/items/properties/value", 4, true),
        ("protocol", "/properties/arguments/prefixItems/10/properties/value/properties/members/items/properties/value", 10, true),
    ];
    for (operator, schema_field, ordinal, list) in fields {
        for value in protocol_enum_values(&schema, operator, schema_field) {
            let spelling = value.as_str().expect("text enum spelling");
            let mut changed = base.clone();
            let position = protocol_node_index(&changed, operator);
            let selected = &mut changed["semantic_graph"]["nodes"][position]["body"]["arguments"]
                [ordinal]["value"];
            if list {
                selected["members"] = Value::Array(vec![protocol_text_term(&text_type, spelling)]);
            } else {
                selected["value"] = value.clone();
            }
            if operator == "queue" && spelling == "loss" {
                let loss = protocol_enum_values(
                    &schema,
                    "queue",
                    "/properties/arguments/prefixItems/3/properties/value/properties/members/items/properties/value",
                )[0]
                    .as_str()
                    .expect("loss spelling");
                changed["semantic_graph"]["nodes"][position]["body"]["arguments"][3]["value"]
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
        let position = protocol_node_index(&changed, "queue");
        changed["semantic_graph"]["nodes"][position]["body"]["arguments"][2]["value"]["value"] =
            Value::from("loss");
        changed["semantic_graph"]["nodes"][position]["body"]["arguments"][3]["value"]["members"] =
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
        let position = protocol_node_index(&changed, "terminal");
        changed["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["value"] =
            mode.clone();
        if spelling == "when" {
            changed["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"]
                ["members"] = Value::Array(vec![serde_json::json!({
                "term":"literal", "type":boolean_type,
                "value_kind":"boolean", "value":true
            })]);
        }
        let result = read(&changed, &document);
        assert!(
            matches!(&result, CheckedPackageV2ReadResult::Admitted(_)),
            "terminal {spelling} must admit: {result:?}"
        );
    }
}

fn append_protocol_control(package: &mut Value, node: Value, name: &str) {
    let mut node = node;
    let run = protocol_node_index(package, "sequence");
    let mut path = package["semantic_graph"]["nodes"][run]["node_id"]["path"]
        .as_array()
        .expect("published run path")
        .clone();
    path.push(Value::from(name));
    node["node_id"]["path"] = Value::Array(path);
    node["declaration"]["qualified_name"] = node["node_id"]["path"].clone();
    package["semantic_graph"]["nodes"][run]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("run sequence children")
        .push(serde_json::json!({"term":"reference", "target":node["node_id"]}));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .push(node);
    fixture_support::settle(package);
}

#[test]
#[trace("TC-048", "FR-038-AC-286")]
fn current_qspec_schema_valid_delivery_bounds_refuse_at_channel_node() {
    let fixtures = fixtures_dir();
    let base = read_json(&fixtures.join("protocol-node-base.json")).expect("QSpec protocol base");
    let manifest = read_json(&fixtures.join("protocol-node-mutation-manifest.json"))
        .expect("QSpec protocol manifest");
    let document = fixtures.join(
        manifest["base"]["domain_document"]
            .as_str()
            .expect("selected domain document"),
    );

    for spelling in ["[4,3]", "[0,18446744073709551616]"] {
        let mut package = base.clone();
        let channel = protocol_node_index(&package, "channel");
        package["semantic_graph"]["nodes"][channel]["body"]["arguments"][4]["value"]["value"] =
            Value::from(spelling);
        assert_current_qspec_protocol_bodies_valid(&package, "channel");

        let result = read(&package, &document);
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{spelling}: schema-valid invalid interval must refuse: {result:?}");
        };
        let pointer = format!("/semantic_graph/nodes/{channel}");
        assert_eq!(
            serde_json::to_value(refusal.code).expect("code"),
            "malformed_wire"
        );
        assert!(refusal.cause.is_none(), "{spelling}: no semantic cause");
        assert_eq!(refusal.loci.len(), 1, "{spelling}: one primary locus");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some(pointer.as_str()),
            "{spelling}: channel node pointer"
        );
        assert!(refusal.locus.is_none(), "{spelling}: no structural locus");
    }
}

fn assert_appended_protocol_node_admits(
    base: &Value,
    node: Value,
    document: &std::path::Path,
    label: &str,
) {
    let mut package = base.clone();
    append_protocol_control(&mut package, node, "Fence");
    assert_current_qspec_protocol_bodies_valid(&package, "fence");
    let result = read(&package, document);
    assert!(
        matches!(&result, CheckedPackageV2ReadResult::Admitted(_)),
        "published {label} layout must admit: {result:?}"
    );
}

fn fresh_parameter(package: &mut Value, template: &Value, name: &str, level: &str) -> Value {
    let mut node = template.clone();
    node["body"]["members"][0]["value"]["value"] = Value::from(name);
    node["body"]["members"][1]["value"]["value"] = Value::from(level);
    node["node_id"]["path"] = serde_json::json!(["value", "parameter", name, level]);
    let target = node["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .push(node);
    target
}

fn published_operator_arguments(fixtures: &std::path::Path, operator: &str) -> Vec<String> {
    let vectors = read_json(&fixtures.join("protocol-node-vectors.json"))
        .expect("current QSpec protocol vectors");
    vectors["operators"]
        .as_array()
        .expect("published operators")
        .iter()
        .find(|row| row["operator"] == operator && row["semantic_form"] == "control")
        .unwrap_or_else(|| panic!("published {operator} control"))["arguments"]
        .as_array()
        .expect("published arguments")
        .iter()
        .map(|name| name.as_str().expect("argument name").to_owned())
        .collect()
}

fn published_bindings(fixtures: &std::path::Path, operator: &str, values: Vec<Value>) -> Value {
    let names = published_operator_arguments(fixtures, operator);
    assert_eq!(names.len(), values.len(), "{operator}: published arity");
    Value::Array(
        names
            .into_iter()
            .zip(values)
            .map(|(name, value)| serde_json::json!({"term":"binding", "name":name, "value":value}))
            .collect(),
    )
}

fn assert_current_qspec_protocol_bodies_valid(package: &Value, operator: &str) {
    let schema = read_json(&proposal_dir_of(&fixtures_dir()).join("schema.json"))
        .expect("current QSpec checked-package schema");
    let application_schema = serde_json::json!({
        "$schema":schema["$schema"],
        "$defs":schema["$defs"],
        "$ref":"#/$defs/ProtocolControlApplication"
    });
    let validator = jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&application_schema)
        .expect("QSpec schema compiles");
    let mut found = false;
    for node in package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
    {
        if node["node_tag"] != "protocol"
            || node["body"]["operation"]["member"]["operator"] != operator
        {
            continue;
        }
        found = true;
        if let Err(errors) = validator.validate(&node["body"]) {
            let details = errors
                .take(3)
                .map(|error| error.to_string())
                .collect::<Vec<_>>();
            panic!("{operator} body must satisfy current QSpec schema: {details:?}");
        };
    }
    assert!(found, "published {operator} body exists");
}

fn qspec_event_package() -> (Value, std::path::PathBuf, Value) {
    let fixtures = fixtures_dir();
    let base = read_json(&fixtures.join("protocol-node-base.json")).expect("QSpec protocol base");
    let manifest = read_json(&fixtures.join("protocol-node-mutation-manifest.json"))
        .expect("QSpec protocol manifest");
    let document = fixtures.join(
        manifest["base"]["domain_document"]
            .as_str()
            .expect("selected domain document"),
    );
    let mut package = base.clone();
    let finish = protocol_node_index(&package, "finish");
    let finish_target = package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    let template = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == finish_target)
        .expect("finish binder")
        .clone();
    let level = template["body"]["members"][1]["value"]["value"]
        .as_str()
        .expect("finish level")
        .parse::<usize>()
        .expect("finish level integer");
    let binder = fresh_parameter(&mut package, &template, "event_result", &level.to_string());
    let finish_parameter = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == finish_target)
        .expect("finish parameter position");
    package["semantic_graph"]["nodes"][finish_parameter]["body"]["members"][1]["value"]["value"] =
        Value::from((level + 1).to_string());
    let role = package["semantic_graph"]["nodes"][protocol_node_index(&package, "attempt")]["body"]
        ["arguments"][0]["value"]
        .clone();
    let boolean_type = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean type")["node_id"]
        .clone();
    let mut event = read_json(&fixtures.join("protocol-node-layout-rows.json"))
        .expect("layout rows")["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .find(|row| row["id"] == "L12")
        .expect("fence template")["edit"][0]["value"]
        .clone();
    event["body"]["operation"]["member"]["operator"] = Value::from("event");
    event["body"]["arguments"] = published_bindings(
        &fixtures,
        "event",
        vec![
            role,
            serde_json::json!({"term":"aggregate","members":[]}),
            serde_json::json!({"term":"reference","target":binder}),
            serde_json::json!({"term":"aggregate","members":[]}),
            serde_json::json!({"term":"literal","type":boolean_type,"value_kind":"boolean","value":true}),
        ],
    );
    append_protocol_control(&mut package, event.clone(), "Event");
    (package, document, event)
}

#[test]
#[trace("TC-048", "FR-038-AC-454")]
fn current_qspec_appended_event_layout_remains_admitted() {
    let (package, document, _) = qspec_event_package();
    assert_current_qspec_protocol_bodies_valid(&package, "event");
    let result = read(&package, &document);
    assert!(
        matches!(&result, CheckedPackageV2ReadResult::Admitted(_)),
        "published event layout must admit: {result:?}"
    );
}

fn append_nested_control(
    package: &mut Value,
    mut node: Value,
    parent: &[Value],
    name: &str,
) -> Value {
    let mut path = parent.to_vec();
    path.push(Value::from(name));
    node["node_id"]["path"] = Value::Array(path);
    node["declaration"]["qualified_name"] = node["node_id"]["path"].clone();
    let target = node["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .push(node);
    target
}

#[test]
#[trace("TC-048", "FR-038-AC-454")]
fn current_qspec_appended_await_layout_remains_admitted() {
    let fixtures = fixtures_dir();
    let (mut package, document, event_template) = qspec_event_package();
    let anchor = package["semantic_graph"]["nodes"][protocol_node_index(&package, "event")]
        ["node_id"]
        .clone();
    let finish = protocol_node_index(&package, "finish");
    let finish_target = package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    let finish_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == finish_target)
        .expect("finish binder");
    let finish_template = package["semantic_graph"]["nodes"][finish_position].clone();
    let level = finish_template["body"]["members"][1]["value"]["value"]
        .as_str()
        .expect("finish level")
        .parse::<usize>()
        .expect("integer level");
    let awaited_binder = fresh_parameter(
        &mut package,
        &finish_template,
        "await_result",
        &level.to_string(),
    );
    let mut awaited_event = event_template;
    awaited_event["body"]["arguments"][2]["value"]["target"] = awaited_binder;
    let run = protocol_node_index(&package, "sequence");
    let mut await_path = package["semantic_graph"]["nodes"][run]["node_id"]["path"]
        .as_array()
        .expect("run path")
        .clone();
    await_path.push(Value::from("Await"));
    let event = append_nested_control(&mut package, awaited_event, &await_path, "Event");
    let layout = read_json(&fixtures.join("protocol-node-layout-rows.json")).expect("layout rows");
    let mut fence = layout["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .find(|row| row["id"] == "L12")
        .expect("fence template")["edit"][0]["value"]
        .clone();
    let schema =
        read_json(&proposal_dir_of(&fixtures).join("schema.json")).expect("current QSpec schema");
    fence["body"]["arguments"][0]["value"]["value"] = protocol_enum_values(
        &schema,
        "fence",
        "/properties/arguments/prefixItems/0/properties/value/properties/value",
    )[0]
    .clone();
    let then = append_nested_control(&mut package, fence.clone(), &await_path, "Then");
    let timeout = append_nested_control(&mut package, fence.clone(), &await_path, "Timeout");
    let scalar = |form: &str| {
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == form)
            .unwrap_or_else(|| panic!("{form} scalar type"))["node_id"]
            .clone()
    };
    let text_type = scalar("text");
    let integer_type = scalar("integer");
    let text = |spelling: &str| protocol_text_term(&text_type, spelling);
    let integer = |spelling: &str| {
        serde_json::json!({
            "term":"literal", "type":integer_type, "value_kind":"integer", "value":spelling
        })
    };
    let binding = |name: &str, value: Value| serde_json::json!({"term":"binding", "name":name, "value":value});
    let reference = |target: Value| serde_json::json!({"term":"reference", "target":target});
    let group = |members: Vec<Value>| serde_json::json!({"term":"aggregate", "members":members});
    let mut awaited = fence;
    awaited["body"]["operation"]["member"]["operator"] = Value::from("await");
    awaited["body"]["arguments"] = published_bindings(
        &fixtures,
        "await",
        vec![
            group(vec![
                binding("kind", text("event")),
                binding("target", reference(anchor)),
            ]),
            text("clock"),
            group(vec![
                binding("lower", integer("0")),
                binding("upper", integer("1")),
            ]),
            reference(event),
            reference(then),
            reference(timeout),
        ],
    );
    append_protocol_control(&mut package, awaited, "Await");
    assert_current_qspec_protocol_bodies_valid(&package, "await");
    assert_current_qspec_protocol_bodies_valid(&package, "event");
    assert_current_qspec_protocol_bodies_valid(&package, "fence");
    let result = read(&package, &document);
    assert!(
        matches!(&result, CheckedPackageV2ReadResult::Admitted(_)),
        "published await layout must admit: {result:?}"
    );
}

#[test]
#[trace("TC-048", "FR-038-AC-454")]
fn current_qspec_appended_fence_layouts_remain_admitted() {
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
}
