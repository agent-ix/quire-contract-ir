// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The published tuple and payload-variant fixture through production intake.

use super::{fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, BoundedDomainForm, CheckedMemberType, CheckedNodeId,
    CheckedNodeKind, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedVariantPayloadType,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

fn selected_document(proposal: &Path) -> Value {
    let documents = super::vector_selected_documents(proposal).expect("selected documents");
    read_json(&proposal.join(super::positive_document(
        "positive-model-tuple-union.json",
        &documents,
    )))
    .expect("selected tuple/union document")
}

fn read_selected_variant_package(package: &Value, document: &Value) -> Box<CheckedPackageV2> {
    let evidence = selected_evidence(document);
    match CheckedPackageV2::read(
        &super::fixture_support::canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        result => panic!("selected model package must admit: {result:?}"),
    }
}

fn selected_evidence(document: &Value) -> CheckedPackageEvidence {
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(
            document["package"]["identity"]
                .as_str()
                .expect("document identity"),
            super::fixture_support::canonical(document),
        )
        .expect("selected document evidence");
    evidence.support_feature("quire.value.complete/v1");
    evidence
}

fn variant_node(package: &Value, identity: &str) -> (usize, CheckedNodeId) {
    let (position, node) = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node["semantic_form"] == "variant_type" && node["owner"]["node"] == identity
        })
        .expect("published selected variant node");
    (
        position,
        serde_json::from_value(node["node_id"].clone()).expect("published variant node id"),
    )
}

fn tuple_node(package: &Value, identity: &str) -> (usize, CheckedNodeId) {
    let (position, node) = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node["semantic_form"] == "tuple_value_type" && node["owner"]["node"] == identity
        })
        .expect("published tuple declaration");
    (
        position,
        serde_json::from_value(node["node_id"].clone()).expect("tuple node id"),
    )
}

fn tuple_positions(document: &Value, identity: &str) -> Vec<(String, String)> {
    document["types"]
        .as_array()
        .expect("selected types")
        .iter()
        .find(|declaration| declaration["identity"] == identity)
        .expect("selected tuple")["positions"]
        .as_array()
        .expect("tuple positions")
        .iter()
        .map(|position| {
            (
                position["identity"]
                    .as_str()
                    .expect("position identity")
                    .to_owned(),
                position["typeRef"]
                    .as_str()
                    .expect("position type")
                    .to_owned(),
            )
        })
        .collect()
}

fn selected_type_position(document: &Value, identity: &str) -> usize {
    document["types"]
        .as_array()
        .expect("selected types")
        .iter()
        .position(|declared| declared["identity"] == identity)
        .expect("selected declaration")
}

fn selected_case_position(document: &Value, type_position: usize, name: &str) -> usize {
    document["types"][type_position]["variants"]
        .as_array()
        .expect("selected variant cases")
        .iter()
        .position(|case| case["name"] == name)
        .expect("selected variant case")
}

type VariantCaseBody = (String, String, usize, usize, Option<String>);

fn asserted_variant_body(
    admitted: &CheckedPackageV2,
    document: &Value,
    node: &CheckedNodeId,
    identity: &str,
    expected_family: &str,
) -> Vec<VariantCaseBody> {
    let view = admitted
        .model_variant_body(node)
        .expect("selected variant body");
    assert_eq!(
        view.selected_document(),
        document["package"]["identity"]
            .as_str()
            .expect("document identity")
    );
    assert_eq!(view.family(), expected_family, "{identity}: variant family");
    let authored = document["types"]
        .as_array()
        .expect("selected types")
        .iter()
        .find(|declared| declared["identity"] == identity)
        .expect("selected variant declaration");
    let mut expected: Vec<VariantCaseBody> = authored["variants"]
        .as_array()
        .expect("authored variant cases")
        .iter()
        .enumerate()
        .map(|(position, case)| {
            let payload = case["payloadType"].as_str().map(str::to_owned);
            (
                case["name"].as_str().expect("case name").to_owned(),
                case["identity"].as_str().expect("case identity").to_owned(),
                position,
                usize::from(payload.is_some()),
                payload,
            )
        })
        .collect();
    expected.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let actual: Vec<VariantCaseBody> = view
        .cases()
        .map(|case| {
            let payload = case.payload_type().map(|type_ref| match type_ref {
                CheckedVariantPayloadType::Native(reference)
                | CheckedVariantPayloadType::Unit(reference)
                | CheckedVariantPayloadType::Declared(reference) => reference.to_owned(),
            });
            (
                case.name().to_owned(),
                case.identity().to_owned(),
                case.position(),
                case.arity(),
                payload,
            )
        })
        .collect();
    assert_eq!(
        actual, expected,
        "{identity}: complete retained variant body"
    );
    actual
}

