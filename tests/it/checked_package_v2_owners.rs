// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Owner-wire admission and refusal cases built from this crate's public wire vocabulary.

use crate::support::checked_package::{
    application_node_key, bounds_body, canonical, evidence_for, integer_key, node_id,
    refresh_identity, settle, sha256_hex, structural_key, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn source_package_form(tag: &str, form: &str, name: &str) -> Value {
    let mut package = v2_nominal();
    let source = package["lock"]["sources"][0].clone();
    let owner = json!({
        "kind": "source", "authority": source["authority"], "identity": source["identity"],
    });
    let preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": tag,
        "semantic_form": form, "semantic_type": null,
        "declaration": {"qualified_name": ["Example", name]},
        "recursion": null, "owner": owner,
        "body": {"term": "aggregate", "members": []},
    });
    let id = node_id(&sha256_hex(&canonical(&preimage)));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": id,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag, "semantic_form": form,
            "semantic_type": id, "dependencies": [],
            "occurrences": [{"role": "declaration", "ordinal": 0}],
            "declaration": {"qualified_name": ["Example", name]},
            "owner": owner,
            "body": {"term": "aggregate", "members": []},
        }));
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({
            "node_id": id, "role": "declaration", "ordinal": 0,
            "regions": [{"source": source, "start": 9, "end": 10}],
        }));
    refresh_identity(&mut package);
    package
}

fn source_package() -> Value {
    source_package_form("composite_type", "record", "Point")
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn model_document() -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": "acme/owners", "version": "1.0.0"},
        "constructs": [{
            "kind": {"module": "acme/owners", "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }, {
            "kind": {"module": "acme/owners", "name": "population"},
            "construct": {"meaning": "quire.meaning.model.population/v1"},
        }],
        "types": [{
            "identity": "ix://acme/owners/Point", "displayName": "Point",
            "kind": {"module": "acme/owners", "name": "entity"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "supertypes": [], "fields": [], "operations": [],
        }],
    })
}

fn model_package() -> (Value, Value) {
    let document = model_document();
    let owner = json!({
        "kind": "model", "identity": "acme/owners", "node": "ix://acme/owners/Point",
    });
    let preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": "model",
        "semantic_form": "object_type", "semantic_type": null,
        "declaration": null, "recursion": null, "owner": owner,
        "body": {"term": "aggregate", "members": []},
    });
    let id = node_id(&sha256_hex(&canonical(&preimage)));
    let mut package = v2_nominal();
    package["lock"]["model_selections"] = json!([{
        "identity": "acme/owners", "digest_domain": "sha256-jcs",
        "digest": sha256_hex(&canonical(&document)),
    }]);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": id, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "model", "semantic_form": "object_type",
            "semantic_type": id, "dependencies": [],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "owner": owner, "body": {"term": "aggregate", "members": []},
        }));
    let source = package["lock"]["sources"][0].clone();
    package["source_map"]
        .as_array_mut()
        .expect("map")
        .push(json!({
            "node_id": id, "role": "type", "ordinal": 0,
            "regions": [{"source": source, "start": 10, "end": 11}],
        }));
    refresh_identity(&mut package);
    (package, document)
}

