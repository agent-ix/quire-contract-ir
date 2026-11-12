// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-987: QSpec's operation-body vectors against the production reader and writer.

use super::operation_effects::{expand_document, normalize_relationship, patch};
use super::{
    cause_word, code_word, fixture_support as fixture, fixtures_dir, proposal_dir_of, read_json,
};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    encode_checked_package_v2, CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageSidecarsRef, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult, OperationBody, OperationBodyRow,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("operation-body-vectors.json"))
        .expect("current QSpec TC-987 vectors")
}

fn text_type() -> Value {
    fixture::structural_id("example", vec![json!("scalar_type"), json!("text")], 0)
}

fn graph_node(entry: &Value) -> Value {
    let key = entry["node"].clone();
    let text = text_type();
    let mut node = json!({
        "node_id": key, "schema_version": "quire.checked-semantic-graph/v2",
        "semantic_type": text, "dependencies": [],
        "occurrences": [{"role": "generated", "ordinal": 0}],
    });
    match entry["form"].as_str().expect("graph node form") {
        "function" => {
            node["node_tag"] = json!("function");
            node["semantic_form"] = key["path"][1].clone();
            node["owner"] = entry["owner"].clone();
            node["body"] = json!({"term": "aggregate", "members": [{
                "term": "binding", "name": "clause",
                "value": {"term": "literal", "type": text, "value_kind": "text",
                    "value": entry["clause"]},
            }]});
        }
        "expression" => {
            node["node_tag"] = json!("value");
            node["semantic_form"] = json!("literal");
            node["body"] = json!({"term": "literal", "type": text, "value_kind": "text",
                "value": "1"});
        }
        form => panic!("unsupported vector graph node form {form}"),
    }
    node
}

/// A function node whose owner member lies outside its owner document.
fn foreign_owner(entry: &Value) -> bool {
    entry["form"] == "function"
        && !entry["owner"]["node"].as_str().is_some_and(|node| {
            node.starts_with(&format!(
                "ix://{}/",
                entry["owner"]["identity"].as_str().unwrap_or_default()
            ))
        })
}