/// Trace: FR-069-AC-4. Compare every published case through the public reader,
/// then change only the selected variant body while its authored node stays put.
#[trace("FR-069-AC-4")]
#[test]
fn published_variant_bodies_and_first_payload_transition_are_retained() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    let package = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let document = selected_document(&proposal);
    let admitted = read_selected_variant_package(&package, &document);
    let mut seen = BTreeSet::new();
    for row in vectors["family_cases"]
        .as_array()
        .expect("published family cases")
    {
        let id = row["name"].as_str().expect("family case id");
        if !matches!(id, "TF-02" | "TF-03") {
            continue;
        }
        assert!(seen.insert(id), "duplicate published family case {id}");
        let name = row["node"].as_str().expect("variant name");
        let identity = format!("ix://acme/shapes/{name}");
        let (_, node) = variant_node(&package, &identity);
        let expected_family = row["expected"].as_str().expect("published family");
        asserted_variant_body(&admitted, &document, &node, &identity, expected_family);
    }
    assert_eq!(seen, BTreeSet::from(["TF-02", "TF-03"]));

    let color = "ix://acme/shapes/Color";
    let (color_node_position, node) = variant_node(&package, color);
    let original_path = node.path.clone();
    let color_type_position = selected_type_position(&document, color);
    let green_position = selected_case_position(&document, color_type_position, "Green");
    let nullary_body = asserted_variant_body(&admitted, &document, &node, color, "enum");
    let mut payload_document = document.clone();
    payload_document["types"][color_type_position]["variants"][green_position]["payloadType"] =
        json!("ix://quire/native/Boolean");
    assert_ne!(payload_document, document, "selected document must change");
    let mut payload_package = package.clone();
    let graph_nodes = payload_package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes");
    let authored_color_id = &graph_nodes[color_node_position]["node_id"];
    let red_position = graph_nodes
        .iter()
        .position(|candidate| {
            candidate["semantic_form"] == "enum_value"
                && candidate["semantic_type"] == *authored_color_id
                && candidate["body"]["value"] == "Red"
        })
        .expect("published Color Red enum value");
    let enum_equality_position = graph_nodes
        .iter()
        .position(|candidate| {
            candidate["semantic_form"] == "binary"
                && candidate["body"]["operation"]["identity"] == "quire.op.enum.eq"
                && candidate["dependencies"]
                    .as_array()
                    .is_some_and(|dependencies| {
                        dependencies.len() == 2
                            && dependencies.iter().all(|dependency| {
                                graph_nodes.iter().any(|operand| {
                                    operand["node_id"] == *dependency
                                        && operand["semantic_type"] == *authored_color_id
                                })
                            })
                    })
        })
        .expect("published Color enum equality");
    let red = &mut payload_package["semantic_graph"]["nodes"][red_position];
    red["semantic_form"] = json!("union_value");
    red["body"] = json!({"term":"aggregate", "members":[{
        "term":"binding", "name":"Red", "value":{"term":"aggregate","members":[]}
    }]});
    red["dependencies"] = json!([]);
    red["node_id"]["path"] = json!(["value", "union_value", "Red", {"node":color_node_position}]);
    red["occurrences"][0]["role"] = json!("expression");
    let removed = payload_package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .remove(enum_equality_position);
    assert_eq!(removed["body"]["operation"]["identity"], "quire.op.enum.eq");
    super::fixture_support::settle(&mut payload_package);
    let payload_admitted = read_selected_variant_package(&payload_package, &payload_document);
    let payload_body =
        asserted_variant_body(&payload_admitted, &payload_document, &node, color, "union");
    assert_ne!(
        payload_body, nullary_body,
        "first payload changes retained body"
    );
    assert_eq!(node.path, original_path, "authored declaration path");
    assert_eq!(
        admitted.structural_key(&node),
        Some(&node),
        "base authored structural identity"
    );
    assert_eq!(
        payload_admitted.structural_key(&node),
        Some(&node),
        "authored structural identity remains stable"
    );

    let mut aliased_document = payload_document.clone();
    let alias = "ix://acme/shapes/BoolAlias";
    let origin = aliased_document["types"][color_type_position]["origin"].clone();
    aliased_document["types"]
        .as_array_mut()
        .expect("selected types")
        .push(json!({
            "identity": alias, "displayName": "BoolAlias", "kind": "alias",
            "target": "ix://quire/native/Boolean", "origin": origin,
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject"
        }));
    aliased_document["types"][color_type_position]["variants"][green_position]["payloadType"] =
        json!(alias);
    let aliased = read_selected_variant_package(&payload_package, &aliased_document);
    let green = aliased
        .model_variant_body(&node)
        .expect("aliased selected variant")
        .cases()
        .find(|case| case.name() == "Green")
        .expect("Green case");
    assert_eq!(
        green.payload_type(),
        Some(CheckedVariantPayloadType::Native(
            "ix://quire/native/Boolean"
        )),
        "payload alias resolves before retention"
    );

    payload_document["types"][color_type_position]["variants"][green_position]
        .as_object_mut()
        .expect("authored case")
        .remove("payloadType")
        .expect("added payload");
    assert_eq!(
        payload_document, document,
        "removed payload restores document"
    );
    let restored = read_selected_variant_package(&package, &payload_document);
    assert_eq!(
        asserted_variant_body(&restored, &payload_document, &node, color, "enum"),
        nullary_body,
        "removing first payload restores complete retained body"
    );
    let non_variant = admitted
        .graph()
        .nodes
        .iter()
        .find(|candidate| candidate.semantic_form.as_ref() == "model_root")
        .expect("published model root");
    assert!(admitted.model_variant_body(&non_variant.node_id).is_none());
    let mut unselected = node.clone();
    unselected.ordinal += 1;
    assert!(admitted.model_variant_body(&unselected).is_none());
}

/// The refusal pointer of the first selected model root: its owner.
fn root_owner_pointer(package: &Value) -> String {
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| node["semantic_form"] == "model_root")
        .expect("a selected model root");
    format!("/semantic_graph/nodes/{position}/owner")
}

/// The published TK-36 bounds, retained in its selected value type and in a
/// real rational-range graph node so public package views can be compared.
fn tk36_rational_shape(package: &mut Value, row: &Value, integer_node: &Value) -> CheckedNodeId {
    let value_type = row["patch"]
        .as_array()
        .expect("TK-36 patches")
        .iter()
        .find(|patch| patch["path"] == "/value_types")
        .expect("TK-36 selected value type")["value"]
        .as_array()
        .expect("TK-36 value types")
        .first()
        .expect("TK-36 Ratio");
    let bound = |name: &str| value_type[name].as_str().expect("published Rational bound");
    let bounds = [
        bound("numerator_min"),
        bound("numerator_max"),
        bound("denominator_min"),
        bound("denominator_max"),
    ];
    assert_eq!(bounds, ["-3", "7", "1", "12"], "published TK-36 bounds");
    let integer_index = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("model nodes")
        .len();
    let integer_id = integer_node["node_id"].clone();
    let mut rational_node = integer_node.clone();
    rational_node["node_id"]["path"] = json!(["scalar_type", "rational"]);
    let rational_id = rational_node["node_id"].clone();
    rational_node["semantic_type"] = rational_id.clone();
    rational_node["semantic_form"] = json!("rational");
    let mut range_node = integer_node.clone();
    range_node["node_id"]["path"] = json!([
        "bounded_domain",
        "rational_range",
        "numerator_min",
        {"node":integer_index},
        bounds[0],
        "numerator_max",
        bounds[1],
        "denominator_min",
        bounds[2],
        "denominator_max",
        bounds[3]
    ]);
    let range_id = range_node["node_id"].clone();
    range_node["node_tag"] = json!("bounded_domain");
    range_node["semantic_form"] = json!("rational_range");
    range_node["semantic_type"] = rational_id;
    let members = [
        "numerator_min",
        "numerator_max",
        "denominator_min",
        "denominator_max",
    ]
    .into_iter()
    .zip(bounds)
    .map(|(name, value)| {
        json!({
            "term":"binding","name":name,"value":{
                "term":"literal","type":integer_id,"value_kind":"integer","value":value
            }
        })
    })
    .collect::<Vec<_>>();
    range_node["body"] = json!({"term":"aggregate","members":members});
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .extend([integer_node.clone(), rational_node, range_node]);
    serde_json::from_value(range_id).expect("TK-36 rational range node ID")
}

