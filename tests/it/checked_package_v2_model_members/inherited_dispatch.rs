// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Strict reader regression over the actual multi-source QSL D10 emission.
use super::*;

const PACKAGE: &str = "example/config-version";
const HISTORY: &str = "ix://example/config-version/config_history";

fn d10_model_document() -> Value {
    fn query(name: &str) -> Value {
        let identity = format!("ix://{PACKAGE}/ConfigVersion/{name}");
        json!({
            "identity": identity,
            "name": name,
            "params": [],
            "returns": {
                "typeRef": "ix://quire/native/Integer",
                "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
                "nullable": false,
            },
            "pre": [], "post": [],
            "origin": {"source": {"sourceIdentity": format!("ix://{PACKAGE}/spec"), "path": "spec.qspec", "startLine": 1, "startColumn": 1}},
            "frame": {"modifies": [], "creates": [], "deletes": []},
        })
    }

    let mut envelope: Value = serde_json::from_str(include_str!(
        "../../fixtures/inherited-dispatch/config-version-model.json"
    ))
    .expect("domain model fixture parses");
    let types = envelope["types"].as_array_mut().expect("types");
    let config = types
        .iter_mut()
        .find(|node| node["identity"] == format!("ix://{PACKAGE}/ConfigVersion"))
        .expect("ConfigVersion type");
    config["operations"] = json!([query("size"), query("other")]);
    let mut sub_size = query("size");
    sub_size["identity"] = json!(format!("ix://{PACKAGE}/Sub/size"));
    sub_size["redefines"] = json!(format!("ix://{PACKAGE}/ConfigVersion/size"));
    let mut probe = query("probe");
    probe["identity"] = json!(format!("ix://{PACKAGE}/Sub/probe"));
    types.push(json!({
        "identity": format!("ix://{PACKAGE}/Sub"),
        "displayName": format!("ix://{PACKAGE}/Sub"),
        "kind": {"module": PACKAGE, "name": "object_type"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "origin": {"generated": {"generatorIdentity": format!("ix://{PACKAGE}/Sub"), "generatorVersion": "1.0.0", "inputIdentities": [format!("ix://{PACKAGE}/Sub")]}},
        "supertypes": [format!("ix://{PACKAGE}/ConfigVersion")], "fields": [],
        "operations": [sub_size, probe],
    }));
    let history = envelope["populations"]
        .as_array_mut()
        .expect("populations")
        .iter_mut()
        .find(|population| population["identity"] == HISTORY)
        .expect("config history population");
    history["members"]
        .as_array_mut()
        .expect("population members")
        .push(json!(format!("ix://{PACKAGE}/Sub")));
    envelope
}

#[trace("TC-472", "FR-354-AC-13")]
#[test]
fn genuine_qsl_d10_retains_static_context_and_complete_selected_family() {
    let package: Value = serde_json::from_str(include_str!(
        "../../fixtures/inherited-dispatch/d10-package.json"
    ))
    .expect("actual emitted package");
    let document = d10_model_document();
    let evidence = evidence_with_document(&package, PACKAGE, &document);
    let bytes = include_bytes!("../../fixtures/inherited-dispatch/d10-package.json");
    let result = CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), &evidence);
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("authentic D10 must admit: {result:?}");
    };
    // The converted fixture is in slot form; the reader's ids are index form.
    let mut resolved = package.clone();
    crate::support::checked_package::resolve_operands(&mut resolved);
    let expected = &resolved["checked_term_data"]["dispatch_tables"][0];
    let application: CheckedNodeId =
        serde_json::from_value(expected["application"].clone()).unwrap();
    let occurrence = serde_json::from_value(expected["occurrence"].clone()).unwrap();
    let view = admitted
        .dispatch_candidates(&application, &occurrence, 3)
        .expect("full family");
    assert_eq!(serde_json::to_value(view.table).unwrap(), *expected);
    assert_eq!(
        serde_json::to_value(&view.table.context.path)
            .unwrap()
            .as_array()
            .unwrap()
            .last(),
        Some(&json!(format!("ix://{PACKAGE}/Sub")))
    );
    assert_ne!(view.table.context, view.table.operation.declaring);
    assert_eq!(view.candidates().len(), 2);
    assert_eq!(
        view.candidates()[0].selection_key.original.node.as_ref(),
        format!("ix://{PACKAGE}/ConfigVersion")
    );
    let call = admitted
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| node.node_id == application)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&call.body).unwrap()["operation"]["member"]["declaration"],
        serde_json::to_value(&view.table.context).unwrap()
    );
}

