// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-976 index recipes and TC-967 body recipes, read from QSpec by reference.

use super::*;
use quire_contract_model::{CheckedSchedulabilityKind, CheckedSchedulingPolicy};

fn aggregate(members: Vec<Value>) -> Value {
    json!({"term":"aggregate","members":members})
}
fn binding(name: &str, value: Value) -> Value {
    json!({"term":"binding","name":name,"value":value})
}
fn literal(type_id: &Value, kind: &str, value: &str) -> Value {
    json!({"term":"literal","type":type_id,"value_kind":kind,"value":value})
}

fn recipes() -> Value {
    let proposal = proposal_dir_of(&fixtures_dir())
        .canonicalize()
        .expect("QSpec proposal directory");
    let root = proposal
        .parent()
        .and_then(Path::parent)
        .expect("QSpec root");
    let text = std::fs::read_to_string(
        root.join("spec/test-cases/TC-967-checked-package-schedulability-claim-node.md"),
    )
    .expect("QSpec TC-967 body recipes");
    let published = text
        .lines()
        .filter_map(|line| line.strip_prefix("| SC-"))
        .map(|line| {
            line.split('|')
                .next()
                .expect("case column")
                .trim()
                .parse::<u32>()
                .expect("SC case number")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        published,
        (1..=22).collect::<Vec<_>>(),
        "every published body recipe is accounted for"
    );
    let vectors = read_json(&proposal.join("schedulability-claim-vectors.json"))
        .expect("QSpec carrier recipes");
    for (list, count) in [
        ("positive", 2),
        ("schema_refusals", 4),
        ("reader_refusals", 6),
    ] {
        assert_eq!(
            vectors[list]
                .as_array()
                .expect("published nonempty recipe list")
                .len(),
            count,
            "{list}"
        );
    }
    vectors
}

fn row_schema() -> jsonschema::JSONSchema {
    let schema = read_json(&proposal_dir_of(&fixtures_dir()).join("schema.json"))
        .expect("QSpec schema by reference");
    let row = json!({"$ref":"#/$defs/SchedulabilityClaimRef","$defs":schema["$defs"]});
    jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .compile(&row)
        .expect("published row schema")
}

fn base() -> Value {
    read_json(&fixtures_dir().join("positive-timed-model-nodes.json"))
        .expect("QSpec published Time and unit")
}
fn position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["semantic_form"] == "schedulability")
        .expect("claim")
}
fn type_id(package: &Value, form: &str) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["semantic_form"] == form)
        .expect("published supporting type")["node_id"]
        .clone()
}

// TC-976's rows address the TC-967 component: its package placeholder and unit
// index are relocated into the published timed-model package used for this run.
fn relocated(mut rows: Value, package: &Value) -> Value {
    let unit = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["semantic_form"] == "unit")
        .unwrap();
    fn visit(value: &mut Value, name: &Value, unit: usize) {
        match value {
            Value::Object(object) => {
                if object.get("package").is_some_and(|v| v == "package") {
                    object.insert("package".into(), name.clone());
                }
                if object.len() == 1 && object.get("node") == Some(&json!(6)) {
                    object.insert("node".into(), json!(unit));
                }
                for child in object.values_mut() {
                    visit(child, name, unit);
                }
            }
            Value::Array(items) => {
                for child in items {
                    visit(child, name, unit);
                }
            }
            _ => {}
        }
    }
    visit(&mut rows, &package["name"], unit);
    rows
}

