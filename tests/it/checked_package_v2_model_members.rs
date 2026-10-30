// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-285: `CheckedPackageV2::read` over a package whose lock selects a real
//! Semantic IR 2.0.0 domain package (non-empty `types`), through the whole
//! chain the reader runs: the document is admitted and read (FR-322 step 1),
//! each declaration's model declaration node key is recovered (step 2), and
//! a `quire.op.model.dispatch_call` on a model-owned operation is resolved
//! and typed (steps 3 and 4) against the graph.
//!
//! The package and the domain package document are built here, node by node,
//! with fixture-authored structural identities; nothing of QSpec is copied in.

use crate::support::checked_package::{
    canonical, evidence_for, evidence_with_document, node_id as fixture_node_id,
    nominal_fixture_members, nominal_package, pointer, rebuild_source_map, refresh_identity,
    refusal_bytes, refusal_cause, select_model, settle, ExpectedRefusal, COMPLETE_VALUE_FEATURE,
    FIXTURE_PACKAGE,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedCollectionKind, CheckedMemberType, CheckedNodeId, CheckedPackageIncomplete,
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedRelationshipEnd, CheckedRelationshipEndError,
};
use serde_json::{json, Value};
use std::sync::LazyLock;

mod reaches_population;

const IDENTITY: &str = "acme/orders";
const VERSION: &str = "1.0.0";
const ORDER: &str = "ix://acme/orders/Order";
const CUSTOMER: &str = "ix://acme/orders/Customer";
const PRIORITY_ORDER: &str = "ix://acme/orders/PriorityOrder";
const PREFERRED_ORDER: &str = "ix://acme/orders/PreferredOrder";
const TIER: &str = "ix://acme/orders/Tier";

/// Stable authored labels for the two native scalars used throughout this fixture.
static INTEGER: LazyLock<String> = LazyLock::new(|| "scalar/integer".to_owned());
static TEXT: LazyLock<String> = LazyLock::new(|| "scalar/text".to_owned());

fn node_id(path: &str) -> Value {
    if let Some(form) = path.strip_prefix("scalar/") {
        return json!({"package": FIXTURE_PACKAGE, "path": ["scalar_type", form], "ordinal": 0});
    }
    if let Some(rest) = path.strip_prefix("model|") {
        let mut parts = rest.splitn(3, '|');
        let form = parts.next().expect("model form");
        let package = parts.next().expect("model package");
        let node = parts.next().expect("model declaration");
        return json!({"package": package, "path": ["model", form, node], "ordinal": 0});
    }
    if let Some(node) = path.strip_prefix("relationship|") {
        return json!({"package": IDENTITY, "path": ["relation", "relationship", node], "ordinal": 0});
    }
    if let Some(target) = path.strip_prefix("reference/") {
        let target = if target.starts_with("model|") {
            node_id(target)
        } else {
            node_id(&model_key("object_type", IDENTITY, target))
        };
        return json!({"package": FIXTURE_PACKAGE, "path": ["composite_type", "reference", target], "ordinal": 0});
    }
    if let Some(rest) = path.strip_prefix("composite/") {
        let (form, declared) = rest
            .split_once('/')
            .expect("composite form and declaration");
        return json!({"package": FIXTURE_PACKAGE, "path": ["composite_type", form, node_id(&model_key("variant_type", IDENTITY, declared))], "ordinal": 0});
    }
    if let Some(target) = path.strip_prefix("option/") {
        return json!({"package": FIXTURE_PACKAGE, "path": ["composite_type", "option", node_id(target)], "ordinal": 0});
    }
    if let Some(rest) = path.strip_prefix("collection/") {
        let (form, element) = rest.split_once('/').expect("collection form and element");
        return json!({"package": FIXTURE_PACKAGE, "path": ["composite_type", form, node_id(element)], "ordinal": 0});
    }
    if let Some(declared) = path.strip_prefix("bounds/ix://") {
        let declared = format!("ix://{declared}");
        let sequence = node_id(&format!("composite/sequence/{declared}"));
        return json!({"package": FIXTURE_PACKAGE, "path": ["bounded_domain", "collection_bounds", "min", node_id("scalar/integer"), "0", "max", "3", sequence], "ordinal": 0});
    }
    if let Some(rest) = path.strip_prefix("bounds/") {
        let (collection_and_lower, upper) = rest.rsplit_once('/').expect("collection upper bound");
        let (collection, lower) = collection_and_lower
            .rsplit_once('/')
            .expect("collection lower bound");
        let (form, element) = collection
            .split_once('/')
            .expect("collection form and element");
        let collection = node_id(&format!("collection/{form}/{element}"));
        return json!({"package": FIXTURE_PACKAGE, "path": ["bounded_domain", "collection_bounds", "min", node_id("scalar/integer"), lower, "max", upper, collection], "ordinal": 0});
    }
    fixture_node_id(path)
}

fn model_key(form: &str, package: &str, node: &str) -> String {
    format!("model|{form}|{package}|{node}")
}

fn dependency_ids(keys: &[&str]) -> Vec<Value> {
    let mut ids = keys.iter().map(|key| node_id(key)).collect::<Vec<_>>();
    ids.sort_by_key(|id| {
        serde_json::from_value::<CheckedNodeId>(id.clone()).expect("dependency identity")
    });
    ids
}

fn wire_node(
    key: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(semantic_type),
        "dependencies": dependency_ids(dependencies),
        "occurrences": [{"role": "type", "ordinal": 0}],
        "body": body,
    })
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn literal(ty: &str, value_kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": node_id(ty), "value_kind": value_kind, "value": value})
}

fn over_body(target: &str) -> Value {
    json!({"term": "aggregate", "members": [reference(target)]})
}

fn bounds_body(lower: &str, upper: &str) -> Value {
    json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "min", "value": literal(&INTEGER, "integer", lower)},
        {"term": "binding", "name": "max", "value": literal(&INTEGER, "integer", upper)},
    ]})
}

fn parameter(name: &str, level: &str, ty: &str) -> (String, Value) {
    let body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value": literal(&TEXT, "text", name)},
        {"term": "binding", "name": "level", "value": literal(&INTEGER, "integer", level)},
    ]});
    let key = format!("parameter/{name}");
    let mut node = wire_node(&key, "value", "parameter", ty, &[], body);
    node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    (key, node)
}

/// One field or operation slot: `typeRef` with the multiplicity `[1, 1]`.
fn slot(identity: Option<&str>, type_ref: &str) -> Value {
    let mut slot = json!({
        "typeRef": type_ref,
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
    });
    if let Some(identity) = identity {
        slot["identity"] = json!(identity);
    }
    slot
}

/// The domain package document: an `Order` with the operation
/// `total(Integer): Integer`, plus `extra` merged into `Order`.
fn domain_document(extra: Value) -> Value {
    let mut order = json!({
        "identity": ORDER, "displayName": "Order",
        "kind": {"module": "acme/orders", "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": [], "fields": [],
        "operations": [{
            "identity": "ix://acme/orders/Order/total",
            "params": [slot(None, "ix://quire/native/Integer")],
            "returns": slot(None, "ix://quire/native/Integer"),
        }],
    });
    if let (Some(order), Some(extra)) = (order.as_object_mut(), extra.as_object()) {
        for (member, value) in extra {
            order.insert(member.clone(), value.clone());
        }
    }
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": IDENTITY, "version": VERSION},
        "constructs": [{
            "kind": {"module": "acme/orders", "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }],
        "types": [order],
    })
}

/// A package selecting `document`, holding `Order`'s model declaration node
/// and `receiver.total(argument)` as a `dispatch_call` of the operation
/// named `name`, plus the evidence that supplies the document.
fn package_over(
    document: &Value,
    name: &str,
) -> (Value, quire_contract_model::CheckedPackageEvidence) {
    let package = package_selecting(name);
    let evidence = evidence_with_document(&package, IDENTITY, document);
    (package, evidence)
}

/// [`package_over`]'s package, whose one `model_root` selects [`IDENTITY`].
fn package_selecting(name: &str) -> Value {
    package_selecting_form(name, "object_type", ORDER)
}

/// The position of the `model_root` of [`IDENTITY`] in `package`.
fn root_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "model_root" && node["owner"]["identity"] == IDENTITY
        })
        .expect("selected root")
}

/// The pointer of the selected root's owner, where document faults are located.
fn root_owner(package: &Value) -> String {
    format!("/semantic_graph/nodes/{}/owner", root_position(package))
}

fn package_selecting_form(name: &str, form: &str, declaring: &str) -> Value {
    let mut package = nominal_package(&[]);

    let order = model_key(form, IDENTITY, declaring);
    let reference_body = json!({"term": "aggregate", "members": [reference(&order)]});
    let reference_type = format!("reference/{order}");
    let (receiver, receiver_node) = parameter("receiver", "0", &reference_type);
    let (argument, argument_node) = parameter("argument", "1", &INTEGER);

    let call_body = json!({
        "term": "application",
        "operator": "call",
        "operation": {
            "identity": "quire.op.model.dispatch_call", "laws": [], "mode": null,
            "member": {"kind": "operation", "declaration": node_id(&order), "name": name},
            "leaves": [],
        },
        "result_type": node_id(&INTEGER),
        "arguments": [reference(&receiver), reference(&argument)],
    });
    let call = "call/dispatch";
    let mut call_dependencies = [receiver.as_str(), argument.as_str(), order.as_str()];
    call_dependencies.sort_unstable();
    let mut declaring_node = wire_node(&order, "model", form, &order, &[], empty());
    declaring_node["owner"] = json!({"kind": "model", "identity": IDENTITY, "node": declaring});
    let nodes = [
        wire_node(&INTEGER, "scalar_type", "integer", &INTEGER, &[], empty()),
        wire_node(&TEXT, "scalar_type", "text", &TEXT, &[], empty()),
        declaring_node,
        wire_node(
            &reference_type,
            "composite_type",
            "reference",
            &reference_type,
            &[order.as_str()],
            reference_body,
        ),
        receiver_node,
        argument_node,
        wire_node(
            call,
            "expression",
            "call",
            &INTEGER,
            &call_dependencies,
            call_body,
        ),
    ];
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes);
    rebuild_source_map(&mut package);
    select_model(&mut package, IDENTITY);
    refresh_identity(&mut package);
    package
}

fn read(
    package: &Value,
    evidence: &quire_contract_model::CheckedPackageEvidence,
) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        evidence,
    )
}

#[trace("TC-048", "FR-038-AC-242")]
#[test]
fn selected_native_parameters_and_results_keep_their_type_nodes() {
    for (native, form) in [
        ("String", "string"),
        ("UUID", "uuid"),
        ("Timestamp", "timestamp"),
    ] {
        let mut document = domain_document(json!({}));
        let type_ref = format!("ix://quire/native/{native}");
        document["types"][0]["fields"] = json!([{
            "identity": format!("{ORDER}/native"), "name": "native", "typeRef": type_ref,
            "presence": "required", "nullable": false, "defaultKind": "none",
            "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
        }]);
        document["types"][0]["operations"][0]["params"] = json!([slot(None, &type_ref)]);
        document["types"][0]["operations"][0]["returns"] = slot(None, &type_ref);
        let (mut package, evidence) = package_over(&document, "total");
        let key = format!("scalar/{form}");
        let (_, argument) = parameter("argument", "1", &key);
        let nodes = package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        nodes[5] = argument;
        nodes[CALL]["body"]["arguments"][1] = reference("parameter/argument");
        nodes[CALL]["body"]["result_type"] = node_id(&key);
        nodes[CALL]["semantic_type"] = node_id(&key);
        nodes.push(wire_node(&key, "scalar_type", form, &key, &[], empty()));
        settle(&mut package);
        let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package, &evidence) else {
            panic!(
                "selected {native} operation refused: {:?}",
                read(&package, &evidence)
            );
        };
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[5].semantic_type).expect("node id"),
            node_id(&key)
        );
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[CALL].semantic_type).expect("node id"),
            node_id(&key)
        );
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[CALL].body).expect("typed body")
                ["result_type"],
            node_id(&key)
        );
        assert_eq!(
            admitted.graph().nodes.last().expect("scalar").semantic_type,
            admitted.graph().nodes.last().expect("scalar").node_id
        );
        for position in [5, CALL, admitted.graph().nodes.len() - 1] {
            let node = &admitted.graph().nodes[position];
            assert!(admitted
                .source_map()
                .iter()
                .any(|entry| { entry.node_id == node.node_id && !entry.regions.is_empty() }));
        }
    }
}

fn refused(
    package: &Value,
    evidence: &quire_contract_model::CheckedPackageEvidence,
) -> CheckedPackageRefusal {
    match read(package, evidence) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected V2 refusal, got {other:?}"),
    }
}