fn family_document() -> Value {
    let mut document = domain_document(json!({}));
    let mut subtype = document["types"][0].clone();
    subtype["identity"] = json!(PRIORITY_ORDER);
    subtype["displayName"] = json!("PriorityOrder");
    subtype["supertypes"] = json!([ORDER]);
    subtype["operations"][0]["identity"] = json!(format!("{PRIORITY_ORDER}/total"));
    subtype["operations"][0]["redefines"] = json!(format!("{ORDER}/total"));
    let mut unrelated = document["types"][0].clone();
    unrelated["identity"] = json!(CUSTOMER);
    unrelated["displayName"] = json!("Customer");
    unrelated["operations"][0]["identity"] = json!(format!("{CUSTOMER}/total"));
    document["types"]
        .as_array_mut()
        .unwrap()
        .extend([subtype, unrelated]);
    document
}

fn family_package(document: &Value, context: &str) -> Value {
    let mut package = package_selecting_form("total", "object_type", context);
    for declaration in document["types"].as_array().unwrap() {
        let identity = declaration["identity"].as_str().unwrap();
        if identity == context {
            continue;
        }
        let key = model_key("object_type", IDENTITY, identity);
        let mut node = wire_node(&key, "model", "object_type", &key, &[], empty());
        node["owner"] = json!({"kind":"model", "identity":IDENTITY, "node":identity});
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(node);
    }
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let body_a = add_dispatch_body_named(&mut package, "qualified:body_a", "body_a", "1");
    let body_b = add_dispatch_body_named(&mut package, "qualified:body_b", "body_b", "2");
    let call = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["semantic_form"] == "call")
        .unwrap()["node_id"]
        .clone();
    let declaring = node_id(&model_key("object_type", IDENTITY, ORDER));
    let context = node_id(&model_key("object_type", IDENTITY, context));
    let witness = json!({"declaring":declaring, "operation":"total"});
    package["checked_term_data"] = json!({
        "equality_decisions":[], "protocol_locations":[], "probabilistic_claims":[],
        "dispatch_tables":[{
            "application":call, "occurrence":{"role":"type", "ordinal":0},
            "operation":witness, "context":context,
            "candidates":[
                {"selected_operation":witness, "selection_key":{"owner_effective_type":null,"original":{"package":IDENTITY,"node":ORDER}}, "operation_body":body_a,"effective_precondition":null},
                {"selected_operation":witness, "selection_key":{"owner_effective_type":null,"original":{"package":IDENTITY,"node":PRIORITY_ORDER}}, "operation_body":body_b,"effective_precondition":null}
            ]
        }]
    });
    package
}

fn assert_family_refusal(package: &Value, document: &Value, expected: &str) {
    let evidence = evidence_with_document(package, IDENTITY, document);
    let result = read(package, &evidence);
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("expected family refusal, got {result:?}");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(refusal.loci[0].path.as_ref().unwrap().as_str(), expected);
}

#[trace("TC-472", "FR-354-AC-13")]
#[test]
fn direct_and_inherited_operations_retain_complete_producer_rows() {
    for (context, redefine) in [
        (ORDER, true),
        (PRIORITY_ORDER, true),
        (PRIORITY_ORDER, false),
    ] {
        let mut document = family_document();
        if !redefine {
            document["types"][1]["operations"] = json!([]);
        }
        let package = family_package(&document, context);
        let evidence = evidence_with_document(&package, IDENTITY, &document);
        let result = read(&package, &evidence);
        let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
            panic!("{context}, redefine={redefine}: {result:?}");
        };
        let actual = admitted.checked_term_data().unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            package["checked_term_data"]
        );
        assert_eq!(actual.dispatch_tables[0].candidates.len(), 2);
    }
}