fn append_claim(
    package: &mut Value,
    declaration: Value,
    occurrences: Vec<u64>,
    policy: &str,
    response: bool,
) {
    let text = type_id(package, "text");
    let integer = type_id(package, "integer");
    let unit = type_id(package, "unit");
    let boolean = type_id(package, "boolean");
    let word = |value: &str| literal(&text, "text", value);
    let quantity = |value: &str| literal(&unit, "quantity", value);
    let tasks = [
        ("t1", "4/1", "1/1", "1"),
        ("t2", "6/1", "2/1", "2"),
        ("t3", "12/1", "5/1", "3"),
    ]
    .into_iter()
    .map(|(name, period, wcet, priority)| {
        aggregate(vec![
            binding("name", word(name)),
            binding("arrival", word("periodic")),
            binding("period", quantity(period)),
            binding("deadline", quantity(period)),
            binding("bcet", quantity("0/1")),
            binding("wcet", quantity(wcet)),
            binding("wcet_hi", aggregate(vec![])),
            binding("criticality", word("lo")),
            binding("priority", literal(&integer, "integer", priority)),
            binding("jitter", quantity("0/1")),
            binding("blocking", quantity("0/1")),
        ])
    })
    .collect();
    package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "node_id":declaration,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"claim","semantic_form":"schedulability",
        "semantic_type":boolean,"dependencies":[],"occurrences":occurrences.into_iter().map(|ordinal| json!({"role":"claim","ordinal":ordinal})).collect::<Vec<_>>(),
        "body":aggregate(vec![binding("kind",word(if response {"response"} else {"schedulable"})),binding("policy",word(policy)),
            binding("task",if response {word("t3")} else {aggregate(vec![])}),binding("bound",if response {quantity("12/1")} else {aggregate(vec![])}),binding("tasks",aggregate(tasks))])
    }));
}
fn finish(package: &mut Value) {
    fixture_support::rebuild_source_map(package);
    fixture_support::refresh_identity(package);
}
fn carrier(vectors: &Value, shared: bool) -> Value {
    let mut package = base();
    let rows = relocated(
        vectors["positive"][usize::from(shared)]["rows"].clone(),
        &package,
    );
    assert_eq!(rows.as_array().unwrap().len(), if shared { 2 } else { 1 });
    append_claim(
        &mut package,
        rows[0]["declaration"].clone(),
        if shared { vec![0, 1] } else { vec![0] },
        "fixed-priority",
        false,
    );
    package["schedulability_claims"] = rows;
    finish(&mut package);
    package
}
fn read(package: &Value) -> CheckedPackageV2ReadResult {
    read_package(package).expect("published selected-model evidence")
}
fn admitted(package: &Value) -> Box<CheckedPackageV2> {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(read) => read,
        other => panic!("TC-967 recipe must admit: {other:?}"),
    }
}
fn refuses(package: &Value, code: Code, cause: Option<Cause>, locus: &str) {
    let result = read(package);
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("{locus}: expected {code:?}, got {result:?}")
    };
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(refusal.loci.len(), 1, "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        locus,
        "{refusal:?}"
    );
}

#[trace("TC-048", "FR-038-AC-396", "FR-038-AC-398", "FR-038-AC-400")]
#[test]
fn published_carrier_positives_authenticate_each_owned_occurrence() {
    let vectors = recipes();
    let schema = row_schema();
    for case in vectors["positive"].as_array().unwrap() {
        for row in case["rows"].as_array().unwrap() {
            assert!(schema.is_valid(row), "positive published carrier row");
        }
    }
    for shared in [false, true] {
        let wire = carrier(&vectors, shared);
        let read = admitted(&wire);
        assert_eq!(
            read.schedulability_claims().len(),
            if shared { 2 } else { 1 }
        );
        let claims = read.schedulability_claims().collect::<Vec<_>>();
        for (ordinal, claim) in claims.iter().enumerate() {
            assert_eq!(claim.occurrence().ordinal, u64::try_from(ordinal).unwrap());
            assert_eq!(claim.policy(), CheckedSchedulingPolicy::FixedPriority);
            assert_eq!(claim.kind(), CheckedSchedulabilityKind::Schedulable);
            assert_eq!(
                claim.tasks().map(|task| task.name()).collect::<Vec<_>>(),
                ["t1", "t2", "t3"]
            );
        }
        if shared {
            assert!(std::ptr::eq(claims[0].body(), claims[1].body()));
        }
    }
}

#[trace("TC-048", "FR-038-AC-397", "FR-038-AC-400")]
#[test]
fn every_published_carrier_schema_refusal_precedes_body_validation() {
    let vectors = recipes();
    let schema = row_schema();
    for case in vectors["schema_refusals"].as_array().unwrap() {
        assert!(!schema.is_valid(&case["row"]), "published malformed row");
        let mut wire = carrier(&vectors, false);
        wire["schedulability_claims"] = json!([relocated(case["row"].clone(), &wire)]);
        let n = position(&wire);
        wire["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"] = json!([]);
        finish(&mut wire);
        assert_eq!(case["expected_code"], "malformed_wire");
        refuses(
            &wire,
            Code::MalformedWire,
            None,
            case["locus"].as_str().unwrap(),
        );
    }
}