fn assert_tk36_resolved(
    admitted: &CheckedPackageV2,
    package: &Value,
    document: &Value,
    range_id: &CheckedNodeId,
) {
    let tuple_identity = document["types"][1]["identity"]
        .as_str()
        .expect("Pair identity");
    let tuple = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .find(|node| {
            node["semantic_form"] == "tuple_value_type" && node["owner"]["node"] == tuple_identity
        })
        .expect("Pair model node");
    let tuple_id: CheckedNodeId =
        serde_json::from_value(tuple["node_id"].clone()).expect("Pair node ID");
    let variant_identity = document["types"][5]["identity"]
        .as_str()
        .expect("Shape identity");
    let (_, variant_id) = variant_node(package, variant_identity);
    let payload_case = document["types"][5]["variants"][0]["name"]
        .as_str()
        .expect("Shape first case");
    let expected = Some(CheckedMemberType::RationalRange {
        numerator_min: -3,
        numerator_max: 7,
        denominator_min: 1,
        denominator_max: 12,
    });
    assert_eq!(
        admitted.model_tuple_position_type(&tuple_id, 0),
        expected,
        "TK-36 tuple position retains exact Rational bounds"
    );
    assert_eq!(
        admitted.model_variant_payload_member_type(&variant_id, payload_case),
        expected,
        "TK-36 variant payload retains exact Rational bounds"
    );
    let (range, kind) = admitted
        .graph()
        .nodes
        .iter()
        .zip(admitted.node_kinds())
        .find(|(node, _)| &node.node_id == range_id)
        .expect("admitted rational range node");
    assert_eq!(
        *kind,
        CheckedNodeKind::BoundedDomain(BoundedDomainForm::RationalRange)
    );
    let body = serde_json::to_value(&range.body).expect("typed rational range body");
    assert_eq!(body["term"], "aggregate");
    assert_eq!(
        body["members"]
            .as_array()
            .expect("rational range members")
            .len(),
        4
    );
    for (index, (name, value)) in [
        ("numerator_min", "-3"),
        ("numerator_max", "7"),
        ("denominator_min", "1"),
        ("denominator_max", "12"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(body["members"][index]["name"], name);
        assert_eq!(body["members"][index]["value"]["value_kind"], "integer");
        assert_eq!(body["members"][index]["value"]["value"], value);
    }
}

/// Apply one published RFC 6902 operation. The vector's patch is the source of
/// both the mutation and its expected package identity.
pub(super) fn apply_wire_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    let operation = patch["op"].as_str().expect("published patch operation");
    match (operation, target) {
        ("add", Value::Array(items)) if member == "-" => items.push(patch["value"].clone()),
        ("replace", Value::Object(map)) => {
            *map.get_mut(member).expect("published replacement target") = patch["value"].clone();
        }
        ("replace", Value::Array(items)) => {
            let index = member.parse::<usize>().expect("published array index");
            *items.get_mut(index).expect("published replacement target") = patch["value"].clone();
        }
        ("add", Value::Object(map)) => {
            assert!(map
                .insert(member.to_owned(), patch["value"].clone())
                .is_none());
        }
        ("remove", Value::Object(map)) => {
            assert!(map.remove(member).is_some(), "published removal target");
        }
        ("remove", Value::Array(items)) => {
            let index = member.parse::<usize>().expect("published array index");
            assert!(index < items.len(), "published removal target");
            items.remove(index);
        }
        _ => panic!("unsupported published patch {operation} at {path}"),
    }
}

