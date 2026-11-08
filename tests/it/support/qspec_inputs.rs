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
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::Value;
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
        let CheckedPackageV2ReadResult::Admitted(dependency) = CheckedPackageV2::read(
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
