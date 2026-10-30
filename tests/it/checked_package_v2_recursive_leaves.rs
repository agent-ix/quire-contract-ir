// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-486: `structural.eq` over a compared type that reaches itself through a
//! record is admitted with its text leaves and its recursion leaves
//! (FR-038, "Recursive compared types"), read end to end through
//! `CheckedPackageV2::read`.
//!
//! Every package below is built here, node by node, from this crate's own
//! vocabulary: a record type node names each field's type by a `reference`
//! term, an optional field is a one-member union over its inner type, and the
//! cycle's members share one `recursion_group`.

use crate::support::checked_package::{
    canonical, evidence_for, mint_grouped_structural_keys, mint_ungrouped_structural_keys, node_id,
    nominal_package, pointer, refusal_at, rekey_application_node, rename_node, settle, sha256_hex,
    structural_key, BOOLEAN_KEY, INTEGER_KEY,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, ContractDependencies,
    ContractPackageReadResult, ContractPackageWireV1, ContractRootViewV1, LoweringProfileRef,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

type Typed = (String, Value);

/// The key of the node called `name`: the derived key (FR-038-AC-134) for
/// the anonymous `integer` and `boolean` nodes, the digest of the name
/// otherwise.
fn key(name: &str) -> String {
    match name {
        "integer" => INTEGER_KEY.to_owned(),
        "boolean" => BOOLEAN_KEY.to_owned(),
        "text" => structural_key("scalar_type", "text", None, &empty()),
        "nfc" => structural_key(
            "bounded_domain",
            "text_bounds",
            Some(&key("text")),
            &json!({"term": "aggregate", "members": [
                binding("text_profile", literal("text", "text", "nfc")),
            ]}),
        ),
        _ => sha256_hex(name.as_bytes()),
    }
}

fn node<S: AsRef<str>>(
    name: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    over: impl IntoIterator<Item = S>,
    role: &str,
    body: Value,
) -> Typed {
    let id = key(name);
    let dependencies: BTreeSet<String> = over.into_iter().map(|name| key(name.as_ref())).collect();
    let node = json!({
        "node_id": node_id(&id),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(&key(semantic_type)),
        "dependencies": dependencies.iter().map(|digest| node_id(digest)).collect::<Vec<_>>(),
        "occurrences": [{"role": role, "ordinal": 0}],
        "body": body,
    });
    (id, node)
}

fn literal(ty: &str, value_kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": node_id(&key(ty)), "value_kind": value_kind, "value": value})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn reference(name: &str) -> Value {
    json!({"term": "reference", "target": node_id(&key(name))})
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn scalar(name: &str, form: &str) -> Typed {
    node(
        name,
        "scalar_type",
        form,
        name,
        [] as [&str; 0],
        "type",
        empty(),
    )
}

/// `Text[nfc]`: a `text_bounds` domain binding the `nfc` profile over `text`.
fn nfc_text() -> Typed {
    let body = json!({"term": "aggregate", "members": [
        binding("text_profile", literal("text", "text", "nfc")),
    ]});
    node(
        "nfc",
        "bounded_domain",
        "text_bounds",
        "text",
        ["text"],
        "type",
        body,
    )
}

/// A parameter named `name` typed at the node called `ty` (QSL FR-092).
fn parameter(name: &str, ty: &str) -> Typed {
    let body = json!({"term": "aggregate", "members": [
        binding("name", literal("text", "text", name)),
        binding("level", literal("integer", "integer", "0")),
    ]});
    let (_, mut parameter) = node(
        name,
        "value",
        "parameter",
        ty,
        [] as [&str; 0],
        "expression",
        body,
    );
    let id = structural_key("value", "parameter", Some(&key(ty)), &parameter["body"]);
    parameter["node_id"] = node_id(&id);
    (id, parameter)
}

/// A record type named `name`, one field per `(field, type name)`.
fn record(name: &str, fields: &[(&str, &str)]) -> Typed {
    let members: Vec<Value> = fields
        .iter()
        .map(|(field, ty)| binding(field, reference(ty)))
        .collect();
    node(
        name,
        "composite_type",
        "record",
        name,
        fields.iter().map(|(_, ty)| *ty),
        "type",
        json!({"term": "aggregate", "members": members}),
    )
}

/// The optional `inner` named `name`, as a union of one member `Some` over
/// `inner`. A derived-shape `option` node cannot sit in a recursion group (its
/// key hashes a preimage with a `null` recursion: FR-038, IR-627-Q4), so the
/// cycles these tests build pass through a union, whose leaves read
/// `member:Some`, `position:0`.
fn some_of(name: &str, inner: &str) -> Typed {
    node(
        name,
        "composite_type",
        "union",
        name,
        [inner],
        "type",
        json!({"term": "aggregate", "members": [
            binding("Some", json!({"term": "aggregate", "members": [reference(inner)]})),
        ]}),
    )
}

/// `Option<inner>` in the QSL wrapped optional-field encoding.
fn option_of(name: &str, inner: &str) -> Typed {
    node(
        name,
        "composite_type",
        "option",
        name,
        [inner],
        "type",
        json!({"term": "aggregate", "members": [reference(inner)]}),
    )
}

fn collection_of(name: &str, form: &str, inner: &str) -> Typed {
    node(
        name,
        "composite_type",
        form,
        name,
        [inner],
        "type",
        json!({"term": "aggregate", "members": [reference(inner)]}),
    )
}

fn alias_of(name: &str, target: &str) -> Typed {
    node(
        name,
        "composite_type",
        "alias",
        target,
        [target],
        "type",
        empty(),
    )
}

/// The leaf segments from a `some_of` node into its payload.
fn some_segments() -> [String; 2] {
    ["member:Some".to_owned(), "position:0".to_owned()]
}

/// A member of the one recursion group the cycle shares.
fn in_group((id, mut node): Typed) -> Typed {
    node["recursion_group"] = json!("g");
    (id, node)
}

/// A catalogued `text_profile` law definition, read from the catalog's home.
fn text_law() -> Value {
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("the catalog is JSON");
    catalog["law_roles"]["text_profile"][0].clone()
}

fn text_leaf(path: &[String]) -> Value {
    json!({
        "path": path,
        "laws": [{"role": "text_profile", "definition": text_law()}],
        "mode": {"kind": "text_profile", "value": "nfc"},
    })
}

fn recursion_leaf(path: &[String]) -> Value {
    json!({"path": path, "laws": [], "mode": null})
}

/// `field:<name>` repeated for each of `names`, then `last`.
fn path(segments: &[&str], repeat: &str, times: usize, last: &str) -> Vec<String> {
    segments
        .iter()
        .map(|segment| (*segment).to_owned())
        .chain(std::iter::repeat_n(repeat.to_owned(), times))
        .chain(std::iter::once(last.to_owned()))
        .collect()
}

/// A package holding `types`, a parameter `p` typed at `root` and the
/// application `structural.eq(p, p)` carrying `leaves`. Returns it with the
/// application's node key.
fn equality_package(types: Vec<Typed>, root: &str, leaves: Vec<Value>) -> (Value, String) {
    let body = json!({
        "term": "application",
        "operator": "binary",
        "operation": {"identity": "quire.op.structural.eq", "laws": [], "mode": null,
            "member": null, "leaves": leaves},
        "result_type": node_id(&key("boolean")),
        "arguments": [reference("p"), reference("p")],
    });
    // FR-322 application key: the reader re-derives it.
    let id = sha256_hex(&canonical(&json!({
        "version": "quire.application-node/v1",
        "node_tag": "expression",
        "semantic_form": "binary",
        "semantic_type": node_id(&key("boolean")),
        "declaration": null,
        "recursion": null,
        "body": body,
    })));
    let (_, mut application) = node(
        "equality",
        "expression",
        "binary",
        "boolean",
        ["p"],
        "expression",
        body,
    );
    application["node_id"] = node_id(&id);

    let mut nodes = vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        scalar("boolean", "boolean"),
        nfc_text(),
        parameter("p", root),
        (id.clone(), application),
    ];
    nodes.extend(types);
    nodes.sort_by(|left, right| left.0.cmp(&right.0));
    let mut package = nominal_package(&[]);
    package["semantic_graph"]["nodes"] =
        Value::Array(nodes.into_iter().map(|(_, node)| node).collect());
    package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("definition selections")
        .push(text_law());
    let parameter_id = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .and_then(|node| node["node_id"]["digest"].as_str())
        .expect("parameter key")
        .to_owned();
    rename_node(&mut package, &key("p"), &parameter_id);
    mint_ungrouped_structural_keys(&mut package);
    mint_grouped_structural_keys(&mut package);
    mint_ungrouped_structural_keys(&mut package);
    let application = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "expression" && node["semantic_form"] == "binary")
        .expect("application");
    rekey_application_node(&mut package, application);
    settle(&mut package);
    let id = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "binary")
        .and_then(|node| node["node_id"]["digest"].as_str())
        .expect("application key")
        .to_owned();
    (package, id)
}

