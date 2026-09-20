// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Shared helpers for the vendored QSpec I04 CheckedPackage fixtures.

#![allow(dead_code)] // Each test binary uses a different subset of these helpers.

use quire_contract_ir::{
    CheckedArtifactLocator, CheckedDomainPackageLocator, CheckedPackageEvidence,
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub const COMPLETE_VALUE_FEATURE: &str = "quire.value.complete/v1";
pub const NODE_DOMAIN: &str = "quire.checked-semantic-node/v1";

/// Exact read-limits `work` boundary that admits `v2_all_families()`: body
/// terms 35 (the sum of each of the 26 nodes' own `validate_body` charge) +
/// graph edges 38 (every `semantic_type`, `dependencies` and body-reference
/// edge the Tarjan recursion walk in `validate_recursion` traverses, one
/// charge per edge: `semantic_type` 25 of the 26 nodes — the one self-typed
/// node contributes none — + `dependencies` 5 + body-reference 8, the
/// non-frame `literal.type`, `application.result_type` and `reference.target`
/// edges that still enter adjacency) = 73. Frame body member entries
/// (`modifies`/`creates`/`deletes`) are
/// declared dependencies, not independent successor edges — FR-340 frame
/// semantics resolve them against `dependencies` alone, so
/// `validate_frame_body` does not forward them to Loop 2's successor
/// collection and they carry no separate edge charge here. The fixture
/// carries no nominal-form node (`enum`/`dimension`/`unit`/enum member) and
/// no diagnostics entries, so neither `validate_nominal_nodes` nor
/// `validate_diagnostics` charges anything here; a fixture that gained
/// either would need its own added term in this sum, and did: IR-216 added
/// `validate_application_keys` (one charge per of the fixture's 4
/// application-bodied nodes — `function.call`, `temporal.clause`,
/// `protocol.control`, `claim.clause` — = 4) and `validate_operations` (one
/// charge per application node plus one per declared law plus one per leaf
/// entry: `function.call` 1 law 0 leaves 0 = 1; each of `temporal.clause`,
/// `protocol.control` and `claim.clause` 1 law, 0 leaves = 2 each = 6;
/// total 7), for 73 + 4 + 7 = 84. Cross-checked against
/// `CheckedPackageV2::read`'s real admit/refuse boundary by
/// `tc_048_v2_reader_reports_exact_and_one_over_limits` and
/// `tc_048_shipped_default_read_limits_are_exact_and_finite`, so a drift
/// between this hand-derived figure and the reader's actual charge fails
/// there rather than silently. Shared between `complete_v1_checked_package`
/// and `checked_package_v2_reader` so a change to the vendored all-families
/// fixture cannot silently move the boundary in only one of them.
pub const ALL_FAMILIES_READ_WORK: u64 = 84;

/// Reads one vendored file under `tests/fixtures/checked-package/`.
pub fn fixture(relative: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/checked-package")
        .join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("vendored fixture {} is readable: {error}", path.display()));
    serde_json::from_str(&text).expect("vendored fixture is JSON")
}

pub fn v2_all_families() -> Value {
    fixture("checked-package-v2/fixtures/positive-all-families.json")
}

pub fn v2_nominal() -> Value {
    fixture("checked-package-v2/fixtures/positive-nominal-identities.json")
}