/// The effect vectors' document expansion, with `process` types given the
/// process meaning they would otherwise lose to the object-type construct.
fn expand_document_with_processes(sketch: &Value) -> Value {
    let identity = sketch["identity"].as_str().expect("document identity");
    let mut document = expand_document(sketch);
    let mut any = false;
    for ty in document["types"].as_array_mut().expect("types") {
        let is_process = sketch["types"]
            .as_array()
            .expect("sketch types")
            .iter()
            .any(|declared| {
                declared["form"] == "process"
                    && ty["identity"]
                        == format!("ix://{identity}/{}", declared["name"].as_str().unwrap())
            });
        for operation in ty["operations"].as_array_mut().expect("operations") {
            // A dispatched call is typed by its operation's result.
            operation["returns"] = json!({"typeRef": "ix://quire/native/Integer",
                "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}});
        }
        if ty["identity"] == format!("ix://{identity}/Premium")
            && sketch["types"]
                .as_array()
                .expect("types")
                .iter()
                .any(|t| t["name"] == "Order")
        {
            ty["supertypes"] = json!([format!("ix://{identity}/Order")]);
        }
        if is_process {
            ty["kind"] = json!({"module": identity, "name": "process"});
            let key = format!("{}/id", ty["identity"].as_str().expect("type identity"));
            ty["fields"] = json!([{"identity": key, "name": "id",
                "typeRef": "ix://quire/native/Integer", "presence": "required",
                "nullable": false, "defaultKind": "none",
                "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}}]);
            ty["identityFields"] = json!([key]);
            ty["steps"] = json!([{"name": "Run", "stepKind": "command",
                "consumes": [], "emits": []}]);
            any = true;
        }
    }
    if any {
        document["constructs"]
            .as_array_mut()
            .expect("constructs")
            .push(json!({
                "kind": {"module": identity, "name": "process"},
                "construct": {"meaning": "quire.meaning.model.process/v1"}
            }));
    }
    document
}

fn scalar(form: &str) -> Value {
    fixture::structural_id("example", vec![json!("scalar_type"), json!(form)], 0)
}

fn reference_term(target: &Value) -> Value {
    json!({"term": "reference", "target": target})
}

fn literal(ty: &Value, kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": ty, "value_kind": kind, "value": value})
}

fn plain_node(
    key: &Value,
    tag: &str,
    form: &str,
    ty: &Value,
    deps: Vec<Value>,
    role: &str,
    body: Value,
) -> Value {
    let mut deps = deps;
    deps.sort_by_key(fixture::structural_fixture_id);
    json!({"node_id": key, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag, "semantic_form": form, "semantic_type": ty, "dependencies": deps,
        "occurrences": [{"role": role, "ordinal": 0}], "body": body})
}

/// One `dispatch_call` of the candidate's selected operation on a receiver of
/// the declaring type, and the dispatch table that holds the candidate.
fn add_dispatch_table(package: &mut Value, candidates: &[Value]) {
    let Some(candidate) = candidates.first() else {
        return;
    };
    let declaring = candidate["selected_operation"]["declaring"].clone();
    let operation = candidate["selected_operation"]["operation"].clone();
    let integer = scalar("integer");
    let reference_type = fixture::structural_id(
        "example",
        vec![
            json!("composite_type"),
            json!("reference"),
            declaring.clone(),
        ],
        0,
    );
    let receiver =
        fixture::structural_id("example", vec![json!("parameter"), json!("receiver")], 0);
    let call = fixture::structural_id("example", vec![json!("call"), json!("dispatch")], 0);
    let text = text_type();
    let empty = json!({"term": "aggregate", "members": []});
    let parameter_body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value": literal(&text, "text", "receiver")},
        {"term": "binding", "name": "level", "value": literal(&integer, "integer", "0")},
    ]});
    let call_body = json!({
        "term": "application", "operator": "call",
        "operation": {"identity": "quire.op.model.dispatch_call", "laws": [], "mode": null,
            "member": {"kind": "operation", "declaration": declaring, "name": operation},
            "leaves": []},
        "result_type": integer, "arguments": [reference_term(&receiver)],
    });
    let nodes = package["semantic_graph"]["nodes"].as_array_mut().unwrap();
    nodes.extend([
        plain_node(
            &integer,
            "scalar_type",
            "integer",
            &integer,
            vec![],
            "type",
            empty,
        ),
        plain_node(
            &reference_type,
            "composite_type",
            "reference",
            &reference_type,
            vec![declaring.clone()],
            "type",
            json!({"term": "aggregate", "members": [reference_term(&declaring)]}),
        ),
        plain_node(
            &receiver,
            "value",
            "parameter",
            &reference_type,
            vec![],
            "expression",
            parameter_body,
        ),
        plain_node(
            &call,
            "expression",
            "call",
            &integer,
            vec![receiver.clone(), declaring.clone()],
            "type",
            call_body,
        ),
    ]);
    package["checked_term_data"] = json!({
        "equality_decisions": [], "protocol_locations": [], "probabilistic_claims": [],
        "dispatch_tables": [{
            "application": call, "occurrence": {"role": "type", "ordinal": 0},
            "operation": candidate["selected_operation"], "context": declaring,
            "candidates": candidates.iter().map(|held| json!({
                "selected_operation": held["selected_operation"],
                "selection_key": {"owner_effective_type": null,
                    "original": {"package": held["selection_type"]["package"],
                        "node": held["selection_type"]["path"][2]}},
                "operation_body": held["operation_body"], "effective_precondition": null,
            })).collect::<Vec<_>>(),
        }],
    });
}

fn read(case: &Value) -> (Value, CheckedPackageEvidence, CheckedPackageV2ReadResult) {
    let mut case = case.clone();
    normalize_relationship(&mut case);
    let mut package = fixture::nominal_package(&[]);
    if !case["operation_bodies"].is_null() {
        package["operation_bodies"] = case["operation_bodies"].clone();
    }
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "node_id": text_type(), "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "scalar_type", "semantic_form": "text", "semantic_type": text_type(),
            "dependencies": [], "occurrences": [{"role": "type", "ordinal": 0}],
            "body": {"term": "aggregate", "members": []},
        }));
    let mut evidence = CheckedPackageEvidence::new();
    for sketch in case["documents"].as_array().expect("sketch documents") {
        let identity = sketch["identity"].as_str().expect("document identity");
        let document = expand_document_with_processes(sketch);
        for ty in sketch["types"].as_array().expect("types") {
            let name = ty["name"].as_str().expect("name");
            let form = ty["form"].as_str().expect("form");
            let member = format!("ix://{identity}/{name}");
            let key = fixture::structural_id(
                identity,
                vec![json!("model"), json!(form), json!(member)],
                0,
            );
            package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
                "node_id":key,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"model",
                "semantic_form":form,"semantic_type":key,"dependencies":[],
                "occurrences":[{"role":"type","ordinal":0}],
                "owner":{"kind":"model","identity":identity,"node":member},
                "body":{"term":"aggregate","members":[]}
            }));
        }
        if sketch["relationships"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty())
        {
            let member = format!("ix://{identity}/relationship/Placed");
            let key = fixture::structural_id(
                identity,
                vec![json!("relation"), json!("relationship"), json!(member)],
                0,
            );
            package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
                "node_id":key,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"relation",
                "semantic_form":"relationship","semantic_type":key,"dependencies":[],
                "occurrences":[{"role":"type","ordinal":0}],
                "owner":{"kind":"model","identity":identity,"node":member},
                "body":{"term":"aggregate","members":[]}
            }));
        }
        fixture::select_model(&mut package, identity);
        evidence
            .insert_domain_package_document(identity, fixture::canonical(&document))
            .expect("one document per identity");
    }
    let referenced = |entry: &Value| {
        case["operation_bodies"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|row| row["body"]["node"] == entry["node"])
    };
    for entry in case["graph_nodes"].as_array().expect("graph nodes") {
        // The owner join refuses a function node whose owner member is not
        // declared by its owner document before any carrier row is read, so
        // such a node is in the package only when a row names it.
        if foreign_owner(entry) && !referenced(entry) {
            continue;
        }
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(graph_node(entry));
    }
    if let Some(candidates) = case["dispatch_candidates"].as_array() {
        let candidates: Vec<Value> = candidates
            .iter()
            .map(|held| {
                let mut held = held.clone();
                if held.get("selection_type").is_none() {
                    held["selection_type"] = held["selected_operation"]["declaring"].clone();
                }
                held
            })
            .collect();
        add_dispatch_table(&mut package, &candidates);
    }
    evidence.support_feature("quire.value.complete/v1");
    fixture::settle(&mut package);
    fixture::index_fixture_node_ids(&mut package);
    let result = crate::read_and_account(
        &fixture::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    (package, evidence, result)
}