fn read(package: &Value, limits: CheckedPackageReadLimits) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(&canonical(package), limits, &evidence_for(package))
}

/// The pointer of `operation.leaves` of the application node keyed `id`.
fn leaves_pointer(package: &Value, id: &str) -> String {
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"]["digest"] == id)
        .expect("the application node");
    format!("/semantic_graph/nodes/{position}/body/operation/leaves")
}

/// Edit the sole application in an already keyed fixture, then derive its
/// dependencies, application key, source map and package identity again.
fn edit_application(package: &mut Value, edit: impl FnOnce(&mut Value)) -> String {
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "expression")
        .expect("application");
    edit(&mut package["semantic_graph"]["nodes"][position]);
    settle(package);
    package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("derived application key")
        .to_owned()
}

fn type_id(package: &Value, form: &str) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "composite_type" && node["semantic_form"] == form)
        .expect("type")["node_id"]
        .clone()
}

fn scalar_id(package: &Value, form: &str) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == form)
        .expect("scalar type")["node_id"]
        .clone()
}

fn collection_of_nfc_id(package: &Value, form: &str) -> Value {
    let nfc_id = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "bounded_domain" && node["semantic_form"] == "text_bounds")
        .expect("NFC text")["node_id"]
        .clone();
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["node_tag"] == "composite_type"
                && node["semantic_form"] == form
                && node["body"]["members"][0]["target"] == nfc_id
        })
        .expect("collection of NFC text")["node_id"]
        .clone()
}

#[trace("TC-048", "FR-038-AC-228")]
#[test]
fn residual_operand_source_reads_value_type_and_direct_type_references() {
    for identity in ["quire.op.structural.eq", "quire.op.structural.ne"] {
        let mut selected = Vec::new();
        for (type_name, leaves) in [
            ("integer", vec![]),
            ("nfc", vec![text_leaf(&[field("value")])]),
        ] {
            let (mut value, _) = equality_package(
                vec![record("r", &[("value", type_name)])],
                "r",
                leaves.clone(),
            );
            let value_application = edit_application(&mut value, |node| {
                node["body"]["operation"]["identity"] = json!(identity);
            });
            let record_id = type_id(&value, "record");
            assert!(
                matches!(
                    read(&value, CheckedPackageReadLimits::bounded()),
                    CheckedPackageV2ReadResult::Admitted(_)
                ),
                "{identity} value reference over {type_name}"
            );
            let mut direct = value.clone();
            edit_application(&mut direct, |node| {
                node["body"]["arguments"] = json!([
                    {"term": "reference", "target": record_id},
                    {"term": "reference", "target": record_id},
                ]);
            });
            assert!(
                matches!(
                    read(&direct, CheckedPackageReadLimits::bounded()),
                    CheckedPackageV2ReadResult::Admitted(_)
                ),
                "{identity} direct type reference over {type_name}"
            );
            if type_name == "nfc" {
                for mut missing in [value, direct] {
                    let application = edit_application(&mut missing, |node| {
                        node["body"]["operation"]["leaves"] = json!([]);
                    });
                    assert_eq!(
                        read(&missing, CheckedPackageReadLimits::bounded()),
                        refusal_at(
                            CheckedPackageRefusalCode::InvalidPackage,
                            &leaves_pointer(&missing, &application),
                            Some(CheckedPackageRefusalCause::OperationLawMissing),
                            &application,
                        ),
                        "{identity} changed to text cannot retain empty leaves"
                    );
                }
            }
            selected.push((record_id, value_application));
        }
        assert_ne!(selected[0].0, selected[1].0, "declared type rekeys record");
        assert_ne!(
            selected[0].1, selected[1].1,
            "declared type rekeys comparison"
        );
    }
}

#[trace("TC-048", "FR-038-AC-228")]
#[test]
fn residual_operand_source_reads_option_none_literal_type() {
    let (mut option_package, _) = equality_package(vec![option_of("r", "integer")], "r", vec![]);
    let option_id = type_id(&option_package, "option");
    edit_application(&mut option_package, |node| {
        let none = json!({"term":"literal","type":option_id,"value_kind":"none","value":null});
        node["body"]["arguments"] = json!([none.clone(), none]);
    });
    let result = read(&option_package, CheckedPackageReadLimits::bounded());
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
}