/// RFC 8785 bytes for the ASCII, integer-only fixtures (sorted members).
pub fn canonical(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("canonical test JSON")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn node_id(digest: &str) -> Value {
    json!({"domain": NODE_DOMAIN, "digest": digest})
}

pub fn typed_node_id(digest: &str) -> quire_contract_ir::CheckedNodeId {
    serde_json::from_value(node_id(digest)).expect("node id")
}

pub fn locator(artifact: &Value) -> CheckedArtifactLocator {
    let text = |key: &str| -> Box<str> {
        artifact[key]
            .as_str()
            .unwrap_or_else(|| panic!("artifact member {key}"))
            .into()
    };
    CheckedArtifactLocator {
        authority: text("authority"),
        identity: text("identity"),
        revision_namespace: artifact["revision"]["namespace"]
            .as_str()
            .expect("revision namespace")
            .into(),
        revision_value: artifact["revision"]["value"]
            .as_str()
            .expect("revision value")
            .into(),
        domain: text("digest_domain"),
    }
}

pub fn domain_package_locator(model: &Value) -> CheckedDomainPackageLocator {
    let text = |key: &str| -> Box<str> {
        model[key]
            .as_str()
            .unwrap_or_else(|| panic!("domain package member {key}"))
            .into()
    };
    CheckedDomainPackageLocator {
        identity: text("identity"),
        version: text("version"),
    }
}

/// Every locked raw byte artifact in `package`, including the diagnostic
/// catalog. Domain package selections are a separate digest domain.
pub fn locked_artifacts(package: &Value) -> Vec<Value> {
    let lock = &package["lock"];
    let mut artifacts = Vec::new();
    let list = |key: &str| lock[key].as_array().cloned().unwrap_or_default();
    artifacts.extend(list("sources"));
    artifacts.push(lock["edition"]["definition"].clone());
    artifacts.extend(
        list("profile_selections")
            .into_iter()
            .map(|selection| selection["definition"].clone()),
    );
    artifacts.extend(list("definition_selections"));
    artifacts.extend(
        list("dependency_selections")
            .into_iter()
            .map(|selection| selection["definition"].clone()),
    );
    artifacts.push(package["diagnostics"]["catalog"].clone());
    artifacts
}

/// Attested evidence for every locked artifact plus the complete-value feature.
pub fn evidence_for(package: &Value) -> CheckedPackageEvidence {
    let mut evidence = CheckedPackageEvidence::new();
    for artifact in locked_artifacts(package) {
        evidence.insert_artifact_digest(
            locator(&artifact),
            artifact["digest"].as_str().expect("artifact digest"),
        );
    }
    // A lock's compiled-model selections are `sha256-jcs` domain packages,
    // typed separately from raw byte artifacts.
    for model in package["lock"]["model_selections"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        let digest = model["digest"].as_str().expect("model digest");
        evidence.insert_domain_package_digest(domain_package_locator(&model), digest);
    }
    evidence.support_feature(COMPLETE_VALUE_FEATURE);
    evidence
}

/// Rebuilds the identity projection from the graph and re-derives the package id.
pub fn refresh_identity(package: &mut Value) {
    let projection = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .cloned()
        .map(|mut node| {
            node.as_object_mut()
                .expect("node object")
                .remove("occurrences");
            node
        })
        .collect::<Vec<_>>();
    package["identity_preimage"]["identity_projection"] = Value::Array(projection);
    for key in [
        "edition",
        "profile_selections",
        "definition_selections",
        "model_selections",
        "required_features",
        "dependency_selections",
    ] {
        package["identity_preimage"][key] = package["lock"][key].clone();
    }
    package["package_id"]["digest"] = json!(sha256_hex(&canonical(&package["identity_preimage"])));
}

/// Nesting depth counted the way the reader charges it (scalars are one).
pub fn json_depth(value: &Value) -> u64 {
    match value {
        Value::Array(values) => 1 + values.iter().map(json_depth).max().unwrap_or(0),
        Value::Object(values) => 1 + values.values().map(json_depth).max().unwrap_or(0),
        _ => 1,
    }
}

/// Applies the RFC 6902 `add`, `remove`, `replace` and `move` operations the
/// vendored vectors use.
pub fn apply_patch(target: &mut Value, patch: &Value) {
    for operation in patch.as_array().expect("patch array") {
        let path = operation["path"].as_str().expect("patch path");
        match operation["op"].as_str().expect("patch op") {
            "replace" => {
                *target.pointer_mut(path).expect("replace target exists") =
                    operation["value"].clone();
            }
            "remove" => {
                remove(target, path);
            }
            "add" => insert(target, path, operation["value"].clone()),
            "move" => {
                let from = operation["from"].as_str().expect("move from");
                let value = remove(target, from);
                insert(target, path, value);
            }
            other => panic!("unsupported patch op {other}"),
        }
    }
}

fn split(path: &str) -> (&str, String) {
    let (parent, last) = path.rsplit_once('/').expect("JSON pointer");
    (parent, last.replace("~1", "/").replace("~0", "~"))
}

fn remove(target: &mut Value, path: &str) -> Value {
    let (parent, key) = split(path);
    match target.pointer_mut(parent).expect("patch parent exists") {
        Value::Object(members) => members.remove(&key).expect("removed member exists"),
        Value::Array(items) => items.remove(key.parse::<usize>().expect("array index")),
        _ => panic!("patch parent is a container"),
    }
}

fn insert(target: &mut Value, path: &str, value: Value) {
    let (parent, key) = split(path);
    match target.pointer_mut(parent).expect("patch parent exists") {
        Value::Object(members) => {
            members.insert(key, value);
        }
        Value::Array(items) => {
            let index = if key == "-" {
                items.len()
            } else {
                key.parse::<usize>().expect("array index")
            };
            items.insert(index, value);
        }
        _ => panic!("patch parent is a container"),
    }
}

/// The vendored nominal node-identity vectors.
pub fn node_identity_vectors() -> Value {
    fixture("checked-package-v2/node-identity-vectors.json")
}

/// Recomputes each preimage's key in list order, rewriting every later
/// preimage that names a changed key. Vectors are listed in dependency order,
/// so one pass carries a rekey through every dependant.
pub fn rekey(preimages: &mut [Value], keys: &[String]) -> Vec<String> {
    let mut current = keys.to_vec();
    for position in 0..preimages.len() {
        let fresh = sha256_hex(&canonical(&preimages[position]));
        if fresh != current[position] {
            for later in preimages.iter_mut().skip(position + 1) {
                replace_digest(later, &current[position], &fresh);
            }
            current[position] = fresh;
        }
    }
    current
}

fn replace_digest(value: &mut Value, stale: &str, fresh: &str) {
    match value {
        Value::String(text) if text == stale => *text = fresh.to_owned(),
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| replace_digest(item, stale, fresh)),
        Value::Object(members) => members
            .values_mut()
            .for_each(|member| replace_digest(member, stale, fresh)),
        _ => {}
    }
}