#[trace("TC-048", "FR-038-AC-397", "FR-038-AC-400")]
#[test]
fn every_published_carrier_semantic_refusal_has_exact_cause_and_locus() {
    let vectors = recipes();
    let schema = row_schema();
    for case in vectors["reader_refusals"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let mut wire = carrier(
            &vectors,
            matches!(id, "SCC-09-duplicate-key" | "SCC-10-missing-claim-coverage"),
        );
        match id {
            "SCC-06-foreign-package" => {
                wire["schedulability_claims"][0]["declaration"]["package"] = json!("foreign")
            }
            "SCC-07-wrong-node-form" => {
                let n = position(&wire);
                wire["semantic_graph"]["nodes"][n]["semantic_form"] = json!("state_graph");
            }
            "SCC-08-unowned-occurrence" => {
                wire["schedulability_claims"][0]["occurrence"] = json!(1)
            }
            "SCC-09-duplicate-key" => {
                wire["schedulability_claims"][1] = wire["schedulability_claims"][0].clone()
            }
            "SCC-10-missing-claim-coverage" => {
                wire["schedulability_claims"].as_array_mut().unwrap().pop();
            }
            "SCC-11-noncanonical-order" => {
                let rows = relocated(case["rows"].clone(), &wire);
                let existing = wire["semantic_graph"]["nodes"].as_array_mut().unwrap();
                existing.retain(|node| node["semantic_form"] != "schedulability");
                for row in rows.as_array().unwrap() {
                    let path = row["declaration"]["path"].as_array().unwrap();
                    let policy = path.iter().position(|v| v == "policy").unwrap();
                    append_claim(
                        &mut wire,
                        row["declaration"].clone(),
                        vec![0],
                        path[policy + 1].as_str().unwrap(),
                        false,
                    );
                }
                wire["schedulability_claims"] = rows;
            }
            _ => panic!("unhandled published carrier recipe {id}"),
        }
        for row in wire["schedulability_claims"].as_array().unwrap() {
            assert!(
                schema.is_valid(row),
                "{id}: semantic refusal requires schema-valid row"
            );
        }
        finish(&mut wire);
        assert_eq!(case["expected_code"], "invalid_package");
        let cause = if case["expected_cause"] == "conflicting-definition" {
            Cause::ConflictingDefinition
        } else {
            assert_eq!(case["expected_cause"], "invalid-value");
            Cause::InvalidValue
        };
        refuses(
            &wire,
            Code::InvalidPackage,
            Some(cause),
            case["locus"].as_str().unwrap(),
        );
    }
}

fn task_value(package: &mut Value, task: usize, member: usize) -> &mut Value {
    let n = position(package);
    &mut package["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"][task]
        ["members"][member]["value"]
}
fn task_path(n: usize, t: usize) -> String {
    format!("/semantic_graph/nodes/{n}/body/members/4/value/members/{t}")
}