#[trace("TC-048", "FR-038-AC-228")]
#[test]
fn residual_operand_source_reads_referenced_application_result_type() {
    let (mut package, _) = equality_package(
        vec![record("r", &[("label", "nfc")])],
        "r",
        vec![text_leaf(&[field("label")])],
    );
    let record_id = type_id(&package, "record");
    let boolean_id = scalar_id(&package, "boolean");
    let parameter_id = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .expect("parameter")["node_id"]
        .clone();
    let body = json!({
        "term": "application",
        "operator": "conditional",
        "operation": {"identity": "quire.op.control.if", "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": record_id,
        "arguments": [
            {"term": "literal", "type": boolean_id, "value_kind": "boolean", "value": true},
            {"term": "reference", "target": parameter_id},
            {"term": "reference", "target": parameter_id},
        ],
    });
    let (_, mut child) = node(
        "child",
        "expression",
        "conditional",
        "r",
        [] as [&str; 0],
        "expression",
        body,
    );
    child["semantic_type"] = record_id;
    let child_id = child["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(child);
    edit_application(&mut package, |outer| {
        outer["body"]["arguments"] = json!([
            {"term":"reference","target":child_id},
            {"term":"reference","target":child_id},
        ]);
    });
    let outcome = read(&package, CheckedPackageReadLimits::bounded());
    assert!(
        matches!(outcome, CheckedPackageV2ReadResult::Admitted(_)),
        "reference to an application-bearing value node: {outcome:?}"
    );

    let mut inline = package;
    let child_body = inline["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "conditional")
        .expect("child application")["body"]
        .clone();
    edit_application(&mut inline, |outer| {
        outer["body"]["arguments"][0] = child_body;
    });
    let position = inline["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "expression" && node["semantic_form"] == "binary")
        .expect("outer comparison");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read(&inline, CheckedPackageReadLimits::bounded())
    else {
        panic!("inline application operand must refuse at wire admission");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        refusal.loci[0].path.clone(),
        Some(pointer(&format!(
            "/semantic_graph/nodes/{position}/body/arguments/0"
        )))
    );
    assert_eq!(refusal.cause, None);
}

#[trace("TC-048", "FR-038-AC-231")]
#[test]
fn residual_compared_type_reaching_ieee_refuses_but_integer_control_admits() {
    for ieee in ["float32", "float64"] {
        for shape in ["record", "tuple", "option", "sequence"] {
            let outer = match shape {
                "record" => record("r", &[("value", ieee)]),
                "tuple" => tuple_of("r", &[ieee]),
                "option" | "sequence" => collection_of("r", shape, ieee),
                _ => unreachable!("test table is exhaustive"),
            };
            let (base, _) = equality_package(vec![outer.clone(), scalar(ieee, ieee)], "r", vec![]);
            let mut graph_only = base;
            graph_only["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .retain(|node| node["node_tag"] != "expression");
            settle(&mut graph_only);
            assert!(
                matches!(
                    read(&graph_only, CheckedPackageReadLimits::bounded()),
                    CheckedPackageV2ReadResult::Admitted(_)
                ),
                "{shape}<{ieee}> has an independently admitted graph"
            );
            for identity in ["quire.op.structural.eq", "quire.op.structural.ne"] {
                for leaves in [vec![], vec![text_leaf(&[field("value")])]] {
                    let (mut package, _) =
                        equality_package(vec![outer.clone(), scalar(ieee, ieee)], "r", leaves);
                    let application = edit_application(&mut package, |node| {
                        node["body"]["operation"]["identity"] = json!(identity);
                    });
                    assert_eq!(
                        read(&package, CheckedPackageReadLimits::bounded()),
                        refusal_at(
                            CheckedPackageRefusalCode::IllTyped,
                            &leaves_pointer(&package, &application),
                            Some(CheckedPackageRefusalCause::OperatorIneligible),
                            &application,
                        ),
                        "{shape}<{ieee}> under {identity} is not equality eligible"
                    );
                }
            }
        }
    }
    for shape in ["record", "tuple", "option", "sequence"] {
        let outer = match shape {
            "record" => record("r", &[("value", "integer")]),
            "tuple" => tuple_of("r", &["integer"]),
            "option" | "sequence" => collection_of("r", shape, "integer"),
            _ => unreachable!("test table is exhaustive"),
        };
        let (integer, _) = equality_package(vec![outer], "r", vec![]);
        assert!(
            matches!(
                read(&integer, CheckedPackageReadLimits::bounded()),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{shape}<Integer> is the admitted control"
        );
    }
}

#[trace("TC-048", "FR-038-AC-180", "FR-038-AC-231")]
#[test]
fn residual_float_rounding_compared_type_refuses_after_graph_admission() {
    for ieee in ["float32", "float64"] {
        let rounded = node(
            "rounded",
            "bounded_domain",
            "float_rounding",
            ieee,
            [ieee],
            "type",
            empty(),
        );
        let types = vec![
            scalar(ieee, ieee),
            rounded,
            record("r", &[("value", "rounded")]),
        ];
        let (base, _) = equality_package(types.clone(), "r", vec![]);
        let mut graph_only = base;
        graph_only["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .retain(|node| node["node_tag"] != "expression");
        settle(&mut graph_only);
        assert!(
            matches!(
                read(&graph_only, CheckedPackageReadLimits::bounded()),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "float_rounding<{ieee}> has an independently admitted graph"
        );
        for identity in ["quire.op.structural.eq", "quire.op.structural.ne"] {
            for leaves in [vec![], vec![text_leaf(&[field("value")])]] {
                let (mut package, _) = equality_package(types.clone(), "r", leaves);
                let application = edit_application(&mut package, |node| {
                    node["body"]["operation"]["identity"] = json!(identity);
                });
                assert_eq!(
                    read(&package, CheckedPackageReadLimits::bounded()),
                    refusal_at(
                        CheckedPackageRefusalCode::IllTyped,
                        &leaves_pointer(&package, &application),
                        Some(CheckedPackageRefusalCause::OperatorIneligible),
                        &application,
                    ),
                    "float_rounding<{ieee}> under {identity} is not equality eligible"
                );
            }
        }
    }
}

#[trace("TC-048", "FR-038-AC-230")]
#[test]
fn residual_result_inner_set_like_elements_require_an_empty_path_leaf() {
    for form in ["set", "bag", "ordered_set"] {
        let (mut package, _) =
            equality_package(vec![collection_of("result", form, "nfc")], "nfc", vec![]);
        let result_id = type_id(&package, form);
        let leaf = text_leaf(&[]);
        edit_application(&mut package, |node| {
            let first_argument = node["body"]["arguments"][0].clone();
            node["semantic_form"] = json!("collection");
            node["semantic_type"] = result_id.clone();
            node["body"]["operator"] = json!("collection");
            node["body"]["operation"]["identity"] = json!(format!("quire.op.collection.{form}"));
            node["body"]["operation"]["leaves"] = json!([leaf]);
            node["body"]["result_type"] = result_id;
            node["body"]["arguments"] = json!([first_argument]);
        });
        assert!(
            matches!(
                read(&package, CheckedPackageReadLimits::bounded()),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{form} with its text leaf must admit"
        );
        let mut missing = package;
        let application = edit_application(&mut missing, |node| {
            node["body"]["operation"]["leaves"] = json!([]);
        });
        assert_eq!(
            read(&missing, CheckedPackageReadLimits::bounded()),
            refusal_at(
                CheckedPackageRefusalCode::InvalidPackage,
                &leaves_pointer(&missing, &application),
                Some(CheckedPackageRefusalCause::OperationLawMissing),
                &application,
            ),
            "{form} without its text leaf"
        );
        let mut prefixed = missing;
        let application = edit_application(&mut prefixed, |node| {
            node["body"]["operation"]["leaves"] = json!([text_leaf(&["inner".to_owned()])]);
        });
        assert_eq!(
            read(&prefixed, CheckedPackageReadLimits::bounded()),
            refusal_at(
                CheckedPackageRefusalCode::InvalidPackage,
                &format!("{}/0/path", leaves_pointer(&prefixed, &application)),
                Some(CheckedPackageRefusalCause::OperationLawMismatch),
                &application,
            ),
            "{form} does not add an inner prefix to the selected element"
        );
    }
}

#[trace("TC-048", "FR-038-AC-229", "FR-038-AC-230")]
#[test]
fn residual_required_leaf_sources_refuse_missing_referenced_types_on_public_read() {
    for leaves in [vec![], vec![text_leaf(&[])]] {
        let (package, application) = equality_package(
            vec![record("r", &[("value", "absent")])],
            "r",
            leaves.clone(),
        );
        assert_eq!(
            read(&package, CheckedPackageReadLimits::bounded()),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &leaves_pointer(&package, &application),
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                &application,
            ),
            "operand:0 outer record family fits but its compared child is absent"
        );

        let (mut contains, _) = equality_package(
            vec![collection_of("r", "sequence", "absent")],
            "r",
            leaves.clone(),
        );
        let application = edit_application(&mut contains, |node| {
            let collection = node["body"]["arguments"][0].clone();
            node["semantic_form"] = json!("collection");
            node["body"]["operator"] = json!("collection");
            node["body"]["operation"]["identity"] = json!("quire.op.collection.contains");
            node["body"]["arguments"] = json!([collection, literal("integer", "integer", "0")]);
        });
        assert_eq!(
            read(&contains, CheckedPackageReadLimits::bounded()),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &leaves_pointer(&contains, &application),
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                &application,
            ),
            "inner:0 collection family fits but its element type is absent"
        );
    }
}

#[trace("TC-048", "FR-038-AC-230")]
#[test]
fn residual_set_result_refuses_missing_element_type_on_public_read() {
    for leaves in [vec![], vec![text_leaf(&[])]] {
        let (mut package, _) = equality_package(
            vec![collection_of("result", "set", "absent")],
            "integer",
            leaves,
        );
        let result_id = type_id(&package, "set");
        let application = edit_application(&mut package, |node| {
            let first = node["body"]["arguments"][0].clone();
            node["semantic_form"] = json!("collection");
            node["semantic_type"] = result_id.clone();
            node["body"]["operator"] = json!("collection");
            node["body"]["operation"]["identity"] = json!("quire.op.collection.set");
            node["body"]["result_type"] = result_id;
            node["body"]["arguments"] = json!([first]);
        });
        assert_eq!(
            read(&package, CheckedPackageReadLimits::bounded()),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &leaves_pointer(&package, &application),
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                &application,
            ),
            "result_inner set family fits but its element type is absent"
        );
    }
}

#[trace("TC-048", "FR-038-AC-230")]
#[test]
fn residual_result_inner_map_flat_map_flatten_and_convert_use_result_element() {
    for identity in ["map", "flat_map", "flatten", "convert"] {
        let (root, types) = match identity {
            "map" | "flat_map" => ("result", vec![collection_of("result", "set", "nfc")]),
            "flatten" => (
                "source",
                vec![
                    collection_of("result", "set", "nfc"),
                    collection_of("source", "set", "result"),
                ],
            ),
            "convert" => (
                "source",
                vec![
                    collection_of("result", "set", "nfc"),
                    collection_of("source", "sequence", "nfc"),
                ],
            ),
            _ => unreachable!("test table is exhaustive"),
        };
        let (mut package, _) = equality_package(types, root, vec![]);
        let result_id = collection_of_nfc_id(&package, "set");
        let nfc_id = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                node["node_tag"] == "bounded_domain" && node["semantic_form"] == "text_bounds"
            })
            .expect("NFC text")["node_id"]
            .clone();
        edit_application(&mut package, |node| {
            let source = node["body"]["arguments"][0].clone();
            node["semantic_form"] = json!(if identity == "convert" {
                "conversion"
            } else {
                "collection"
            });
            node["semantic_type"] = result_id.clone();
            node["body"]["operator"] = json!(if identity == "convert" {
                "convert"
            } else {
                "collection"
            });
            node["body"]["operation"]["identity"] =
                json!(format!("quire.op.collection.{identity}"));
            node["body"]["operation"]["leaves"] = json!([text_leaf(&[])]);
            node["body"]["result_type"] = result_id.clone();
            node["body"]["arguments"] = match identity {
                "map" => json!([source, {"term":"binding","name":"x","value":{
                    "term":"literal","type":nfc_id,"value_kind":"text","value":"a"}}]),
                "flat_map" => json!([source.clone(), {"term":"binding","name":"x","value":source}]),
                "flatten" | "convert" => json!([source]),
                _ => unreachable!("test table is exhaustive"),
            };
            if identity == "convert" {
                node["body"]["operation"]["member"] = json!({
                    "kind":"type_argument", "declaration":result_id
                });
            }
        });
        let outcome = read(&package, CheckedPackageReadLimits::bounded());
        assert!(
            matches!(outcome, CheckedPackageV2ReadResult::Admitted(_)),
            "{identity} positive control: {outcome:?}"
        );
        let mut missing = package;
        let application = edit_application(&mut missing, |node| {
            node["body"]["operation"]["leaves"] = json!([]);
        });
        assert_eq!(
            read(&missing, CheckedPackageReadLimits::bounded()),
            refusal_at(
                CheckedPackageRefusalCode::InvalidPackage,
                &leaves_pointer(&missing, &application),
                Some(CheckedPackageRefusalCause::OperationLawMissing),
                &application,
            ),
            "{identity} missing leaf"
        );
    }
}

#[trace("TC-048", "FR-038-AC-230")]
#[test]
fn residual_result_inner_sequence_results_expect_no_leaves() {
    for identity in ["map", "flat_map", "flatten", "convert"] {
        let (root, types) = match identity {
            "map" | "flat_map" => ("result", vec![collection_of("result", "sequence", "nfc")]),
            "flatten" => (
                "source",
                vec![
                    collection_of("result", "sequence", "nfc"),
                    collection_of("source", "sequence", "result"),
                ],
            ),
            "convert" => (
                "source",
                vec![
                    collection_of("result", "sequence", "nfc"),
                    collection_of("source", "set", "nfc"),
                ],
            ),
            _ => unreachable!("test table is exhaustive"),
        };
        let (mut package, _) = equality_package(types, root, vec![]);
        let result_id = collection_of_nfc_id(&package, "sequence");
        let nfc_id = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                node["node_tag"] == "bounded_domain" && node["semantic_form"] == "text_bounds"
            })
            .expect("NFC text")["node_id"]
            .clone();
        edit_application(&mut package, |node| {
            let source = node["body"]["arguments"][0].clone();
            node["semantic_form"] = json!(if identity == "convert" {
                "conversion"
            } else {
                "collection"
            });
            node["semantic_type"] = result_id.clone();
            node["body"]["operator"] = json!(if identity == "convert" {
                "convert"
            } else {
                "collection"
            });
            node["body"]["operation"]["identity"] =
                json!(format!("quire.op.collection.{identity}"));
            node["body"]["result_type"] = result_id.clone();
            node["body"]["arguments"] = match identity {
                "map" => json!([source, {"term":"binding","name":"x","value":{
                    "term":"literal","type":nfc_id,"value_kind":"text","value":"a"}}]),
                "flat_map" => json!([source.clone(), {"term":"binding","name":"x","value":source}]),
                "flatten" | "convert" => json!([source]),
                _ => unreachable!("test table is exhaustive"),
            };
            if identity == "convert" {
                node["body"]["operation"]["member"] = json!({
                    "kind":"type_argument", "declaration":result_id
                });
            }
        });
        let outcome = read(&package, CheckedPackageReadLimits::bounded());
        assert!(
            matches!(outcome, CheckedPackageV2ReadResult::Admitted(_)),
            "{identity} sequence result with no leaves: {outcome:?}"
        );
        let mut extra = package;
        let application = edit_application(&mut extra, |node| {
            node["body"]["operation"]["leaves"] = json!([text_leaf(&[])]);
        });
        assert_eq!(
            read(&extra, CheckedPackageReadLimits::bounded()),
            refusal_at(
                CheckedPackageRefusalCode::InvalidPackage,
                &format!("{}/0", leaves_pointer(&extra, &application)),
                Some(CheckedPackageRefusalCause::OperationLawMismatch),
                &application,
            ),
            "{identity} sequence result rejects an extraneous leaf"
        );
    }
}

#[trace("TC-048", "FR-038-AC-231")]
#[test]
fn residual_flatten_sequence_result_does_not_compare_its_ieee_element() {
    for ieee in ["float32", "float64"] {
        let (mut package, _) = equality_package(
            vec![
                scalar(ieee, ieee),
                collection_of("inner", "sequence", ieee),
                collection_of("outer", "sequence", "inner"),
            ],
            "outer",
            vec![],
        );
        let ieee_id = scalar_id(&package, ieee);
        let inner_id = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                node["node_tag"] == "composite_type"
                    && node["semantic_form"] == "sequence"
                    && node["body"]["members"][0]["target"] == ieee_id
            })
            .expect("inner sequence")["node_id"]
            .clone();
        edit_application(&mut package, |node| {
            let argument = node["body"]["arguments"][0].clone();
            node["semantic_form"] = json!("collection");
            node["semantic_type"] = inner_id.clone();
            node["body"]["operator"] = json!("collection");
            node["body"]["operation"]["identity"] = json!("quire.op.collection.flatten");
            node["body"]["result_type"] = inner_id;
            node["body"]["arguments"] = json!([argument]);
        });
        assert!(
            matches!(
                read(&package, CheckedPackageReadLimits::bounded()),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "flatten over sequence<{ieee}> has no compared leaf source"
        );
    }
}

#[trace("TC-048", "FR-038-AC-231")]
#[test]
fn residual_contains_sequence_compares_its_ieee_element() {
    for form in ["float32", "float64", "integer"] {
        let mut types = vec![collection_of("r", "sequence", form)];
        if form != "integer" {
            types.push(scalar(form, form));
        }
        if form != "integer" {
            let (mut graph_only, _) = equality_package(types.clone(), "r", vec![]);
            graph_only["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .retain(|node| node["node_tag"] != "expression");
            settle(&mut graph_only);
            assert!(
                matches!(
                    read(&graph_only, CheckedPackageReadLimits::bounded()),
                    CheckedPackageV2ReadResult::Admitted(_)
                ),
                "sequence<{form}> has an independently admitted graph"
            );
        }
        let leaf_lists = if form == "integer" {
            vec![vec![]]
        } else {
            vec![vec![], vec![text_leaf(&[])]]
        };
        for leaves in leaf_lists {
            let (mut package, _) = equality_package(types.clone(), "r", leaves);
            let element_id = scalar_id(&package, form);
            let application = edit_application(&mut package, |node| {
                let collection = node["body"]["arguments"][0].clone();
                node["semantic_form"] = json!("collection");
                node["body"]["operator"] = json!("collection");
                node["body"]["operation"]["identity"] = json!("quire.op.collection.contains");
                node["body"]["arguments"] = json!([
                    collection, {"term":"reference","target":element_id}
                ]);
            });
            let outcome = read(&package, CheckedPackageReadLimits::bounded());
            if form == "integer" {
                assert!(
                    matches!(outcome, CheckedPackageV2ReadResult::Admitted(_)),
                    "integer element admits: {outcome:?}"
                );
            } else {
                assert_eq!(
                    outcome,
                    refusal_at(
                        CheckedPackageRefusalCode::IllTyped,
                        &leaves_pointer(&package, &application),
                        Some(CheckedPackageRefusalCause::OperatorIneligible),
                        &application,
                    ),
                    "sequence<{form}> contains compares the element"
                );
            }
        }
    }
}

#[trace("TC-048", "FR-038-AC-229")]
#[test]
fn residual_compared_operand_rejects_binder_and_aggregate_families_first() {
    let (base, _) = equality_package(vec![record("r", &[("n", "integer")])], "r", vec![]);
    assert!(matches!(
        read(&base, CheckedPackageReadLimits::bounded()),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    for family in ["binder", "aggregate"] {
        let mut package = base.clone();
        let application = edit_application(&mut package, |node| {
            let value = node["body"]["arguments"][0].clone();
            node["body"]["arguments"][0] = match family {
                "binder" => json!({"term":"binding","name":"x","value":value}),
                "aggregate" => json!({"term":"aggregate","members":[value]}),
                _ => unreachable!("test table is exhaustive"),
            };
        });
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"]["digest"] == application)
            .expect("application");
        assert_eq!(
            read(&package, CheckedPackageReadLimits::bounded()),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &format!("/semantic_graph/nodes/{position}/body/arguments/0"),
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                &application,
            ),
            "{family} refuses before leaves"
        );
    }
}

#[trace("TC-048", "FR-038-AC-229")]
#[test]
fn residual_non_member_terms_refuse_before_operand_family_and_leaves() {
    let (base, _) = equality_package(vec![record("r", &[("n", "integer")])], "r", vec![]);
    for term in [
        json!({"term":"frame","modifies":[],"creates":[],"deletes":[],"deletes_scoped":[]}),
        json!({"term":"abstraction_relation","members":[]}),
    ] {
        let mut package = base.clone();
        let application = edit_application(&mut package, |node| {
            node["body"]["arguments"][0] = term.clone();
        });
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"]["digest"] == application)
            .expect("application");
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read(&package, CheckedPackageReadLimits::bounded())
        else {
            panic!("non-MemberTerm operand must refuse before leaf selection");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "{refusal:?}"
        );
        assert_eq!(
            refusal.loci[0].path.clone(),
            Some(pointer(&format!(
                "/semantic_graph/nodes/{position}/body/arguments/0"
            )))
        );
        assert_eq!(refusal.cause, None);
    }
}

#[trace("TC-048", "FR-038-AC-229")]
#[test]
fn residual_dependency_reference_cannot_be_a_comparison_operand() {
    let (mut package, _) = equality_package(vec![record("r", &[("n", "integer")])], "r", vec![]);
    let package_id = package["package_id"].clone();
    let parameter_id = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .expect("parameter")["node_id"]
        .clone();
    let application = edit_application(&mut package, |node| {
        node["body"]["arguments"][0] = json!({
            "term":"dependency_reference", "package":package_id, "node":parameter_id
        });
    });
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"]["digest"] == application)
        .expect("application");
    let outcome = read(&package, CheckedPackageReadLimits::bounded());
    assert_eq!(
        outcome,
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!("/semantic_graph/nodes/{position}/body/arguments/0/package"),
            Some(CheckedPackageRefusalCause::MissingSelection),
            &application,
        ),
        "dependency reference is refused by placement before leaves"
    );
}

/// `record Node { label: Text[nfc]; next?: Node; }`.
fn node_types() -> Vec<Typed> {
    vec![
        in_group(record("node", &[("label", "nfc"), ("next", "node_next")])),
        in_group(some_of("node_next", "node")),
    ]
}

#[trace("FR-035-AC-4", "FR-035-AC-19")]
#[test]
fn recursive_group_round_trips_with_original_node_keys() {
    let [some, payload] = some_segments();
    let (source, application) = equality_package(
        node_types(),
        "node",
        vec![
            text_leaf(&[field("label")]),
            recursion_leaf(&[field("next"), some, payload, "recursion:0".into()]),
        ],
    );
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read(&source, CheckedPackageReadLimits::bounded())
    else {
        panic!("recursive source must admit");
    };
    let source_group = admitted
        .graph()
        .nodes
        .iter()
        .filter(|node| node.recursion_group.is_some())
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(source_group.len(), 2);
    assert!(
        source_group[0] > source_group[1],
        "fixture must distinguish graph order from key order"
    );
    let root: CheckedNodeId = serde_json::from_value(node_id(&application)).expect("root key");
    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let result = admitted.lower(std::slice::from_ref(&root), &profile);
    let bytes = result.package.canonical_bytes().expect("complete package");
    let outcome = ContractPackageWireV1::read(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    );
    let ContractPackageReadResult::Admitted(decoded) = outcome else {
        panic!("lowered recursion group must strict-read: {outcome:?}");
    };
    let producer = result.package.wire().expect("producer package");
    for node in &decoded.wire().semantic_graph.nodes {
        let position = admitted
            .graph()
            .nodes
            .iter()
            .position(|source| source.node_id == node.node_id)
            .expect("represented source node");
        let kind = admitted.node_kinds()[position];
        assert_eq!(producer.node_kind(&node.node_id), Some(kind));
        assert_eq!(decoded.node_kind(&node.node_id), Some(kind));
    }
    let ContractRootViewV1::Lowered(closure) = decoded.reached_closure(&root).expect("closure")
    else {
        panic!("lowered root");
    };
    assert_eq!(
        closure.root_kind,
        decoded.node_kind(&root).expect("root kind")
    );
    for reached in &closure.nodes {
        assert_eq!(Some(reached.kind), decoded.node_kind(&reached.node.node_id));
    }
    let emitted_group = decoded
        .wire()
        .semantic_graph
        .nodes
        .iter()
        .filter(|node| node.recursion_group.is_some())
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(emitted_group, source_group);
    let again = admitted.lower(std::slice::from_ref(&root), &profile);
    assert_eq!(again.package.canonical_bytes(), Some(bytes));
}

#[trace("TC-047", "FR-035-AC-6")]
#[test]
fn reached_view_visits_an_admitted_recursion_group_once() {
    let [some, payload] = some_segments();
    let (source, application) = equality_package(
        node_types(),
        "node",
        vec![
            text_leaf(&[field("label")]),
            recursion_leaf(&[field("next"), some, payload, "recursion:0".into()]),
        ],
    );
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read(&source, CheckedPackageReadLimits::bounded())
    else {
        panic!("recursion group must admit");
    };
    let root: CheckedNodeId = serde_json::from_value(node_id(&application)).expect("root key");
    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let result = admitted.lower(std::slice::from_ref(&root), &profile);
    let ContractRootViewV1::Lowered(view) = result
        .package
        .wire()
        .expect("wire")
        .reached_closure(&root)
        .expect("view")
    else {
        panic!("recursive request must lower");
    };
    let keys = view
        .nodes
        .iter()
        .map(|entry| entry.node.node_id.clone())
        .collect::<Vec<_>>();
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        keys.iter().cloned().collect::<BTreeSet<_>>().len(),
        keys.len()
    );
    let nodes = admitted
        .graph()
        .nodes
        .iter()
        .map(|node| (&node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    let mut independent = BTreeSet::new();
    let mut pending = VecDeque::from([root.clone()]);
    while let Some(key) = pending.pop_front() {
        if !independent.insert(key.clone()) {
            continue;
        }
        let node = nodes.get(&key).expect("admitted local edge");
        pending.push_back(node.semantic_type.clone());
        pending.extend(node.dependencies.iter().cloned());
        fn body_targets(value: &Value, pending: &mut VecDeque<CheckedNodeId>) {
            match value {
                Value::Object(object) => {
                    if let Some(term) = object.get("term").and_then(Value::as_str) {
                        let member = match term {
                            "reference" => Some("target"),
                            "literal" => Some("type"),
                            "application" => Some("result_type"),
                            _ => None,
                        };
                        if let Some(target) = member.and_then(|member| object.get(member)) {
                            pending.push_back(
                                serde_json::from_value(target.clone()).expect("body key"),
                            );
                        }
                    }
                    for child in object.values() {
                        body_targets(child, pending);
                    }
                }
                Value::Array(items) => {
                    for item in items {
                        body_targets(item, pending);
                    }
                }
                _ => {}
            }
        }
        body_targets(
            &serde_json::to_value(&node.body).expect("typed body"),
            &mut pending,
        );
    }
    assert_eq!(keys.iter().cloned().collect::<BTreeSet<_>>(), independent);
    let grouped = view
        .nodes
        .iter()
        .filter(|entry| entry.node.recursion_group.is_some())
        .map(|entry| entry.node.node_id.clone())
        .collect::<BTreeSet<_>>();
    let admitted_group = admitted
        .graph()
        .nodes
        .iter()
        .filter(|node| node.recursion_group.is_some())
        .map(|node| node.node_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(grouped, admitted_group);
    assert_eq!(grouped.len(), 2);
}

/// The QSL record-field encoding of `next?: List` contributes one field edge.
fn wrapped_list(text_fields: &[&str]) -> Vec<Typed> {
    let mut fields = vec![("number", "integer")];
    if let Some(label) = text_fields.first() {
        fields.push((label, "nfc"));
    }
    fields.push(("next", "list_next"));
    fields.extend(text_fields.iter().skip(1).map(|name| (*name, "nfc")));
    let (id, mut list) = record("list", &fields);
    let next = list["body"]["members"]
        .as_array_mut()
        .expect("record members")
        .iter_mut()
        .find(|field| field["name"] == "next")
        .expect("next field");
    next["value"] = json!({"term": "aggregate", "members": [
        binding("optional", reference("list_next")),
    ]});
    vec![
        in_group((id, list)),
        in_group(option_of("list_next", "list")),
    ]
}

fn list_leaves(text_fields: &[&str]) -> Vec<Value> {
    let mut leaves: Vec<Value> = text_fields
        .first()
        .map(|name| text_leaf(&[field(name)]))
        .into_iter()
        .collect();
    leaves.push(recursion_leaf(&[
        field("next"),
        "inner".into(),
        "recursion:0".into(),
    ]));
    leaves.extend(
        text_fields
            .iter()
            .skip(1)
            .map(|name| text_leaf(&[field(name)])),
    );
    leaves
}

#[trace("TC-048", "FR-038-AC-151")]
#[test]
fn tc_048_wrapped_optional_record_fields_keep_their_leaf_order() {
    let limits = CheckedPackageReadLimits::bounded();
    let (integer_list, _) = equality_package(wrapped_list(&[]), "list", vec![]);
    assert!(matches!(
        read(&integer_list, limits),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    for text_fields in [&["label"][..], &["label", "tail"][..]] {
        let (package, _) =
            equality_package(wrapped_list(text_fields), "list", list_leaves(text_fields));
        match read(&package, limits) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("wrapped optional with {text_fields:?} must admit: {other:?}"),
        }
    }

    let (package, id) = equality_package(
        wrapped_list(&["label"]),
        "list",
        vec![text_leaf(&[field("label")])],
    );
    assert_eq!(
        read(&package, limits),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &leaves_pointer(&package, &id),
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            &id,
        ),
    );

    let mut direct = wrapped_list(&["label"]);
    direct[0].1["body"]["members"]
        .as_array_mut()
        .expect("record members")
        .iter_mut()
        .find(|field| field["name"] == "next")
        .expect("next field")["value"] = reference("list_next");
    let (package, _) = equality_package(direct, "list", list_leaves(&["label"]));
    assert!(matches!(
        read(&package, limits),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-048", "FR-038-AC-152")]
#[test]
fn tc_048_malformed_optional_record_fields_refuse_at_operation_leaves() {
    let malformed = [
        json!({"term": "aggregate", "members": []}),
        json!({"term": "aggregate", "members": [
            binding("optional", reference("list_next")),
            binding("optional", reference("list_next")),
        ]}),
        json!({"term": "aggregate", "members": [binding("other", reference("list_next"))]}),
        json!({"term": "aggregate", "members": [reference("list_next")]}),
        json!({"term": "aggregate", "members": [binding("optional", reference("integer"))]}),
        json!({"term": "aggregate", "members": [binding("optional", literal("integer", "integer", "1"))]}),
    ];
    for value in malformed {
        let mut types = wrapped_list(&["label"]);
        types[0].1["body"]["members"]
            .as_array_mut()
            .expect("record members")
            .iter_mut()
            .find(|field| field["name"] == "next")
            .expect("next field")["value"] = value;
        // Keep the group's independent cycle valid while `next` is malformed.
        // The operation step must still examine the malformed optional field.
        types[0].1["body"]["members"]
            .as_array_mut()
            .expect("record members")
            .push(binding("cycle", reference("list_next")));
        let (package, id) = equality_package(types, "list", list_leaves(&["label"]));
        assert_eq!(
            read(&package, CheckedPackageReadLimits::bounded()),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &leaves_pointer(&package, &id),
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                &id,
            ),
            "malformed next: {}",
            package["semantic_graph"]["nodes"],
        );
    }
}

#[trace("FR-038-AC-147", "FR-038-AC-152")]
#[test]
fn tc_226_a_severed_labelled_group_reaches_the_optional_body_refusal() {
    let mut types = wrapped_list(&["label"]);
    types[0].1["body"]["members"]
        .as_array_mut()
        .expect("record members")
        .iter_mut()
        .find(|field| field["name"] == "next")
        .expect("next field")["value"] = empty();
    let (package, application) = equality_package(types, "list", list_leaves(&["label"]));
    assert_eq!(
        read(&package, CheckedPackageReadLimits::bounded()),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &leaves_pointer(&package, &application),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            &application,
        ),
    );
}

fn field(name: &str) -> String {
    format!("field:{name}")
}

/// Equality over `Node` is admitted end to end with its text leaf and its
/// recursion leaf; the text leaf alone refuses `operation-law-missing` at
/// `operation.leaves`, and a further text leaf refuses
/// `operation-law-mismatch` at that entry.
#[trace("TC-048", "FR-038-AC-70")]
#[test]
fn tc_048_the_reader_admits_equality_over_a_recursive_record() {
    let label = text_leaf(&[field("label")]);
    let [some, payload] = some_segments();
    let recursion = recursion_leaf(&[field("next"), some, payload, "recursion:0".into()]);
    let limits = CheckedPackageReadLimits::bounded();

    let (package, _) =
        equality_package(node_types(), "node", vec![label.clone(), recursion.clone()]);
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected V2 admission, got {other:?}"),
    }

    let (package, id) = equality_package(node_types(), "node", vec![label.clone()]);
    assert_eq!(
        read(&package, limits),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &leaves_pointer(&package, &id),
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            &id,
        ),
        "the text leaf alone"
    );

    let (package, id) =
        equality_package(node_types(), "node", vec![label.clone(), recursion, label]);
    assert_eq!(
        read(&package, limits),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("{}/2", leaves_pointer(&package, &id)),
            Some(CheckedPackageRefusalCause::OperationLawMismatch),
            &id,
        ),
        "a text leaf with no place left"
    );
}

/// A tuple type named `name`, one position per type name.
fn tuple_of(name: &str, positions: &[&str]) -> Typed {
    let members: Vec<Value> = positions.iter().map(|ty| reference(ty)).collect();
    node(
        name,
        "composite_type",
        "tuple",
        name,
        positions.iter().copied(),
        "type",
        json!({"term": "aggregate", "members": members}),
    )
}

/// `record Cell { item: (Text[nfc], Option<Cell>); }`: the cycle passes
/// through a tuple and an option, and the record is entered at the empty path,
/// so its recursion leaf reads `recursion:0` (QSpec FR-322 "Structural leaf
/// walk"). The text leaf alone and a recursion leaf reading `recursion:1` are
/// refused.
#[trace("TC-048", "FR-038-AC-70")]
#[test]
fn tc_048_the_reader_admits_equality_over_a_record_cycling_through_a_tuple() {
    let types = || {
        vec![
            in_group(record("cell", &[("item", "cell_item")])),
            in_group(tuple_of("cell_item", &["nfc", "cell_next"])),
            in_group(some_of("cell_next", "cell")),
        ]
    };
    let item = |last: &[&str]| -> Vec<String> {
        [field("item")]
            .into_iter()
            .chain(last.iter().map(|segment| (*segment).to_owned()))
            .collect()
    };
    let text = text_leaf(&item(&["position:0"]));
    let recursion =
        |depth: &str| recursion_leaf(&item(&["position:1", "member:Some", "position:0", depth]));
    let limits = CheckedPackageReadLimits::bounded();

    let (package, _) = equality_package(
        types(),
        "cell",
        vec![text.clone(), recursion("recursion:0")],
    );
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected V2 admission, got {other:?}"),
    }

    let (package, id) = equality_package(types(), "cell", vec![text.clone()]);
    assert_eq!(
        read(&package, limits),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &leaves_pointer(&package, &id),
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            &id,
        ),
        "the text leaf alone"
    );

    let (package, id) = equality_package(types(), "cell", vec![text, recursion("recursion:1")]);
    assert_eq!(
        read(&package, limits),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("{}/1/path", leaves_pointer(&package, &id)),
            Some(CheckedPackageRefusalCause::OperationLawMismatch),
            &id,
        ),
        "a recursion leaf with the wrong depth"
    );
}