/// Replay every published wire row, including rows added after this test.
#[trace("TC-048", "FR-069-AC-3", "FR-069-AC-5", "FR-069-AC-8")]
#[test]
fn published_model_tuple_union_wire_cases_match_reader_refusals() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    assert_eq!(vectors["base"], "fixtures/positive-model-tuple-union.json");
    let selected = selected_document(&proposal);
    let base = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(
            selected["package"]["identity"]
                .as_str()
                .expect("document identity"),
            super::fixture_support::canonical(&selected),
        )
        .expect("selected document evidence");
    evidence.support_feature("quire.value.complete/v1");
    let rows = vectors["wire_cases"]
        .as_array()
        .expect("published wire cases");
    let mut seen = BTreeSet::new();
    let mut mismatches = Vec::new();
    for row in rows {
        let id = row["id"].as_str().expect("published wire case id");
        assert!(seen.insert(id), "duplicate published wire case {id}");
        let mut package = base.clone();
        for patch in row["patch"].as_array().expect("published patch list") {
            apply_wire_patch(&mut package, patch);
        }
        let result = CheckedPackageV2::read(
            &super::fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        let expected = row["outcome"].as_str().expect("published outcome");
        match (expected, result) {
            ("admitted", CheckedPackageV2ReadResult::Admitted(_)) => {}
            (expected, CheckedPackageV2ReadResult::Refused(refusal))
                if expected.strip_prefix("refused:") == Some(super::code_word(refusal.code)) =>
            {
                if refusal.cause.map(super::cause_word) != row["cause"].as_str() {
                    mismatches.push(format!("{id}: wrong refusal cause {refusal:?}"));
                }
                let actual_path = refusal.loci[0].path.as_ref().map(ToString::to_string);
                if actual_path.as_deref() != row["locus"].as_str() {
                    mismatches.push(format!(
                        "{id}: expected locus {}, got {refusal:?}",
                        row["locus"]
                    ));
                }
            }
            (expected, CheckedPackageV2ReadResult::Admitted(_)) => {
                mismatches.push(format!("{id}: expected {expected}, got admitted"));
            }
            (expected, CheckedPackageV2ReadResult::Refused(refusal)) => {
                mismatches.push(format!("{id}: expected {expected}, got {refusal:?}"));
            }
            (expected, actual) => {
                mismatches.push(format!("{id}: expected {expected}, got {actual:?}"))
            }
        }
    }
    for ordinal in (0..=26).filter(|ordinal| *ordinal != 5) {
        let id = format!("TW-{ordinal:02}");
        assert!(
            seen.contains(id.as_str()),
            "missing published wire case {id}"
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// Translate the published projection mutation to the corresponding member of
/// QSpec's full Semantic IR document. The projection ordinals are represented
/// by position identities in that document.
fn apply_published_intake_patch(document: &mut Value, row: &Value) {
    let name = row["name"].as_str().expect("published intake case name");
    let patches = row["patch"].as_array().expect("published intake patch");
    for patch in patches {
        let path = patch["path"].as_str().expect("published patch path");
        let operation = patch["op"].as_str().expect("published patch operation");
        let value = &patch["value"];
        if path == "/value_types" {
            assert_eq!(operation, "add", "{name}: value-type patch");
            for declared in value.as_array().expect("published value types") {
                let identity = declared["node"].as_str().expect("value type identity");
                let native = declared["native"].as_str().expect("value type native");
                let scalar = match native {
                    "ix://quire/native/Text" => "string",
                    "ix://quire/native/Decimal" => "decimal",
                    "ix://quire/native/Rational" => "rational",
                    _ => panic!("{name}: unsupported published value type {native}"),
                };
                let origin = document["types"][1]["origin"].clone();
                let mut constraints = Vec::new();
                for (member, keyword) in [
                    (
                        "lower",
                        if scalar == "string" {
                            "minLength"
                        } else {
                            "min"
                        },
                    ),
                    (
                        "upper",
                        if scalar == "string" {
                            "maxLength"
                        } else {
                            "max"
                        },
                    ),
                ] {
                    if let Some(bound) = declared[member].as_str() {
                        let bound = if scalar == "string" {
                            json!(bound.parse::<u64>().expect("published length bound"))
                        } else {
                            json!(bound)
                        };
                        constraints.push(json!({
                            "identity":format!("{identity}/constraint/{keyword}"),
                            "appliesTo":identity,
                            "diagnosticCode":format!("acme.shapes.{}_{}", identity.rsplit('/').next().expect("name").to_ascii_uppercase(), keyword.to_ascii_uppercase()),
                            "keyword":keyword,"operands":{"value":bound},"origin":origin
                        }));
                    }
                }
                for (member, keyword) in [
                    ("numerator_min", "minNumerator"),
                    ("numerator_max", "maxNumerator"),
                    ("denominator_min", "minDenominator"),
                    ("denominator_max", "maxDenominator"),
                ] {
                    if let Some(bound) = declared[member].as_str() {
                        let bound = bound.parse::<i64>().expect("published rational bound");
                        constraints.push(json!({
                            "identity":format!("{identity}/constraint/{keyword}"),
                            "appliesTo":identity,
                            "diagnosticCode":format!("acme.shapes.{}_{}", identity.rsplit('/').next().expect("name").to_ascii_uppercase(), keyword.to_ascii_uppercase()),
                            "keyword":keyword,"operands":{"value":bound},"origin":origin
                        }));
                    }
                }
                if let Some(profile) = declared["profile"].as_str() {
                    constraints.push(json!({
                        "identity":format!("{identity}/constraint/format"),"appliesTo":identity,
                        "diagnosticCode":format!("acme.shapes.{}_PROFILE", identity.rsplit('/').next().expect("name").to_ascii_uppercase()),
                        "keyword":"format","operands":{"name":format!("text-profile:{profile}")},"origin":origin
                    }));
                }
                let kind = json!({"module":"acme/shapes", "name":"value_type"});
                if !document["constructs"]
                    .as_array()
                    .expect("constructs")
                    .iter()
                    .any(|entry| entry["kind"] == kind)
                {
                    document["constructs"].as_array_mut().expect("constructs").push(json!({
                        "kind":kind, "moduleVersion":"1.0.0",
                        "construct":{"identity":"value","meaning":"quire.meaning.model.value-type/v1","shape":"record","members":{}}
                    }));
                }
                let mut value_type = json!({
                    "identity":identity,"displayName":identity.rsplit('/').next().expect("name"),
                    "kind":kind,"scalar":scalar,"constraints":constraints,"origin":origin,
                    "roles":[],"extensions":[],"unknownPolicy":"reject"
                });
                if let Some(unit) = declared.get("unit") {
                    value_type["unit"] = unit.clone();
                }
                document["types"]
                    .as_array_mut()
                    .expect("types")
                    .push(value_type);
            }
            continue;
        }
        if path == "/aliases" {
            assert_eq!(operation, "add", "{name}: alias patch");
            for alias in value.as_array().expect("published aliases") {
                let identity = alias["node"].as_str().expect("alias identity");
                let origin = document["types"][1]["origin"].clone();
                document["types"].as_array_mut().expect("types").push(json!({
                    "identity":identity,"displayName":identity.rsplit('/').next().expect("name"),
                    "kind":"alias","target":alias["target"],"origin":origin,
                    "roles":[],"constraints":[],"extensions":[],"unknownPolicy":"reject"
                }));
            }
            continue;
        }
        let segments: Vec<_> = path.trim_start_matches('/').split('/').collect();
        let (group, ordinal, rest) = match segments.as_slice() {
            [group @ ("tuple_types" | "variant_types"), ordinal, rest @ ..] => (
                *group,
                ordinal.parse::<usize>().expect("declaration index"),
                rest,
            ),
            _ => panic!("{name}: unsupported published patch {path}"),
        };
        let kind = if group == "tuple_types" {
            "tuple"
        } else {
            "variant"
        };
        let index = document["types"]
            .as_array()
            .expect("types")
            .iter()
            .enumerate()
            .filter(|(_, declared)| declared["kind"]["name"] == kind)
            .nth(ordinal)
            .map(|(index, _)| index)
            .expect("published declaration");
        let declared = &mut document["types"][index];
        match (group, rest) {
            ("tuple_types", ["positions"]) => {
                assert_eq!(operation, "replace", "{name}: positions patch");
                assert!(
                    value.as_array().expect("positions").is_empty(),
                    "{name}: empty positions"
                );
                declared["positions"] = json!([]);
            }
            ("tuple_types", ["fields"]) => {
                assert_eq!(operation, "add", "{name}: fields patch");
                declared["fields"] = value.clone();
            }
            ("tuple_types", ["positions", slot, "ordinal"]) => {
                assert_eq!(operation, "replace", "{name}: ordinal patch");
                let slot = slot.parse::<usize>().expect("position index");
                let owner = declared["identity"].as_str().expect("tuple identity");
                declared["positions"][slot]["identity"] =
                    json!(format!("{owner}/p{}", value.as_u64().expect("ordinal")));
            }
            ("tuple_types", ["positions", slot, "typeRef"]) => {
                let slot = slot.parse::<usize>().expect("position index");
                if operation == "remove" {
                    assert!(
                        declared["positions"][slot]
                            .as_object_mut()
                            .expect("position")
                            .remove("typeRef")
                            .is_some(),
                        "{name}: removed position type"
                    );
                } else {
                    assert_eq!(operation, "replace", "{name}: position type patch");
                    declared["positions"][slot]["typeRef"] = value.clone();
                }
            }
            ("variant_types", ["cases", slot, "payloadType"]) => {
                assert!(
                    matches!(operation, "replace" | "add"),
                    "{name}: payload patch"
                );
                let slot = slot.parse::<usize>().expect("case index");
                declared["variants"][slot]["payloadType"] = value.clone();
            }
            _ => panic!("{name}: unsupported published patch {path}"),
        }
    }
}

/// Replay every published intake row through the selected document join.
/// Trace: TC-048, FR-069-AC-8.
#[trace("TC-048", "FR-069-AC-8")]
#[test]
fn published_model_tuple_union_intake_cases_refuse_at_exact_member() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    assert_eq!(vectors["base"], "fixtures/positive-model-tuple-union.json");
    let selected = selected_document(&proposal);
    let mut base = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let integer_node = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
        .expect("published Integer node")
        .clone();
    // Intake mutations can change the types of the published expression
    // controls. Keep the model declarations and root for this reader replay.
    base["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("published nodes")
        .retain(|node| node["node_tag"] == "model");
    let model_ids: Vec<_> = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("model nodes")
        .iter()
        .map(|node| node["node_id"].clone())
        .collect();
    base["source_map"]
        .as_array_mut()
        .expect("published source map")
        .retain(|entry| model_ids.contains(&entry["node_id"]));
    let rows = vectors["intake_cases"]
        .as_array()
        .expect("published intake cases");
    let mut seen = BTreeSet::new();
    for row in rows {
        let name = row["name"].as_str().expect("published intake case name");
        assert!(seen.insert(name), "duplicate published intake case {name}");
        let mut document = selected.clone();
        let mut package = base.clone();
        apply_published_intake_patch(&mut document, row);
        let tk36_range =
            (name == "TK-36").then(|| tk36_rational_shape(&mut package, row, &integer_node));
        super::fixture_support::settle(&mut package);

        let mut evidence = CheckedPackageEvidence::new();
        evidence
            .insert_domain_package_document(
                document["package"]["identity"]
                    .as_str()
                    .expect("document identity"),
                super::fixture_support::canonical(&document),
            )
            .expect("selected document evidence");
        evidence.support_feature("quire.value.complete/v1");
        let result = CheckedPackageV2::read(
            &super::fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        if row["expected"]["admitted"] == true {
            let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
                panic!("{name}: expected admission, got {result:?}");
            };
            if let Some(range_id) = tk36_range.as_ref() {
                assert_tk36_resolved(&admitted, &package, &document, range_id);
            }
            continue;
        }
        let expected = &row["expected"]["refused"];
        assert_eq!(
            expected["stage"], "reader",
            "{name}: published reader stage"
        );
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{name}: expected published refusal, got {result:?}");
        };
        assert_eq!(
            super::code_word(refusal.code),
            expected["code"].as_str().expect("code"),
            "{name}"
        );
        assert_eq!(
            refusal.cause.map(super::cause_word),
            expected["cause"].as_str(),
            "{name}"
        );
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(root_owner_pointer(&package).as_str()),
            "{name}"
        );
        let projection_locus = expected["at"].as_str().expect("published refusal locus");
        let document_pointer = if let Some(suffix) = projection_locus.strip_prefix("/tuple_types/0")
        {
            format!("/types/1{suffix}")
        } else if let Some(suffix) = projection_locus.strip_prefix("/variant_types/0/cases") {
            format!("/types/5/variants{suffix}")
        } else {
            panic!("{name}: unexpected published refusal locus {projection_locus}")
        };
        assert_eq!(
            refusal
                .document_pointer
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(document_pointer.as_str()),
            "{name}"
        );
    }
    for ordinal in 1..=35 {
        let id = format!("TK-{ordinal:02}");
        assert!(
            seen.contains(id.as_str()),
            "missing published intake case {id}"
        );
    }
    assert!(
        seen.contains("TK-36"),
        "missing published bounded Rational intake case"
    );
}

/// An admitted Duration position is usable as the
/// Quantity<s> result of a tuple projection when s is one second of its
/// owner's Time. The upstream fixtures are read at test time.
#[test]
fn duration_tuple_position_projects_to_resolved_second_unit() {
    for scenario in [
        DurationCase::Source,
        DurationCase::Selected,
        DurationCase::Missing,
        DurationCase::InvalidSelectedWithSource,
        DurationCase::BothSelected,
        DurationCase::BothSourceResult,
    ] {
        duration_projection_control(scenario);
    }
}

#[derive(Clone, Copy, Debug)]
enum DurationCase {
    Source,
    Selected,
    Missing,
    InvalidSelectedWithSource,
    BothSelected,
    BothSourceResult,
}

fn duration_projection_control(scenario: DurationCase) {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let selected_unit = matches!(
        scenario,
        DurationCase::Selected
            | DurationCase::InvalidSelectedWithSource
            | DurationCase::BothSelected
            | DurationCase::BothSourceResult
    );
    let source_unit = matches!(
        scenario,
        DurationCase::Source
            | DurationCase::InvalidSelectedWithSource
            | DurationCase::BothSelected
            | DurationCase::BothSourceResult
    );
    let selected_valid = !matches!(scenario, DurationCase::InvalidSelectedWithSource);
    let mut selected =
        read_json(&proposal.join("domain-package-acme-shapes.json")).expect("selected Semantic IR");
    selected["types"][1]["positions"][1]["typeRef"] = json!("ix://quire/native/Duration");
    if selected_unit {
        let origin = selected["types"][1]["origin"].clone();
        let dimension = if selected_valid { "Time" } else { "Length" };
        selected["dimensions"] = json!([{
            "identity":format!("ix://acme/shapes/{dimension}"), "name":dimension,
            "base":true, "terms":[], "origin":origin
        }]);
        selected["units"] = json!([{
            "identity":"ix://acme/shapes/unit/s", "name":"s",
            "dimension":format!("ix://acme/shapes/{dimension}"), "scale":"1", "offset":"0",
            "origin":origin
        }]);
    }

    let mut package = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let retained = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 14, 32, 37];
    let original_count = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph")
        .len();
    let mut old_to_new = vec![usize::MAX; original_count];
    for (new, old) in retained.iter().copied().enumerate() {
        old_to_new[old] = new;
    }
    let mut nodes: Vec<Value> = retained
        .iter()
        .map(|index| package["semantic_graph"]["nodes"][*index].clone())
        .collect();
    let ids: BTreeSet<_> = nodes
        .iter()
        .map(|node| node["node_id"].to_string())
        .collect();
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .retain(|entry| ids.contains(&entry["node_id"].to_string()));
    for node in &mut nodes {
        super::node_index::remap_graph_indexes(node, &old_to_new);
    }
    super::node_index::remap_graph_indexes(&mut package["source_map"], &old_to_new);
    let mut nominal_maps = Vec::new();
    let mut source_id = None;
    let mut selected_id = None;
    if source_unit {
        let (pair, entries, unit_id) = duration_nominal_pair(&fixtures, false, true);
        nodes.extend(pair);
        nominal_maps.extend(entries);
        source_id = Some(unit_id);
    }
    if selected_unit {
        let (pair, entries, unit_id) = duration_nominal_pair(&fixtures, true, selected_valid);
        nodes.extend(pair);
        nominal_maps.extend(entries);
        selected_id = Some(unit_id);
    }
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .extend(nominal_maps);
    let result = package["semantic_graph"]["nodes"][11]["body"]["result_type"].clone();
    assert_eq!(result, package["semantic_graph"]["nodes"][8]["node_id"]);
    let result_unit = if matches!(scenario, DurationCase::BothSourceResult) {
        source_id.as_ref()
    } else {
        selected_id.as_ref().or(source_id.as_ref())
    };
    if let Some(unit_id) = result_unit {
        package["semantic_graph"]["nodes"][11]["body"]["result_type"] = unit_id.clone();
        package["semantic_graph"]["nodes"][11]["semantic_type"] = unit_id.clone();
    }
    super::fixture_support::settle(&mut package);
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(
            selected["package"]["identity"]
                .as_str()
                .expect("document identity"),
            super::fixture_support::canonical(&selected),
        )
        .expect("selected evidence");
    evidence.support_feature("quire.value.complete/v1");
    let expected_locus = package["semantic_graph"]["nodes"][11]["node_id"].clone();
    let read = CheckedPackageV2::read(
        &super::fixture_support::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    match (scenario, read) {
        (
            DurationCase::Source | DurationCase::Selected | DurationCase::BothSelected,
            CheckedPackageV2ReadResult::Admitted(_),
        ) => {}
        (DurationCase::Missing, CheckedPackageV2ReadResult::Refused(refusal)) => {
            assert_eq!(refusal.code, Code::MissingDeclaration);
            assert_eq!(refusal.cause, Some(Cause::MissingName));
            assert_eq!(
                refusal.loci[0]
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref(),
                Some("/semantic_graph/nodes/11")
            );
            assert_eq!(
                refusal
                    .locus
                    .as_ref()
                    .map(|id| serde_json::to_value(id).expect("locus")),
                Some(expected_locus.clone())
            );
        }
        (
            DurationCase::InvalidSelectedWithSource | DurationCase::BothSourceResult,
            CheckedPackageV2ReadResult::Refused(refusal),
        ) => {
            assert_eq!(refusal.code, Code::IllTyped);
            assert_eq!(refusal.cause, Some(Cause::OperatorIneligible));
            assert_eq!(
                refusal.loci[0]
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref(),
                Some("/semantic_graph/nodes/11")
            );
            assert_eq!(
                refusal
                    .locus
                    .as_ref()
                    .map(|id| serde_json::to_value(id).expect("locus")),
                Some(expected_locus.clone())
            );
        }
        (case, result) => panic!("{case:?}: unexpected Duration projection outcome {result:?}"),
    }
}

/// Re-key the authoritative timed fixture's Time/s pair for one owner. The
/// selected invalid case uses a base Length so its nominal graph remains
/// valid while its Duration member has no type.
fn duration_nominal_pair(
    fixtures: &Path,
    selected: bool,
    valid: bool,
) -> (Vec<Value>, Vec<Value>, Value) {
    let mut timed =
        read_json(&fixtures.join("positive-timed-model-nodes.json")).expect("published time units");
    let dimension = 42;
    let unit = 43;
    let (id_package, qualified_prefix) = if selected {
        ("acme/shapes", vec!["acme", "shapes"])
    } else {
        // The source-owned implicit second is the unqualified `s`.
        ("example", Vec::new())
    };
    let dimension_name = if valid { "Time" } else { "Length" };
    let dimension_identity = format!("ix://acme/shapes/{dimension_name}");
    let dimension_name = qualified_prefix
        .iter()
        .copied()
        .chain(std::iter::once(dimension_name))
        .collect::<Vec<_>>();
    let old_dimension = timed["semantic_graph"]["nodes"][dimension]["node_id"].clone();
    let new_dimension = super::fixture_support::structural_id(
        id_package,
        if selected {
            vec![
                json!("scalar_type"),
                json!("dimension"),
                json!(dimension_identity),
            ]
        } else {
            dimension_name.iter().map(|part| json!(part)).collect()
        },
        0,
    );
    super::fixture_support::reassign_node_id(&mut timed, &old_dimension, &new_dimension);
    if selected {
        let node = &mut timed["semantic_graph"]["nodes"][dimension];
        node.as_object_mut()
            .expect("dimension node")
            .remove("declaration");
        node["owner"] = json!({"kind":"model","identity":"acme/shapes","node":dimension_identity});
        node["body"] = json!({"term":"aggregate","members":[]});
        node["occurrences"] = json!([{"role":"generated","ordinal":0}]);
    } else {
        timed["semantic_graph"]["nodes"][dimension]["declaration"]["qualified_name"] =
            json!(dimension_name);
    }
    let unit_name = qualified_prefix
        .into_iter()
        .chain(std::iter::once("s"))
        .collect::<Vec<_>>();
    let old_unit = timed["semantic_graph"]["nodes"][unit]["node_id"].clone();
    let new_unit = super::fixture_support::structural_id(
        id_package,
        if selected {
            vec![
                json!("scalar_type"),
                json!("unit"),
                json!("ix://acme/shapes/unit/s"),
            ]
        } else {
            unit_name.iter().map(|part| json!(part)).collect()
        },
        0,
    );
    super::fixture_support::reassign_node_id(&mut timed, &old_unit, &new_unit);
    if selected {
        let node = &mut timed["semantic_graph"]["nodes"][unit];
        node.as_object_mut()
            .expect("unit node")
            .remove("declaration");
        node["owner"] =
            json!({"kind":"model","identity":"acme/shapes","node":"ix://acme/shapes/unit/s"});
        node["body"] = json!({"term":"aggregate","members":[]});
        node["dependencies"] = json!([new_dimension]);
        node["occurrences"] = json!([{"role":"generated","ordinal":0}]);
        assert_eq!(node["semantic_type"], new_dimension);
    } else {
        timed["semantic_graph"]["nodes"][unit]["declaration"]["qualified_name"] = json!(unit_name);
    }
    let nodes: Vec<_> = [dimension, unit]
        .iter()
        .map(|index| timed["semantic_graph"]["nodes"][*index].clone())
        .collect();
    let maps = nodes
        .iter()
        .map(|node| {
            timed["source_map"]
                .as_array()
                .expect("timed source map")
                .iter()
                .find(|entry| entry["node_id"] == node["node_id"])
                .expect("nominal source map")
                .clone()
        })
        .map(|mut entry| {
            if selected {
                entry["role"] = json!("generated");
            }
            entry
        })
        .collect();
    (
        nodes,
        maps,
        timed["semantic_graph"]["nodes"][unit]["node_id"].clone(),
    )
}

/// QSpec FR-322 reads duplicate tuple ordinals before the resulting gap and
/// checks all tuples before any variant payload in one selected document.
/// Provenance: IR-1059 partial AC-6 combined-defect order; full AC-6 remains open.
#[test]
fn combined_tuple_and_variant_defects_keep_published_intake_order() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    let selected = selected_document(&proposal);
    let base = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let rows = vectors["intake_cases"].as_array().expect("intake cases");
    for (first, expected_cause) in [
        ("TK-06", "conflicting-binding"),
        ("TK-02", "malformed-declaration"),
    ] {
        let mut document = selected.clone();
        let mut package = base.clone();
        for name in [first, "TK-07"] {
            let row = rows
                .iter()
                .find(|row| row["name"] == name)
                .expect("published intake row");
            apply_published_intake_patch(&mut document, row);
        }
        super::fixture_support::settle(&mut package);
        let mut evidence = CheckedPackageEvidence::new();
        evidence
            .insert_domain_package_document(
                document["package"]["identity"]
                    .as_str()
                    .expect("document identity"),
                super::fixture_support::canonical(&document),
            )
            .expect("selected document evidence");
        evidence.support_feature("quire.value.complete/v1");
        let result = CheckedPackageV2::read(
            &super::fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{first}: expected tuple defect before variant defect: {result:?}");
        };
        assert_eq!(refusal.code, Code::InvalidModelBinding, "{first}");
        assert_eq!(
            refusal.cause.map(super::cause_word),
            Some(expected_cause),
            "{first}"
        );
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(root_owner_pointer(&package).as_str()),
            "{first}"
        );
        assert_eq!(
            refusal
                .document_pointer
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("/types/1/positions/1"),
            "{first}"
        );
    }
}

