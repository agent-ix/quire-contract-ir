// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Existing QSpec fixture setup shared by its conformance driver and the
//! whole-read allocation registry. These functions only prepare inputs.

use crate::support::checked_package as fixture_support;
use fixture_support::retention_definition;
use quire_contract_model::{
    CheckedArtifactRef, CheckedExtensionClauseSchema, CheckedExtensionDefinition,
    CheckedExtensionGrammarSchema, CheckedExtensionNodeSchema, CheckedExtensionProduction,
    CheckedExtensionTypedNodeSchema, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn read_json(path: &Path) -> Result<Value, String> {
    let mut value = read_json_wire(path)?;
    // Tests author and mutate packages in resolved index form; `read_package`
    // writes the wire form back.
    fixture_support::resolve_operands(&mut value);
    fixture_support::resolve_vector_rows(&mut value);
    Ok(value)
}

pub(crate) fn read_json_wire(path: &Path) -> Result<Value, String> {
    let text = std::fs::read(path)
        .map_err(|error| format!("{} cannot be read: {error}", path.display()))?;
    serde_json::from_slice(&text)
        .map_err(|error| format!("{} is not JSON: {error}", path.display()))
}

pub(crate) fn quota_definition() -> CheckedExtensionDefinition {
    CheckedExtensionDefinition {
        grammar_schema: CheckedExtensionGrammarSchema {
            productions: vec![CheckedExtensionProduction {
                keyword: "quota".into(),
                clauses: vec!["limit".into()],
            }],
        },
        typed_node_schema: CheckedExtensionTypedNodeSchema {
            nodes: vec![CheckedExtensionNodeSchema {
                keyword: "quota".into(),
                node_kind: "quota".into(),
                clauses: vec![CheckedExtensionClauseSchema {
                    name: "limit".into(),
                    type_ref: "Int[1, 100]".into(),
                }],
            }],
        },
    }
}

pub(crate) fn policy_definition() -> CheckedExtensionDefinition {
    CheckedExtensionDefinition {
        grammar_schema: CheckedExtensionGrammarSchema {
            productions: vec![CheckedExtensionProduction {
                keyword: "policy".into(),
                clauses: vec!["retain".into()],
            }],
        },
        typed_node_schema: CheckedExtensionTypedNodeSchema {
            nodes: vec![CheckedExtensionNodeSchema {
                keyword: "policy".into(),
                node_kind: "policy".into(),
                clauses: vec![CheckedExtensionClauseSchema {
                    name: "retain".into(),
                    type_ref: "retention".into(),
                }],
            }],
        },
    }
}

pub(crate) fn evidence_for_published(
    package: &Value,
    name: &str,
    domain_document: &Path,
    hold_retention: bool,
    document_override: Option<&[u8]>,
    fixtures_dir: &Path,
) -> Result<CheckedPackageEvidence, String> {
    let mut evidence = CheckedPackageEvidence::new();
    if hold_retention
        && package["lock"]["definition_selections"]
            .as_array()
            .is_some_and(|rows| {
                rows.iter()
                    .any(|row| row["authority"] == "example" && row["identity"] == "retention")
            })
    {
        evidence
            .insert_extension_definition(
                CheckedArtifactRef {
                    authority: "example".into(),
                    identity: "retention".into(),
                },
                retention_definition(),
            )
            .map_err(|error| error.to_string())?;
    }
    if package["lock"]["definition_selections"]
        .as_array()
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row["authority"] == "example" && row["identity"] == "quota")
        })
    {
        evidence
            .insert_extension_definition(
                CheckedArtifactRef {
                    authority: "example".into(),
                    identity: "quota".into(),
                },
                quota_definition(),
            )
            .map_err(|error| error.to_string())?;
    }
    if package["lock"]["definition_selections"]
        .as_array()
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row["authority"] == "example" && row["identity"] == "policy")
        })
    {
        evidence
            .insert_extension_definition(
                CheckedArtifactRef {
                    authority: "example".into(),
                    identity: "policy".into(),
                },
                policy_definition(),
            )
            .map_err(|error| error.to_string())?;
    }
    for feature in package["lock"]["required_features"]
        .as_array()
        .ok_or("lock.required_features is not an array")?
    {
        evidence.support_feature(
            feature
                .as_str()
                .ok_or("a required feature is not a string")?,
        );
    }
    if package["lock"]["dependency_selections"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["identity"] == "test/predicates"))
    {
        let dependency = read_json(&fixtures_dir.join("positive-dependency-library.json"))?;
        let dependency_evidence = evidence_for_published(
            &dependency,
            "published dependency",
            domain_document,
            true,
            None,
            fixtures_dir,
        )?;
        let CheckedPackageV2ReadResult::Admitted(dependency) = crate::read_and_account(
            &fixture_support::canonical(&dependency),
            CheckedPackageReadLimits::bounded(),
            &dependency_evidence,
        ) else {
            return Err("published dependency library did not admit".into());
        };
        evidence
            .insert_dependency_package("test/predicates", dependency)
            .map_err(|error| format!("dependency evidence insertion failed: {error}"))?;
    }
    // Supply the one domain document this harness owns. Leave malformed or
    // unselected root owners to the reader's own refusal path.
    let roots = package["semantic_graph"]["nodes"]
        .as_array()
        .ok_or_else(|| format!("{name}: semantic_graph.nodes is not an array"))?
        .iter()
        .filter(|node| node["semantic_form"] == "model_root");
    for root in roots {
        let Some(identity) = root["owner"]["identity"].as_str() else {
            continue;
        };
        if identity != "acme/orders" && identity != "acme/shapes" {
            continue;
        }
        let selected_document = if identity == "acme/shapes" {
            fixtures_dir
                .join("..")
                .join("domain-package-acme-shapes.json")
        } else {
            domain_document.to_path_buf()
        };
        let document = if identity == "acme/orders" {
            match document_override {
                Some(bytes) => bytes.to_vec(),
                None => std::fs::read(&selected_document).map_err(|error| {
                    format!("{} cannot be read: {error}", selected_document.display())
                })?,
            }
        } else {
            std::fs::read(&selected_document).map_err(|error| {
                format!("{} cannot be read: {error}", selected_document.display())
            })?
        };
        serde_json::from_slice::<Value>(&document)
            .map_err(|error| format!("{} is not JSON: {error}", selected_document.display()))?;
        evidence
            .insert_domain_package_document(identity, document)
            .map_err(|error| format!("{name}: evidence insertion failed: {error}"))?;
    }
    Ok(evidence)
}