/// A ring of `size` records, each holding one text field and an optional
/// field naming the next, the last naming the first, and the leaves of the
/// ring compared at the first: its `size` text leaves and its one recursion
/// leaf (the last record's field reenters the first).
fn ring_of_text_records(size: usize) -> (Value, String) {
    let names: Vec<String> = (0..size).map(|at| format!("ring{at}")).collect();
    let optionals: Vec<String> = (0..size).map(|at| format!("ring{at}_next")).collect();
    let mut types = Vec::new();
    for at in 0..size {
        let following = (at + 1) % size;
        types.push(in_group(record(
            &names[at],
            &[("t", "nfc"), ("next", &optionals[at])],
        )));
        types.push(in_group(some_of(&optionals[at], &names[following])));
    }
    let step = |times: usize| -> Vec<String> {
        (0..times)
            .flat_map(|_| {
                let [some, payload] = some_segments();
                [field("next"), some, payload]
            })
            .collect()
    };
    let mut leaves: Vec<Value> = (0..size)
        .map(|at| {
            let mut leaf = step(at);
            leaf.push(field("t"));
            text_leaf(&leaf)
        })
        .collect();
    let mut reentry = step(size);
    reentry.push("recursion:0".to_owned());
    leaves.push(recursion_leaf(&reentry));
    equality_package(types, &names[0], leaves)
}