/// IR-1059 slice 1: local owner and selected declaration intake.
/// The published TC-940 vector and full FR-069 matrix remain open.
#[test]
fn selected_tuple_and_payload_variant_declarations_admit() {
    let tuple = format!("ix://{IDENTITY}/Pair");
    let variant = format!("ix://{IDENTITY}/Maybe");
    let mut document = domain_document(json!({}));
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .extend([
            json!({"kind":{"module":IDENTITY,"name":"tuple"},
                "construct":{"meaning":"quire.meaning.model.tuple-value-type/v1"}}),
            json!({"kind":{"module":IDENTITY,"name":"variant"},
                "construct":{"meaning":"quire.meaning.model.variant-type/v1"}}),
        ]);
    document["types"].as_array_mut().expect("types").extend([
        json!({
            "identity":tuple, "displayName":"Pair",
            "kind":{"module":IDENTITY,"name":"tuple"},
            "positions":[
                {"identity":format!("{tuple}/p0"),"typeRef":"ix://quire/native/Boolean"},
                {"identity":format!("{tuple}/p1"),"typeRef":"ix://quire/native/Integer"}
            ]
        }),
        json!({
            "identity":variant, "displayName":"Maybe",
            "kind":{"module":IDENTITY,"name":"variant"},
            "variants":[
                {"identity":format!("{variant}/None"),"name":"None"},
                {"identity":format!("{variant}/Some"),"name":"Some","payloadType":tuple}
            ]
        }),
    ]);
    let (mut package, evidence) = package_over(&document, "total");
    for (name, form) in [(&tuple, "tuple_value_type"), (&variant, "variant_type")] {
        let owner = json!({"kind":"model", "identity":IDENTITY, "node":name});
        let key = model_key(form, IDENTITY, name);
        let mut node = wire_node(&key, "model", form, &key, &[], empty());
        node["owner"] = owner;
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
    }
    settle(&mut package);
    let result = read(&package, &evidence);
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
}

/// IR-1059 slice 1: a selected payload variant is a union in operation typing.
#[test]
fn selected_payload_variant_uses_union_family_in_binary_operations() {
    let mut document = variant_document_named(TIER, &["Gold", "2x"]);
    document["types"][1]["variants"][0]["payloadType"] = json!("ix://quire/native/Integer");
    let (mut package, _, variant_key) = variant_package(&["Gold", "2x"], &[], false);
    let boolean = "scalar/boolean".to_owned();
    let (left, left_node) = parameter("left", "3", &variant_key);
    let (right, right_node) = parameter("right", "4", &variant_key);
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    nodes.extend([
        wire_node(&boolean, "scalar_type", "boolean", &boolean, &[], empty()),
        left_node,
        right_node,
    ]);
    let body = json!({
        "term":"application", "operator":"binary",
        "operation":{"identity":"quire.op.structural.eq", "laws":[], "mode":null,
            "member":null, "leaves":[]},
        "result_type":node_id(&boolean),
        "arguments":[reference(&left), reference(&right)],
    });
    let mut equality = wire_node(
        "binary/structural-eq",
        "expression",
        "binary",
        &boolean,
        &[&left, &right],
        body,
    );
    equality["occurrences"] = json!([{"role":"expression","ordinal":0}]);
    nodes.push(equality);
    settle(&mut package);
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    let result = read(&package, &evidence);
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );

    let equality_index = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "quire.op.structural.eq")
        .expect("structural equality");
    package["semantic_graph"]["nodes"][equality_index]["body"]["operation"]["identity"] =
        json!("quire.op.enum.eq");
    settle(&mut package);
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::IllTyped,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        ),
        "{refusal:?}"
    );
}

/// A selected unordered variant declaration and its exact semantic cases.
fn variant_document_named(declared: &str, cases: &[&str]) -> Value {
    let mut document = domain_document(json!({}));
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": "acme/orders", "name": "variant"},
            "construct": {"meaning": "quire.meaning.model.variant-type/v1"},
        }));
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(json!({
            "identity": declared, "displayName": "Tier",
            "kind": {"module": "acme/orders", "name": "variant"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "variants": cases.iter().map(|case| json!({
                "identity": format!("{declared}/{case}"), "name": case,
            })).collect::<Vec<_>>(),
        }));
    document
}

fn variant_package(
    cases: &[&str],
    emitted: &[&str],
    typed: bool,
) -> (Value, quire_contract_model::CheckedPackageEvidence, String) {
    variant_package_named(TIER, cases, emitted, typed)
}

fn variant_package_named(
    declared: &str,
    cases: &[&str],
    emitted: &[&str],
    typed: bool,
) -> (Value, quire_contract_model::CheckedPackageEvidence, String) {
    let mut document = variant_document_named(declared, cases);
    if typed {
        let order = &mut document["types"][0];
        let one = json!({"lower": 1, "upper": 1, "ordered": false, "unique": true});
        order["fields"] = json!([
            edge_field(ORDER, "tier", declared, "required", one.clone()),
            edge_field(ORDER, "maybeTier", declared, "optional", one),
            edge_field(
                ORDER,
                "tiers",
                declared,
                "required",
                json!({"lower": 0, "upper": 3, "ordered": true, "unique": false})
            ),
        ]);
        order["operations"][0]["params"] = json!([slot(None, declared)]);
        order["operations"][0]["returns"] = slot(None, declared);
    }
    let (mut package, evidence) = package_over(&document, "total");
    let key = model_key("variant_type", IDENTITY, declared);
    let mut variant = wire_node(&key, "model", "variant_type", &key, &[], empty());
    variant["owner"] = json!({"kind": "model", "identity": IDENTITY, "node": declared});
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    if typed {
        let (argument, argument_node) = parameter("argument", "1", &key);
        nodes[5] = argument_node;
        let body = &mut nodes[CALL]["body"];
        body["arguments"][1] = reference(&argument);
        body["result_type"] = node_id(&key);
        let call_key = "call/dispatch";
        let order = model_key("object_type", IDENTITY, ORDER);
        let receiver = "parameter/receiver".to_owned();
        let mut dependencies = [argument.as_str(), order.as_str(), receiver.as_str()];
        dependencies.sort_unstable();
        nodes[CALL]["node_id"] = node_id(call_key);
        nodes[CALL]["semantic_type"] = node_id(&key);
        nodes[CALL]["dependencies"] = json!(dependency_ids(&dependencies));
    }
    nodes.insert(5, variant);
    for case in emitted {
        let member_key = format!("enum-value/{declared}/{case}");
        let body = literal(&key, "enum", case);
        let mut member = wire_node(&member_key, "value", "enum_value", &key, &[&key], body);
        member["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
        nodes.push(member);
    }
    if typed {
        for form in ["option", "sequence"] {
            let body = over_body(&key);
            let composite = format!("composite/{form}/{declared}");
            nodes.push(wire_node(
                &composite,
                "composite_type",
                form,
                &composite,
                &[&key],
                body,
            ));
            if form == "sequence" {
                let bounds = bounds_body("0", "3");
                let bounded = format!("bounds/{declared}");
                nodes.push(wire_node(
                    &bounded,
                    "bounded_domain",
                    "collection_bounds",
                    &composite,
                    &[&composite],
                    bounds,
                ));
            }
        }
    }
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    (package, evidence, key)
}

#[trace("TC-048", "FR-038-AC-237")]
#[test]
fn selected_variant_accepts_exact_utf8_cases() {
    for case in ["Gold", "Gold Plus", "2x", "Café"] {
        let (package, evidence, _) =
            variant_package(&["Gold", "Gold Plus", "2x", "Café"], &[case], false);
        let result = read(&package, &evidence);
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
            "{case}: {result:?}"
        );
    }
}