#[trace("TC-048", "FR-038-AC-398", "FR-038-AC-400")]
#[test]
fn published_body_refusals_cover_shape_literals_fit_order_units_and_task_premises() {
    let vectors = recipes();
    for case in [
        "SC-08",
        "SC-09",
        "SC-10",
        "SC-11",
        "SC-12",
        "SC-13",
        "SC-14",
        "SC-15",
        "SC-16",
        "SC-17",
        "SC-18",
        "SC-19",
        "SC-21-negative",
        "SC-21-above",
        "SC-21-missing",
        "HI-bcet-above",
    ] {
        let mut wire = carrier(&vectors, false);
        let n = position(&wire);
        let body = format!("/semantic_graph/nodes/{n}/body");
        let tasks = format!("{body}/members/4");
        let mut code = Code::InvalidSemanticGraph;
        let mut cause = None;
        let locus = match case {
            "SC-08" => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"] =
                    json!([]);
                tasks.clone()
            }
            "SC-09" => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"]
                    .as_array_mut()
                    .unwrap()
                    .swap(0, 1);
                body.clone()
            }
            "SC-10" => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][1]["value"]["value"] =
                    json!("rate-monotonic");
                code = Code::InvalidPackage;
                cause = Some(Cause::InvalidValue);
                format!("{body}/members/1")
            }
            "SC-11" => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][0]["value"]["value"] =
                    json!("response");
                format!("{body}/members/2")
            }
            "SC-12" => {
                *task_value(&mut wire, 0, 6) = task_value(&mut wire, 1, 5).clone();
                format!("{}/members/6", task_path(n, 0))
            }
            "SC-13" => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"]
                    .as_array_mut()
                    .unwrap()
                    .swap(0, 1);
                tasks.clone()
            }
            "SC-14" => {
                let text = type_id(&wire, "text");
                let unit = type_id(&wire, "unit");
                wire["semantic_graph"]["nodes"][n]["body"]["members"][0]["value"]["value"] =
                    json!("response");
                wire["semantic_graph"]["nodes"][n]["body"]["members"][2]["value"] =
                    literal(&text, "text", "t9");
                wire["semantic_graph"]["nodes"][n]["body"]["members"][3]["value"] =
                    literal(&unit, "quantity", "12/1");
                format!("{body}/members/2")
            }
            "SC-15" => {
                // A separately declared source dimension and unit are valid graph facts,
                // but cannot type this task-set time value.
                let nodes = wire["semantic_graph"]["nodes"].as_array().unwrap();
                let mut dimension = nodes
                    .iter()
                    .find(|node| node["semantic_form"] == "dimension")
                    .unwrap()
                    .clone();
                let mut unit = nodes
                    .iter()
                    .find(|node| node["semantic_form"] == "unit")
                    .unwrap()
                    .clone();
                let old = dimension["node_id"].clone();
                let mut fresh = old.clone();
                fresh["path"] = json!(["Example", "Length"]);
                dimension["node_id"] = fresh.clone();
                dimension["semantic_type"] = fresh.clone();
                dimension["declaration"]["qualified_name"] = json!(["Example", "Length"]);
                unit["node_id"]["path"] = json!(["Example", "metre"]);
                unit["declaration"]["qualified_name"] = json!(["Example", "metre"]);
                unit["semantic_type"] = fresh.clone();
                unit["dependencies"] = json!([fresh]);
                unit["body"]["dimension_node_id"] = unit["semantic_type"].clone();
                task_value(&mut wire, 1, 2)["type"] = unit["node_id"].clone();
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .extend([dimension, unit]);
                code = Code::IllTyped;
                cause = Some(Cause::TypeMismatch);
                format!("{}/members/2/value", task_path(n, 1))
            }
            "SC-19" => {
                wire["semantic_graph"]["nodes"][n]["dependencies"] =
                    json!([type_id(&wire, "unit")]);
                format!("/semantic_graph/nodes/{n}/dependencies")
            }
            "SC-21-missing" => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"][0]
                    ["members"]
                    .as_array_mut()
                    .unwrap()
                    .remove(4);
                task_path(n, 0)
            }
            _ => {
                let (task, member, value) = match case {
                    "SC-16" => (0, 2, "0/1"),
                    "SC-17" => (1, 9, "-1/1"),
                    "SC-18" => (2, 8, "1"),
                    "SC-21-negative" => (0, 4, "-1/1"),
                    "SC-21-above" => (0, 4, "2/1"),
                    "HI-bcet-above" => (0, 4, "3/1"),
                    _ => unreachable!(),
                };
                if case == "HI-bcet-above" {
                    *task_value(&mut wire, 0, 6) = task_value(&mut wire, 1, 5).clone();
                    task_value(&mut wire, 0, 7)["value"] = json!("hi");
                }
                task_value(&mut wire, task, member)["value"] = json!(value);
                code = Code::InvalidTimedModel;
                cause = Some(Cause::InvalidTaskSet);
                task_path(n, task)
            }
        };
        finish(&mut wire);
        refuses(&wire, code, cause, &locus);
    }
}