/// The least work limit under which `package` reads to admission, found by
/// bisection against the real reader.
fn work_read_used(package: &Value) -> u64 {
    let bounded = CheckedPackageReadLimits::bounded();
    let (mut low, mut high) = (0_u64, bounded.work);
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut limits = bounded;
        limits.work = middle;
        match read(package, limits) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            _ => low = middle,
        }
    }
    high
}

/// A reader-admissible operation with one forwarding edge at the requested
/// structural caller. The established package builder mints graph keys and
/// source entries; `settle` updates the edited application's key and identity.
fn forwarding_operation_package(route: &str) -> (Value, String) {
    let types = match route {
        "inner:0" => vec![
            collection_of("set", "set", "integer"),
            alias_of("forward", "set"),
        ],
        "result_inner" => vec![
            collection_of("set", "set", "integer"),
            alias_of("forward", "set"),
            collection_of("source", "sequence", "set"),
        ],
        "leaf edge" => vec![
            alias_of("forward", "integer"),
            record("record", &[("value", "forward")]),
        ],
        _ => panic!("unknown structural route"),
    };
    let root = match route {
        "inner:0" => "forward",
        "result_inner" => "source",
        "leaf edge" => "record",
        _ => unreachable!(),
    };
    let (mut package, _) = equality_package(types, root, Vec::new());
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["term"] == "application")
        .expect("application");
    if route != "leaf edge" {
        let first_argument =
            package["semantic_graph"]["nodes"][position]["body"]["arguments"][0].clone();
        let forward = (route == "result_inner").then(|| {
            package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .find(|node| node["semantic_form"] == "alias")
                .expect("forwarding alias")["node_id"]
                .clone()
        });
        let application = &mut package["semantic_graph"]["nodes"][position];
        application["semantic_form"] = json!("collection");
        application["body"]["operator"] = json!("collection");
        application["body"]["operation"]["identity"] = json!(if route == "inner:0" {
            "quire.op.collection.contains"
        } else {
            "quire.op.collection.flatten"
        });
        if route == "inner:0" {
            application["body"]["arguments"] =
                json!([first_argument, literal("integer", "integer", "0")]);
        } else {
            let forward = forward.expect("result type alias");
            application["semantic_type"] = forward.clone();
            application["body"]["result_type"] = forward;
            application["body"]["arguments"] = json!([first_argument]);
        }
        settle(&mut package);
    }
    let id = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("application key")
        .to_owned();
    (package, id)
}