#[trace("TC-048", "FR-038-AC-237")]
#[test]
fn selected_variant_case_keys_bind_exact_case_and_declaration() {
    let (tier, tier_evidence, _) = variant_package_named(
        TIER,
        &["Gold", "Gold Plus", "2x", "Café"],
        &["Gold", "Gold Plus", "2x", "Café"],
        false,
    );
    assert!(matches!(
        read(&tier, &tier_evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let tier_nodes = tier["semantic_graph"]["nodes"].as_array().expect("nodes");
    let case_keys: std::collections::BTreeSet<_> = tier_nodes[9..13]
        .iter()
        .map(|node| {
            serde_json::from_value::<CheckedNodeId>(node["node_id"].clone()).expect("case identity")
        })
        .collect();
    assert_eq!(case_keys.len(), 4);

    let (rank, rank_evidence, _) =
        variant_package_named("ix://acme/orders/Rank", &["Gold"], &["Gold"], false);
    assert!(matches!(
        read(&rank, &rank_evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert_ne!(
        tier_nodes[9]["node_id"],
        rank["semantic_graph"]["nodes"][9]["node_id"],
    );

    // A selected document's display alias changes its content, but the
    // declaration identity and exact case still determine the member key.
    let mut alias_document = variant_document_named(TIER, &["Gold", "Gold Plus", "2x", "Café"]);
    alias_document["types"][1]["displayName"] = json!("Level");
    let mut alias_package = tier.clone();
    refresh_identity(&mut alias_package);
    let alias_evidence = evidence_with_document(&alias_package, IDENTITY, &alias_document);
    assert!(matches!(
        read(&alias_package, &alias_evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert_eq!(
        tier_nodes[9]["node_id"],
        alias_package["semantic_graph"]["nodes"][9]["node_id"],
    );
}

#[trace("TC-048", "FR-038-AC-236")]
#[test]
fn selected_variant_types_field_parameter_and_result() {
    let (package, evidence, tier) = variant_package(&["Gold"], &["Gold"], true);
    let admitted = match read(&package, &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("expected selected field/parameter/result admission: {other:?}"),
    };
    for form in ["option", "sequence", "collection_bounds"] {
        assert!(
            admitted
                .graph()
                .nodes
                .iter()
                .any(|node| node.semantic_form.as_ref() == form),
            "admitted graph omits {form} over Tier"
        );
    }
    let order = model_key("object_type", IDENTITY, ORDER);
    let fields = admitted
        .model_object_fields(&serde_json::from_value(node_id(&order)).expect("node id"))
        .expect("Order fields");
    assert!(matches!(
        fields.field("tier").and_then(|field| field.member_type()),
        Some(CheckedMemberType::Variant(id)) if serde_json::to_value(id).expect("variant id") == node_id(&tier)
    ));
    assert!(matches!(
        fields.field("maybeTier").and_then(|field| field.member_type()),
        Some(CheckedMemberType::Option(inner))
            if matches!(inner.as_ref(), CheckedMemberType::Variant(id) if serde_json::to_value(id).expect("variant id") == node_id(&tier))
    ));
    assert!(matches!(
        fields.field("tiers").and_then(|field| field.member_type()),
        Some(CheckedMemberType::Collection { kind: CheckedCollectionKind::Sequence, element, bounds: Some((0, 3)) })
            if matches!(element.as_ref(), CheckedMemberType::Variant(id) if serde_json::to_value(id).expect("variant id") == node_id(&tier))
    ));
}

fn variant_binary_package(
    operation: &str,
) -> (Value, quire_contract_model::CheckedPackageEvidence) {
    let (mut package, evidence, _) = variant_package(&["Gold", "2x"], &["Gold", "2x"], false);
    let boolean = "scalar/boolean".to_owned();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let left = format!("enum-value/{TIER}/Gold");
    let right = format!("enum-value/{TIER}/2x");
    nodes.push(wire_node(
        &boolean,
        "scalar_type",
        "boolean",
        &boolean,
        &[],
        empty(),
    ));
    let body = json!({
        "term": "application", "operator": "binary",
        "operation": {"identity": operation, "laws": [], "mode": null, "member": null,
            "leaves": []},
        "result_type": node_id(&boolean),
        "arguments": [reference(&left), reference(&right)],
    });
    let key = "binary/variant-comparison";
    let mut dependencies = [left.as_str(), right.as_str()];
    dependencies.sort_unstable();
    let mut application = wire_node(key, "expression", "binary", &boolean, &dependencies, body);
    application["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    nodes.push(application);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    (package, evidence)
}

#[trace("TC-048", "FR-038-AC-237")]
#[test]
fn selected_variant_admits_enum_equality_and_refuses_order_or_structural_equality() {
    for operation in ["quire.op.enum.eq", "quire.op.enum.ne"] {
        let (package, evidence) = variant_binary_package(operation);
        let result = read(&package, &evidence);
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
            "{operation}: {result:?}"
        );
    }
    for operation in ["quire.op.enum.lt", "quire.op.structural.eq"] {
        let (package, evidence) = variant_binary_package(operation);
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len()
            - 1;
        let refusal = refused(&package, &evidence);
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::IllTyped,
            "{operation}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            "{operation}"
        );
        let expected_path = if operation == "quire.op.structural.eq" {
            format!("/semantic_graph/nodes/{position}")
        } else {
            format!("/semantic_graph/nodes/{position}/body/arguments/0")
        };
        assert_eq!(
            refusal.loci[0].path.clone(),
            Some(pointer(&expected_path)),
            "{operation}"
        );
    }
}

#[trace("TC-048", "FR-038-AC-238")]
#[test]
fn selected_variant_refuses_absent_case_and_empty_declaration() {
    let (package, evidence, _) = variant_package(&["Gold"], &["Silver"], false);
    let refusal = refused(&package, &evidence);
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.loci[0].path.clone(),
        Some(pointer("/semantic_graph/nodes/9"))
    );

    let (package, evidence, _) = variant_package(&[], &[], false);
    let refusal = refused(&package, &evidence);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
    assert_eq!(
        refusal.loci[0].path.clone(),
        Some(pointer(&root_owner(&package)))
    );
}

#[trace("TC-048", "FR-038-AC-236")]
#[test]
fn selected_variant_requires_its_exact_model_owner() {
    let (package, evidence, _) = variant_package(&["Gold"], &["Gold"], false);
    let mut missing = package.clone();
    missing["semantic_graph"]["nodes"][5]
        .as_object_mut()
        .expect("variant node")
        .remove("owner");
    refresh_identity(&mut missing);
    let refusal = refused(&missing, &evidence);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        refusal.loci[0].path.clone(),
        Some(pointer("/semantic_graph/nodes/5"))
    );

    for owner_node in ["ix://acme/orders/Missing", ORDER] {
        let mut changed = package.clone();
        changed["semantic_graph"]["nodes"][5]["owner"]["node"] = json!(owner_node);
        refresh_identity(&mut changed);
        let refusal = refused(&changed, &evidence);
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MissingSelection)
        );
        assert_eq!(
            refusal.loci[0].path.clone(),
            Some(pointer("/semantic_graph/nodes/5/owner"))
        );
    }
}

/// The position of the `dispatch_call` application in [`package_over`]'s graph.
const CALL: usize = 6;

const RELATIONSHIP: &str = "ix://acme/orders/relationship/Order-links-Order";

fn authored_relationship() -> Value {
    json!({
        "identity": RELATIONSHIP, "category": "structural", "composite": false,
        "direction": "bidirectional",
        "sourceEnd": {"type": ORDER, "role": "links", "multiplicity":
            {"lower": 0, "upper": 1, "ordered": false, "unique": true}},
        "targetEnd": {"type": ORDER, "role": "linkedBy", "multiplicity":
            {"lower": 1, "upper": 1, "ordered": false, "unique": true}},
        "origin": {"source": {"sourceIdentity": ORDER, "path": "models/Order.md",
            "startLine": 1, "startColumn": 1}},
    })
}

fn relationship_package(
    relationship: Value,
    role: &str,
) -> (Value, quire_contract_model::CheckedPackageEvidence) {
    relationship_operation_package(relationship, role, false)
}

pub(crate) fn model_member_control(
    identity: &str,
) -> (Value, quire_contract_model::CheckedPackageEvidence) {
    match identity {
        "quire.op.model.navigate" => relationship_package(authored_relationship(), "links"),
        "quire.op.model.dispatch_call" => package_over(&domain_document(json!({})), "total"),
        _ => unreachable!("only the two model member controls are requested"),
    }
}

fn relationship_operation_package(
    relationship: Value,
    role: &str,
    reaches: bool,
) -> (Value, quire_contract_model::CheckedPackageEvidence) {
    let destination = if relationship["sourceEnd"]["role"] == role {
        &relationship["targetEnd"]["type"]
    } else {
        &relationship["sourceEnd"]["type"]
    };
    let destination = destination.as_str().expect("destination type");
    let destination_type = node_id(&model_key("object_type", IDENTITY, destination));
    let document = domain_document(json!({"relationships": [relationship]}));
    let mut package = package_selecting("total");
    let relationship_key = format!("relationship|{RELATIONSHIP}");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let receiver = "parameter/receiver".to_owned();
    let reference_type = format!("reference/{ORDER}");
    let boolean = "scalar/boolean".to_owned();
    let result_type = if reaches { &boolean } else { &reference_type };
    let form = if reaches { "reachability" } else { "query" };
    let operator = if reaches { "reaches" } else { "query" };
    let operation = if reaches {
        "quire.op.model.reaches"
    } else {
        "quire.op.model.navigate"
    };
    let second = reaches.then(|| parameter("other", "1", &reference_type));
    let arguments = if reaches {
        vec![
            reference(&receiver),
            reference(&second.as_ref().expect("second operand").0),
        ]
    } else {
        vec![reference(&receiver)]
    };
    let body = json!({
        "term": "application", "operator": operator,
        "operation": {"identity": operation, "laws": [], "mode": null,
            "member": {"kind": "relationship_end", "declaration": node_id(&relationship_key),
                "name": role, "destination_type": destination_type},
            "leaves": []},
        "result_type": node_id(result_type),
        "arguments": arguments,
    });
    let application_key = format!("application/{form}");
    let mut dependencies = vec![receiver.as_str(), relationship_key.as_str()];
    if let Some((key, _)) = &second {
        dependencies.push(key);
    }
    dependencies.sort_unstable();
    nodes[CALL] = wire_node(
        &application_key,
        "expression",
        form,
        result_type,
        &dependencies,
        body,
    );
    if let Some((_, second_node)) = second {
        nodes.push(second_node);
    }
    let mut relation_node = wire_node(
        &relationship_key,
        "relation",
        "relationship",
        &relationship_key,
        &[],
        empty(),
    );
    relation_node["owner"] = json!({"kind": "model", "identity": IDENTITY, "node": RELATIONSHIP});
    nodes.push(relation_node);
    if reaches {
        nodes.push(wire_node(
            &boolean,
            "scalar_type",
            "boolean",
            &boolean,
            &[],
            empty(),
        ));
    }
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    (package, evidence)
}

#[derive(Clone, Copy)]
enum NavigationResult {
    Reference,
    OrderReference,
    Option,
    Set(u64, u64),
    Bag(u64, u64),
}

/// Appends a `set` or `bag` composite over `element` and its bounded domain
/// to `nodes`, returning the bounded-domain node key a result names.
fn bounded_collection(
    nodes: &mut Vec<Value>,
    element: &str,
    form: &str,
    lower: u64,
    upper: u64,
) -> String {
    let body = over_body(element);
    let collection = format!("collection/{form}/{element}");
    nodes.push(wire_node(
        &collection,
        "composite_type",
        form,
        &collection,
        &[element],
        body,
    ));
    let bounds = bounds_body(&lower.to_string(), &upper.to_string());
    let key = format!("bounds/{form}/{element}/{lower}/{upper}");
    nodes.push(wire_node(
        &key,
        "bounded_domain",
        "collection_bounds",
        &collection,
        &[&collection],
        bounds,
    ));
    key
}

/// A complete selected package whose application names an independently
/// authored result node. The result is supplied separately from the FCD end
/// so a changed destination can reject the formerly admitted result.
fn navigation_with_result(
    relationship: Value,
    result: NavigationResult,
) -> (Value, quire_contract_model::CheckedPackageEvidence) {
    let (mut package, _) = relationship_package(relationship.clone(), "links");
    let mut document = domain_document(json!({"relationships": [relationship.clone()]}));
    let destination = relationship["targetEnd"]["type"]
        .as_str()
        .expect("destination type");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let reference_type = if destination == ORDER {
        format!("reference/{ORDER}")
    } else {
        document["types"]
            .as_array_mut()
            .expect("types")
            .push(json!({
                "identity": destination, "displayName": "Customer",
                "kind": {"module": IDENTITY, "name": "entity"},
                "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
                "supertypes": [], "fields": [], "operations": [],
            }));
        let object = model_key("object_type", IDENTITY, destination);
        let mut declaration = wire_node(&object, "model", "object_type", &object, &[], empty());
        declaration["owner"] = json!({"kind": "model", "identity": IDENTITY, "node": destination});
        nodes.push(declaration);
        let body = over_body(&object);
        let reference = format!("reference/{destination}");
        nodes.push(wire_node(
            &reference,
            "composite_type",
            "reference",
            &reference,
            &[&object],
            body,
        ));
        reference
    };
    let result_type = match result {
        NavigationResult::Reference => reference_type,
        NavigationResult::OrderReference => format!("reference/{ORDER}"),
        NavigationResult::Option => {
            let body = over_body(&reference_type);
            let key = format!("option/{reference_type}");
            nodes.push(wire_node(
                &key,
                "composite_type",
                "option",
                &key,
                &[&reference_type],
                body,
            ));
            key
        }
        NavigationResult::Set(lower, upper) => {
            bounded_collection(nodes, &reference_type, "set", lower, upper)
        }
        NavigationResult::Bag(lower, upper) => {
            bounded_collection(nodes, &reference_type, "bag", lower, upper)
        }
    };
    let call = &mut nodes[CALL];
    call["body"]["result_type"] = node_id(&result_type);
    call["semantic_type"] = node_id(&result_type);
    call["node_id"] = node_id("application/query");
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    (package, evidence)
}

/// Two independently keyed reaches operands, each typed at its authored
/// object declaration. An unrelated second owner must be checked even when
/// operand 0 is eligible.
fn reaches_with_operands(
    first_type: &str,
    second_type: &str,
) -> (Value, quire_contract_model::CheckedPackageEvidence) {
    reaches_over(authored_relationship(), "links", first_type, second_type)
}

fn reaches_over(
    relation: Value,
    role: &str,
    first_type: &str,
    second_type: &str,
) -> (Value, quire_contract_model::CheckedPackageEvidence) {
    let (mut package, _) = relationship_operation_package(relation.clone(), role, true);
    let mut document = domain_document(json!({"relationships": [relation.clone()]}));
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let order_reference = format!("reference/{ORDER}");
    let mut references = vec![(ORDER, order_reference)];
    let mut operand_keys = Vec::new();
    for (position, ty) in [first_type, second_type].into_iter().enumerate() {
        if !references.iter().any(|(known, _)| *known == ty) {
            let supertype = if matches!(ty, PRIORITY_ORDER | PREFERRED_ORDER) {
                vec![ORDER]
            } else {
                vec![]
            };
            document["types"]
                .as_array_mut()
                .expect("types")
                .push(json!({
                    "identity": ty, "displayName": ty.rsplit('/').next().expect("name"),
                    "kind": {"module": IDENTITY, "name": "entity"},
                    "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
                    "supertypes": supertype, "fields": [], "operations": [],
                }));
            let object = model_key("object_type", IDENTITY, ty);
            let mut declaration = wire_node(&object, "model", "object_type", &object, &[], empty());
            declaration["owner"] = json!({"kind": "model", "identity": IDENTITY, "node": ty});
            nodes.push(declaration);
            let body = json!({"term": "aggregate", "members": [reference(&object)]});
            let reference = format!("reference/{ty}");
            nodes.push(wire_node(
                &reference,
                "composite_type",
                "reference",
                &reference,
                &[&object],
                body,
            ));
            references.push((ty, reference));
        }
        let reference_type = references
            .iter()
            .find(|(known, _)| *known == ty)
            .expect("operand reference")
            .1
            .as_str();
        let (key, operand) = parameter(
            &format!("operand{position}"),
            &position.to_string(),
            reference_type,
        );
        nodes.push(operand);
        operand_keys.push(key);
    }
    for end in ["sourceEnd", "targetEnd"] {
        let ty = relation[end]["type"].as_str().expect("end type");
        if !document["types"]
            .as_array()
            .expect("types")
            .iter()
            .any(|object| object["identity"] == ty)
        {
            document["types"]
                .as_array_mut()
                .expect("types")
                .push(json!({
                    "identity": ty, "displayName": ty.rsplit('/').next().expect("name"),
                    "kind": {"module": IDENTITY, "name": "entity"},
                    "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
                    "supertypes": [], "fields": [], "operations": [],
                }));
        }
    }
    let relation_key = format!("relationship|{RELATIONSHIP}");
    nodes[CALL]["body"]["arguments"] =
        json!([reference(&operand_keys[0]), reference(&operand_keys[1])]);
    let mut dependencies = [
        operand_keys[0].as_str(),
        operand_keys[1].as_str(),
        relation_key.as_str(),
    ];
    dependencies.sort_unstable();
    nodes[CALL]["dependencies"] = json!(dependency_ids(&dependencies));
    nodes[CALL]["node_id"] = node_id("application/reachability");
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    (package, evidence)
}

#[trace(
    "TC-048",
    "FR-038-AC-165",
    "FR-038-AC-166",
    "FR-038-AC-169",
    "FR-038-AC-170",
    "FR-038-AC-171",
    "FR-062-AC-3"
)]
#[test]
fn tc_048_a_selected_fcd_relationship_navigates_through_its_named_end() {
    let (package, evidence) = relationship_package(authored_relationship(), "links");
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let (package, evidence) = relationship_package(authored_relationship(), "missing");
    let refusal = refused(&package, &evidence);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/semantic_graph/nodes/6/body/operation/member/name")
    );
    assert_eq!(refusal.terminal_duplicate, None);
    assert_eq!(
        refusal.locus.as_ref(),
        Some(
            &serde_json::from_value(package["semantic_graph"]["nodes"][CALL]["node_id"].clone())
                .expect("application id")
        )
    );
    let mut renamed = authored_relationship();
    renamed["sourceEnd"]["role"] = json!("billedTo");
    let (package, evidence) = relationship_package(renamed.clone(), "links");
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingName)
        )
    );
    let (package, evidence) = relationship_package(renamed.clone(), "billedTo");
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    renamed["targetEnd"]["role"] = json!("billedTo");
    let (package, evidence) = relationship_package(renamed, "billedTo");
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            Some(CheckedPackageRefusalCause::AmbiguousName)
        )
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/semantic_graph/nodes/6/body/operation/member/name")
    );
}

/// The undeclared model-owned relationship retains its exact assigned
/// structural identity after owner validation and reader admission.
#[trace("TC-048", "FR-048-AC-2")]
#[test]
fn tc_048_model_relationship_identity_survives_reader_admission() {
    let (package, evidence) = relationship_package(authored_relationship(), "links");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let position = nodes
        .iter()
        .position(|node| node["node_tag"] == "relation" && node["semantic_form"] == "relationship")
        .expect("relationship");
    let authored = serde_json::from_value::<CheckedNodeId>(nodes[position]["node_id"].clone())
        .expect("authored structural identity");
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package, &evidence) else {
        panic!("model relationship package must admit");
    };
    assert_eq!(admitted.graph().nodes[position].node_id, authored);
}