/// Provenance: IR-1059 slice 2. Full TC-940 and FR-069 conformance remains open.
#[test]
fn published_model_tuple_union_fixture_admits() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let document = selected_document(&proposal);
    let package = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");

    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(
            document["package"]["identity"]
                .as_str()
                .expect("document identity"),
            super::fixture_support::canonical(&document),
        )
        .expect("selected document evidence");
    evidence.support_feature("quire.value.complete/v1");
    let result = CheckedPackageV2::read(
        &super::fixture_support::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
}

/// Trace: FR-069-AC-1, FR-069-AC-2, TC-940.
#[trace("TC-940")]
#[test]
fn published_tuples_retain_named_identity_and_ordered_selected_positions() {
    let fixtures = fixtures_dir();
    let document = selected_document(&proposal_dir_of(&fixtures));
    let package = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let admitted = read_selected_variant_package(&package, &document);
    let names = ["Pair", "Swap", "Nest", "Cell"];
    let mut paths = BTreeSet::new();
    for name in names {
        let identity = format!("ix://acme/shapes/{name}");
        let (index, id) = tuple_node(&package, &identity);
        let node = &admitted.graph().nodes[index];
        assert_eq!(node.node_id, id, "{name}: structural id");
        assert_eq!(node.node_id.package.as_ref(), "acme/shapes");
        assert_eq!(
            serde_json::to_value(&node.node_id.path).expect("path"),
            json!(["model", "tuple_value_type", identity]),
            "{name}: qualified structural path"
        );
        assert_eq!(
            package["semantic_graph"]["nodes"][index]["owner"]["identity"],
            "acme/shapes"
        );
        assert!(
            paths.insert(id.path.clone()),
            "{name}: unique declaration path"
        );
        let positions = tuple_positions(&document, &identity);
        assert_eq!(positions.len(), 2, "{name}: published arity");
        for (position, (position_identity, type_ref)) in positions.iter().enumerate() {
            assert_eq!(position_identity, &format!("{identity}/p{position}"));
            let expected = match type_ref.as_str() {
                "ix://quire/native/Boolean" => CheckedMemberType::Boolean,
                "ix://quire/native/Integer" => CheckedMemberType::Integer,
                "ix://acme/shapes/Pair" => {
                    CheckedMemberType::Tuple(tuple_node(&package, type_ref).1)
                }
                "ix://acme/shapes/Shape" | "ix://acme/shapes/List" => {
                    CheckedMemberType::Variant(variant_node(&package, type_ref).1)
                }
                other => panic!("unexpected published position type {other}"),
            };
            assert_eq!(
                admitted.model_tuple_position_type(&id, position),
                Some(expected),
                "{name} position {position}"
            );
            for id in package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .map(|node| &node["node_id"])
                .chain(
                    package["source_map"]
                        .as_array()
                        .expect("source map")
                        .iter()
                        .map(|entry| &entry["node_id"]),
                )
            {
                assert!(
                    !id["path"]
                        .as_array()
                        .expect("structural path")
                        .iter()
                        .any(|segment| segment == position_identity),
                    "{name} position {position} must not be a graph or source-map node"
                );
            }
        }
        assert_eq!(
            admitted.model_tuple_position_type(&id, positions.len()),
            None
        );
    }

    let pair = "ix://acme/shapes/Pair";
    let (_, pair_id) = tuple_node(&package, pair);
    let swap = "ix://acme/shapes/Swap";
    let (_, swap_id) = tuple_node(&package, swap);
    let mut same_shape = document.clone();
    let swap_index = selected_type_position(&same_shape, swap);
    let positions = same_shape["types"][swap_index]["positions"]
        .as_array_mut()
        .expect("Swap positions");
    positions.swap(0, 1);
    for (position, row) in positions.iter_mut().enumerate() {
        row["identity"] = json!(format!("{swap}/p{position}"));
    }
    let same_shape_admitted = read_selected_variant_package(&package, &same_shape);
    assert_eq!(
        same_shape_admitted.graph().nodes[tuple_node(&package, swap).0].node_id,
        swap_id
    );
    assert_ne!(
        tuple_positions(&document, swap),
        tuple_positions(&same_shape, swap)
    );
    assert_eq!(
        tuple_positions(&same_shape, swap),
        vec![
            (format!("{swap}/p0"), "ix://quire/native/Boolean".to_owned()),
            (format!("{swap}/p1"), "ix://quire/native/Integer".to_owned()),
        ],
        "reordered declaration retains its exact authored position content"
    );
    assert_ne!(
        pair_id.path, swap_id.path,
        "named tuple paths stay distinct"
    );
    for (position, expected) in [CheckedMemberType::Boolean, CheckedMemberType::Integer]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            same_shape_admitted.model_tuple_position_type(&swap_id, position),
            Some(expected.clone())
        );
        assert_eq!(
            same_shape_admitted.model_tuple_position_type(&pair_id, position),
            Some(expected)
        );
    }

    let mut aliased_document = same_shape.clone();
    let alias = "ix://acme/shapes/BoolAlias";
    let origin = aliased_document["types"][swap_index]["origin"].clone();
    aliased_document["types"]
        .as_array_mut()
        .expect("selected types")
        .push(json!({
            "identity": alias, "displayName": "BoolAlias", "kind": "alias",
            "target": "ix://quire/native/Boolean", "origin": origin,
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject"
        }));
    aliased_document["types"][swap_index]["positions"][0]["typeRef"] = json!(alias);
    let aliased = read_selected_variant_package(&package, &aliased_document);
    let (swap_node_index, _) = tuple_node(&package, swap);
    assert_eq!(aliased.graph().nodes[swap_node_index].node_id, swap_id);
    assert_eq!(
        serde_json::to_value(&aliased.graph().nodes[swap_node_index].node_id.path).expect("path"),
        json!(["model", "tuple_value_type", swap]),
        "alias must not replace the declared qualified name"
    );
    assert_eq!(
        serde_json::to_value(&aliased.graph().nodes[swap_node_index].owner).expect("owner"),
        json!({"kind":"model", "identity":"acme/shapes", "node":swap}),
        "alias must not replace the admitted owner declaration"
    );
    assert_eq!(
        aliased.model_tuple_position_type(&swap_id, 0),
        Some(CheckedMemberType::Boolean)
    );
}