#[trace("TC-048", "FR-038-AC-224", "FR-038-AC-229", "FR-038-AC-230")]
#[test]
fn structural_forwarding_reader_routes_have_exact_and_one_less_work_boundaries() {
    for route in ["inner:0", "result_inner", "leaf edge"] {
        let (package, id) = forwarding_operation_package(route);
        let used = work_read_used(&package);
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = used;
        assert!(
            matches!(
                read(&package, limits),
                CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{route} must admit at its measured limit"
        );
        limits.work = used - 1;
        let operation = leaves_pointer(&package, &id)
            .strip_suffix("/leaves")
            .expect("operation path")
            .to_owned();
        match read(&package, limits) {
            CheckedPackageV2ReadResult::Incomplete(incomplete) => {
                assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work, "{route}");
                assert_eq!(incomplete.limit, used - 1, "{route}");
                assert_eq!(incomplete.consumed, used, "{route}");
            }
            other => panic!("{route} one-less Work must reach the reader: {other:?}"),
        }
        // The reader still charges graph-cycle edges after the operation.
        // Find a limit that cuts off one of this route's operation charges,
        // and assert that the public outcome keeps the operation's Work type
        // and locus instead of changing it to an ill-typed refusal.
        let at_operation = (0..used).find_map(|work| {
            let mut limits = CheckedPackageReadLimits::bounded();
            limits.work = work;
            match read(&package, limits) {
                CheckedPackageV2ReadResult::Incomplete(incomplete)
                    if incomplete.path == Some(pointer(&operation)) =>
                {
                    Some(incomplete)
                }
                _ => None,
            }
        });
        let incomplete = at_operation.expect("reader must reach structural Work");
        assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work, "{route}");
        assert_eq!(incomplete.consumed, incomplete.limit + 1, "{route}");
    }
}