pub(crate) const POSITIVE_ALL: [&str; 19] = [
    "positive-all-families.json",
    "positive-clause-operations.json",
    "positive-control-operations.json",
    "positive-dependency-call.json",
    "positive-dependency-library.json",
    "positive-extension-nodes.json",
    "positive-integer-literal-references.json",
    "positive-model-tuple-union.json",
    "positive-nominal-identities.json",
    "positive-operation-identities.json",
    "positive-recursive-records.json",
    "positive-state-transition.json",
    "positive-tick-timed-model-nodes.json",
    "positive-timed-clause-nodes.json",
    "positive-timed-model-nodes.json",
    "positive-timed-slot-nodes.json",
    "positive-two-owners-a.json",
    "positive-two-owners-b.json",
    "positive-union-nodes.json",
];

pub(crate) const DOMAIN_DOCUMENT: &str = "domain-package-acme-orders.json";

pub(crate) fn vector_selected_documents(
    proposal: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let mapping = read_json(&proposal.join("selected-documents.json"))?;
    selected_document_assignments(&mapping, proposal)
}

pub(crate) fn selected_document_assignments(
    mapping: &Value,
    proposal: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let assignments = mapping["selected_documents"]
        .as_object()
        .ok_or("selected-documents.json: selected_documents is not an object")?;
    if assignments.is_empty() {
        return Err("selected-documents.json: selected_documents is empty".into());
    }
    let mut documents = BTreeMap::new();
    for (fixture, document) in assignments {
        let name = fixture
            .strip_prefix("fixtures/")
            .ok_or_else(|| format!("selected-documents.json: invalid fixture path {fixture:?}"))?;
        if !POSITIVE_ALL.contains(&name) {
            return Err(format!(
                "selected-documents.json: unlisted positive fixture {fixture}"
            ));
        }
        let document = document.as_str().ok_or_else(|| {
            format!("selected-documents.json: {fixture} document is not a string")
        })?;
        if !document.starts_with("domain-package-")
            || !document.ends_with(".json")
            || document.contains('/')
            || document.contains('\\')
        {
            return Err(format!(
                "selected-documents.json: invalid document {document:?}"
            ));
        }
        read_json(&proposal.join(document))?;
        documents.insert(name.to_owned(), document.to_owned());
    }
    Ok(documents)
}

pub(crate) fn positive_document<'a>(
    name: &str,
    documents: &'a BTreeMap<String, String>,
) -> &'a str {
    documents
        .get(name)
        .map(String::as_str)
        .unwrap_or(DOMAIN_DOCUMENT)
}