#[trace("TC-048", "FR-038-AC-170")]
#[test]
fn tc_048_navigation_admits_canonical_finite_results_and_observes_destination_changes() {
    let mut relation = authored_relationship();
    for (lower, upper, unique, result) in [
        (1, 1, true, NavigationResult::Reference),
        (0, 1, true, NavigationResult::Option),
        (2, 3, true, NavigationResult::Set(2, 3)),
        (2, 3, false, NavigationResult::Bag(2, 3)),
    ] {
        relation["targetEnd"]["multiplicity"] = json!({
            "lower": lower, "upper": upper, "ordered": false, "unique": unique,
        });
        let (package, evidence) = navigation_with_result(relation.clone(), result);
        assert!(
            matches!(
                read(&package, &evidence),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "result for {lower}..{upper}, unique={unique}"
        );
    }
    let (package, evidence) = navigation_with_result(relation.clone(), NavigationResult::Set(2, 3));
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::IllTyped,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        )
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/semantic_graph/nodes/6/body/result_type")
    );

    relation["targetEnd"]["multiplicity"]["unique"] = json!(true);
    let (package, evidence) = navigation_with_result(relation.clone(), NavigationResult::Set(1, 3));
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::IllTyped,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        )
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/semantic_graph/nodes/6/body/result_type")
    );

    relation["targetEnd"]["multiplicity"] = json!({
        "lower": 1, "upper": 1, "ordered": false, "unique": true,
    });
    relation["targetEnd"]["type"] = json!(CUSTOMER);
    let (package, evidence) =
        navigation_with_result(relation.clone(), NavigationResult::OrderReference);
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::IllTyped,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        )
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/semantic_graph/nodes/6/body/result_type")
    );
    let (package, evidence) = navigation_with_result(relation, NavigationResult::Reference);
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-048", "FR-038-AC-169", "FR-038-AC-172")]
#[test]
fn tc_048_a_selected_self_relationship_reaches_through_either_named_end() {
    for role in ["links", "linkedBy"] {
        let (package, evidence) =
            relationship_operation_package(authored_relationship(), role, true);
        assert_ne!(
            package["semantic_graph"]["nodes"][CALL]["body"]["arguments"][0],
            package["semantic_graph"]["nodes"][CALL]["body"]["arguments"][1],
            "the two operands name distinct nodes"
        );
        let result = read(&package, &evidence);
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
            "{role}: {result:?}"
        );
    }
    let mut set_edge = authored_relationship();
    set_edge["targetEnd"]["multiplicity"]["lower"] = json!(2);
    set_edge["targetEnd"]["multiplicity"]["upper"] = json!(3);
    let (package, evidence) = relationship_operation_package(set_edge, "links", true);
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let (package, evidence) = reaches_with_operands(PRIORITY_ORDER, PREFERRED_ORDER);
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    for (first, second, expected_path) in [
        (ORDER, CUSTOMER, "/semantic_graph/nodes/6/body/arguments/1"),
        (CUSTOMER, ORDER, "/semantic_graph/nodes/6/body/arguments/0"),
    ] {
        let (package, evidence) = reaches_with_operands(first, second);
        let refusal = refused(&package, &evidence);
        assert_eq!(
            (refusal.code, refusal.cause),
            (
                CheckedPackageRefusalCode::IllTyped,
                Some(CheckedPackageRefusalCause::OperatorIneligible)
            )
        );
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(expected_path)
        );
    }
}

/// The reaches application and its result type node, as the reader admits them.
fn admitted_reaches(
    package: &Value,
    evidence: &quire_contract_model::CheckedPackageEvidence,
) -> (Value, String, usize) {
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(package, evidence) else {
        panic!("expected an admitted reaches package");
    };
    let nodes = &admitted.graph().nodes;
    let call = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.body,
                quire_contract_model::CheckedTerm::Application {
                    operator: quire_contract_model::ApplicationOperator::Reaches,
                    ..
                }
            )
        })
        .expect("admitted reaches application");
    let result = nodes
        .iter()
        .find(|node| node.node_id == call.semantic_type)
        .expect("admitted result type node");
    let operands = match &call.body {
        quire_contract_model::CheckedTerm::Application { arguments, .. } => arguments.len(),
        _ => 0,
    };
    (
        serde_json::to_value(&call.body).expect("typed reaches body"),
        result.semantic_form.to_string(),
        operands,
    )
}

#[trace("TC-048", "FR-038-AC-252")]
#[test]
fn tc_048_reaches_finite_multivalued_ends_keep_the_single_valued_node_shape() {
    for (role, end) in [("links", "targetEnd"), ("linkedBy", "sourceEnd")] {
        let mut baseline = None;
        for (lower, upper, unique) in [
            (0, 1, true),
            (1, 1, true),
            (0, 3, true),
            (0, 3, false),
            (2, 3, true),
        ] {
            let mut relation = authored_relationship();
            relation[end]["multiplicity"] = json!({
                "lower": lower, "upper": upper, "ordered": false, "unique": unique,
            });
            let (package, evidence) = relationship_operation_package(relation, role, true);
            // Everything below is read back from the admitted graph, so a
            // reader that altered the node for a multi-valued end would fail.
            let (body, result_form, operands) = admitted_reaches(&package, &evidence);
            assert_eq!(operands, 2, "{lower}..{upper}, unique={unique}");
            assert_eq!(result_form, "boolean", "{lower}..{upper}, unique={unique}");
            assert_eq!(body["operation"]["member"]["kind"], "relationship_end");
            match &baseline {
                None => baseline = Some(body),
                Some(single) => assert_eq!(
                    &body, single,
                    "{lower}..{upper}, unique={unique}: the end multiplicity must not alter the admitted reaches body"
                ),
            }
            // The node shape is closed: a multiplicity member is not admitted.
            let mut tampered = package.clone();
            tampered["semantic_graph"]["nodes"][CALL]["body"]["multiplicity"] = json!({
                "lower": lower, "upper": upper,
            });
            let nodes = tampered["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes");
            let call = &mut nodes[CALL];
            call["node_id"] = node_id("application/reachability");
            rebuild_source_map(&mut tampered);
            refresh_identity(&mut tampered);
            let refusal = refused(&tampered, &evidence);
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "{lower}..{upper}, unique={unique}: a multiplicity member must be refused"
            );
            assert_eq!(
                refusal.loci[0]
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref(),
                Some("/semantic_graph/nodes/6/body"),
                "{lower}..{upper}, unique={unique}"
            );
        }
    }
}

#[trace("TC-048", "FR-038-AC-253")]
#[test]
fn tc_048_reaches_checks_operand_one_against_receiver_before_edge_defects() {
    let mut finite = authored_relationship();
    finite["targetEnd"]["multiplicity"] = json!({
        "lower": 0, "upper": 3, "ordered": false, "unique": false,
    });
    let (package, evidence) = reaches_over(finite.clone(), "links", ORDER, PRIORITY_ORDER);
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    for (relation, role) in [
        (finite.clone(), "links"),
        (
            {
                let mut relation = finite.clone();
                relation["direction"] = json!("source-to-target");
                relation["sourceEnd"]["multiplicity"] =
                    json!({"lower": 0, "upper": 3, "ordered": false, "unique": false});
                relation
            },
            "linkedBy",
        ),
        (
            {
                let mut relation = finite.clone();
                relation["targetEnd"]["multiplicity"]
                    .as_object_mut()
                    .expect("multiplicity")
                    .remove("upper");
                relation
            },
            "links",
        ),
        (
            {
                let mut relation = finite.clone();
                relation["targetEnd"]["multiplicity"]["ordered"] = json!(true);
                relation
            },
            "links",
        ),
        (
            {
                let mut relation = finite.clone();
                relation["targetEnd"]["type"] = json!(CUSTOMER);
                relation
            },
            "links",
        ),
    ] {
        let (package, evidence) = reaches_over(relation, role, ORDER, CUSTOMER);
        let refusal = refused(&package, &evidence);
        assert_eq!(
            (refusal.code, refusal.cause),
            (
                CheckedPackageRefusalCode::IllTyped,
                Some(CheckedPackageRefusalCause::OperatorIneligible)
            )
        );
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("/semantic_graph/nodes/6/body/arguments/1")
        );
    }
}

#[trace("TC-048", "FR-038-AC-254", "FR-038-AC-255", "FR-038-AC-256")]
#[test]
fn tc_048_reaches_refusal_order_is_direction_then_multiplicity_then_type() {
    let mut relation = authored_relationship();
    relation["direction"] = json!("source-to-target");
    relation["sourceEnd"]["multiplicity"]
        .as_object_mut()
        .expect("multiplicity")
        .remove("upper");
    let cases = [
        (
            relation,
            "linkedBy",
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
        ),
        (
            {
                let mut r = authored_relationship();
                r["targetEnd"]["multiplicity"]
                    .as_object_mut()
                    .expect("multiplicity")
                    .remove("upper");
                r
            },
            "links",
            CheckedPackageRefusalCode::UnsupportedConstruct,
            CheckedPackageRefusalCause::ExpressionForm,
        ),
        (
            {
                let mut r = authored_relationship();
                r["targetEnd"]["multiplicity"]["ordered"] = json!(true);
                r
            },
            "links",
            CheckedPackageRefusalCode::UnsupportedConstruct,
            CheckedPackageRefusalCause::ExpressionForm,
        ),
        (
            {
                let mut r = authored_relationship();
                r["targetEnd"]["type"] = json!(CUSTOMER);
                r["targetEnd"]["multiplicity"]
                    .as_object_mut()
                    .expect("multiplicity")
                    .remove("upper");
                r
            },
            "links",
            CheckedPackageRefusalCode::UnsupportedConstruct,
            CheckedPackageRefusalCause::ExpressionForm,
        ),
        (
            {
                let mut r = authored_relationship();
                r["targetEnd"]["type"] = json!(CUSTOMER);
                r
            },
            "links",
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
        ),
        (
            {
                let mut r = authored_relationship();
                r["targetEnd"]["type"] = json!(CUSTOMER);
                r["targetEnd"]["multiplicity"]["upper"] = json!(3);
                r
            },
            "links",
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
        ),
    ];
    for (relation, role, code, cause) in cases {
        let (package, evidence) = reaches_over(relation, role, ORDER, ORDER);
        let refusal = refused(&package, &evidence);
        assert_eq!((refusal.code, refusal.cause), (code, Some(cause)));
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("/semantic_graph/nodes/6/body/operation/member/name")
        );
    }
}

#[trace("TC-048", "FR-038-AC-257")]
#[test]
fn tc_048_reaches_multivalued_end_requires_boolean_result() {
    // Each result is the end's own navigation type, so only the Boolean
    // requirement distinguishes it from the navigate result.
    for (form, unique) in [("set", true), ("bag", false)] {
        let mut relation = authored_relationship();
        relation["targetEnd"]["multiplicity"] =
            json!({"lower": 1, "upper": 3, "ordered": false, "unique": unique});
        let (mut package, evidence) = relationship_operation_package(relation, "links", true);
        let reference = format!("reference/{ORDER}");
        let nodes = package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        let result_type = bounded_collection(nodes, &reference, form, 1, 3);
        let call = &mut nodes[CALL];
        call["body"]["result_type"] = node_id(&result_type);
        call["semantic_type"] = node_id(&result_type);
        call["node_id"] = node_id("application/reachability");
        rebuild_source_map(&mut package);
        refresh_identity(&mut package);
        let refusal = refused(&package, &evidence);
        assert_eq!(
            (refusal.code, refusal.cause),
            (
                CheckedPackageRefusalCode::IllTyped,
                Some(CheckedPackageRefusalCause::OperatorIneligible)
            ),
            "{form}"
        );
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("/semantic_graph/nodes/6/body/result_type"),
            "{form}"
        );
    }
}

#[trace("TC-048", "FR-038-AC-171")]
#[test]
fn tc_048_relationship_navigation_total_work_limit_reports_its_first_unpayable_edge() {
    let (package, evidence) = relationship_package(authored_relationship(), "links");
    let bytes = canonical(&package);
    let read_at = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        CheckedPackageV2::read(&bytes, limits, &evidence)
    };
    let (mut insufficient, mut admitted) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(matches!(
        read_at(admitted),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    while insufficient + 1 < admitted {
        let middle = insufficient + (admitted - insufficient) / 2;
        match read_at(middle) {
            CheckedPackageV2ReadResult::Admitted(_) => admitted = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => insufficient = middle,
            other => panic!("expected work boundary, got {other:?}"),
        }
    }
    assert!(matches!(
        read_at(admitted),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    match read_at(admitted - 1) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(incomplete.limit, admitted - 1);
            assert_eq!(
                incomplete.path.map(|path| path.to_string()).as_deref(),
                // `select_model` adds the root at node 7; node 8 is the
                // reaches operation's `other` parameter, whose second
                // binding is the Integer-typed level literal.
                Some("/semantic_graph/nodes/8/body/members/1/value/type")
            );
        }
        other => panic!("expected total-work exhaustion, got {other:?}"),
    }
}

#[trace(
    "TC-048",
    "FR-038-AC-165",
    "FR-038-AC-166",
    "FR-038-AC-167",
    "FR-038-AC-168",
    "FR-038-AC-173"
)]
#[test]
fn tc_048_selected_relationship_declaration_refusals_precede_navigation() {
    let mut cases = Vec::new();
    let mut wrong_identity = authored_relationship();
    wrong_identity["identity"] = json!("ix://acme/orders/Order/links");
    cases.push((
        wrong_identity,
        CheckedPackageRefusalCode::InvalidModelBinding,
        CheckedPackageRefusalCause::MalformedDeclaration,
    ));
    let mut missing_direction = authored_relationship();
    missing_direction
        .as_object_mut()
        .expect("relationship")
        .remove("direction");
    cases.push((
        missing_direction,
        CheckedPackageRefusalCode::InvalidModelBinding,
        CheckedPackageRefusalCause::MalformedDeclaration,
    ));
    let mut no_role = authored_relationship();
    no_role["sourceEnd"]
        .as_object_mut()
        .expect("source end")
        .remove("role");
    cases.push((
        no_role,
        CheckedPackageRefusalCode::InvalidModelBinding,
        CheckedPackageRefusalCause::MalformedDeclaration,
    ));
    let mut unresolved = authored_relationship();
    unresolved["targetEnd"]["type"] = json!("ix://acme/orders/Ghost");
    cases.push((
        unresolved,
        CheckedPackageRefusalCode::MissingDeclaration,
        CheckedPackageRefusalCause::MissingName,
    ));
    let mut reversed = authored_relationship();
    reversed["targetEnd"]["multiplicity"]["lower"] = json!(3);
    cases.push((
        reversed,
        CheckedPackageRefusalCode::InvalidModelBinding,
        CheckedPackageRefusalCause::UnpreservedModelMeaning,
    ));
    for (relation, code, cause) in cases {
        let (package, evidence) = relationship_package(relation.clone(), "missing");
        let refusal = refused(&package, &evidence);
        assert_eq!(
            (refusal.code, refusal.cause),
            (code, Some(cause)),
            "{relation}"
        );
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(root_owner(&package).as_str())
        );
    }
    let mut no_inverse = authored_relationship();
    no_inverse["targetEnd"]
        .as_object_mut()
        .expect("target end")
        .remove("role");
    let (package, evidence) = relationship_package(no_inverse, "linkedBy");
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingName)
        )
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/semantic_graph/nodes/6/body/operation/member/name")
    );
}