/// A ring of 12 records with its 12 text leaves and its recursion leaf
/// admits at the exact work a read used, and one below that returns
/// `incomplete` for `work`.
#[trace("TC-048", "FR-038-AC-72")]
#[test]
fn tc_048_a_ring_of_twelve_records_is_decided_at_its_exact_work() {
    let (package, _) = ring_of_text_records(12);
    let used = work_read_used(&package);
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = used;
    assert!(
        matches!(
            read(&package, limits),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "the exact work admits"
    );
    limits.work = used - 1;
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(incomplete.limit, used - 1);
        }
        other => panic!("one below the work a read used must be incomplete, got {other:?}"),
    }
}

/// Ten records that each hold a text field and an optional field naming every
/// other record, compared at the first with `leaves` empty, return
/// `incomplete` for `work` at `operation.leaves` under the default read
/// limits: the `(n - 1)!` leaves they unfold to are never listed.
#[trace("TC-048", "FR-038-AC-72")]
#[test]
fn tc_048_ten_records_naming_each_other_exhaust_the_work_budget() {
    let size = 10;
    let names: Vec<String> = (0..size).map(|at| format!("r{at}")).collect();
    let optionals: Vec<String> = (0..size).map(|at| format!("o{at}")).collect();
    let mut types = Vec::new();
    for at in 0..size {
        let mut fields: Vec<(&str, &str)> = vec![("t", "nfc")];
        fields.extend(
            (0..size)
                .filter(|other| *other != at)
                .map(|other| (optionals[other].as_str(), optionals[other].as_str())),
        );
        types.push(in_group(record(&names[at], &fields)));
        types.push(in_group(some_of(&optionals[at], &names[at])));
    }
    let (package, id) = equality_package(types, &names[0], Vec::new());
    let limits = CheckedPackageReadLimits::bounded();
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(incomplete.limit, limits.work);
            assert_eq!(
                incomplete.path,
                Some(pointer(&leaves_pointer(&package, &id)))
            );
        }
        other => panic!("expected incomplete for work, got {other:?}"),
    }
}