fn read_with_model(package: &Value, document: &Value) -> CheckedPackageV2ReadResult {
    let mut evidence = evidence_for(package);
    evidence
        .insert_domain_package_document(sha256_hex(&canonical(document)), canonical(document))
        .expect("domain evidence declarations agree on exact bytes");
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn declared_population_package_for(population: &str) -> (Value, Value) {
    declared_population_package_for_owner(population, "acme/owners")
}

fn raw_population_record(identity: &str, module: &str, member: &str) -> Value {
    json!({
        "identity": identity,
        "displayName": identity.rsplit('/').next().expect("population name"),
        "kind": {"module": module, "name": "population"},
        "members": [member],
        "extent": "closed",
        "origin": {"source": {
            "path": "spec/populations.md",
            "sourceIdentity": format!("ix://{module}/spec"),
            "startColumn": 1,
            "startLine": 1,
        }},
    })
}

fn declared_population_package_for_owner(population: &str, owner_identity: &str) -> (Value, Value) {
    let (mut package, mut document) = model_package();
    document["populations"] = json!([raw_population_record(
        population,
        "acme/owners",
        "ix://acme/owners/Point",
    )]);
    package["lock"]["model_selections"][0]["digest"] = json!(sha256_hex(&canonical(&document)));

    let node = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .last_mut()
        .expect("declaration node");
    node["node_tag"] = json!("relation");
    node["semantic_form"] = json!("population");
    node["owner"]["identity"] = json!(owner_identity);
    node["owner"]["node"] = json!(population);
    let preimage = json!({
        "version": "quire.structural-node/v1",
        "node_tag": "relation", "semantic_form": "population",
        "semantic_type": null, "declaration": null, "recursion": null,
        "owner": node["owner"],
        "body": {"term": "aggregate", "members": []},
    });
    let id = node_id(&sha256_hex(&canonical(&preimage)));
    node["node_id"] = id.clone();
    node["semantic_type"] = id.clone();
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .last_mut()
        .expect("population occurrence")["node_id"] = id;
    refresh_identity(&mut package);
    (package, document)
}

fn declared_population_package() -> (Value, Value) {
    declared_population_package_for("ix://acme/owners/counters")
}

#[trace("TC-229", "FR-038-AC-303", "FR-038-AC-304")]
#[test]
fn tc_229_raw_population_record_joins_by_identity() {
    let (mut package, mut document) = declared_population_package();
    document["populations"] = json!([{
        "identity": "ix://acme/owners/counters",
        "displayName": "counters",
        "kind": {"module": "acme/owners", "name": "population"},
        "members": ["ix://acme/owners/Point"],
        "extent": "closed",
        "origin": {"generated": {
            "generatorIdentity": "ix://acme/owners/spec",
            "generatorVersion": "1.0.0",
            "inputIdentities": ["ix://acme/owners/counters"],
        }},
    }]);
    package["lock"]["model_selections"][0]["digest"] = json!(sha256_hex(&canonical(&document)));
    refresh_identity(&mut package);

    assert!(matches!(
        read_with_model(&package, &document),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-229", "FR-038-AC-303", "FR-038-AC-304")]
#[test]
fn tc_229_population_node_without_identity_refuses_at_selection() {
    let (mut package, mut document) = declared_population_package();
    document["populations"] = json!([{"node": "ix://acme/owners/counters"}]);
    package["lock"]["model_selections"][0]["digest"] = json!(sha256_hex(&canonical(&document)));
    refresh_identity(&mut package);

    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&package, &document) else {
        panic!("a raw population without identity must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        "/lock/model_selections/0"
    );
}

#[trace("TC-229", "FR-038-AC-304")]
#[test]
fn tc_229_population_with_unresolved_construct_refuses() {
    let (mut package, mut document) = declared_population_package();
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .retain(|entry| entry["kind"]["name"] != "population");
    package["lock"]["model_selections"][0]["digest"] = json!(sha256_hex(&canonical(&document)));
    refresh_identity(&mut package);

    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&package, &document) else {
        panic!("a population with an unresolved construct must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
}

#[trace("TC-229", "FR-038-AC-304")]
#[test]
fn tc_229_population_kind_with_wrong_meaning_refuses() {
    let (mut package, mut document) = declared_population_package();
    document["populations"][0]["kind"]["name"] = json!("entity");
    package["lock"]["model_selections"][0]["digest"] = json!(sha256_hex(&canonical(&document)));
    refresh_identity(&mut package);

    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&package, &document) else {
        panic!("a population kind with object-type meaning must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
}

#[trace("TC-229", "FR-038-AC-303", "FR-038-AC-304")]
#[test]
fn tc_229_selected_declared_population_reference_admits() {
    let (package, document) = declared_population_package();
    assert!(matches!(
        read_with_model(&package, &document),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-229", "FR-038-AC-303", "FR-038-AC-304")]
#[test]
fn tc_229_distinct_population_declarations_have_distinct_keys() {
    let (counters, counters_document) = declared_population_package();
    let (archive, archive_document) = declared_population_package_for("ix://acme/owners/archive");
    assert_ne!(
        counters["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["node_id"],
        archive["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["node_id"]
    );
    for (package, document) in [
        (&counters, &counters_document),
        (&archive, &archive_document),
    ] {
        assert!(matches!(
            read_with_model(package, document),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
    }
}

#[trace("TC-229", "FR-038-AC-303", "FR-038-AC-304")]
#[test]
fn tc_229_population_owner_identity_is_joined_independently_of_node() {
    let (base, _) = declared_population_package();
    let (foreign_owner, document) =
        declared_population_package_for_owner("ix://acme/owners/counters", "acme/elsewhere");
    let base_node = base["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .last()
        .unwrap();
    let changed_node = foreign_owner["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .last()
        .unwrap();
    assert_eq!(changed_node["owner"]["node"], base_node["owner"]["node"]);
    assert_ne!(changed_node["node_id"], base_node["node_id"]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&foreign_owner, &document)
    else {
        panic!("unselected owner identity must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!(
            "/semantic_graph/nodes/{}/node_id",
            foreign_owner["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .len()
                - 1
        )
    );
}

#[trace("TC-229", "FR-038-AC-303", "FR-038-AC-304")]
#[test]
fn tc_229_population_reference_requires_owner_and_selected_population() {
    let (base, document) = declared_population_package();
    let position = base["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
    let mut ownerless = base.clone();
    ownerless["semantic_graph"]["nodes"][position]
        .as_object_mut()
        .unwrap()
        .remove("owner");
    refresh_identity(&mut ownerless);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&ownerless, &document)
    else {
        panic!("ownerless population reference must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);

    let mut ownerless_projection = base.clone();
    ownerless_projection["identity_preimage"]["identity_projection"][position]
        .as_object_mut()
        .unwrap()
        .remove("owner");
    ownerless_projection["package_id"]["digest"] = json!(sha256_hex(&canonical(
        &ownerless_projection["identity_preimage"]
    )));
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_model(&ownerless_projection, &document)
    else {
        panic!("ownerless population projection must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);

    let mut wrong_shape = base.clone();
    wrong_shape["semantic_graph"]["nodes"][position]["body"] = json!({
        "term":"aggregate","members":[{"term":"binding","name":"max",
            "value":{"term":"literal","type":base["semantic_graph"]["nodes"][0]["node_id"],
                "value_kind":"integer","value":"2"}}]
    });
    refresh_identity(&mut wrong_shape);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&wrong_shape, &document)
    else {
        panic!("population reference with a maximum in its body must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );

    let mut wrong_dependencies = base.clone();
    wrong_dependencies["semantic_graph"]["nodes"][position]["dependencies"] =
        json!([base["semantic_graph"]["nodes"][0]["node_id"]]);
    refresh_identity(&mut wrong_dependencies);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_model(&wrong_dependencies, &document)
    else {
        panic!("population reference with a dependency must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );

    for (name, node_tag, form, declared) in [
        (
            "missing population",
            "relation",
            "population",
            "ix://acme/owners/missing",
        ),
        (
            "object claimed as population",
            "relation",
            "population",
            "ix://acme/owners/Point",
        ),
        (
            "population claimed as object",
            "model",
            "object_type",
            "ix://acme/owners/counters",
        ),
        (
            "population claimed as relationship",
            "relation",
            "relationship",
            "ix://acme/owners/counters",
        ),
    ] {
        let mut changed = base.clone();
        let node = &mut changed["semantic_graph"]["nodes"][position];
        node["node_tag"] = json!(node_tag);
        node["semantic_form"] = json!(form);
        node["owner"]["node"] = json!(declared);
        refresh_identity(&mut changed);
        let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&changed, &document)
        else {
            panic!("{name} must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MissingDeclaration,
            "{name}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MissingSelection),
            "{name}"
        );
        assert_eq!(
            refusal.path.expect("path").as_str(),
            format!("/semantic_graph/nodes/{position}/node_id"),
            "{name}"
        );
    }
}

#[trace("TC-229", "FR-038-AC-305")]
#[test]
fn tc_229_declared_population_is_query_operand() {
    let mut shared_population = None;
    let mut unbounded_application = None;
    for (lookup, bounded) in [(false, false), (false, true), (true, false)] {
        let (mut package, document) = declared_population_package();
        let population = package["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["node_id"]
            .clone();
        let (object_package, _) = model_package();
        let object_node = object_package["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .clone();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(object_node);
        let point = package["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["semantic_form"] == "object_type")
            .unwrap()["node_id"]
            .clone();
        let reference_body = json!({"term":"aggregate","members":[
        {"term":"reference","target":point.clone()}]});
        let reference = structural_key("composite_type", "reference", None, &reference_body);
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "node_id":node_id(&reference), "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"composite_type", "semantic_form":"reference",
                "semantic_type":node_id(&reference), "dependencies":[],
                "occurrences":[{"role":"type","ordinal":0}], "body":reference_body
            }));
        let set_body = json!({"term":"aggregate","members":[
        {"term":"reference","target":node_id(&reference)}]});
        let set = structural_key("composite_type", "set", None, &set_body);
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "node_id":node_id(&set), "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"composite_type", "semantic_form":"set",
                "semantic_type":node_id(&set), "dependencies":[],
                "occurrences":[{"role":"type","ordinal":0}], "body":set_body
            }));
        let (result, mode, arguments) = if lookup {
            let option_body = json!({"term":"aggregate","members":[
            {"term":"reference","target":node_id(&reference)}]});
            let option = structural_key("composite_type", "option", None, &option_body);
            package["semantic_graph"]["nodes"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "node_id":node_id(&option), "schema_version":"quire.checked-semantic-graph/v2",
                    "node_tag":"composite_type", "semantic_form":"option",
                    "semantic_type":node_id(&option), "dependencies":[],
                    "occurrences":[{"role":"type","ordinal":0}], "body":option_body
                }));
            let identity_body = json!({"term":"reference","target":point.clone()});
            let identity =
                structural_key("expression", "reference", Some(&reference), &identity_body);
            package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
            "node_id":node_id(&identity), "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"expression", "semantic_form":"reference",
            "semantic_type":node_id(&reference), "dependencies":[],
            "occurrences":[{"role":"expression","ordinal":0}], "body":identity_body
        }));
            (
                option,
                json!({"kind":"absence","value":"option"}),
                json!([{"term":"reference","target":population},
                {"term":"reference","target":node_id(&identity)}]),
            )
        } else {
            let result = if bounded {
                let integer = integer_key();
                if !package["semantic_graph"]["nodes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|node| node["node_id"] == node_id(&integer))
                {
                    package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
                            "node_id":node_id(&integer), "schema_version":"quire.checked-semantic-graph/v2",
                            "node_tag":"scalar_type", "semantic_form":"integer",
                            "semantic_type":node_id(&integer), "dependencies":[],
                            "occurrences":[{"role":"type","ordinal":0}],
                            "body":{"term":"aggregate","members":[]}
                        }));
                }
                let body = bounds_body("0", "2");
                let bounded_key =
                    structural_key("bounded_domain", "collection_bounds", Some(&set), &body);
                package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
                        "node_id":node_id(&bounded_key), "schema_version":"quire.checked-semantic-graph/v2",
                        "node_tag":"bounded_domain", "semantic_form":"collection_bounds",
                        "semantic_type":node_id(&set), "dependencies":[node_id(&set)],
                        "occurrences":[{"role":"type","ordinal":0}], "body":body
                    }));
                bounded_key
            } else {
                set
            };
            (
                result,
                Value::Null,
                json!([{"term":"reference","target":population}]),
            )
        };
        let body = json!({"term":"application","operator":"query",
        "operation":{"identity":if lookup {"quire.op.model.lookup"} else {"quire.op.model.all_instances"},
            "laws":[],"mode":mode,
            "member":{"kind":"type_argument","declaration":point},"leaves":[]},
        "result_type":node_id(&result),"arguments":arguments});
        let application = application_node_key("expression", "query", &result, &body);
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "node_id":node_id(&application), "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"expression", "semantic_form":"query",
                "semantic_type":node_id(&result), "dependencies":[],
                "occurrences":[{"role":"expression","ordinal":0}], "body":body
            }));
        settle(&mut package);
        if let Some(shared) = &shared_population {
            assert_eq!(&population, shared, "one declaration keeps its node key");
        } else {
            shared_population = Some(population.clone());
        }
        let application_id = package["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["node_id"]
            .clone();
        if bounded {
            assert_ne!(
                Some(&application_id),
                unbounded_application.as_ref(),
                "a unit maximum changes the application, not the population reference"
            );
        } else if !lookup {
            unbounded_application = Some(application_id);
        }
        assert!(
            matches!(
                read_with_model(&package, &document),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "lookup={lookup}, bounded={bounded}: {:?}",
            read_with_model(&package, &document)
        );
    }
}

#[trace("TC-229", "FR-038-AC-305", "FR-038-AC-307")]
#[test]
fn tc_229_qspec_published_population_binding_fixture_admits() {
    let root = match std::env::var("QUIRE_SPECIFICATION_DIR") {
        Ok(root) if !root.is_empty() => root,
        _ if std::env::var_os("QSPEC_CONFORMANCE").is_none() => return,
        _ => panic!("QUIRE_SPECIFICATION_DIR is required for QSpec conformance"),
    };
    let proposal = std::path::Path::new(&root).join("proposals/checked-package-v2");
    let package: Value = serde_json::from_slice(
        &std::fs::read(proposal.join("fixtures/positive-operation-identities.json"))
            .expect("QSpec positive operation fixture"),
    )
    .expect("QSpec positive operation JSON");
    let document: Value = serde_json::from_slice(
        &std::fs::read(proposal.join("domain-package-acme-orders.json"))
            .expect("QSpec selected domain document"),
    )
    .expect("QSpec selected domain JSON");
    assert!(matches!(
        read_with_model(&package, &document),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let population = nodes
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "census"
        })
        .expect("published population binding");
    let population_id = &population["node_id"];
    for identity in ["quire.op.model.all_instances", "quire.op.model.lookup"] {
        let application = nodes
            .iter()
            .find(|node| node["body"]["operation"]["identity"] == identity)
            .expect("published population query");
        assert_eq!(
            application["body"]["arguments"][0]["target"],
            *population_id
        );
        assert!(application["dependencies"]
            .as_array()
            .unwrap()
            .contains(population_id));
    }
}

#[trace(
    "TC-229",
    "FR-038-AC-304",
    "FR-038-AC-305",
    "FR-038-AC-306",
    "FR-038-AC-307"
)]
#[test]
fn tc_229_qspec_population_reference_cases_match_reader() {
    let root = match std::env::var("QUIRE_SPECIFICATION_DIR") {
        Ok(root) if !root.is_empty() => root,
        _ if std::env::var_os("QSPEC_CONFORMANCE").is_none() => return,
        _ => panic!("QUIRE_SPECIFICATION_DIR is required for QSpec conformance"),
    };
    let proposal = std::path::Path::new(&root).join("proposals/checked-package-v2");
    let read = |name: &str| -> Value {
        serde_json::from_slice(&std::fs::read(proposal.join(name)).expect("QSpec fixture"))
            .expect("QSpec JSON")
    };
    let cases = read("fixtures/adverse.json");
    let rows = cases["population_reference_cases"]
        .as_array()
        .expect("population cases");
    assert_eq!(rows.len(), 13, "QSpec population case inventory");
    let mut admitted = std::collections::BTreeMap::new();
    for case in rows {
        let id = case["id"].as_str().expect("case id");
        assert_eq!(case["base"], "positive-operation-identities", "{id}");
        if case["operation"] == "published" {
            assert_eq!(id, "parameter-operand-admitted");
            continue;
        }
        let mut package = read("fixtures/positive-operation-identities.json");
        let mut document = read("domain-package-acme-orders.json");
        document["constructs"]
            .as_array_mut()
            .expect("constructs")
            .push(json!({
                "kind": {"module": "acme/orders", "name": "population"},
                "construct": {"meaning": "quire.meaning.model.population/v1"},
            }));
        document["populations"] = if case["document_populations"] == "node-only" {
            Value::Array(
                cases["selected_document_populations"]
                    .as_array()
                    .expect("QSpec selected populations")
                    .iter()
                    .map(|entry| json!({"node": entry["identity"]}))
                    .collect(),
            )
        } else {
            cases["selected_document_populations"].clone()
        };
        package["lock"]["model_selections"][0]["digest"] = json!(sha256_hex(&canonical(&document)));
        let owner = case["owner_node"]
            .as_str()
            .map(|node| json!({"kind":"model","identity":"acme/orders","node":node}));
        let preimage = json!({"version":"quire.structural-node/v1",
            "node_tag":"relation","semantic_form":"population","semantic_type":null,
            "declaration":null,"recursion":null,"owner":owner,
            "body":{"term":"aggregate","members":[]}});
        let population = node_id(&sha256_hex(&canonical(&preimage)));
        let mut population_node = json!({
            "node_id":population,"schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"relation","semantic_form":"population",
            "semantic_type":population,"dependencies":[],
            "occurrences":[{"role":"generated","ordinal":0}],
            "body":{"term":"aggregate","members":[]}
        });
        if let Some(owner) = owner {
            population_node["owner"] = owner;
        }
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(population_node);
        let operation = format!(
            "quire.op.{}",
            case["operation"].as_str().expect("operation")
        );
        let application = package["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|node| node["body"]["operation"]["identity"] == operation)
            .unwrap_or_else(|| panic!("{id}: published operation {operation}"));
        if case["member_declaration"] == true {
            application["body"]["operation"]["member"]["declaration"] = population.clone();
        } else if case["member_population"] == true {
            application["body"]["operation"]["member"]["population"] = population.clone();
        } else {
            application["body"]["arguments"][0]["target"] = population.clone();
        }
        if let Some(also) = case["also"].as_str() {
            let also = format!("quire.op.{also}");
            let application = package["semantic_graph"]["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|node| node["body"]["operation"]["identity"] == also)
                .unwrap_or_else(|| panic!("{id}: second published operation {also}"));
            application["body"]["arguments"][0]["target"] = population.clone();
        }
        settle(&mut package);
        let source = package["lock"]["sources"][0].clone();
        package["source_map"] = Value::Array(
            package["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(position, node)| {
                    json!({"node_id":node["node_id"],
                    "role":node["occurrences"][0]["role"],
                    "ordinal":node["occurrences"][0]["ordinal"],
                    "regions":[{"source":source,"start":position,"end":position+1}]})
                })
                .collect(),
        );
        match read_with_model(&package, &document) {
            CheckedPackageV2ReadResult::Admitted(_) if case["outcome"] == "admitted" => {
                let nodes = package["semantic_graph"]["nodes"].as_array().unwrap();
                let application = nodes
                    .iter()
                    .find(|node| node["body"]["operation"]["identity"] == operation)
                    .unwrap();
                assert!(
                    application["dependencies"]
                        .as_array()
                        .unwrap()
                        .contains(&population),
                    "{id}"
                );
                if let Some(also) = case["also"].as_str() {
                    let also = format!("quire.op.{also}");
                    let second = nodes
                        .iter()
                        .find(|node| node["body"]["operation"]["identity"] == also)
                        .expect("second population query");
                    assert_eq!(second["body"]["arguments"][0]["target"], population, "{id}");
                    assert_eq!(
                        nodes
                            .iter()
                            .filter(|node| node["semantic_form"] == "population")
                            .count(),
                        1,
                        "{id}"
                    );
                }
                admitted.insert(
                    id.to_owned(),
                    (
                        population.clone(),
                        application["node_id"].clone(),
                        package["package_id"].clone(),
                    ),
                );
            }
            CheckedPackageV2ReadResult::Refused(refusal) if case["outcome"] != "admitted" => {
                let expected_code = case["expected_code"].as_str().expect("expected code");
                assert_eq!(
                    serde_json::to_value(refusal.code).unwrap(),
                    json!(expected_code),
                    "{id}"
                );
                if let Some(cause) = case["expected_cause"].as_str() {
                    assert_eq!(
                        serde_json::to_value(refusal.cause).unwrap(),
                        json!(cause),
                        "{id}"
                    );
                }
                let nodes = package["semantic_graph"]["nodes"].as_array().unwrap();
                let locus = match id {
                    "population-entry-without-an-identity-refused" => {
                        "/lock/model_selections/0".to_owned()
                    }
                    "population-owned-by-an-object-type"
                    | "population-owned-by-an-undeclared-node" => {
                        format!("/semantic_graph/nodes/{}/node_id", nodes.len() - 1)
                    }
                    "population-without-an-owner" => {
                        format!("/semantic_graph/nodes/{}", nodes.len() - 1)
                    }
                    other => {
                        let position = nodes
                            .iter()
                            .position(|node| node["body"]["operation"]["identity"] == operation)
                            .unwrap_or_else(|| panic!("{other}: application"));
                        let suffix = match other {
                            "population-as-a-relationship-declaration" => {
                                "/body/operation/member/declaration"
                            }
                            "population-as-a-field-declaration"
                            | "population-as-a-collection-operand" => "/body/arguments/0",
                            "population-as-an-operation-declaration" => {
                                "/body/operation/member/name"
                            }
                            "population-as-a-reaches-population-link" => {
                                "/body/operation/member/population"
                            }
                            _ => panic!("{other}: unexpected refusal case"),
                        };
                        format!("/semantic_graph/nodes/{position}{suffix}")
                    }
                };
                assert_eq!(
                    refusal.path.as_ref().map(|path| path.as_str()),
                    Some(locus.as_str()),
                    "{id}"
                );
            }
            outcome => panic!("{id}: unexpected {outcome:?}"),
        }
    }
    let counters = &admitted["counters-all-instances-admitted"];
    let archive = &admitted["archive-all-instances-admitted"];
    assert_ne!(counters.0, archive.0);
    assert_ne!(counters.1, archive.1);
    assert_ne!(counters.2, archive.2);
}

fn owned_clause(mut package: Value, document: &Value, declared: &str, clause: &str) -> Value {
    package["lock"]["model_selections"][0]["digest"] = json!(sha256_hex(&canonical(document)));
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let literal_type = package["semantic_graph"]["nodes"][0]["node_id"].clone();
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["node_tag"] = json!("function");
    node["semantic_form"] = json!("pure_function");
    node["owner"]["node"] = json!(declared);
    let body = json!({"term": "aggregate", "members": [{
        "term": "binding", "name": "clause",
        "value": {"term": "literal", "type": literal_type,
                  "value_kind": "text", "value": clause},
    }]});
    node["body"] = body.clone();
    let preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": "function",
        "semantic_form": "pure_function", "semantic_type": null,
        "declaration": null, "recursion": null,
        "owner": node["owner"], "body": body,
    });
    let id = node_id(&sha256_hex(&canonical(&preimage)));
    node["node_id"] = id.clone();
    node["semantic_type"] = id.clone();
    package["source_map"]
        .as_array_mut()
        .expect("map")
        .last_mut()
        .expect("entry")["node_id"] = id;
    refresh_identity(&mut package);
    package
}

fn refused(package: &Value, code: CheckedPackageRefusalCode, path: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!("owner mutation must refuse");
    };
    assert_eq!(refusal.code, code);
    assert_eq!(refusal.path.expect("refusal path").as_str(), path);
}

#[trace("TC-228", "FR-038-AC-153")]
#[test]
fn tc_228_declared_record_tuple_and_function_carry_their_source_owner() {
    for (tag, form, name) in [
        ("composite_type", "record", "Point"),
        ("composite_type", "tuple", "Pair"),
        ("function", "pure_function", "compute"),
    ] {
        let package = source_package_form(tag, form, name);
        let node = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .last()
            .expect("record");
        let projection = package["identity_preimage"]["identity_projection"]
            .as_array()
            .expect("projection")
            .last()
            .expect("record projection");
        assert_eq!(node["owner"], projection["owner"]);
        assert!(matches!(
            read(&package),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
    }
}

#[trace("TC-228", "FR-038-AC-153")]
#[test]
fn tc_228_anonymous_locus_nominal_and_application_nodes_are_owner_free() {
    let package = v2_all_families();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let projection = package["identity_preimage"]["identity_projection"]
        .as_array()
        .expect("projection");
    for (name, find) in [
        ("anonymous type", ("scalar_type", "boolean")),
        ("source locus", ("correspondence", "source_locus")),
    ] {
        let position = nodes
            .iter()
            .position(|node| node["node_tag"] == find.0 && node["semantic_form"] == find.1)
            .expect(name);
        assert!(nodes[position].get("owner").is_none(), "{name}");
        assert!(projection[position].get("owner").is_none(), "{name}");
    }
    let nominal_package = v2_nominal();
    let nominal_nodes = nominal_package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nominal nodes");
    let nominal_projection = nominal_package["identity_preimage"]["identity_projection"]
        .as_array()
        .expect("nominal projection");
    let nominal = nominal_nodes
        .iter()
        .position(|node| node.get("nominal_identity_preimage").is_some())
        .expect("nominal node");
    assert!(nominal_nodes[nominal].get("owner").is_none());
    assert!(nominal_projection[nominal].get("owner").is_none());
    let application = nodes
        .iter()
        .position(|node| node["body"]["term"] == "application")
        .expect("application-keyed node");
    assert!(nodes[application].get("owner").is_none());
    assert!(projection[application].get("owner").is_none());
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert!(matches!(
        read(&nominal_package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-228", "FR-038-AC-154")]
#[test]
fn tc_228_required_and_forbidden_owner_shapes_refuse_before_identity() {
    let base = source_package();
    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let node_path = format!("/semantic_graph/nodes/{position}");
    let projection_path = format!("/identity_preimage/identity_projection/{position}");

    let mut missing_node = base.clone();
    missing_node["semantic_graph"]["nodes"][position]
        .as_object_mut()
        .expect("record")
        .remove("owner");
    refused(
        &missing_node,
        CheckedPackageRefusalCode::MalformedWire,
        &node_path,
    );

    let mut missing_projection = base.clone();
    missing_projection["identity_preimage"]["identity_projection"][position]
        .as_object_mut()
        .expect("projection")
        .remove("owner");
    refused(
        &missing_projection,
        CheckedPackageRefusalCode::MalformedWire,
        &projection_path,
    );

    let mut null_owner = base.clone();
    null_owner["semantic_graph"]["nodes"][position]["owner"] = Value::Null;
    refused(
        &null_owner,
        CheckedPackageRefusalCode::MalformedWire,
        &format!("{node_path}/owner"),
    );

    let mut wrong_kind = base.clone();
    wrong_kind["semantic_graph"]["nodes"][position]["owner"] =
        json!({"kind": "model", "identity": "acme/orders", "node": "Order"});
    refused(
        &wrong_kind,
        CheckedPackageRefusalCode::MalformedWire,
        &format!("{node_path}/owner"),
    );

    let mut forbidden = base.clone();
    forbidden["semantic_graph"]["nodes"][0]["owner"] =
        json!({"kind": "source", "authority": "test", "identity": "test"});
    refused(
        &forbidden,
        CheckedPackageRefusalCode::MalformedWire,
        "/semantic_graph/nodes/0/owner",
    );

    let mut forbidden_projection = base.clone();
    forbidden_projection["identity_preimage"]["identity_projection"][0]["owner"] =
        json!({"kind": "source", "authority": "test", "identity": "test"});
    refused(
        &forbidden_projection,
        CheckedPackageRefusalCode::MalformedWire,
        "/identity_preimage/identity_projection/0/owner",
    );

    let mut different = base;
    different["identity_preimage"]["identity_projection"][position]["owner"]["identity"] =
        json!("other");
    refresh_package_id(&mut different);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&different) else {
        panic!("different projection source owner must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("{projection_path}/owner")
    );
}

fn refresh_package_id(package: &mut Value) {
    package["package_id"]["digest"] = json!(crate::support::checked_package::sha256_hex(
        &canonical(&package["identity_preimage"])
    ));
}

#[trace("TC-228", "FR-038-AC-154")]
#[test]
fn tc_228_projection_model_owner_difference_refuses_at_owner_before_join() {
    let mut package = v2_all_families();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "relation" && node["semantic_form"] == "relationship")
        .expect("relationship node");
    package["identity_preimage"]["identity_projection"][position]["owner"]["node"] =
        json!("ix://acme/orders/Order");
    refresh_package_id(&mut package);

    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("different projection model owner must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/identity_preimage/identity_projection/{position}/owner")
    );
}

#[trace("TC-228", "FR-038-AC-155")]
#[test]
fn tc_228_source_owner_joins_selected_source_and_declaration_region() {
    let base = source_package();
    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let mut unselected = base.clone();
    unselected["semantic_graph"]["nodes"][position]["owner"]["identity"] = json!("other");
    refresh_identity(&mut unselected);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&unselected) else {
        panic!("unselected source must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{position}/node_id")
    );

    let mut wrong_region = base;
    let second_source = wrong_region["lock"]["sources"][1].clone();
    wrong_region["source_map"]
        .as_array_mut()
        .expect("map")
        .last_mut()
        .expect("entry")["regions"][0]["source"] = second_source;
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_region) else {
        panic!("a region from another selected source must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    let source_entry = wrong_region["source_map"].as_array().expect("map").len() - 1;
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/source_map/{source_entry}")
    );
}

#[trace("TC-228", "FR-038-AC-153", "FR-038-AC-155")]
#[test]
fn tc_228_an_unreachable_model_node_joins_its_declaring_object_type() {
    let (base, document) = model_package();
    assert!(matches!(
        read_with_model(&base, &document),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    for (name, identity, node) in [
        ("unselected package", "acme/other", "ix://acme/owners/Point"),
        ("unresolved node", "acme/owners", "ix://acme/owners/Missing"),
    ] {
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][position]["owner"]["identity"] = json!(identity);
        changed["semantic_graph"]["nodes"][position]["owner"]["node"] = json!(node);
        refresh_identity(&mut changed);
        let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&changed, &document)
        else {
            panic!("{name} must refuse even when nothing reaches the node");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MissingDeclaration,
            "{name}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MissingSelection),
            "{name}"
        );
        assert_eq!(
            refusal.path.expect("path").as_str(),
            format!("/semantic_graph/nodes/{position}/node_id")
        );
    }

    let mut relation_document = document.clone();
    let relationship = "ix://acme/owners/relationship/Point-related-Point";
    relation_document["types"][0]["relationships"] = json!([{
        "identity": relationship,
        "category": "structural", "composite": false, "direction": "source-to-target",
        "sourceEnd": {
            "type": "ix://acme/owners/Point", "role": "related",
            "multiplicity": {"lower": 0, "upper": 1, "ordered": false, "unique": true},
        },
        "targetEnd": {
            "type": "ix://acme/owners/Point",
            "multiplicity": {"lower": 0, "upper": 1, "ordered": false, "unique": true},
        },
        "origin": {"source": {
            "sourceIdentity": "ix://acme/owners/Point", "path": "models/Point.md",
            "startLine": 1, "startColumn": 1,
        }},
    }]);
    let mut wrong_kind = base.clone();
    wrong_kind["lock"]["model_selections"][0]["digest"] =
        json!(sha256_hex(&canonical(&relation_document)));
    wrong_kind["semantic_graph"]["nodes"][position]["owner"]["node"] = json!(relationship);
    refresh_identity(&mut wrong_kind);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_model(&wrong_kind, &relation_document)
    else {
        panic!("a relationship cannot back an object type");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{position}/node_id")
    );

    let mut interface_document = document.clone();
    interface_document["constructs"][0]["construct"]["meaning"] =
        json!("quire.meaning.systems.interface/v1");
    interface_document["types"][0]["interfaceFeatures"] = json!({});
    let mut wrong_subtype = base.clone();
    wrong_subtype["lock"]["model_selections"][0]["digest"] =
        json!(sha256_hex(&canonical(&interface_document)));
    refresh_identity(&mut wrong_subtype);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_model(&wrong_subtype, &interface_document)
    else {
        panic!("an interface cannot back model/object_type");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{position}/node_id")
    );

    let mut missing_interface = base.clone();
    missing_interface["semantic_graph"]["nodes"][position]["semantic_form"] =
        json!("systems_interface");
    refresh_identity(&mut missing_interface);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_model(&missing_interface, &document)
    else {
        panic!("an ordinary object cannot back model/systems_interface");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{position}/node_id")
    );
}

#[trace("TC-228", "FR-038-AC-153", "FR-038-AC-155")]
#[test]
fn tc_228_operation_and_invariant_clause_owners_join_only_their_kind() {
    let (base, mut document) = model_package();
    let object = "ix://acme/owners/Point";
    let operation = format!("{object}/op");
    let absent_operation = format!("{object}/absent");
    let field = format!("{object}/field");
    document["types"][0]["operations"] = json!([{"identity": operation, "params": []}]);
    document["types"][0]["fields"] = json!([{
        "identity": field, "name": "field", "typeRef": "ix://quire/native/Integer",
        "presence": "required", "nullable": false, "defaultKind": "none",
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
    }]);
    for (name, declared, clause, expected_admission) in [
        ("operation clause", operation.as_str(), "precondition", true),
        ("body clause", operation.as_str(), "body", true),
        ("invariant clause", object, "invariant", true),
        (
            "absent operation member on an unreachable clause",
            absent_operation.as_str(),
            "precondition",
            false,
        ),
        (
            "field cannot back a clause",
            field.as_str(),
            "precondition",
            false,
        ),
        (
            "object cannot back an operation clause",
            object,
            "precondition",
            false,
        ),
        (
            "operation cannot back an invariant",
            operation.as_str(),
            "invariant",
            false,
        ),
    ] {
        let package = owned_clause(base.clone(), &document, declared, clause);
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len()
            - 1;
        match read_with_model(&package, &document) {
            CheckedPackageV2ReadResult::Admitted(_) if expected_admission => {}
            CheckedPackageV2ReadResult::Refused(refusal) if !expected_admission => {
                assert_eq!(
                    refusal.code,
                    CheckedPackageRefusalCode::MissingDeclaration,
                    "{name}"
                );
                assert_eq!(
                    refusal.cause,
                    Some(CheckedPackageRefusalCause::MissingSelection),
                    "{name}"
                );
                assert_eq!(
                    refusal.path.expect("path").as_str(),
                    format!("/semantic_graph/nodes/{position}/node_id"),
                    "{name}"
                );
            }
            other => panic!("{name}: unexpected {other:?}"),
        }
    }
}

/// An owner-bearing function's body participates in its FR-092 key.
#[trace("TC-228", "FR-038-AC-148")]
#[test]
fn tc_228_owner_bearing_function_refuses_a_stale_body_key() {
    let (base, mut document) = model_package();
    let operation = "ix://acme/owners/Point/op";
    document["types"][0]["operations"] = json!([{"identity": operation, "params": []}]);
    let mut package = owned_clause(base, &document, operation, "precondition");
    assert!(matches!(
        read_with_model(&package, &document),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["members"][0]["value"]["value"] = json!("body");
    let would_be_key = sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "function",
        "semantic_form": "pure_function", "semantic_type": null,
        "declaration": null, "recursion": null,
        "owner": node["owner"], "body": node["body"],
    })));
    assert_ne!(node["node_id"]["digest"], would_be_key);
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&package, &document) else {
        panic!("changed owner-bearing function must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/node_id").as_str())
    );
    assert_eq!(
        refusal
            .expected_node_id()
            .expect("derived key")
            .digest
            .as_ref(),
        would_be_key
    );
}

#[trace("TC-228", "FR-038-AC-155")]
#[test]
fn tc_228_one_model_owner_lookup_consumes_one_validation_visit() {
    let (package, document) = model_package();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let owner_path = format!("/semantic_graph/nodes/{position}/node_id");
    let mut evidence = evidence_for(&package);
    evidence
        .insert_domain_package_document(sha256_hex(&canonical(&document)), canonical(&document))
        .expect("domain evidence declarations agree on exact bytes");
    let bytes = canonical(&package);
    for work in 0..4096 {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        let outcome = CheckedPackageV2::read(&bytes, limits, &evidence);
        let CheckedPackageV2ReadResult::Incomplete(incomplete) = outcome else {
            continue;
        };
        if incomplete
            .path
            .as_ref()
            .is_some_and(|path| path.as_str() == owner_path)
        {
            assert_eq!(incomplete.consumed, work + 1);
            limits.work = work + 1;
            let next = CheckedPackageV2::read(&bytes, limits, &evidence);
            // The owner join uses this charge. FR-038-AC-150 subsequently
            // charges the structural key at the same node_id pointer.
            let CheckedPackageV2ReadResult::Incomplete(key_charge) = next else {
                panic!("the structural-key visit follows the owner lookup");
            };
            assert_eq!(
                key_charge.path.as_ref().map(|path| path.as_str()),
                Some(owner_path.as_str())
            );
            assert_eq!(key_charge.consumed, work + 2);
            return;
        }
    }
    panic!("the bounded reader did not expose the owner lookup charge");
}

#[trace("TC-228", "FR-038-AC-155")]
#[test]
fn tc_228_owner_defects_use_digest_order_ahead_of_a_lower_stale_key() {
    let mut package = v2_all_families();
    let application = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .enumerate()
        .filter(|(_, node)| node["body"]["term"] == "application")
        .min_by_key(|(_, node)| {
            node["node_id"]["digest"]
                .as_str()
                .expect("digest")
                .to_owned()
        })
        .map(|(position, _)| position)
        .expect("application-keyed node");
    let stale_key = package["semantic_graph"]["nodes"][application]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let operation = package["semantic_graph"]["nodes"][application]["body"]["operation"]
        ["identity"]
        .as_str()
        .expect("operation")
        .to_owned();
    package["semantic_graph"]["nodes"][application]["body"]["operation"]["identity"] =
        json!(format!("{operation}.changed"));
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(stale_refusal) = read(&package) else {
        panic!("the changed application preimage must have a stale key");
    };
    assert_eq!(
        stale_refusal.code,
        CheckedPackageRefusalCode::InvalidPackage
    );
    assert_eq!(
        stale_refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );
    assert_eq!(
        stale_refusal.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{application}/node_id")
    );

    let mut owners = Vec::new();
    for index in 0..64 {
        let form = if index % 2 == 0 { "record" } else { "tuple" };
        let fixture = source_package_form("composite_type", form, &format!("Owner{index}"));
        let node = fixture["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .last()
            .expect("owner node")
            .clone();
        let digest = node["node_id"]["digest"].as_str().expect("digest");
        if digest > stale_key.as_str() {
            let entry = fixture["source_map"]
                .as_array()
                .expect("map")
                .last()
                .expect("entry")
                .clone();
            owners.push((node, entry));
            if owners.len() == 2 {
                break;
            }
        }
    }
    assert_eq!(
        owners.len(),
        2,
        "two owner keys above the stale application key"
    );
    owners.sort_by(|left, right| {
        right.0["node_id"]["digest"]
            .as_str()
            .cmp(&left.0["node_id"]["digest"].as_str())
    });
    for (node, entry) in owners {
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
        package["source_map"]
            .as_array_mut()
            .expect("map")
            .push(entry);
    }
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let positions = [nodes.len() - 2, nodes.len() - 1];
    for position in positions {
        nodes[position]["owner"]["identity"] = json!("unselected");
    }
    let first = positions
        .into_iter()
        .min_by_key(|position| {
            nodes[*position]["node_id"]["digest"]
                .as_str()
                .expect("digest")
                .to_owned()
        })
        .expect("owner positions");
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("the first unselected source owner must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{first}/node_id")
    );
}

#[trace("TC-228", "FR-038-AC-154")]
#[test]
fn tc_228_model_owner_is_closed_and_required_in_both_wire_positions() {
    let (base, _) = model_package();
    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    for projection in [false, true] {
        let prefix = if projection {
            "/identity_preimage/identity_projection"
        } else {
            "/semantic_graph/nodes"
        };
        let mut absent = base.clone();
        let node = if projection {
            &mut absent["identity_preimage"]["identity_projection"][position]
        } else {
            &mut absent["semantic_graph"]["nodes"][position]
        };
        node.as_object_mut().expect("node").remove("owner");
        refused(
            &absent,
            CheckedPackageRefusalCode::MalformedWire,
            &format!("{prefix}/{position}"),
        );

        let mut versioned = base.clone();
        let owner = if projection {
            &mut versioned["identity_preimage"]["identity_projection"][position]["owner"]
        } else {
            &mut versioned["semantic_graph"]["nodes"][position]["owner"]
        };
        owner["version"] = json!("1.0.0");
        refused(
            &versioned,
            CheckedPackageRefusalCode::MalformedWire,
            &format!("{prefix}/{position}/owner"),
        );
    }
}