#[trace("TC-048", "FR-038-AC-29")]
#[test]
fn tc_048_a_model_owned_operation_admits_through_a_real_semantic_ir_document() {
    let (package, evidence) = package_over(&domain_document(json!({})), "total");
    match read(&package, &evidence) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected admission, got {other:?}"),
    }
}

/// Trace: FR-068-AC-2, QSpec FR-322-AC-72
#[trace("TC-463")]
#[test]
fn selected_flow_process_go_resolves_its_operation_and_reference_receiver() {
    let flow = format!("ix://{IDENTITY}/Flow");
    let mut document = domain_document(json!({}));
    document["constructs"][0]["construct"]["meaning"] = json!("quire.meaning.model.process/v1");
    document["types"][0]["identity"] = json!(flow);
    document["types"][0]["displayName"] = json!("Flow");
    document["types"][0]["operations"][0]["identity"] = json!(format!("{flow}/go"));
    document["types"][0]["fields"] = json!([{
        "identity":format!("{flow}/n"), "name":"n",
        "typeRef":"ix://quire/native/Integer", "presence":"required",
        "nullable":false, "defaultKind":"none",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
    }]);
    document["types"][0]["identityFields"] = json!([format!("{flow}/n")]);
    document["types"][0]["steps"] = json!([{
        "identity":format!("{flow}/advance"), "name":"advance",
        "stepKind":"command", "consumes":[], "emits":[]
    }]);
    let package = package_selecting_form("go", "process", &flow);
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package, &evidence) else {
        panic!("selected Flow.go and Reference<Flow> must admit");
    };
    assert_eq!(admitted.graph().nodes[CALL].semantic_form.as_ref(), "call");
    assert_eq!(
        admitted.graph().nodes[3].semantic_form.as_ref(),
        "reference"
    );
}

fn unit_dispatch_call(
    argument_family: &str,
    result_unit: bool,
) -> (Value, quire_contract_model::CheckedPackageEvidence) {
    let mut document = domain_document(json!({}));
    document["types"][0]["operations"][0]["params"] = json!([slot(None, "ix://quire/unit/ms")]);
    if result_unit {
        document["types"][0]["operations"][0]["returns"] = slot(None, "ix://quire/unit/ms");
    }
    let mut package = package_selecting("total");
    let mut members = nominal_fixture_members();
    let unit = members
        .iter_mut()
        .find(|(preimage, _)| preimage["version"] == "quire.unit-node/v1")
        .expect("unit");
    unit.0["qualified_declaration"] = json!(["ms"]);
    unit.1 = "ms".to_owned();
    let mut second_preimage = unit.0.clone();
    second_preimage["qualified_declaration"] = json!(["s"]);
    second_preimage["target_unit_node_id"] = node_id(&unit.1);
    second_preimage["scale"]["numerator"] = json!("2");
    let second = "s".to_owned();
    let ms = unit.1.clone();
    members.push((second_preimage, second.clone()));
    let nominal = nominal_package(&members);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .extend(
            nominal["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .cloned(),
        );
    let argument_type = match argument_family {
        "integer" => None,
        "ms" => Some(ms.as_str()),
        "s" => Some(second.as_str()),
        other => panic!("unknown argument family {other}"),
    };
    if let Some(argument_type) = argument_type {
        let (argument, node) = parameter("argument", "1", argument_type);
        package["semantic_graph"]["nodes"][5] = node;
        package["semantic_graph"]["nodes"][CALL]["body"]["arguments"][1] = reference(&argument);
        settle(&mut package);
    }
    if result_unit {
        package["semantic_graph"]["nodes"][CALL]["semantic_type"] = node_id(&ms);
        package["semantic_graph"]["nodes"][CALL]["body"]["result_type"] = node_id(&ms);
        settle(&mut package);
    }
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    (package, evidence)
}

#[trace("TC-230", "FR-038-AC-300", "FR-038-AC-302")]
#[test]
fn tc_230_dispatch_call_requires_the_selected_checked_unit() {
    for result_unit in [false, true] {
        let (package, evidence) = unit_dispatch_call("ms", result_unit);
        assert!(matches!(
            read(&package, &evidence),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
    }
    for argument_family in ["integer", "s"] {
        let (package, evidence) = unit_dispatch_call(argument_family, false);
        let refusal = refused(&package, &evidence);
        assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!("/semantic_graph/nodes/{CALL}/body/arguments/1")),
        );
    }
}

#[trace("FR-038-AC-150", "FR-038-AC-185", "TC-048")]
#[test]
fn tc_048_selected_model_declaration_shape_refuses_at_its_node_before_the_call() {
    let (mut package, evidence) = package_over(&domain_document(json!({})), "total");
    let position = 2;
    let original = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["semantic_type"] =
        package["semantic_graph"]["nodes"][0]["node_id"].clone();
    refresh_identity(&mut package);
    let refusal = match read(&package, &evidence) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("selected model declaration must refuse: {other:?}"),
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}/semantic_type"))
    );
    assert_eq!(
        serde_json::to_value(refusal.locus.as_ref().expect("node locus"))
            .expect("structural node locus"),
        original
    );
}

#[trace("TC-048", "FR-038-AC-45")]
#[test]
fn tc_048_a_declaration_node_key_is_unchanged_when_the_selected_document_changes() {
    let (selected, selected_evidence) = package_over(&domain_document(json!({})), "total");
    match read(&selected, &selected_evidence) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected admission, got {other:?}"),
    }

    // A document that differs only in content the keys do not read (its
    // display name): the keys are unchanged (FR-038-AC-45).
    let renamed = domain_document(json!({"displayName": "Order renamed"}));
    let (renamed_package, renamed_evidence) = package_over(&renamed, "total");
    match read(&renamed_package, &renamed_evidence) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected admission under the changed document, got {other:?}"),
    }
    let declarations = |package: &Value| {
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .filter(|node| node["semantic_form"] != "model_root")
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(declarations(&selected), declarations(&renamed_package));

    // The same declarations in a document whose only difference is its
    // package version, which the selection no longer names: the keys are
    // version-free (`tc_048_a_selected_document_needs_no_package_version`).
    assert_ne!(
        declaration_key_of(ORDER, IDENTITY),
        declaration_key_of(ORDER, OTHER_PACKAGE),
        "the owner's identity, not its version, distinguishes declaration keys"
    );
}

#[trace("TC-048", "FR-038-AC-45")]
#[test]
fn tc_048_a_declaration_node_keyed_under_another_domain_package_refuses() {
    // The member's declaration is `Order` keyed under a package the lock does
    // not select: the key resolves to no selected declaration.
    let (package, evidence, _) = reaches_package(
        (ORDER, OTHER_PACKAGE),
        "parent",
        Operand::Reference("unselected"),
        Operand::Reference("unselected"),
    );
    let refusal = refused(&package, &evidence);
    assert_eq!(
        (refusal.code, refusal.cause),
        (
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingSelection)
        )
    );
    let unselected = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["owner"]["identity"] == OTHER_PACKAGE)
        .expect("unselected model node");
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some(format!("/semantic_graph/nodes/{unselected}/node_id").as_str())
    );
    // The selected package's own key for the same node resolves.
    let (package, evidence, _) = reaches_package(
        (ORDER, IDENTITY),
        "parent",
        Operand::Reference(ORDER),
        Operand::Reference(ORDER),
    );
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// A selection binds by identity: the document's own
/// `package.version` is neither required nor read, so a matching document
/// with no version, a non-string one or any string admits. The model-owned node keys carry no version
/// by construction (the owner is `{kind, identity, node}`, pinned by
/// `tc_048_a_declaration_node_key_is_unchanged_when_the_selected_document_changes`),
/// so admission per version is the oracle here, not graph equality.
#[trace("TC-048", "FR-038-AC-27", "FR-038-AC-45", "FR-038-AC-64")]
#[test]
fn tc_048_a_selected_document_needs_no_package_version() {
    let versioned = |version: Option<Value>| {
        let mut document = domain_document(json!({}));
        let package = document["package"].as_object_mut().expect("package");
        match version {
            Some(version) => package.insert("version".to_owned(), version),
            None => package.remove("version"),
        };
        document
    };
    for (case, version) in [
        ("version 1.0.0", Some(json!("1.0.0"))),
        ("version 2.0.0", Some(json!("2.0.0"))),
        ("no version", None),
        ("a non-string version", Some(json!(7))),
        ("a null version", Some(Value::Null)),
    ] {
        let (package, evidence) = package_over(&versioned(version), "total");
        match read(&package, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{case}: expected admission, got {other:?}"),
        }
    }

    // A document naming another identity is refused at the root's identity,
    // whatever its version.
    let mut other = versioned(Some(json!("1.0.0")));
    other["package"]["identity"] = json!("acme/other");
    let (package, evidence) = package_over(&other, "total");
    assert_eq!(
        refused(&package, &evidence),
        refusal_cause(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("{}/identity", root_owner(&package)),
            CheckedPackageRefusalCause::WrongModelSelection
        )
    );
}

#[trace("TC-048", "FR-038-AC-29")]
#[test]
fn tc_048_a_model_owned_operation_the_document_does_not_declare_refuses() {
    let (package, evidence) = package_over(&domain_document(json!({})), "missing");
    let refusal = refused(&package, &evidence);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some(format!("/semantic_graph/nodes/{CALL}/body/operation/member/name").as_str())
    );
}

#[trace("TC-048", "FR-038-AC-27", "FR-038-AC-28")]
#[test]
fn tc_048_a_declaration_refusal_is_located_at_its_model_root() {
    let dangling = domain_document(json!({"supertypes": ["ix://acme/orders/Nope"]}));
    let (package, evidence) = package_over(&dangling, "total");
    assert_eq!(
        refused(&package, &evidence),
        refusal_cause(
            CheckedPackageRefusalCode::MissingDeclaration,
            &root_owner(&package),
            CheckedPackageRefusalCause::MissingName
        )
    );
}

#[trace("TC-048", "FR-038-AC-30")]
#[test]
fn tc_048_reading_a_domain_package_is_charged_to_the_work_limit() {
    let (package, evidence) = package_over(&domain_document(json!({})), "total");
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = 0;
    match CheckedPackageV2::read(&canonical(&package), limits, &evidence) {
        CheckedPackageV2ReadResult::Incomplete(CheckedPackageIncomplete {
            limit_kind,
            limit,
            path,
            ..
        }) => {
            assert_eq!(limit_kind, CheckedPackageLimit::Work);
            assert_eq!(limit, 0);
            assert_eq!(
                path.map(|path| path.to_string()).as_deref(),
                Some(root_owner(&package).as_str())
            );
        }
        other => panic!("expected an incomplete read, got {other:?}"),
    }
}

/// The bytes of [`domain_document`] with the JSON number text `number` at
/// `/package/<member>`, spelled exactly as given.
fn document_bytes_with_number(member: &str, number: &str) -> Vec<u8> {
    let mut document = domain_document(json!({}));
    document["package"][member] = json!(0);
    let text = String::from_utf8(canonical(&document)).expect("utf-8");
    let zero = format!("\"{member}\":0");
    assert_eq!(text.matches(&zero).count(), 1);
    text.replace(&zero, &format!("\"{member}\":{number}"))
        .into_bytes()
}

/// A read of a package whose one root selects [`IDENTITY`], with `bytes`
/// supplied as its document.
fn read_selecting(bytes: &[u8]) -> CheckedPackageV2ReadResult {
    let package = package_selecting("total");
    let mut evidence = quire_contract_model::CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(IDENTITY.to_owned(), bytes.to_vec())
        .expect("domain evidence declarations agree on exact bytes");
    evidence.support_feature(COMPLETE_VALUE_FEATURE);
    read(&package, &evidence)
}

fn number_refusal(document_pointer: &str, cause: CheckedPackageRefusalCause) -> ExpectedRefusal {
    ExpectedRefusal {
        document_pointer: Some(pointer(document_pointer)),
        ..refusal_cause(
            CheckedPackageRefusalCode::NoncanonicalWire,
            &format!("{}/identity", root_owner(&package_selecting("total"))),
            cause,
        )
    }
}