/// A cycle of 20000 record nodes, each holding an integer field and naming
/// the next, the last naming the first and also holding one text field,
/// admits with its one 20000-segment text leaf and its recursion leaf on a
/// thread whose stack is 256 KiB, under byte, node, edge and work limits
/// raised to admit it; the default limits read it as `incomplete` for
/// `bytes`, since the package is larger than the default 1 MiB.
#[trace("TC-048", "FR-038-AC-72")]
#[test]
fn tc_048_a_cycle_of_twenty_thousand_records_is_decided_on_a_small_stack() {
    const SIZE: usize = 20_000;
    let names: Vec<String> = (0..SIZE).map(|at| format!("link{at}")).collect();
    let mut types = Vec::with_capacity(SIZE);
    for at in 0..SIZE {
        let following = &names[(at + 1) % SIZE];
        let mut fields = vec![("k", "integer"), ("next", following.as_str())];
        if at + 1 == SIZE {
            fields.insert(1, ("t", "nfc"));
        }
        types.push(in_group(record(&names[at], &fields)));
    }
    let text = path(&[], &field("next"), SIZE - 1, &field("t"));
    let reentry = path(&[], &field("next"), SIZE, "recursion:0");
    assert_eq!(text.len(), SIZE);
    let (package, _) = equality_package(
        types,
        &names[0],
        vec![text_leaf(&text), recursion_leaf(&reentry)],
    );

    let mut limits = CheckedPackageReadLimits::bounded();
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Bytes);
        }
        other => panic!("the default byte limit is below the package, got {other:?}"),
    }
    limits.bytes = 64 * 1024 * 1024;
    limits.nodes = 100_000;
    limits.edges = 1_000_000;
    limits.occurrences = 1_000_000;
    limits.work = 1_000_000_000;
    let reader = std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || match read(&package, limits) {
            CheckedPackageV2ReadResult::Admitted(_) => Ok(()),
            other => Err(format!("{other:?}")),
        })
        .expect("spawn the 256 KiB reader thread");
    assert_eq!(reader.join().expect("the reader did not overflow"), Ok(()));
}