fn case(vectors: &Value, id: &str) -> Value {
    let base = vectors["positive"]
        .as_array()
        .expect("positive vectors")
        .iter()
        .find(|row| row["id"] == id)
        .expect("published base");
    let mut result = base.clone();
    result["operation_bodies"] = result["rows"].clone();
    result
}

#[trace("TC-987", "FR-322-AC-397", "FR-038-AC-541", "FR-019-AC-28")]
#[test]
fn tc_987_positive_operation_body_vectors_read_and_round_trip() {
    let vectors = vectors();
    let positives = vectors["positive"].as_array().expect("positive vectors");
    assert_eq!(positives.len(), 6);
    for positive in positives {
        let id = positive["id"].as_str().expect("vector id");
        let input = case(&vectors, id);
        let (_, evidence, result) = read(&input);
        let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
            panic!("{id}: {result:?}");
        };
        let expected: Vec<OperationBodyRow> =
            serde_json::from_value(input["operation_bodies"].clone()).expect("published rows");
        assert_eq!(
            admitted.operation_body_rows(),
            Some(expected.as_slice()),
            "{id}"
        );
        for row in &expected {
            assert_eq!(
                admitted.operation_body(&row.document, &row.declaring, &row.operation),
                Some(&row.body),
                "{id}"
            );
        }
        let encoded = encode_checked_package_v2(
            admitted.name(),
            admitted.lock(),
            admitted.graph(),
            CheckedPackageSourceCorrespondenceRef {
                source_map: admitted.source_map(),
                refinement_step_rows: admitted.refinement_step_rows(),
            },
            CheckedPackageSidecarsRef {
                checked_term_data: admitted.checked_term_data(),
                temporal_bindings: admitted.temporal_bindings(),
                temporal_clause_names: admitted.temporal_clause_names(),
                schedulability_claims: None,
                operation_effects: None,
                operation_bodies: Some(&expected),
            },
            admitted.capability_report(),
            admitted.diagnostics(),
            Limits::new(1 << 24),
        )
        .expect("typed rows encode");
        let CheckedPackageV2ReadResult::Admitted(again) =
            CheckedPackageV2::read(&encoded, CheckedPackageReadLimits::bounded(), &evidence)
        else {
            panic!("{id}: encoded typed rows must admit");
        };
        assert_eq!(
            again.operation_body_rows(),
            Some(expected.as_slice()),
            "{id}"
        );
    }
}