#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_a_model_document_number_past_2_pow_53_refuses_with_its_document_pointer() {
    for text in [
        "9007199254740993",
        "-9007199254740993",
        "9.007199254740993e15",
        "1e20",
        "18446744073709551617",
    ] {
        let bytes = document_bytes_with_number("count", text);
        match read_selecting(&bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => {
                assert_eq!(
                    refused,
                    number_refusal("/package/count", CheckedPackageRefusalCause::InexactInteger),
                    "{text}"
                );
            }
            other => panic!("{text}: expected a refusal, read {other:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_a_model_document_number_at_or_under_2_pow_53_is_admitted() {
    for (text, _) in [
        ("9007199254740992", "9007199254740992"),
        ("-9007199254740992", "-9007199254740992"),
        ("9.007199254740992e15", "9007199254740992"),
        ("0.5", "0.5"),
    ] {
        let bytes = document_bytes_with_number("count", text);
        assert!(
            matches!(
                read_selecting(&bytes),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{text} is admitted"
        );
    }
}

/// [`domain_document`] with an integer value type `Count` whose `max` bound is
/// `maximum`, a JSON number or a decimal string.
fn document_with_count_bound(maximum: Value) -> Value {
    let mut document = domain_document(json!({}));
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": "acme/orders", "name": "count"},
            "construct": {"meaning": "quire.meaning.model.value-type/v1"},
        }));
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(json!({
            "identity": "ix://acme/orders/Count", "displayName": "Count",
            "kind": {"module": "acme/orders", "name": "count"},
            "roles": [], "extensions": [], "unknownPolicy": "reject", "scalar": "integer",
            "constraints": [
                {"keyword": "min", "operands": {"value": 0}},
                {"keyword": "max", "operands": {"value": maximum}},
            ],
        }));
    document
}