/// Trace: FR-069-AC-7, TC-940.
#[trace("TC-940")]
#[test]
fn published_tuple_union_typed_writer_round_trips() {
    let fixtures = fixtures_dir();
    let document = selected_document(&proposal_dir_of(&fixtures));
    let package = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let admitted = read_selected_variant_package(&package, &document);
    let encoded = encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        quire_contract_model::CheckedPackageSidecarsRef {
            temporal_clause_names: admitted.temporal_clause_names(),
            operation_effects: None,
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
            schedulability_claims: None,
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("typed tuple and union package encodes");
    assert_eq!(encoded, super::fixture_support::canonical(&package));
    let evidence = selected_evidence(&document);
    let reread =
        match CheckedPackageV2::read(&encoded, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(value) => value,
            other => panic!("typed tuple and union readback: {other:?}"),
        };
    assert_eq!(reread.name(), admitted.name());
    assert_eq!(reread.lock(), admitted.lock(), "selection rows");
    for name in ["Pair", "Swap", "Nest", "Cell"] {
        let identity = format!("ix://acme/shapes/{name}");
        let (_, id) = tuple_node(&package, &identity);
        assert!(reread.graph().nodes.iter().any(|node| node.node_id == id));
        for position in 0..tuple_positions(&document, &identity).len() {
            assert_eq!(
                reread.model_tuple_position_type(&id, position),
                admitted.model_tuple_position_type(&id, position)
            );
        }
    }
    for name in ["Shape", "Color", "List"] {
        let identity = format!("ix://acme/shapes/{name}");
        let (_, id) = variant_node(&package, &identity);
        assert!(reread.graph().nodes.iter().any(|node| node.node_id == id));
        let original = admitted.model_variant_body(&id).expect("original variant");
        let roundtrip = reread.model_variant_body(&id).expect("roundtrip variant");
        assert_eq!(roundtrip.selected_document(), original.selected_document());
        assert_eq!(roundtrip.family(), original.family());
        let original_cases: Vec<_> = original.cases().collect();
        let roundtrip_cases: Vec<_> = roundtrip.cases().collect();
        assert_eq!(roundtrip_cases, original_cases);
    }
    for invalid in [
        json!("bad"),
        package["semantic_graph"]["nodes"][0]["node_id"].clone(),
    ] {
        let mut malformed = package.clone();
        malformed["semantic_graph"]["nodes"][1]["node_id"] = invalid;
        let result = CheckedPackageV2::read(
            &super::fixture_support::canonical(&malformed),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Refused(_)),
            "invalid node id must refuse: {result:?}"
        );
    }
}