pub(crate) fn apply_vector_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    match (patch["op"].as_str().expect("published operation"), target) {
        ("add", Value::Array(items)) if member == "-" => items.push(patch["value"].clone()),
        ("add", Value::Object(map)) => {
            map.insert(member.to_owned(), patch["value"].clone());
        }
        ("replace", Value::Object(map)) => {
            *map.get_mut(member).expect("replaced member") = patch["value"].clone();
        }
        ("remove", Value::Array(items)) => {
            items.remove(member.parse::<usize>().expect("array index"));
        }
        ("remove", Value::Object(map)) => {
            map.remove(member).expect("removed member");
        }
        (operation, _) => panic!("unsupported published patch {operation}: {path}"),
    }
}

/// Add the authored siblings around the original derived Integer node. The
/// source-map rows follow each new identity, so the reader decides the graph
/// identity rule rather than failing on missing source correspondence.
pub(crate) fn ordinal_case_package(base: &Value, case: &Value) -> (Value, Vec<usize>) {
    let mut package = base.clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph nodes");
    let original = nodes
        .iter()
        .position(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
        .expect("published derived Integer node");
    let template = nodes[original].clone();
    assert_eq!(template["node_id"]["ordinal"], 0);
    assert_eq!(template["semantic_type"], template["node_id"]);
    let source = package["source_map"]
        .as_array()
        .expect("published source map")
        .iter()
        .find(|row| row["node_id"] == template["node_id"])
        .expect("Integer source correspondence")
        .clone();
    let siblings = case["siblings"].as_array().expect("authored siblings");
    assert!(!siblings.is_empty(), "{} has no siblings", case["id"]);
    let before = match case["placement"].as_str().expect("placement") {
        "before" => true,
        "after" => false,
        other => panic!("unknown placement {other}"),
    };
    let mut positions = Vec::with_capacity(siblings.len());
    for (index, sibling) in siblings.iter().enumerate() {
        let ordinal = sibling["ordinal"].as_u64().expect("producer ordinal");
        let mut node = template.clone();
        node["node_id"]["ordinal"] = json!(ordinal);
        node["semantic_type"] = node["node_id"].clone();
        let position = if before {
            original + index
        } else {
            package["semantic_graph"]["nodes"]
                .as_array()
                .expect("graph nodes")
                .len()
        };
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("graph nodes")
            .insert(position, node.clone());
        let mut correspondence = source.clone();
        correspondence["node_id"] = node["node_id"].clone();
        package["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(correspondence);
        positions.push(position);
    }
    (package, positions)
}

pub(crate) fn apply_node_index_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("published patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    let operation = patch["op"].as_str().expect("published patch operation");
    match (operation, target) {
        ("replace", Value::Array(items)) => {
            let index: usize = member.parse().expect("published array index");
            *items.get_mut(index).expect("published replacement target") = patch["value"].clone();
        }
        ("add", Value::Array(items)) if member == "-" => items.push(patch["value"].clone()),
        ("remove", Value::Array(items)) => {
            let index: usize = member.parse().expect("published array index");
            items.remove(index);
        }
        ("add", Value::Array(items)) => {
            let index: usize = member.parse().expect("published array index");
            assert!(index <= items.len(), "{path}: published insertion target");
            items.insert(index, patch["value"].clone());
        }
        _ => panic!("unsupported published patch {operation} at {path}"),
    }
}

/// The original NI stage-only oracle: later semantic refusals are permitted,
/// but operand grammar and the identified duplicate-key refusal must pass.
pub(crate) fn operand_checks_pass(actual: &CheckedPackageV2ReadResult) -> bool {
    match actual {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            refusal.code != quire_contract_model::CheckedPackageRefusalCode::MalformedWire
                && refusal.loci.iter().all(|locus| {
                    locus
                        .path
                        .as_ref()
                        .is_none_or(|path| path.as_str() != "/semantic_graph/nodes/4/node_id")
                })
        }
        _ => true,
    }
}

/// The replacement of `package` at `pointer`, or why it cannot be made.
pub(crate) fn replace_at(
    package: &mut Value,
    pointer: &str,
    replacement: &Value,
) -> Result<(), String> {
    let slot = package
        .pointer_mut(pointer)
        .ok_or_else(|| format!("pointer {pointer} resolves to nothing in the package"))?;
    *slot = replacement.clone();
    Ok(())
}

/// The original flattened-control oracle rejects malformed wire, while
/// permitting later semantic refusals. An unavailable catalogue is not a run.
pub(crate) fn flat_body_control_passes(actual: &CheckedPackageV2ReadResult) -> bool {
    match actual {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            refusal.code != quire_contract_model::CheckedPackageRefusalCode::MalformedWire
        }
        CheckedPackageV2ReadResult::CatalogUnavailable(_) => false,
        _ => true,
    }
}