#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_an_integer_value_type_bound_past_2_pow_53_is_refused_as_a_number() {
    // Written as a JSON number, the bound is refused at its own pointer, where
    // the reader admitted it before.
    let past = document_with_count_bound(json!(9_007_199_254_740_993_i64));
    let bytes = canonical(&past);
    match read_selecting(&bytes) {
        CheckedPackageV2ReadResult::Refused(refused) => {
            assert_eq!(
                refused,
                number_refusal(
                    "/types/1/constraints/1/operands/value",
                    CheckedPackageRefusalCause::InexactInteger
                )
            );
        }
        other => panic!("expected a refusal, read {other:?}"),
    }
    // At 2^53 the same document is admitted.
    let at = document_with_count_bound(json!(9_007_199_254_740_992_i64));
    let bytes = canonical(&at);
    assert!(matches!(
        read_selecting(&bytes),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    // Written as a decimal string, the bound is not a number of the document:
    // exact integers past 2^53 travel this way, so it is admitted.
    let spelled = document_with_count_bound(json!("9007199254740993"));
    let bytes = canonical(&spelled);
    assert!(matches!(
        read_selecting(&bytes),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_the_first_number_in_document_order_past_2_pow_53_is_named() {
    let document = domain_document(json!({}));
    let text = String::from_utf8(canonical(&document)).expect("utf-8");
    // `/b` is written first and `/a/0` second, so document order is not the
    // sorted order that would name `/a/0` first.
    let spelled = format!(
        "{{\"b\":9007199254740993,\"a\":[9007199254740995],{}",
        &text[1..]
    );
    let bytes = spelled.into_bytes();
    match read_selecting(&bytes) {
        CheckedPackageV2ReadResult::Refused(refused) => {
            assert_eq!(
                refused,
                number_refusal("/b", CheckedPackageRefusalCause::InexactInteger)
            );
        }
        other => panic!("expected a refusal, read {other:?}"),
    }
}

/// The JSON object prefix `members` placed before the members of a model
/// document's canonical text, so the numbers in `members` come first in
/// document order.
fn document_bytes_led_by(members: &str) -> Vec<u8> {
    let document = domain_document(json!({}));
    let text = String::from_utf8(canonical(&document)).expect("utf-8");
    format!("{{{members},{}", &text[1..]).into_bytes()
}

#[trace("TC-048", "FR-038-AC-109")]
#[test]
fn tc_048_a_model_document_number_with_no_exact_rfc_8785_spelling_refuses_inexact_number() {
    for text in [
        "0.1000000000000000000001",
        "9007199254740993.5",
        "-0.1000000000000000000001",
        "4.9e-324",
        "1e-400",
        // The odd-digit spellings of three ties, whose even-digit spellings are
        // admitted below.
        "1125899906842624.3",
        "1500000000000000.3",
        "2.9802322387695313e-8",
    ] {
        let bytes = document_bytes_with_number("ratio", text);
        match read_selecting(&bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                number_refusal("/package/ratio", CheckedPackageRefusalCause::InexactNumber),
                "{text}"
            ),
            other => panic!("{text}: expected a refusal, read {other:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-109")]
#[test]
fn tc_048_a_model_document_number_whose_value_its_encoding_keeps_is_admitted() {
    // Each pair is the number as spelled and the text `quire-canonical` writes
    // for it: the value is the same, so the spelling does not matter.
    for (text, _) in [
        ("0.1", "0.1"),
        ("0.5", "0.5"),
        ("1.5", "1.5"),
        ("-0.25", "-0.25"),
        ("5e-324", "5e-324"),
        ("2.5e-10", "2.5e-10"),
        ("1.0", "1"),
        ("-0", "0"),
        ("1e2", "100"),
        ("1125899906842624.2", "1125899906842624.2"),
        ("1500000000000000.2", "1500000000000000.2"),
        ("2.9802322387695312e-8", "2.9802322387695312e-8"),
    ] {
        let bytes = document_bytes_with_number("ratio", text);
        assert!(
            matches!(
                read_selecting(&bytes),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{text} is admitted"
        );
    }
}

#[trace("TC-048", "FR-038-AC-110")]
#[test]
fn tc_048_the_first_inexact_number_in_document_order_is_named_with_its_own_cause() {
    // An inexact number at `/b` before a whole number past 2^53 at `/a/0`
    // names `/b`; the reverse order names the whole number. `9007199254740992.5`
    // is not whole, so it is never an integer cause though its nearest double
    // is 2^53.
    for (members, pointer, cause) in [
        (
            "\"b\":9007199254740992.5",
            "/b",
            CheckedPackageRefusalCause::InexactNumber,
        ),
        (
            "\"b\":0.1000000000000000000001,\"a\":[9007199254740993]",
            "/b",
            CheckedPackageRefusalCause::InexactNumber,
        ),
        (
            "\"b\":9007199254740993,\"a\":[0.1000000000000000000001]",
            "/b",
            CheckedPackageRefusalCause::InexactInteger,
        ),
        (
            "\"a\":[9007199254740993.0],\"b\":0.1000000000000000000001",
            "/a/0",
            CheckedPackageRefusalCause::InexactInteger,
        ),
    ] {
        let bytes = document_bytes_led_by(members);
        match read_selecting(&bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => {
                assert_eq!(refused, number_refusal(pointer, cause), "{members}");
            }
            other => panic!("{members}: expected a refusal, read {other:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-110")]
#[test]
fn tc_048_a_number_past_the_double_range_refuses_inexact_integer_with_its_pointer() {
    for text in ["1e400", "-1e400", "1e309"] {
        // Nested: at a member of the model document.
        let bytes = document_bytes_with_number("ratio", text);
        match read_selecting(&bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                number_refusal("/package/ratio", CheckedPackageRefusalCause::InexactInteger),
                "{text}"
            ),
            other => panic!("{text}: expected a refusal, read {other:?}"),
        }

        // In an array, so the pointer carries an index.
        let bytes = document_bytes_led_by(&format!("\"a\":[0,{text}]"));
        match read_selecting(&bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                number_refusal("/a/1", CheckedPackageRefusalCause::InexactInteger),
                "{text}"
            ),
            other => panic!("{text}: expected a refusal, read {other:?}"),
        }
        // At the top level the document is the number and the pointer is empty.
        let bytes = text.as_bytes().to_vec();
        match read_selecting(&bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                number_refusal("", CheckedPackageRefusalCause::InexactInteger),
                "{text}"
            ),
            other => panic!("{text}: expected a refusal, read {other:?}"),
        }
    }
}

/// The refusal of a model document the reader does not parse.
fn unreadable_refusal() -> ExpectedRefusal {
    refusal_cause(
        CheckedPackageRefusalCode::MalformedWire,
        &format!("{}/identity", root_owner(&package_selecting("total"))),
        CheckedPackageRefusalCause::MalformedJson,
    )
}

#[trace("TC-048", "FR-038-AC-110")]
#[test]
fn tc_048_the_first_reader_fault_decides_when_faults_coexist() {
    // QSL FR-056: the first fault `quire_canonical::read` returns decides. A
    // number with no finite double is raised when it is read; a repeated
    // member name only when its object closes. Hand-written expectations.
    let number = |pointer: &str, cause| number_refusal(pointer, cause);
    let led = |members: &str| document_bytes_led_by(members);
    let cases: [(Vec<u8>, ExpectedRefusal); 11] = [
        // The reader checks that the whole input is UTF-8 before it reads any
        // value, so invalid UTF-8 after the number is the first fault.
        (b"[1e400,\"\xFF\"]".to_vec(), unreadable_refusal()),
        // A lone surrogate escape is found when the string is read, after the
        // number.
        (
            b"[1e400,\"\\ud800\"]".to_vec(),
            number("/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        // The out-of-range number is read before the object closes, so it is
        // named ahead of the earlier repeated name.
        (
            led("\"a\":1,\"a\":2,\"n\":1e400"),
            number("/n", CheckedPackageRefusalCause::InexactInteger),
        ),
        // The repeated name is in an object that closes before `1e400`, so it
        // is the first fault and the bytes are unreadable.
        (b"[{\"a\":1,\"a\":2},1e400]".to_vec(), unreadable_refusal()),
        // An inexact-number-to-be (`1e-400` reads as zero) is no reader
        // fault, so the repeated name is the first fault.
        (led("\"a\":1,\"a\":2,\"n\":1e-400"), unreadable_refusal()),
        // Truncation after the number: the number is read first.
        (
            b"[1e400".to_vec(),
            number("/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        // The number before the repeated name, in the same object.
        (
            led("\"n\":1e400,\"a\":1,\"a\":2"),
            number("/n", CheckedPackageRefusalCause::InexactInteger),
        ),
        // An out-of-range number is named ahead of an earlier inexact number
        // (a reader-decided one is refused only after the read succeeds).
        (
            led("\"b\":0.1000000000000000000001,\"a\":[1e400]"),
            number("/a/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        (
            led("\"b\":9007199254740993,\"a\":[1e400]"),
            number("/a/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        (
            led("\"b\":9007199254740993,\"a\":[-1e400]"),
            number("/a/0", CheckedPackageRefusalCause::InexactInteger),
        ),
        // An out-of-range number that is not whole is an inexact number.
        (
            led(&format!(
                "\"b\":9007199254740993,\"a\":[{}.5]",
                "9".repeat(400)
            )),
            number("/a/0", CheckedPackageRefusalCause::InexactNumber),
        ),
    ];
    for (bytes, expected) in cases {
        let shown = String::from_utf8_lossy(&bytes)
            .chars()
            .take(60)
            .collect::<String>();
        match read_selecting(&bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => {
                assert_eq!(refused, expected, "{shown}")
            }
            other => panic!("{shown}: expected a refusal, read {other:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-110")]
#[test]
fn tc_048_a_non_whole_number_past_the_double_range_refuses_inexact_number() {
    // 400 nines then `.5`: past the double range, but not a whole value, so
    // the cause is `inexact-number`, never `inexact-integer`.
    let text = format!("{}.5", "9".repeat(400));
    for number in [text.clone(), format!("-{text}")] {
        let bytes = document_bytes_with_number("ratio", &number);
        match read_selecting(&bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                number_refusal("/package/ratio", CheckedPackageRefusalCause::InexactNumber),
            ),
            other => panic!("expected a refusal, read {other:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-109")]
#[test]
fn tc_048_a_number_below_the_double_range_refuses_inexact_number_on_its_text() {
    // These read as zero with their text kept, so the rule is IR's own, on
    // the text: zero is not their exact value.
    for text in ["1e-400", "-1e-400"] {
        let bytes = document_bytes_with_number("ratio", text);
        match read_selecting(&bytes) {
            CheckedPackageV2ReadResult::Refused(refused) => assert_eq!(
                refused,
                number_refusal("/package/ratio", CheckedPackageRefusalCause::InexactNumber),
                "{text}"
            ),
            other => panic!("{text}: expected a refusal, read {other:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-93")]
#[test]
fn tc_048_a_package_stream_refusal_carries_no_document_pointer() {
    let package = package_selecting("total");
    let mut bytes = canonical(&package);
    bytes.push(b' ');
    match CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package),
    ) {
        CheckedPackageV2ReadResult::Refused(refused) => {
            assert_eq!(
                refused,
                refusal_bytes(CheckedPackageRefusalCode::NoncanonicalWire)
            );
            assert_eq!(refused.document_pointer, None);
        }
        other => panic!("expected a refusal, read {other:?}"),
    }
}

#[trace("TC-048", "FR-038-AC-94")]
#[test]
fn tc_048_a_model_document_is_read_and_encoded_under_the_byte_limit() {
    let mut document = domain_document(json!({}));
    document["package"]["padding"] = json!("x".repeat(30_000));
    let bytes = canonical(&document);
    let length = u64::try_from(bytes.len()).expect("length");
    let package = package_selecting("total");
    let package_bytes = canonical(&package);
    assert!(
        u64::try_from(package_bytes.len()).expect("length") < length,
        "the package is shorter than the document"
    );
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.bytes = length;
    assert!(
        matches!(
            CheckedPackageV2::read(&package_bytes, limits, &evidence),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "a document exactly `limits.bytes` long is read and matches"
    );
    limits.bytes = length - 1;
    match CheckedPackageV2::read(&package_bytes, limits, &evidence) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Bytes);
            assert_eq!(incomplete.limit, length - 1);
            assert_eq!(incomplete.consumed, length);
            assert_eq!(incomplete.path, None);
        }
        other => panic!("expected an incomplete read, got {other:?}"),
    }
    // The work limit charged for the same document is located at the root.
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = length.div_ceil(1024) - 1;
    match CheckedPackageV2::read(&package_bytes, limits, &evidence) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(
                incomplete.path.map(|path| path.to_string()).as_deref(),
                Some(root_owner(&package).as_str())
            );
        }
        other => panic!("expected an incomplete read, got {other:?}"),
    }
}

const SUB: &str = "ix://acme/orders/Sub";
const SUBSUB: &str = "ix://acme/orders/SubSub";
const INVOICE: &str = "ix://acme/orders/Invoice";
const TAGGED: &str = "ix://acme/orders/Tagged";
const CODED: &str = "ix://acme/orders/Coded";
const BOTH: &str = "ix://acme/orders/Both";
const MODELS: [&str; 7] = [ORDER, SUB, SUBSUB, INVOICE, TAGGED, CODED, BOTH];
/// A domain package identity the lock never selects.
const OTHER_PACKAGE: &str = "acme/other";

fn edge_field(
    owner: &str,
    name: &str,
    type_ref: &str,
    presence: &str,
    multiplicity: Value,
) -> Value {
    json!({
        "identity": format!("{owner}/{name}"), "name": name, "typeRef": type_ref,
        "presence": presence, "nullable": false, "defaultKind": "none",
        "multiplicity": multiplicity,
    })
}

fn object_type_declaration(node: &str, supertypes: &[&str], fields: Vec<Value>) -> Value {
    json!({
        "identity": node, "displayName": node,
        "kind": {"module": "acme/orders", "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes, "fields": fields, "operations": [],
    })
}

/// A document whose `Order` declares one edge-shaped and several
/// non-edge-shaped fields; `Sub` specializes `Order` and declares an
/// optional `Sub` edge, `SubSub` specializes `Sub`, `Invoice` is unrelated,
/// and `Both` inherits a `code` field from each of `Tagged` and `Coded`.
fn edge_document() -> Value {
    let one = json!({"lower": 1, "upper": 1, "ordered": false, "unique": true});
    let order = |name: &str, presence: &str, multiplicity: Value| {
        edge_field(ORDER, name, ORDER, presence, multiplicity)
    };
    let fields = vec![
        order("parent", "optional", one.clone()),
        order("origin", "required", one.clone()),
        order(
            "prior",
            "required",
            json!({"lower": 0, "upper": 3, "ordered": true, "unique": false}),
        ),
        order(
            "peers",
            "required",
            json!({"lower": 0, "upper": 3, "ordered": false, "unique": true}),
        ),
        order(
            "trail",
            "required",
            json!({"lower": 0, "ordered": true, "unique": false}),
        ),
        edge_field(ORDER, "invoice", INVOICE, "required", one.clone()),
        edge_field(
            ORDER,
            "total",
            "ix://quire/native/Integer",
            "required",
            one.clone(),
        ),
    ];
    let code = |owner: &str| {
        vec![edge_field(
            owner,
            "code",
            "ix://quire/native/Integer",
            "required",
            one.clone(),
        )]
    };
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": IDENTITY, "version": VERSION},
        "constructs": [{
            "kind": {"module": "acme/orders", "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }],
        "types": [
            object_type_declaration(ORDER, &[], fields),
            object_type_declaration(
                SUB,
                &[ORDER],
                vec![edge_field(SUB, "child", SUB, "optional", one.clone())],
            ),
            object_type_declaration(SUBSUB, &[SUB], vec![]),
            object_type_declaration(INVOICE, &[], vec![]),
            object_type_declaration(TAGGED, &[], code(TAGGED)),
            object_type_declaration(CODED, &[], code(CODED)),
            object_type_declaration(BOTH, &[TAGGED, CODED], vec![]),
        ],
    })
}

/// The authored model declaration identity of `node` under `identity`.
fn declaration_key_of(node: &str, identity: &str) -> String {
    model_key("object_type", identity, node)
}

/// What an operand of [`reaches_package`] is typed as.
#[derive(Clone, Copy)]
enum Operand {
    /// `Reference<node>`.
    Reference(&'static str),
    /// `node` itself, an object.
    Object(&'static str),
}

/// A package over [`edge_document`] holding `reaches(source, target, name)`
/// as a `quire.op.model.reaches_field` application whose member declaration
/// is `declaring` (keyed under the domain package `package`), with the
/// operands typed as given. Returns the package, its evidence, and the
/// application's node position.
fn reaches_package(
    (declaring, package_identity): (&str, &str),
    name: &str,
    source: Operand,
    target: Operand,
) -> (Value, quire_contract_model::CheckedPackageEvidence, usize) {
    let document = edge_document();
    let mut package = nominal_package(&[]);

    let boolean = "scalar/boolean".to_owned();
    let mut models: Vec<(String, String)> = MODELS
        .into_iter()
        .map(|node| (node.to_owned(), declaration_key_of(node, IDENTITY)))
        .collect();
    if package_identity == OTHER_PACKAGE
        || matches!(source, Operand::Reference("unselected"))
        || matches!(target, Operand::Reference("unselected"))
    {
        models.push((
            "unselected".to_owned(),
            declaration_key_of(ORDER, OTHER_PACKAGE),
        ));
    }
    let reference_types: Vec<(String, String, Value)> = models
        .iter()
        .map(|(node, key)| {
            let body = json!({"term": "aggregate", "members": [reference(key)]});
            let type_key = format!("reference/{key}");
            (node.clone(), type_key, body)
        })
        .collect();
    let model_of = |node: &str, owner: &str| {
        let name = if owner == IDENTITY {
            node
        } else {
            "unselected"
        };
        models
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, key)| key.clone())
            .expect("declared model")
    };
    // A reference operand is a parameter of the reference type; an object
    // operand names the model declaration node itself.
    let operand = |operand: Operand, name: &str, level: &str| match operand {
        Operand::Reference(node) => {
            let type_key = reference_types
                .iter()
                .find(|(candidate, _, _)| candidate == node)
                .map(|(_, key, _)| key.clone())
                .expect("reference type");
            let (key, node) = parameter(name, level, &type_key);
            (key, Some(node))
        }
        Operand::Object(node) => (model_of(node, IDENTITY), None),
    };
    let (receiver, receiver_node) = operand(source, "receiver", "0");
    let (argument, argument_node) = operand(target, "argument", "1");
    let declaring_key = model_of(declaring, package_identity);
    let body = json!({
        "term": "application",
        "operator": "reaches",
        "operation": {
            "identity": "quire.op.model.reaches_field", "laws": [], "mode": null,
            "member": {"kind": "field", "declaration": node_id(&declaring_key), "name": name},
            "leaves": [],
        },
        "result_type": node_id(&boolean),
        "arguments": [reference(&receiver), reference(&argument)],
    });
    let reaches = "application/reaches-field";
    let mut dependencies = [receiver.as_str(), argument.as_str(), declaring_key.as_str()];
    dependencies.sort_unstable();
    let dependencies: Vec<&str> = {
        let mut unique = dependencies.to_vec();
        unique.dedup();
        unique
    };
    let mut nodes = vec![
        wire_node(&INTEGER, "scalar_type", "integer", &INTEGER, &[], empty()),
        wire_node(&TEXT, "scalar_type", "text", &TEXT, &[], empty()),
        wire_node(&boolean, "scalar_type", "boolean", &boolean, &[], empty()),
    ];
    for (declared, key) in &models {
        let mut node = wire_node(key, "model", "object_type", key, &[], empty());
        let (identity, model_node) = if declared == "unselected" {
            (OTHER_PACKAGE, ORDER)
        } else {
            (IDENTITY, declared.as_str())
        };
        node["owner"] = json!({"kind": "model", "identity": identity, "node": model_node});
        nodes.push(node);
    }
    for ((_, model), (_, type_key, type_body)) in models.iter().zip(&reference_types) {
        nodes.push(wire_node(
            type_key,
            "composite_type",
            "reference",
            type_key,
            &[model.as_str()],
            type_body.clone(),
        ));
    }
    nodes.extend(receiver_node.into_iter().chain(argument_node));
    nodes.push(wire_node(
        reaches,
        "expression",
        "reachability",
        &boolean,
        &dependencies,
        body,
    ));
    let position = nodes.len() - 1;
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes);
    rebuild_source_map(&mut package);
    select_model(&mut package, IDENTITY);
    refresh_identity(&mut package);
    let evidence = evidence_with_document(&package, IDENTITY, &document);
    (package, evidence, position)
}

const SELECTED: &str = IDENTITY;

fn reaches(
    declaring: &'static str,
    name: &str,
    source: &'static str,
    target: &'static str,
) -> (Value, quire_contract_model::CheckedPackageEvidence, usize) {
    reaches_package(
        (declaring, SELECTED),
        name,
        Operand::Reference(source),
        Operand::Reference(target),
    )
}

/// FR-322 "Reaches over a field": an optional, a required and a bounded
/// sequence reference to the owner are edges, including through a subtype
/// declaring node, a subtype target and a target two levels down.
#[trace("TC-056", "FR-040-AC-13")]
#[test]
fn tc_056_reaches_field_admits_each_reference_edge_shape() {
    let admitted = [
        (ORDER, "parent", ORDER, ORDER),
        (ORDER, "prior", ORDER, ORDER),
        (ORDER, "origin", ORDER, ORDER),
        (SUB, "parent", SUB, ORDER),
        (ORDER, "parent", ORDER, SUB),
        (ORDER, "parent", ORDER, SUBSUB),
        (SUB, "child", SUB, SUBSUB),
    ];
    for (declaring, name, source, target) in admitted {
        let (package, evidence, _) = reaches(declaring, name, source, target);
        match read(&package, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{name} over {source}/{target}: expected admission, got {other:?}"),
        }
    }
}

/// The package reader's relationship-edge check resolves `parent` on each
/// declaring type. `Sub` inherits `Order`'s seven fields and declares `child`,
/// so its field table has one more entry than `Order`'s table.
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_a_relationship_edge_read_charges_its_effective_table_end_to_end() {
    let smallest = |declaring, source| {
        let (package, evidence, _) = reaches(declaring, "parent", source, ORDER);
        let read = |work| {
            let mut limits = CheckedPackageReadLimits::bounded();
            limits.work = work;
            CheckedPackageV2::read(&canonical(&package), limits, &evidence)
        };
        let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
        assert!(matches!(
            read(high),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        while low < high {
            let middle = low + (high - low) / 2;
            match read(middle) {
                CheckedPackageV2ReadResult::Admitted(_) => high = middle,
                CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
                other => panic!("expected admission or work limit, got {other:?}"),
            }
        }
        assert!(matches!(
            read(low - 1),
            CheckedPackageV2ReadResult::Incomplete(_)
        ));
        low
    };
    assert_eq!(smallest(SUB, SUB), smallest(ORDER, ORDER) + 1);
}

/// FR-322 "Reaches over a field": a field that is not a reference edge to
/// its owner, an operand that does not name the declaring node, a target
/// that is not a subtype of the edge's owner (a supertype included), an
/// object operand, an undeclared field, an ambiguous field name and an
/// unselected declaration each refuse with their own code at the operand or
/// member at fault.
#[trace("TC-056", "FR-040-AC-13")]
#[test]
fn tc_056_reaches_field_refuses_an_invalid_edge_where_it_fails() {
    use CheckedPackageRefusalCause as Cause;
    use CheckedPackageRefusalCode as Code;
    let ineligible = (Code::IllTyped, Cause::OperatorIneligible);
    let name = "body/operation/member/name";
    let first = "body/arguments/0";
    let second = "body/arguments/1";
    let by_reference = |declaring, member, source, target| {
        reaches_package(
            (declaring, SELECTED),
            member,
            Operand::Reference(source),
            Operand::Reference(target),
        )
    };
    let cases = [
        (by_reference(ORDER, "peers", ORDER, ORDER), ineligible, name),
        (by_reference(ORDER, "trail", ORDER, ORDER), ineligible, name),
        (
            by_reference(ORDER, "invoice", ORDER, ORDER),
            ineligible,
            name,
        ),
        (by_reference(ORDER, "total", ORDER, ORDER), ineligible, name),
        (
            by_reference(ORDER, "missing", ORDER, ORDER),
            ineligible,
            name,
        ),
        (
            by_reference(ORDER, "parent", ORDER, INVOICE),
            ineligible,
            second,
        ),
        (by_reference(ORDER, "parent", SUB, ORDER), ineligible, first),
        // An edge to `Sub` does not reach an `Order`, a supertype of its owner.
        (by_reference(SUB, "child", SUB, ORDER), ineligible, second),
        (
            reaches_package(
                (ORDER, SELECTED),
                "parent",
                Operand::Reference(ORDER),
                Operand::Object(ORDER),
            ),
            ineligible,
            second,
        ),
        (
            by_reference(BOTH, "code", BOTH, BOTH),
            (Code::AmbiguousDeclaration, Cause::AmbiguousName),
            name,
        ),
    ];
    for ((package, evidence, position), (code, cause), path) in cases {
        let refusal = refused(&package, &evidence);
        let expected = format!("/semantic_graph/nodes/{position}/{path}");
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(expected.as_str()),
            "{refusal:?}"
        );
        assert_eq!(
            (refusal.code, refusal.cause),
            (code, Some(cause)),
            "{expected}"
        );
    }
    // An unselected owner is now refused at its own node before the
    // application can inspect the member that names it (FR-038-AC-155).
    let (package, evidence, _) = reaches_package(
        (ORDER, OTHER_PACKAGE),
        "parent",
        Operand::Reference("unselected"),
        Operand::Reference("unselected"),
    );
    let unselected = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["owner"]["identity"] == OTHER_PACKAGE)
        .expect("unselected owner");
    let refusal = refused(&package, &evidence);
    assert_eq!(refusal.code, Code::MissingDeclaration);
    assert_eq!(refusal.cause, Some(Cause::MissingSelection));
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some(format!("/semantic_graph/nodes/{unselected}/node_id").as_str())
    );
}

// --- FR-038: the typed relationship end of a reaches application ---

fn admitted(
    package: &Value,
    evidence: &quire_contract_model::CheckedPackageEvidence,
) -> CheckedPackageV2 {
    match read(package, evidence) {
        CheckedPackageV2ReadResult::Admitted(admitted) => *admitted,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn typed_id(key: &str) -> CheckedNodeId {
    serde_json::from_value(node_id(key)).expect("a node id")
}

/// The node of the first application of `operation` in the admitted graph.
fn application_of(package: &CheckedPackageV2, operation: &str) -> CheckedNodeId {
    package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            matches!(&node.body, quire_contract_model::CheckedTerm::Application { operation: record, .. }
                if record.identity() == Some(operation))
        })
        .expect("an application of the operation")
        .node_id
        .clone()
}

/// The relationship declaration node's key, built from the authored identity.
fn relationship_id() -> CheckedNodeId {
    typed_id(&format!("relationship|{RELATIONSHIP}"))
}

fn end_json(role: Option<&str>, lower: u64, upper: u64, unique: bool, ordered: bool) -> Value {
    let mut end = json!({"type": ORDER, "multiplicity":
        {"lower": lower, "upper": upper, "ordered": ordered, "unique": unique}});
    if let Some(role) = role {
        end["role"] = json!(role);
    }
    end
}

/// A bidirectional self-relationship of `Order` with the given ends.
fn relationship_of(source: Value, target: Value) -> Value {
    let mut relation = authored_relationship();
    relation["sourceEnd"] = source;
    relation["targetEnd"] = target;
    relation
}

/// The fields of the expected result, independent of the accessor.
fn expected_end(
    application: &CheckedNodeId,
    destination: &CheckedNodeId,
    role: Option<&str>,
    bounds: (u64, u64, bool),
    forward: bool,
) -> CheckedRelationshipEnd {
    CheckedRelationshipEnd {
        application: application.clone(),
        relationship: relationship_id(),
        role: role.map(Box::from),
        destination_type: destination.clone(),
        lower: bounds.0,
        upper: Some(bounds.1),
        ordered: false,
        unique: bounds.2,
        forward,
    }
}

fn order_declaration_id() -> CheckedNodeId {
    typed_id(&declaration_key_of(ORDER, IDENTITY))
}

/// The reaches package of `relation` naming `role`, admitted.
fn admitted_reaching(relation: Value, role: &str) -> (CheckedPackageV2, CheckedNodeId) {
    let (package, evidence) = relationship_operation_package(relation, role, true);
    let package = admitted(&package, &evidence);
    let application = application_of(&package, "quire.op.model.reaches");
    (package, application)
}

#[trace("TC-048", "FR-038-AC-262")]
#[test]
fn tc_048_relationship_end_reads_forward_and_inverse_ends_of_unequal_bounds() {
    let relation = relationship_of(
        end_json(Some("links"), 1, 1, false, false),
        end_json(Some("peer"), 2, 3, true, false),
    );
    let order = order_declaration_id();
    let (package, forward) = admitted_reaching(relation.clone(), "links");
    assert_eq!(
        package.relationship_end(&forward),
        Ok(expected_end(
            &forward,
            &order,
            Some("peer"),
            (2, 3, true),
            true
        ))
    );
    let (package, inverse) = admitted_reaching(relation, "peer");
    assert_eq!(
        package.relationship_end(&inverse),
        Ok(expected_end(
            &inverse,
            &order,
            Some("links"),
            (1, 1, false),
            false
        ))
    );
}

#[trace("TC-048", "FR-038-AC-263")]
#[test]
fn tc_048_relationship_end_returns_the_declared_bounds_role_and_derived_type() {
    let order = order_declaration_id();
    for (lower, upper, unique) in [
        (0, 3, true),
        (0, 3, false),
        (2, 3, true),
        (0, 1, true),
        (1, 1, true),
    ] {
        let relation = relationship_of(
            end_json(Some("links"), 1, 1, true, false),
            end_json(Some("peer"), lower, upper, unique, false),
        );
        let (package, application) = admitted_reaching(relation, "links");
        assert_eq!(
            package.relationship_end(&application),
            Ok(expected_end(
                &application,
                &order,
                Some("peer"),
                (lower, upper, unique),
                true
            )),
            "{lower}..{upper} unique={unique}"
        );
    }
    let relation = relationship_of(
        end_json(Some("links"), 1, 1, true, false),
        end_json(None, 0, 3, true, false),
    );
    let (package, application) = admitted_reaching(relation, "links");
    assert_eq!(
        package.relationship_end(&application),
        Ok(expected_end(&application, &order, None, (0, 3, true), true))
    );

    // A subtype operand does not change the declared end type.
    let (subtype, evidence) = reaches_over(
        authored_relationship(),
        "links",
        PRIORITY_ORDER,
        PREFERRED_ORDER,
    );
    let package = admitted(&subtype, &evidence);
    let application = application_of(&package, "quire.op.model.reaches");
    assert_eq!(
        package.relationship_end(&application),
        Ok(expected_end(
            &application,
            &order,
            Some("linkedBy"),
            (1, 1, true),
            true
        ))
    );

    // The same package without any graph node holding the declared end type.
    let mut missing = subtype;
    let nodes = missing["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let mut removed = vec![serde_json::to_value(&order).expect("structural order id")];
    loop {
        let before = nodes.len();
        nodes.retain(|node| {
            let key = node["node_id"].clone();
            let gone = |id: &Value| removed.contains(id);
            let depends = node["dependencies"]
                .as_array()
                .expect("dependencies")
                .iter()
                .any(gone);
            let typed_at_removed = gone(&node["semantic_type"]) && node["semantic_type"] != key;
            if depends || typed_at_removed || removed.contains(&key) {
                removed.push(key);
                false
            } else {
                true
            }
        });
        if nodes.len() == before {
            break;
        }
    }
    rebuild_source_map(&mut missing);
    refresh_identity(&mut missing);
    let package = admitted(&missing, &evidence);
    assert!(
        !package
            .graph()
            .nodes
            .iter()
            .any(|node| node.node_id == order),
        "the graph holds no node of the declared end type"
    );
    let application = application_of(&package, "quire.op.model.reaches");
    assert_eq!(
        package.relationship_end(&application),
        Ok(expected_end(
            &application,
            &order,
            Some("linkedBy"),
            (1, 1, true),
            true
        ))
    );
}

#[trace("TC-048", "FR-038-AC-264")]
#[test]
fn tc_048_relationship_end_refuses_every_node_that_is_not_a_reaches_application() {
    let refusal = Err(CheckedRelationshipEndError::NotReachesApplication);
    let (navigate, evidence) = relationship_package(authored_relationship(), "links");
    let package = admitted(&navigate, &evidence);
    assert_eq!(
        package.relationship_end(&typed_id(&"f".repeat(64))),
        Err(CheckedRelationshipEndError::UnknownNode)
    );
    for (label, node) in [
        (
            "navigate",
            application_of(&package, "quire.op.model.navigate"),
        ),
        ("relationship", relationship_id()),
        ("object declaration", order_declaration_id()),
    ] {
        assert_eq!(package.relationship_end(&node), refusal, "{label}");
    }

    let (project, evidence) = crate::checked_package_v2_model_fields::record_project_control();
    let package = admitted(&project, &evidence);
    let read = application_of(&package, "quire.op.record.project");
    assert_eq!(package.relationship_end(&read), refusal, "field read");

    let package = crate::checked_package_v2_model_fields::integer_add_control();
    let add = application_of(&package, "quire.op.integer.add");
    assert_eq!(package.relationship_end(&add), refusal, "integer.add");
    let literal = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_tag.as_ref() == "value" && node.semantic_form.as_ref() == "literal")
        .expect("a literal node")
        .node_id
        .clone();
    assert_eq!(package.relationship_end(&literal), refusal, "literal");
}

#[trace("TC-048", "FR-038-AC-266")]
#[test]
fn tc_048_retaining_the_relationship_end_changes_neither_equality_nor_work() {
    let relation = relationship_of(
        end_json(Some("links"), 1, 1, false, false),
        end_json(Some("peer"), 2, 3, true, false),
    );
    let (package, evidence) = relationship_operation_package(relation, "links", true);
    let first = admitted(&package, &evidence);
    let second = admitted(&package, &evidence);
    let application = application_of(&first, "quire.op.model.reaches");
    let clone = first.clone();
    let result = first.relationship_end(&application);
    assert!(result.is_ok());
    assert_eq!(first.relationship_end(&application), result);
    assert_eq!(clone.relationship_end(&application), result);
    assert_eq!(second.relationship_end(&application), result);
    assert_eq!(first, clone);
    assert_eq!(first, second);
    // `package_id()` reads the wire and the reader checks it before retention is
    // built, so no retention change can alter it: it is not asserted here. Equal
    // admissions carry equal retention, so the equality above cannot catch
    // retention leaking into equality; the in-crate test
    // `tc_048_admitted_retention_is_read_and_outside_package_equality` does, by
    // comparing an admitted package with one whose retention is cleared. The
    // charged work (below) is the guard this test owns.

    // The accessor leaves the authored package and its exact admission work
    // boundary intact. The independent per-visit tally is TC-226/AC-149.
    let bytes = canonical(&package);
    let read_at = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        CheckedPackageV2::read(&bytes, limits, &evidence)
    };
    let (mut insufficient, mut sufficient) = (0, CheckedPackageReadLimits::bounded().work);
    while insufficient + 1 < sufficient {
        let middle = insufficient + (sufficient - insufficient) / 2;
        match read_at(middle) {
            CheckedPackageV2ReadResult::Admitted(_) => sufficient = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => insufficient = middle,
            other => panic!("expected work boundary, got {other:?}"),
        }
    }
    assert!(matches!(
        read_at(sufficient),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    match read_at(sufficient - 1) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(incomplete.limit, sufficient - 1);
            assert_eq!(incomplete.consumed, sufficient);
            assert_eq!(
                incomplete.path.as_ref().map(ToString::to_string).as_deref(),
                Some("/semantic_graph/nodes/6/body/arguments/0/target")
            );
        }
        other => panic!("expected first unpaid work visit, got {other:?}"),
    }
}

#[trace("TC-048", "FR-038-AC-267")]
#[test]
fn tc_048_relationship_end_api_is_nine_fields_and_a_closed_error() {
    let (package, application) = admitted_reaching(authored_relationship(), "links");
    let result = package.relationship_end(&application);
    // The result is built field by field with no `..` rest pattern: a tenth
    // public field, or a missing one, fails to compile.
    let CheckedRelationshipEnd {
        application: read_application,
        relationship,
        role,
        destination_type,
        lower,
        upper,
        ordered,
        unique,
        forward,
    } = result.expect("an admitted reaches application");
    let rebuilt = CheckedRelationshipEnd {
        application: read_application,
        relationship,
        role,
        destination_type,
        lower,
        upper,
        ordered,
        unique,
        forward,
    };
    assert_eq!(rebuilt.application, application);
    let describe = |error: CheckedRelationshipEndError| match error {
        CheckedRelationshipEndError::UnknownNode => "unknown",
        CheckedRelationshipEndError::NotReachesApplication => "not reaches",
        CheckedRelationshipEndError::UnresolvedEnd => "unresolved",
    };
    assert_eq!(
        package
            .relationship_end(&typed_id(&"f".repeat(64)))
            .map_err(describe),
        Err("unknown")
    );
}

#[trace("TC-048", "FR-038-AC-268")]
#[test]
fn tc_048_relationship_end_is_not_a_field_and_exports_no_selected_population_item() {
    let (package, _) = admitted_reaching(authored_relationship(), "links");
    let fields = package
        .model_object_fields(&order_declaration_id())
        .expect("the object's fields");
    assert!(
        fields.fields().is_empty(),
        "a relationship end is not an edge field"
    );
    assert!(fields.field("links").is_none() && fields.field("linkedBy").is_none());

    let root = include_str!("../../crates/quire-contract-model/src/lib.rs");
    let facade = include_str!("../../crates/quire-contract-model/src/selected_population.rs");
    let exported = facade
        .split_once("pub use crate::checked_package::{")
        .expect("the facade names its exports")
        .1
        .split_once("};")
        .expect("the export list closes")
        .0;
    let reexports: String = root
        .split("pub use ")
        .skip(1)
        .map(|item| item.split_once(';').expect("a use item ends").0)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!reexports.contains("selected_population"));
    let words: std::collections::BTreeSet<&str> = reexports
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .collect();
    for item in exported
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        assert!(
            !words.contains(item),
            "{item} must not be re-exported at the model root"
        );
    }
    let ir_root = include_str!("../../src/lib.rs");
    assert!(!ir_root.contains("selected_population"));
}