/// Unsupported selected payloads fail at model intake with the selected
/// document member retained as a public location.
#[test]
fn unsupported_selected_variant_payloads_refuse_at_document_member() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let selected = read_json(&proposal.join("domain-package-acme-shapes.json"))
        .expect("selected full Semantic IR document");
    let base = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    for payload_type in [
        "ix://quire/native/Rational",
        "ix://quire/native/Decimal",
        "ix://quire/native/Text",
        "ix://acme/shapes/Raw",
    ] {
        let mut document = selected.clone();
        let mut package = base.clone();
        if payload_type == "ix://acme/shapes/Raw" {
            document["constructs"]
                .as_array_mut()
                .expect("constructs")
                .push(json!({"kind":{"module":"acme/shapes","name":"raw"},
                    "construct":{"meaning":"quire.meaning.model.value-type/v1"}}));
            document["types"]
                .as_array_mut()
                .expect("types")
                .push(json!({"identity":payload_type,
                    "kind":{"module":"acme/shapes","name":"raw"},
                    "scalar":"integer","constraints":[]}));
        }
        document["types"][5]["variants"][0]["payloadType"] = json!(payload_type);
        super::fixture_support::settle(&mut package);

        let mut evidence = CheckedPackageEvidence::new();
        evidence
            .insert_domain_package_document(
                document["package"]["identity"]
                    .as_str()
                    .expect("document identity"),
                super::fixture_support::canonical(&document),
            )
            .expect("selected document evidence");
        evidence.support_feature("quire.value.complete/v1");
        let result = CheckedPackageV2::read(
            &super::fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{payload_type}: expected unsupported payload intake refusal: {result:?}");
        };
        assert_eq!(refusal.code, Code::UnsupportedConstruct, "{payload_type}");
        assert_eq!(
            refusal.cause,
            Some(Cause::DeclarationForm),
            "{payload_type}"
        );
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(root_owner_pointer(&package).as_str()),
            "{payload_type}",
        );
        assert_eq!(
            refusal
                .document_pointer
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("/types/5/variants/0/payloadType"),
            "{payload_type}",
        );
    }
}