#[trace("TC-472", "FR-354-AC-14")]
#[test]
fn unrelated_same_named_selected_witness_refuses() {
    let document = family_document();
    let mut package = family_package(&document, PRIORITY_ORDER);
    package["checked_term_data"]["dispatch_tables"][0]["operation"]["declaring"] =
        node_id(&model_key("object_type", IDENTITY, CUSTOMER));
    assert_family_refusal(
        &package,
        &document,
        "/checked_term_data/dispatch_tables/0/operation",
    );
}

#[trace("TC-472", "FR-354-AC-15")]
#[test]
fn ancestry_and_same_spelling_do_not_replace_the_operation_family() {
    let mut document = family_document();
    // Hide the ancestor's `total` under a differently named redefinition;
    // the subtype's independent `total` is uniquely exposed but another family.
    let mut renamed = document["types"][1]["operations"][0].clone();
    renamed["identity"] = json!(format!("{PRIORITY_ORDER}/other"));
    document["types"][1]["operations"][0]
        .as_object_mut()
        .unwrap()
        .remove("redefines");
    document["types"][1]["operations"]
        .as_array_mut()
        .unwrap()
        .push(renamed);
    let mut package = family_package(&document, PRIORITY_ORDER);
    // Independently prove the same call graph admits without the sidecar.
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    let sidecar = package
        .as_object_mut()
        .unwrap()
        .remove("checked_term_data")
        .unwrap();
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["checked_term_data"] = sidecar;
    assert_family_refusal(
        &package,
        &document,
        "/checked_term_data/dispatch_tables/0/operation",
    );
}

#[trace("TC-472", "FR-354-AC-16")]
#[test]
fn wrong_kind_unrelated_and_cyclic_redefinition_links_refuse() {
    for malformed in ["field", "unrelated", "same-owner", "cycle"] {
        let mut document = family_document();
        let mut hidden = document["types"][0]["operations"][0].clone();
        hidden["identity"] = json!(format!("{ORDER}/hidden"));
        // The public call still resolves `total` uniquely. Its outgoing edge
        // has no bearing on that name's exposure, but cannot prove a family.
        let target = match malformed {
            "field" => {
                document["types"][0]["fields"] = json!([{"identity":format!("{ORDER}/marker"),"typeRef":"ix://quire/native/Integer","multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true},"presence":"required"}]);
                format!("{ORDER}/marker")
            }
            "unrelated" => format!("{CUSTOMER}/total"),
            "same-owner" | "cycle" => {
                if malformed == "cycle" {
                    hidden["redefines"] = json!(format!("{ORDER}/hidden"));
                }
                document["types"][0]["operations"]
                    .as_array_mut()
                    .unwrap()
                    .push(hidden);
                format!("{ORDER}/hidden")
            }
            _ => unreachable!(),
        };
        document["types"][0]["operations"][0]["redefines"] = json!(target);
        let package = family_package(&document, PRIORITY_ORDER);
        assert_family_refusal(
            &package,
            &document,
            "/checked_term_data/dispatch_tables/0/operation",
        );
    }
}

#[trace("TC-472", "FR-354-AC-16")]
#[test]
fn multi_hop_family_charges_exact_reader_work_and_preserves_owner_locus() {
    let mut document = family_document();
    let mut middle = document["types"][1].clone();
    middle["identity"] = json!(PREFERRED_ORDER);
    middle["displayName"] = json!("PreferredOrder");
    middle["operations"][0]["identity"] = json!(format!("{PREFERRED_ORDER}/total"));
    document["types"][1]["supertypes"] = json!([PREFERRED_ORDER]);
    document["types"][1]["operations"][0]["redefines"] = json!(format!("{PREFERRED_ORDER}/total"));
    document["types"].as_array_mut().unwrap().push(middle);
    let package = family_package(&document, PRIORITY_ORDER);
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    let bytes = indexed_bytes(&package);
    let read_at = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        CheckedPackageV2::read(&bytes, limits, &evidence)
    };
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(
        matches!(read_at(high), CheckedPackageV2ReadResult::Admitted(_)),
        "sufficient read: {:?}",
        read_at(high)
    );
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        match read_at(middle) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle,
            other => panic!("expected work boundary: {other:?}"),
        }
    }
    assert!(
        matches!(read_at(high), CheckedPackageV2ReadResult::Admitted(_)),
        "sufficient read: {:?}",
        read_at(high)
    );
    let CheckedPackageV2ReadResult::Incomplete(incomplete) = read_at(high - 1) else {
        panic!("one below must exhaust");
    };
    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
    assert_eq!(incomplete.limit, high - 1);
    assert_eq!(incomplete.consumed, high);
    assert_eq!(
        incomplete.path.as_ref().unwrap().as_str(),
        root_owner(&package)
    );
}