/// Builds a canonical V2 package from nominal `(preimage, key)` pairs in the
/// recorded nominal fixture's lock, deriving each node's family, type,
/// dependencies and body from its preimage.
pub fn nominal_package(members: &[(Value, String)]) -> Value {
    let mut package = v2_nominal();
    let source = package["lock"]["sources"][0].clone();
    let mut nodes = Vec::with_capacity(members.len());
    let mut source_map = Vec::with_capacity(members.len());
    for (position, (preimage, key)) in members.iter().enumerate() {
        let id = node_id(key);
        // Every nominal form here is a named, source-declared `scalar_type`
        // (or the `enum_value` its enum declares); the schema's
        // `DeclarationTagRules`/`DeclarationOccurrenceRule` (FR-208) require
        // a `declaration` member on the former, exactly matching each node's
        // `declaration`-role occurrence below, and forbid it on the latter.
        // FR-322's `declaration-nominal-mismatch` rule pins a nominal node's
        // `declaration.qualified_name` to its own preimage's
        // `qualified_declaration`, so that member is reused verbatim rather
        // than invented.
        let declaration_name = preimage["qualified_declaration"].clone();
        let (tag, form, semantic_type, dependencies, body, declaration) =
            match preimage["version"].as_str().expect("preimage version") {
                "quire.enum-declaration-node/v1" => (
                    "scalar_type",
                    "enum",
                    id.clone(),
                    json!([]),
                    json!({"term":"aggregate","members":[]}),
                    Some(json!({"qualified_name": declaration_name})),
                ),
                "quire.enum-member-node/v1" => (
                    "value",
                    "enum_value",
                    preimage["declaration_node_id"].clone(),
                    json!([preimage["declaration_node_id"]]),
                    json!({
                        "term": "literal",
                        "type": preimage["declaration_node_id"],
                        "value_kind": "enum",
                        "value": preimage["case"],
                    }),
                    None,
                ),
                "quire.dimension-node/v1" => (
                    "scalar_type",
                    "dimension",
                    id.clone(),
                    Value::Array(
                        preimage["terms"]
                            .as_array()
                            .expect("terms")
                            .iter()
                            .map(|term| term["dimension_node_id"].clone())
                            .collect(),
                    ),
                    json!({"term":"aggregate","members":[]}),
                    Some(json!({"qualified_name": declaration_name})),
                ),
                "quire.unit-node/v1" => {
                    let mut dependencies = vec![preimage["dimension_node_id"].clone()];
                    if !preimage["target_unit_node_id"].is_null() {
                        dependencies.push(preimage["target_unit_node_id"].clone());
                    }
                    (
                        "scalar_type",
                        "unit",
                        preimage["dimension_node_id"].clone(),
                        Value::Array(dependencies),
                        json!({"term":"aggregate","members":[]}),
                        Some(json!({"qualified_name": declaration_name})),
                    )
                }
                other => panic!("unknown nominal preimage {other}"),
            };
        let mut node = json!({
            "node_id": id,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag,
            "semantic_form": form,
            "semantic_type": semantic_type,
            "dependencies": dependencies,
            "occurrences": [{"role":"declaration","ordinal":0}],
            "nominal_identity_preimage": preimage,
            "body": body,
        });
        if let Some(declaration) = declaration {
            node.as_object_mut()
                .expect("node object")
                .insert("declaration".to_owned(), declaration);
        }
        nodes.push(node);
        source_map.push(json!({
            "node_id": id,
            "role": "declaration",
            "ordinal": 0,
            "regions": [{"source": source, "start": position, "end": position + 1}],
        }));
    }
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    package["source_map"] = Value::Array(source_map);
    refresh_identity(&mut package);
    package
}