/// Published projection rows drive the selected declaration and result type;
/// unrelated expression controls are removed so they cannot mask the join.
#[trace("TC-940", "FR-069-AC-3")]
#[test]
fn published_tuple_positions_execute_the_reader_application_join() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple vectors");
    let document = selected_document(&proposal);
    let base =
        read_json(&fixtures.join("positive-model-tuple-union.json")).expect("published package");
    let template = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.tuple.project")
        .expect("published projection")
        .clone();
    let parameter_id = template["body"]["arguments"][0]["target"].clone();
    let mut seen = BTreeSet::new();
    for row in vectors["position_cases"]
        .as_array()
        .expect("published positions")
    {
        let name = row["name"].as_str().expect("row name");
        assert!(seen.insert(name), "duplicate {name}");
        let identity = format!("ix://acme/shapes/{}", row["tuple"].as_str().expect("tuple"));
        let (_, declaration) = tuple_node(&base, &identity);
        let mut package = base.clone();
        let nodes = package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        nodes.retain(|node| node["node_tag"] != "expression");
        nodes
            .iter_mut()
            .find(|node| node["node_id"] == parameter_id)
            .expect("projection parameter")["semantic_type"] =
            serde_json::to_value(&declaration).expect("id");
        let mut application = template.clone();
        application["body"]["operation"]["member"]["declaration"] =
            serde_json::to_value(&declaration).expect("id");
        application["body"]["operation"]["member"]["position"] = row["position"].clone();
        if let Some(position) = row["position"]
            .as_u64()
            .and_then(|position| usize::try_from(position).ok())
        {
            if let Some((_, type_ref)) = tuple_positions(&document, &identity).get(position) {
                let result = base["semantic_graph"]["nodes"]
                    .as_array()
                    .expect("nodes")
                    .iter()
                    .find(|node| match type_ref.as_str() {
                        "ix://quire/native/Boolean" => {
                            node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean"
                        }
                        "ix://quire/native/Integer" => {
                            node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer"
                        }
                        _ => node["node_tag"] == "model" && node["owner"]["node"] == *type_ref,
                    })
                    .expect("published result type")["node_id"]
                    .clone();
                application["semantic_type"] = result.clone();
                application["body"]["result_type"] = result;
            }
        }
        nodes.push(application);
        super::fixture_support::settle(&mut package);
        let result = CheckedPackageV2::read(
            &super::fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &selected_evidence(&document),
        );
        if row["expected"].get("refused").is_some() {
            let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                panic!("{name}: must refuse: {result:?}")
            };
            assert_eq!(refusal.code, Code::IllTyped, "{name}");
            assert_eq!(refusal.cause, Some(Cause::OperatorIneligible), "{name}");
            assert!(
                refusal.loci[0]
                    .path
                    .as_ref()
                    .expect("member locus")
                    .to_string()
                    .ends_with("/body/operation/member"),
                "{name}: {refusal:?}"
            );
        } else {
            let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
                panic!("{name}: {result:?}")
            };
            let position = usize::try_from(row["position"].as_u64().expect("position"))
                .expect("host position");
            let expected = &row["expected"]["member_type"];
            let expected_type = match expected["type"].as_str() {
                Some("Boolean") => CheckedMemberType::Boolean,
                Some("Integer") => CheckedMemberType::Integer,
                Some(other) => panic!("{name}: unsupported native position {other}"),
                None => {
                    let identity = format!(
                        "ix://acme/shapes/{}",
                        expected["declaration"].as_str().expect("declared position")
                    );
                    if document["types"]
                        .as_array()
                        .expect("selected types")
                        .iter()
                        .any(|declaration| {
                            declaration["identity"] == identity
                                && declaration["kind"]["name"] == "tuple"
                        })
                    {
                        CheckedMemberType::Tuple(tuple_node(&base, &identity).1)
                    } else {
                        CheckedMemberType::Variant(variant_node(&base, &identity).1)
                    }
                }
            };
            assert_eq!(
                admitted.model_tuple_position_type(&declaration, position),
                Some(expected_type),
                "{name}: exact published selected type"
            );
            assert_eq!(
                tuple_positions(&document, &identity)[position].0,
                row["expected"]["resolves"]
                    .as_str()
                    .expect("published position identity"),
                "{name}"
            );
        }
    }
    assert_eq!(seen.len(), 7, "all published position controls");
}

#[trace("TC-940", "FR-069-AC-5")]
#[test]
fn published_recursive_tuple_union_declarations_keep_finite_structural_identity() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published recursion controls");
    let document = selected_document(&proposal);
    let package = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let admitted = read_selected_variant_package(&package, &document);
    let (_, cell) = tuple_node(&package, "ix://acme/shapes/Cell");
    let (_, list) = variant_node(&package, "ix://acme/shapes/List");
    assert_eq!(
        admitted.model_tuple_position_type(&cell, 0),
        Some(CheckedMemberType::Integer)
    );
    assert_eq!(
        admitted.model_tuple_position_type(&cell, 1),
        Some(CheckedMemberType::Variant(list.clone()))
    );
    let list_body = admitted
        .model_variant_body(&list)
        .expect("recursive List body");
    let cons = list_body
        .cases()
        .find(|case| case.name() == "Cons")
        .expect("Cons");
    assert_eq!(
        cons.payload_type(),
        Some(CheckedVariantPayloadType::Declared("ix://acme/shapes/Cell"))
    );
    for id in [&cell, &list] {
        let node = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                serde_json::from_value::<CheckedNodeId>(node["node_id"].clone()).expect("id") == *id
            })
            .expect("recursive declaration");
        assert!(node.get("recursion_group").is_none_or(Value::is_null));
        assert_eq!(id.path.len(), 3, "owner-only declaration identity");
    }
    let recursion = vectors["recursion_cases"]
        .as_array()
        .expect("recursion cases");
    assert_eq!(recursion.len(), 4);
    let leaf_control = recursion
        .iter()
        .find(|row| row["name"] == "TR-04")
        .expect("leaf control");
    assert_eq!(leaf_control["expected"]["leaves"], json!([]));
    let list_key = serde_json::to_value(&list).expect("list id");
    let equality = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["body"]["operation"]["identity"] == leaf_control["operation"]
                && node["body"]["arguments"].as_array().is_some_and(|args| {
                    args.iter().all(|argument| {
                        package["semantic_graph"]["nodes"]
                            .as_array()
                            .expect("nodes")
                            .iter()
                            .any(|operand| {
                                operand["node_id"] == argument["target"]
                                    && operand["semantic_type"] == list_key
                            })
                    })
                })
        })
        .expect("admitted recursive structural equality");
    assert_eq!(
        equality["body"]["operation"]["leaves"],
        leaf_control["expected"]["leaves"]
    );
}