#[trace("TC-048", "FR-038-AC-398", "FR-038-AC-400")]
#[test]
fn published_body_positive_variants_retain_response_high_work_and_fixed_work() {
    let vectors = recipes();
    for case in [
        "SC-02",
        "SC-03",
        "SC-04-wcet",
        "SC-04-bcet",
        "SC-05",
        "SC-07",
        "SC-18-edf",
        "SC-20",
        "SC-22-lo",
        "SC-22-hi-interval",
        "SC-22-hi-fixed",
    ] {
        let mut wire = carrier(&vectors, false);
        let n = position(&wire);
        match case {
            "SC-02" => {
                let text = type_id(&wire, "text");
                let unit = type_id(&wire, "unit");
                wire["semantic_graph"]["nodes"][n]["body"]["members"][0]["value"]["value"] =
                    json!("response");
                wire["semantic_graph"]["nodes"][n]["body"]["members"][2]["value"] =
                    literal(&text, "text", "t3");
                wire["semantic_graph"]["nodes"][n]["body"]["members"][3]["value"] =
                    literal(&unit, "quantity", "12/1");
            }
            "SC-03" | "SC-18-edf" => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][1]["value"]["value"] =
                    json!("edf");
                if case == "SC-18-edf" {
                    task_value(&mut wire, 2, 8)["value"] = json!("1");
                }
            }
            "SC-04-wcet" => task_value(&mut wire, 2, 5)["value"] = json!("6/1"),
            "SC-04-bcet" => task_value(&mut wire, 2, 4)["value"] = json!("1/1"),
            "SC-05" => {}
            "SC-07" => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][1]["value"]["value"] =
                    json!("amc");
                task_value(&mut wire, 0, 7)["value"] = json!("hi");
                *task_value(&mut wire, 0, 6) = task_value(&mut wire, 1, 5).clone();
            }
            "SC-20" => {
                task_value(&mut wire, 0, 4)["value"] = json!("1/1");
                task_value(&mut wire, 1, 4)["value"] = json!("2/1");
            }
            "SC-22-lo" => task_value(&mut wire, 0, 4)["value"] = json!("1/1"),
            "SC-22-hi-interval" | "SC-22-hi-fixed" => {
                task_value(&mut wire, 0, 7)["value"] = json!("hi");
                task_value(&mut wire, 0, 4)["value"] = json!("2/1");
                task_value(&mut wire, 0, 5)["value"] = json!("2/1");
                *task_value(&mut wire, 0, 6) = task_value(&mut wire, 0, 5).clone();
                if case == "SC-22-hi-interval" {
                    task_value(&mut wire, 0, 6)["value"] = json!("3/1");
                }
            }
            _ => unreachable!(),
        }
        finish(&mut wire);
        let read = admitted(&wire);
        let claim = read
            .schedulability_claims()
            .next()
            .expect("authenticated variant");
        assert_eq!(claim.tasks().len(), 3, "{case}");
        assert_eq!(
            serde_json::to_value(claim.body()).unwrap(),
            wire["semantic_graph"]["nodes"][n]["body"],
            "{case}: complete admitted premises"
        );
    }
}

#[trace("TC-048", "FR-038-AC-398", "FR-038-AC-400")]
#[test]
fn body_validation_stages_choose_the_first_defect() {
    let vectors = recipes();
    for stage in 0..5 {
        let mut wire = carrier(&vectors, false);
        let n = position(&wire);
        let body = format!("/semantic_graph/nodes/{n}/body");
        let (code, cause, locus) = match stage {
            0 => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"] =
                    json!([]);
                wire["semantic_graph"]["nodes"][n]["body"]["members"][1]["value"]["value"] =
                    json!("unknown");
                (
                    Code::InvalidSemanticGraph,
                    None,
                    format!("{body}/members/4"),
                )
            }
            1 => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][1]["value"]["value"] =
                    json!("unknown");
                wire["semantic_graph"]["nodes"][n]["body"]["members"][0]["value"]["value"] =
                    json!("response");
                (
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    format!("{body}/members/1"),
                )
            }
            2 => {
                wire["semantic_graph"]["nodes"][n]["body"]["members"][0]["value"]["value"] =
                    json!("response");
                wire["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"]
                    .as_array_mut()
                    .unwrap()
                    .swap(0, 1);
                (
                    Code::InvalidSemanticGraph,
                    None,
                    format!("{body}/members/2"),
                )
            }
            3 => {
                let wrong = type_id(&wire, "integer");
                task_value(&mut wire, 0, 2)["type"] = wrong;
                wire["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"]
                    .as_array_mut()
                    .unwrap()
                    .swap(0, 1);
                (
                    Code::InvalidSemanticGraph,
                    None,
                    format!("{body}/members/4"),
                )
            }
            4 => {
                let wrong = type_id(&wire, "integer");
                task_value(&mut wire, 0, 2)["type"] = wrong;
                task_value(&mut wire, 0, 2)["value"] = json!("0/1");
                (
                    Code::IllTyped,
                    Some(Cause::TypeMismatch),
                    format!("{}/members/2/value", task_path(n, 0)),
                )
            }
            _ => unreachable!(),
        };
        finish(&mut wire);
        refuses(&wire, code, cause, &locus);
    }
}