#[trace("TC-987", "FR-322-AC-397", "FR-038-AC-541", "FR-019-AC-28")]
#[test]
fn tc_987_published_refusal_vectors_refuse_with_the_recorded_code_and_locus() {
    let vectors = vectors();
    let mut seen = BTreeSet::new();
    for (group, count) in [("schema_refusals", 13), ("reader_refusals", 21)] {
        let rows = vectors[group].as_array().expect("refusal vectors");
        assert_eq!(rows.len(), count, "{group}");
        for row in rows {
            let id = row["id"].as_str().expect("vector id");
            assert!(seen.insert(id), "duplicate vector {id}");
            let mut input = case(&vectors, row["base"].as_str().expect("base id"));
            for mutation in row["patch"].as_array().expect("patches") {
                patch(&mut input, mutation);
            }
            if let Some(candidates) = row.get("dispatch_candidates") {
                input["dispatch_candidates"] = candidates.clone();
            }
            let (_, _, actual) = read(&input);
            let CheckedPackageV2ReadResult::Refused(refusal) = actual else {
                panic!("{id}: unexpectedly admitted");
            };
            if id.starts_with("OBR-21") {
                // STD-661: the vector cannot reach its recorded code and locus,
                // because the package's own owner join refuses the node first.
                assert_eq!(code_word(refusal.code), "missing_declaration", "{id}");
                assert_eq!(
                    refusal.cause.map(cause_word),
                    Some("missing-selection"),
                    "{id}"
                );
                let path = refusal.loci[0].path.as_ref().expect("path").to_string();
                assert!(path.ends_with("/node_id"), "{id}: {path}");
                continue;
            }
            assert_eq!(
                code_word(refusal.code),
                row["expected_code"].as_str().unwrap(),
                "{id}"
            );
            assert_eq!(
                refusal.cause.map(cause_word),
                row["expected_cause"].as_str(),
                "{id}"
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().expect("path").to_string(),
                row["locus"],
                "{id}"
            );
        }
    }
}

#[trace("FR-322-AC-397", "FR-038-AC-541", "FR-019-AC-28")]
#[test]
fn a_package_without_the_member_has_no_operation_bodies() {
    let vectors = vectors();
    let mut input = case(&vectors, "OBC-02-no-body");
    input.as_object_mut().unwrap().remove("operation_bodies");
    let (package, _, result) = read(&input);
    assert!(package.get("operation_bodies").is_none());
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("a package without the member admits: {result:?}");
    };
    assert_eq!(admitted.operation_body_rows(), None);
    let declaring: CheckedNodeId =
        serde_json::from_value(input["rows"][0]["declaring"].clone()).unwrap();
    assert_eq!(
        admitted.operation_body("acme/orders", &declaring, "touch"),
        None
    );
    let none = OperationBody::NoBody;
    assert_eq!(
        none,
        serde_json::from_value(json!({"kind": "no_body"})).unwrap()
    );
}

/// A dispatch table with two candidates for one selected operation, each with
/// its own body node, beside a correct carrier is the intended admitted case.
#[trace("TC-987", "FR-322-AC-397", "FR-038-AC-541")]
#[test]
#[ignore = "STD-661"]
fn two_dispatch_candidates_with_their_own_bodies_admit_beside_the_carrier() {
    let vectors = vectors();
    let mut input = case(&vectors, "OBC-05-dispatch-candidate");
    let order = input["dispatch_candidates"][0]["selected_operation"]["declaring"].clone();
    let mut second = input["dispatch_candidates"][0].clone();
    second["selection_type"] = fixture::structural_id(
        "acme/orders",
        vec![
            json!("model"),
            json!("object_type"),
            json!("ix://acme/orders/Premium"),
        ],
        0,
    );
    second["selected_operation"]["declaring"] = order;
    second["operation_body"] = fixture::structural_id(
        "acme/orders",
        vec![
            json!("function"),
            json!("pure_function"),
            json!("ix://acme/orders/Order/settle/other"),
        ],
        0,
    );
    input["dispatch_candidates"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let (_, _, result) = read(&input);
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
}