#[trace("TC-472", "FR-354-AC-13")]
#[test]
fn static_context_and_each_candidate_stay_bound_to_the_selected_family() {
    let document = family_document();
    let package = family_package(&document, PRIORITY_ORDER);
    let mut wrong_context = package.clone();
    wrong_context["checked_term_data"]["dispatch_tables"][0]["context"] =
        node_id(&model_key("object_type", IDENTITY, ORDER));
    assert_family_refusal(
        &wrong_context,
        &document,
        "/checked_term_data/dispatch_tables/0/context",
    );
    let mut wrong_key = package.clone();
    wrong_key["checked_term_data"]["dispatch_tables"][0]["candidates"][0]["selection_key"]
        ["original"]["node"] = json!(CUSTOMER);
    assert_family_refusal(
        &wrong_key,
        &document,
        "/checked_term_data/dispatch_tables/0/candidates/0/selection_key",
    );
    let mut wrong_witness = package;
    wrong_witness["checked_term_data"]["dispatch_tables"][0]["candidates"][0]
        ["selected_operation"]["declaring"] =
        node_id(&model_key("object_type", IDENTITY, PRIORITY_ORDER));
    assert_family_refusal(
        &wrong_witness,
        &document,
        "/checked_term_data/dispatch_tables/0/candidates/0/selected_operation",
    );
}

/// The D10 fixture is the QSL #950 emission converted to the FR-322 operand
/// slot form (see the fixture README). Resolving its slots must give back the
/// index-form graph the admitted package holds, node for node.
#[trace("TC-472", "FR-354-AC-13")]
#[test]
fn converted_d10_fixture_resolves_to_the_admitted_index_form_graph() {
    use crate::support::checked_package::{resolve_operands, wire_operands};
    let wire: Value = serde_json::from_str(include_str!(
        "../../fixtures/inherited-dispatch/d10-package.json"
    ))
    .expect("converted package");
    let mut resolved = wire.clone();
    resolve_operands(&mut resolved);
    let mut rewired = resolved.clone();
    wire_operands(&mut rewired);
    assert_eq!(
        rewired, wire,
        "the slot form is the deterministic conversion"
    );
    let document = d10_model_document();
    let evidence = evidence_with_document(&wire, PACKAGE, &document);
    let bytes = include_bytes!("../../fixtures/inherited-dispatch/d10-package.json");
    let result = CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), &evidence);
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("converted D10 must admit: {result:?}");
    };
    let expected: Vec<CheckedNodeId> = resolved["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .map(|node| serde_json::from_value(node["node_id"].clone()).expect("index-form node id"))
        .collect();
    let actual: Vec<_> = admitted
        .resolved_graph()
        .nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect();
    assert_eq!(actual, expected);
}