/// Parses a vendored `refused:<code>` outcome.
pub fn refusal_code(outcome: &str) -> CheckedPackageRefusalCode {
    match outcome.strip_prefix("refused:").expect("refused outcome") {
        "unknown_contract_version" => CheckedPackageRefusalCode::UnknownContractVersion,
        "malformed_wire" => CheckedPackageRefusalCode::MalformedWire,
        "duplicate_member" => CheckedPackageRefusalCode::DuplicateMember,
        "unknown_member" => CheckedPackageRefusalCode::UnknownMember,
        "noncanonical_wire" => CheckedPackageRefusalCode::NoncanonicalWire,
        "stale_dependency" => CheckedPackageRefusalCode::StaleDependency,
        "digest_domain_mismatch" => CheckedPackageRefusalCode::DigestDomainMismatch,
        "unknown_required_capability" => CheckedPackageRefusalCode::UnknownRequiredCapability,
        "invalid_semantic_graph" => CheckedPackageRefusalCode::InvalidSemanticGraph,
        "invalid_source_map" => CheckedPackageRefusalCode::InvalidSourceMap,
        "unsupported_node_tag" => CheckedPackageRefusalCode::UnsupportedNodeTag,
        other => panic!("unknown vendored refusal code {other}"),
    }
}

pub fn refusal(code: CheckedPackageRefusalCode, path: &str) -> CheckedPackageRefusal {
    CheckedPackageRefusal {
        code,
        path: path.into(),
        cause: None,
        locus: None,
    }
}

/// A refusal located at a specific graph node (FR-340 frame refusals):
/// carries the cause tag (absent for a canonical-order defect) and the node
/// key of the offending entry or node.
pub fn refusal_at(
    code: CheckedPackageRefusalCode,
    path: &str,
    cause: Option<CheckedPackageRefusalCause>,
    locus_digest: &str,
) -> CheckedPackageRefusal {
    CheckedPackageRefusal {
        code,
        path: path.into(),
        cause,
        locus: Some(typed_node_id(locus_digest)),
    }
}

/// Parses a vendored `frame_mutations` vector's `expected_code`.
pub fn frame_refusal_code(expected: &str) -> CheckedPackageRefusalCode {
    match expected {
        "invalid_semantic_graph" => CheckedPackageRefusalCode::InvalidSemanticGraph,
        "missing_declaration" => CheckedPackageRefusalCode::MissingDeclaration,
        "invalid_model_binding" => CheckedPackageRefusalCode::InvalidModelBinding,
        other => panic!("unknown vendored frame refusal code {other}"),
    }
}

/// Parses a vendored `frame_mutations` vector's `expected_cause`.
pub fn frame_refusal_cause(expected: Option<&str>) -> Option<CheckedPackageRefusalCause> {
    match expected {
        None => None,
        Some("missing-name") => Some(CheckedPackageRefusalCause::MissingName),
        Some("malformed-declaration") => Some(CheckedPackageRefusalCause::MalformedDeclaration),
        Some(other) => panic!("unknown vendored frame refusal cause {other}"),
    }
}

pub fn incomplete(
    kind: CheckedPackageLimit,
    limit: u64,
    consumed: u64,
) -> CheckedPackageIncomplete {
    CheckedPackageIncomplete {
        limit_kind: kind,
        limit,
        consumed,
    }
}
