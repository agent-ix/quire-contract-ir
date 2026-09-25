// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Shared helpers for the I04 CheckedPackage integration tests.
//!
//! The package documents these helpers used to read were copied from a
//! private repository and were removed when this repository's contents were
//! contained; see AGE-1961.
//!
//! [`v2_all_families`], [`v2_nominal`] and [`positive_operation_identities`]
//! are regenerated below: every document they return is *built* by
//! [`build_v2_all_families`]/[`nominal_package`]/[`build_operation_identities`]
//! from this crate's own public vocabulary (`CheckedNodeTag`, the
//! `quire.application-node/v1` key-derivation formula the reader implements in
//! `v2/operations.rs`, and the nominal identity preimages `v2/identity.rs`
//! implements) — SHA-256/RFC-8785 canonicalization done with the same
//! `sha256_hex`/`canonical` helpers every other test in this module already
//! used, never a copy of bytes from anywhere else. The node-identity-vectors
//! conformance oracle is the one piece that stays unavailable: it was an
//! *independent* oracle (what an outside producer computed), so regenerating
//! it from this crate would make every assertion against it tautological —
//! see AGE-1961. Every test that read it was removed rather than rewritten.

#![allow(dead_code)] // Each test binary uses a different subset of these helpers.

use quire_contract_ir::{
    CheckedArtifactLocator, CheckedDomainPackageLocator, CheckedPackageEvidence,
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, JsonPointer,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub const COMPLETE_VALUE_FEATURE: &str = "quire.value.complete/v1";
pub const NODE_DOMAIN: &str = "quire.checked-semantic-node/v1";

/// The exact minimum `work` `CheckedPackageV2::read` charges to admit
/// `v2_all_families()`. Measured by binary search directly against the real
/// reader — the same technique
/// `tc_048_shipped_default_read_limits_are_exact_and_finite` uses — rather
/// than hand-derived from the validation stages' documented charges: a
/// hand-derived figure is exactly the kind of number that silently drifts
/// from what the reader actually charges the next time this generator's
/// fixture changes (AGE-1961 exists because the
/// previous fixture tree could not be regenerated when that happened).
/// Shared between `complete_v1_checked_package` and `checked_package_v2_reader`
/// so a change to `build_v2_all_families` cannot silently move the boundary
/// in only one of them.
pub fn all_families_read_work() -> u64 {
    let value = v2_all_families();
    let bytes = canonical(&value);
    let evidence = evidence_for(&value);
    let bounded = CheckedPackageReadLimits::bounded();
    let (mut lo, mut hi) = (0_u64, bounded.work);
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        let mut limits = bounded;
        limits.work = mid;
        match CheckedPackageV2::read(&bytes, limits, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => hi = mid,
            _ => lo = mid,
        }
    }
    hi
}

/// The real value [`all_families_read_work`] measures today, pinned by hand
/// so this file matches its own convention of hand-pinning other worked-out
/// charges (e.g. `tc_048_v2_reader_reports_exact_and_one_over_limits`'s
/// `work: 21`). A binary search against the reader under test can only ever
/// agree with that same reader — it is not, on its own, a gate that a change
/// to the reader's charging logic can fail. Both callers assert
/// `all_families_read_work() == ALL_FAMILIES_READ_WORK`, so a future change
/// to `v2/lower.rs`'s charge model that moves the real boundary is caught
/// here instead of silently absorbed by a measurement that moves with it.
pub const ALL_FAMILIES_READ_WORK: u64 = 78;

pub fn v2_all_families() -> Value {
    build_v2_all_families()
}

pub fn v2_nominal() -> Value {
    nominal_package(&nominal_fixture_members())
}

/// A `quire.checked-package/v2` document exercising `declaration`,
/// `literal.type` and `application.operation`/`application.result_type` —
/// the shape `tc_048_deleting_a_declared_wire_member_refuses_before_the_projection_compare`
/// needs a node carrying each of, so it can delete one member at a time and
/// assert the closed-schema refusal that member's absence produces.
pub fn positive_operation_identities() -> Value {
    build_operation_identities()
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
    let mut package = nominal_skeleton();
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

/// Parses an expected RFC 6901 pointer.
pub fn pointer(text: &str) -> JsonPointer {
    JsonPointer::parse(text).unwrap_or_else(|| panic!("not an RFC 6901 pointer: {text:?}"))
}

/// A refusal about the value at the RFC 6901 pointer `path`.
pub fn refusal(code: CheckedPackageRefusalCode, path: &str) -> CheckedPackageRefusal {
    CheckedPackageRefusal {
        code,
        path: Some(pointer(path)),
        cause: None,
        locus: None,
        contract_version: None,
    }
}

/// A refusal about the byte stream rather than a value: it carries no path.
pub fn refusal_bytes(code: CheckedPackageRefusalCode) -> CheckedPackageRefusal {
    CheckedPackageRefusal {
        code,
        path: None,
        cause: None,
        locus: None,
        contract_version: None,
    }
}

/// `unknown_contract_version` at `/contract_version`, carrying the version
/// string the reader read there.
pub fn unknown_version(version: &str) -> CheckedPackageRefusal {
    CheckedPackageRefusal {
        code: CheckedPackageRefusalCode::UnknownContractVersion,
        path: Some(pointer("/contract_version")),
        cause: None,
        locus: None,
        contract_version: Some(version.into()),
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
        path: Some(pointer(path)),
        cause,
        locus: Some(typed_node_id(locus_digest)),
        contract_version: None,
    }
}

/// The incomplete outcome for `kind`, charged at the value `path` names
/// (`None` only for the byte limit).
pub fn incomplete(
    kind: CheckedPackageLimit,
    limit: u64,
    consumed: u64,
    path: Option<&str>,
) -> CheckedPackageIncomplete {
    CheckedPackageIncomplete {
        limit_kind: kind,
        limit,
        consumed,
        path: path.map(pointer),
    }
}

// ---------------------------------------------------------------------------
// Fixture generator (AGE-1961).
//
// Every document below is *built*, here, from this crate's own public
// vocabulary: `CheckedNodeTag`'s closed families and forms, the
// `quire.application-node/v1` key-derivation formula
// `v2::operations::validate_application_keys` implements (re-derived in
// `application_key` below with the same `sha256_hex`/`canonical` this module
// already used for nominal preimages), and the catalogued operation
// identities `quire-verification-contracts` publishes and this crate's own
// `v2::operations` module validates against (`quire.op.function.call`,
// `quire.op.temporal.clause`, `quire.op.protocol.control`,
// `quire.op.claim.clause` — all four are catalogued with a fixed,
// zero-or-one-law shape this generator matches exactly). No fixture node's
// digest, body or lock entry is read from a file or copied from anywhere
// else; every one is computed by the functions below.

const FIXTURE_SOURCE_AUTHORITY: &str = "agent-ix";
const FIXTURE_SOURCE_IDENTITY: &str = "quire.fixture.source/v1";
const FIXTURE_DEFINITION_DOMAIN: &str = "quire.definition.bytes/v1";
const FIXTURE_SOURCE_DOMAIN: &str = "quire.source.bytes/v1";
const APPLICATION_NODE_VERSION: &str = "quire.application-node/v1";

/// A `CheckedArtifactRef`-shaped locked artifact. `digest` need not be a real
/// content hash: [`evidence_for`] attests whatever digest an artifact records
/// here directly (it never re-hashes real bytes), so any syntactically valid
/// 64-hex-digit string proves the lock current against the evidence this
/// generator's own callers build.
fn artifact(authority: &str, identity: &str, domain: &str, digest: &str) -> Value {
    json!({
        "authority": authority,
        "identity": identity,
        "revision": {"namespace": "quire-draft", "value": "1"},
        "digest_domain": domain,
        "digest": digest,
    })
}

/// The `quire.application-node/v1` key this crate's reader
/// (`v2::operations::validate_application_keys`) re-derives for every node
/// whose body is an `application` term: SHA-256 of the RFC-8785 canonical
/// bytes of `{version, node_tag, semantic_form, semantic_type, declaration,
/// recursion, body}`. Every application-bodied fixture node this generator
/// builds carries no `declaration` and no `recursion_group`, so both are
/// `null` here unconditionally.
fn application_key(
    node_tag: &str,
    semantic_form: &str,
    semantic_type: &Value,
    body: &Value,
) -> String {
    application_preimage_key(
        node_tag,
        semantic_form,
        semantic_type,
        &Value::Null,
        &Value::Null,
        body,
    )
}

/// SHA-256 of the RFC-8785 bytes of FR-322's `quire.application-node/v1`
/// preimage, the one builder both fixture keys and grouped keys use.
fn application_preimage_key(
    node_tag: &str,
    semantic_form: &str,
    semantic_type: &Value,
    declaration: &Value,
    recursion: &Value,
    body: &Value,
) -> String {
    let preimage = json!({
        "version": APPLICATION_NODE_VERSION,
        "node_tag": node_tag,
        "semantic_form": semantic_form,
        "semantic_type": semantic_type,
        "declaration": declaration,
        "recursion": recursion,
        "body": body,
    });
    sha256_hex(&canonical(&preimage))
}
/// QSpec FR-322's `quire.application-node/v1` key for a node inside a
/// recursion group: `recursion` is `{size, ordinal}` of `node` among
/// `group` (member node ids in graph order), and each body `reference` to a
/// group member is keyed as `{term: "group_reference", ordinal}`. Written
/// here from FR-322's text, independently of the reader.
pub fn application_key_in_group(node: &Value, group: &[Value]) -> String {
    fn rewrite(term: &Value, group: &[Value]) -> Value {
        let mut term = term.clone();
        if term["term"] == "reference" {
            if let Some(ordinal) = group.iter().position(|member| *member == term["target"]) {
                return json!({"term": "group_reference", "ordinal": ordinal});
            }
        }
        for key in ["arguments", "members"] {
            if let Some(items) = term.get(key).and_then(Value::as_array).cloned() {
                term[key] = Value::Array(items.iter().map(|item| rewrite(item, group)).collect());
            }
        }
        if term["term"] == "binding" {
            let value = rewrite(&term["value"], group);
            term["value"] = value;
        }
        term
    }
    let ordinal = group
        .iter()
        .position(|member| *member == node["node_id"])
        .expect("node is a group member");
    application_preimage_key(
        node["node_tag"].as_str().expect("tag"),
        node["semantic_form"].as_str().expect("form"),
        &node["semantic_type"],
        &node.get("declaration").cloned().unwrap_or(Value::Null),
        &json!({"size": group.len(), "ordinal": ordinal}),
        &rewrite(&node["body"], group),
    )
}

/// The exact 64-hex-digit node key this generator assigns to a family or
/// supporting node's `prefix` in [`build_v2_all_families`]: `prefix.repeat(16)`
/// for every plain node, except the four whose body is a real `application`
/// term (`ffff`/`4040`/`5050`/`6060` — the function/temporal/protocol/claim
/// families), whose node key must be the actual [`application_key`] of their
/// own preimage or the reader refuses them as `stale-node-key`.
/// `tests/checked_package_v2_lowering.rs`'s own `key()` calls this directly,
/// so the fixture and the test that indexes it by prefix can never drift.
pub fn family_key(prefix: &str) -> String {
    match prefix {
        "ffff" => application_key(
            "function",
            "pure_function",
            &node_id(&family_key("aaaa")),
            &function_call_body(),
        ),
        "4040" => application_key(
            "temporal",
            "temporal_clause",
            &node_id(&family_key("aaaa")),
            &temporal_clause_body(),
        ),
        "5050" => application_key(
            "protocol",
            "protocol_clause",
            &node_id(&family_key("aaaa")),
            &protocol_control_body(),
        ),
        "6060" => application_key(
            "claim",
            "verification_claim",
            &node_id(&family_key("aaaa")),
            &claim_clause_body(),
        ),
        _ => prefix.repeat(16),
    }
}

fn function_call_body() -> Value {
    json!({
        "term": "application",
        "operator": "call",
        "operation": {
            "identity": "quire.op.function.call",
            "laws": Value::Array(Vec::new()),
            "mode": Value::Null,
            "member": Value::Null,
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [{"term": "reference", "target": node_id(&family_key("8080"))}],
    })
}

/// The lock's own `temporal_profile` selection, reused verbatim as the
/// `laws[]` entry every `temporal.clause`/`claim.clause` application node
/// declares (both are catalogued with that one required role) — the same
/// `CheckedArtifactRef` value in both places, so the operation-law lock join
/// (`is_profile_role` → `lock.profile_selections`) is satisfied by
/// construction rather than by two independently-typed copies.
fn temporal_profile_law() -> Value {
    json!({
        "role": "temporal_profile",
        "definition": artifact(
            FIXTURE_SOURCE_AUTHORITY,
            "quire.fixture.temporal-profile/v1",
            FIXTURE_DEFINITION_DOMAIN,
            &"2".repeat(64),
        ),
    })
}

/// The lock's own `protocol_profile` selection; see [`temporal_profile_law`].
fn protocol_profile_law() -> Value {
    json!({
        "role": "protocol_profile",
        "definition": artifact(
            FIXTURE_SOURCE_AUTHORITY,
            "quire.fixture.protocol-profile/v1",
            FIXTURE_DEFINITION_DOMAIN,
            &"3".repeat(64),
        ),
    })
}

fn temporal_clause_body() -> Value {
    json!({
        "term": "application",
        "operator": "temporal",
        "operation": {
            "identity": "quire.op.temporal.clause",
            "laws": [temporal_profile_law()],
            "mode": Value::Null,
            "member": {"kind": "profile_operator"},
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": Value::Array(Vec::new()),
    })
}

fn protocol_control_body() -> Value {
    json!({
        "term": "application",
        "operator": "protocol_control",
        "operation": {
            "identity": "quire.op.protocol.control",
            "laws": [protocol_profile_law()],
            "mode": Value::Null,
            "member": {"kind": "profile_operator"},
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": Value::Array(Vec::new()),
    })
}

fn claim_clause_body() -> Value {
    json!({
        "term": "application",
        "operator": "claim",
        "operation": {
            "identity": "quire.op.claim.clause",
            "laws": [temporal_profile_law()],
            "mode": Value::Null,
            "member": {"kind": "profile_operator"},
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": Value::Array(Vec::new()),
    })
}

/// A plain graph node: `dependencies` is the node's own wire `dependencies`
/// array (identity-based, distinct from the lowering closure), and
/// `occurrences` is a single `generated`-role occurrence. No fixture node
/// this generator builds carries a `declaration`-role occurrence or a
/// nominal preimage, so `declaration` and `nominal_identity_preimage` (both
/// optional wire members) are simply omitted.
fn plain_node(
    digest: &str,
    node_tag: &str,
    semantic_form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": node_tag,
        "semantic_form": semantic_form,
        "semantic_type": node_id(semantic_type),
        "dependencies": dependencies.iter().map(|d| node_id(d)).collect::<Vec<_>>(),
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": body,
    })
}

/// An empty `aggregate` term: the simplest closed `SemanticTerm` body,
/// charging exactly one unit of `validate_body` work and naming no reference.
fn empty_aggregate() -> Value {
    json!({"term": "aggregate", "members": Value::Array(Vec::new())})
}

/// An application-bodied node, keyed by [`application_key`] rather than a
/// placeholder digest. `result_type` doubles as the node's own `semantic_type`:
/// an application node's semantic type is the type of the value it produces,
/// the same type its own body already names as `result_type`. Every caller in
/// `build_v2_all_families` happens to pass `"aaaa"` for both, but
/// `build_operation_identities` does not — its call node's real type is its
/// `result_type` argument (`&root`), not the `"aaaa"` family node that
/// document never builds; hardcoding `"aaaa"` here previously left that
/// document's call node with a dangling `semantic_type` that refused
/// admission with `InvalidSemanticGraph` at `semantic_graph.nodes.semantic_type`.
fn application_node(
    node_tag: &str,
    semantic_form: &str,
    operator: &str,
    operation: Value,
    result_type: &str,
    arguments: Vec<Value>,
    dependencies: &[&str],
) -> Value {
    let semantic_type = node_id(result_type);
    let body = json!({
        "term": "application",
        "operator": operator,
        "operation": operation,
        "result_type": node_id(result_type),
        "arguments": arguments,
    });
    let digest = application_key(node_tag, semantic_form, &semantic_type, &body);
    json!({
        "node_id": node_id(&digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": node_tag,
        "semantic_form": semantic_form,
        "semantic_type": semantic_type,
        "dependencies": dependencies.iter().map(|d| node_id(d)).collect::<Vec<_>>(),
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": body,
    })
}

fn fixture_source() -> Value {
    artifact(
        FIXTURE_SOURCE_AUTHORITY,
        FIXTURE_SOURCE_IDENTITY,
        FIXTURE_SOURCE_DOMAIN,
        &"1".repeat(64),
    )
}

/// A second locked source, distinct from [`fixture_source`]. Nominal
/// packages carry two locked sources so
/// `tc_048_package_id_covers_exactly_the_identity_preimage` can point a
/// source-map entry at the *other* locked source and show that which source
/// a node's occurrences cite is outside the identity preimage; a single
/// locked source could never distinguish that from citing the same source
/// again.
fn fixture_secondary_source() -> Value {
    artifact(
        FIXTURE_SOURCE_AUTHORITY,
        "quire.fixture.source.secondary/v1",
        FIXTURE_SOURCE_DOMAIN,
        &"9".repeat(64),
    )
}

/// A locked definition artifact, unreferenced by any node body. Nominal
/// packages carry one so `tc_048_package_id_covers_exactly_the_identity_preimage`
/// can select it into `profile_selections` under a fresh role and show that a
/// preimage member's *value* (not merely its presence) is what the identity
/// covers, reusing an artifact evidence already attests rather than a
/// digest invented for the mutation. It is also load-bearing for a second,
/// unrelated role: `nominal_fixture_members`'s dimension preimage owns it as
/// a `NominalOwner::Definition` (rather than the source-owned declaration and
/// unit), so `tc_048_nominal_cross_field_contradictions_refuse`'s "owner
/// outside lock" case — which mutates `lock.definition_selections[0].identity`
/// — has a real join to break; without this second role that mutation would
/// have nothing in the fixture to affect.
fn fixture_definition_selection() -> Value {
    artifact(
        FIXTURE_SOURCE_AUTHORITY,
        "quire.fixture.definition.selection/v1",
        FIXTURE_DEFINITION_DOMAIN,
        &"a".repeat(64),
    )
}

fn fixture_edition_definition() -> Value {
    artifact(
        FIXTURE_SOURCE_AUTHORITY,
        "quire.fixture.edition/v1",
        FIXTURE_DEFINITION_DOMAIN,
        &"6".repeat(64),
    )
}

fn fixture_diagnostics_catalog() -> Value {
    artifact(
        FIXTURE_SOURCE_AUTHORITY,
        "quire.fixture.diagnostics-catalog/v1",
        FIXTURE_DEFINITION_DOMAIN,
        &"7".repeat(64),
    )
}

fn fixture_capability_report() -> Value {
    json!([{"feature": COMPLETE_VALUE_FEATURE, "disposition": "available"}])
}

fn fixture_lock(profile_selections: Vec<Value>) -> Value {
    json!({
        "sources": [fixture_source()],
        "edition": {"role": "edition", "definition": fixture_edition_definition()},
        "profile_selections": profile_selections,
        "definition_selections": Value::Array(Vec::new()),
        "model_selections": Value::Array(Vec::new()),
        "required_features": [COMPLETE_VALUE_FEATURE],
        "dependency_selections": Value::Array(Vec::new()),
    })
}

/// Rebuilds a fixture package's source map from its nodes' first
/// occurrences, after a test changed a node's occurrences.
pub fn rebuild_source_map(package: &mut Value) {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    package["source_map"] = source_map_for(&nodes, &fixture_source());
}

/// Re-derives the application key of the node at `position` from its own
/// members, after a test changed its body, and renames it everywhere it is
/// referenced.
pub fn rekey_application_node(package: &mut Value, position: usize) {
    let node = package["semantic_graph"]["nodes"][position].clone();
    let fresh = application_key(
        node["node_tag"].as_str().expect("tag"),
        node["semantic_form"].as_str().expect("form"),
        &node["semantic_type"],
        &node["body"],
    );
    let stale = node["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    replace_digest(package, &stale, &fresh);
}

/// One entry per node, sourced from the node's own single occurrence rather
/// than a hardcoded `generated` role: every node this generator builds
/// carries exactly one entry in its own `occurrences` array, but that entry's
/// role is `declaration` for a declaring node (see `build_operation_identities`'s
/// `declaring_node`) and `generated` for every other one. Reading the role
/// and ordinal back from the node itself, rather than re-asserting
/// `generated` here, is what keeps this function correct for both — a
/// hardcoded `generated` silently produced a `source_map` whose entry
/// disagreed with the declaring node's own recorded occurrence, which
/// `validate_source_map_entries` refuses as `invalid_source_map`.
fn source_map_for(nodes: &[Value], source: &Value) -> Value {
    let entries = nodes
        .iter()
        .enumerate()
        .map(|(position, node)| {
            let occurrence = &node["occurrences"][0];
            json!({
                "node_id": node["node_id"],
                "role": occurrence["role"],
                "ordinal": occurrence["ordinal"],
                "regions": [{"source": source, "start": position, "end": position + 1}],
            })
        })
        .collect::<Vec<_>>();
    Value::Array(entries)
}

/// A structurally valid but not-yet-derived `identity_preimage`/`package_id`
/// pair: every member [`refresh_identity`] does not itself overwrite
/// (`version`, `domain`, `algorithm`) is set to its real closed-schema value
/// here; every member it does overwrite (`identity_projection` and the six
/// lock-mirrored fields, plus `package_id.digest`) is a placeholder,
/// replaced by the `refresh_identity(&mut package)` call every builder below
/// makes as its last step.
fn placeholder_identity_preimage() -> Value {
    json!({
        "version": "quire.checked-package-id/v2",
        "edition": Value::Null,
        "profile_selections": Value::Array(Vec::new()),
        "definition_selections": Value::Array(Vec::new()),
        "model_selections": Value::Array(Vec::new()),
        "required_features": Value::Array(Vec::new()),
        "dependency_selections": Value::Array(Vec::new()),
        "identity_projection": Value::Array(Vec::new()),
    })
}

fn placeholder_package_id() -> Value {
    json!({
        "domain": "quire.package.semantic/v2",
        "algorithm": "sha256",
        "digest": "0".repeat(64),
    })
}

/// The base lock/diagnostics/capability skeleton [`nominal_package`] overlays
/// with a caller-supplied node set. Kept separate from [`v2_nominal`] (which
/// *is* one particular overlay of this skeleton — see [`nominal_fixture_members`])
/// so the two do not recurse into each other.
fn nominal_skeleton() -> Value {
    let mut lock = fixture_lock(Vec::new());
    lock["sources"] = json!([fixture_source(), fixture_secondary_source()]);
    lock["definition_selections"] = json!([fixture_definition_selection()]);
    json!({
        "contract_version": "quire.checked-package/v2",
        "identity_preimage": placeholder_identity_preimage(),
        "package_id": placeholder_package_id(),
        "lock": lock,
        "semantic_graph": {
            "graph_version": "quire.checked-semantic-graph/v2",
            "nodes": Value::Array(Vec::new()),
        },
        "source_map": Value::Array(Vec::new()),
        "capability_report": fixture_capability_report(),
        "diagnostics": {
            "catalog": fixture_diagnostics_catalog(),
            "entries": Value::Array(Vec::new()),
        },
    })
}

/// The four nominal preimages [`v2_nominal`] carries, in `(preimage, key)`
/// pairs, in the exact array order every test in this crate that indexes
/// `v2_nominal()`'s nodes by position assumes: `[0] = enum member,
/// [1] = enum declaration, [2] = unit, [3] = dimension`. Digests are computed
/// in dependency order (declaration before its member, dimension before its
/// unit) — required, since each dependant preimage embeds the digest of the
/// node it names — and only then reordered into that display order.
pub fn nominal_fixture_members() -> Vec<(Value, String)> {
    let owner = json!({
        "kind": "source",
        "authority": FIXTURE_SOURCE_AUTHORITY,
        "identity": FIXTURE_SOURCE_IDENTITY,
    });
    // The dimension is owned by the locked definition selection, not the
    // locked source, so `tc_048_nominal_cross_field_contradictions_refuse`'s
    // "owner outside lock" case — which mutates
    // `lock.definition_selections[0].identity` — has a real
    // `NominalOwner::Definition` join to break; the declaration and unit stay
    // source-owned, matching `validate_owner`'s other join arm.
    let definition = fixture_definition_selection();
    let definition_owner = json!({
        "kind": "definition",
        "authority": definition["authority"],
        "identity": definition["identity"],
    });

    let declaration_preimage = json!({
        "version": "quire.enum-declaration-node/v1",
        "owner": owner,
        "qualified_declaration": ["Example", "Status"],
        "ordered": false,
        "members": ["ACTIVE", "DONE"],
    });
    let declaration_key = sha256_hex(&canonical(&declaration_preimage));

    let member_preimage = json!({
        "version": "quire.enum-member-node/v1",
        "declaration_node_id": node_id(&declaration_key),
        "case": "ACTIVE",
    });
    let member_key = sha256_hex(&canonical(&member_preimage));

    let dimension_preimage = json!({
        "version": "quire.dimension-node/v1",
        "owner": definition_owner,
        "qualified_declaration": ["Example", "Length"],
        "terms": Value::Array(Vec::new()),
    });
    let dimension_key = sha256_hex(&canonical(&dimension_preimage));

    let unit_preimage = json!({
        "version": "quire.unit-node/v1",
        "owner": owner,
        "qualified_declaration": ["Example", "Metre"],
        "dimension_node_id": node_id(&dimension_key),
        "target_unit_node_id": Value::Null,
        "scale": {"numerator": "1", "denominator": "1"},
        "offset": {"numerator": "0", "denominator": "1"},
    });
    let unit_key = sha256_hex(&canonical(&unit_preimage));

    vec![
        (member_preimage, member_key),
        (declaration_preimage, declaration_key),
        (unit_preimage, unit_key),
        (dimension_preimage, dimension_key),
    ]
}

/// Builds `v2_all_families()`: one node per V2 semantic-node family (in
/// `CheckedNodeTag::ALL` order, positions 0-12, keyed `aaaa`.."7070"), plus
/// five supporting nodes the family nodes' own required members reference —
/// a second `pure_function` the `function` family's real `application`
/// argument names, a `field_declaration`/`object_type`/`process` triple the
/// `state` family's frame body's `modifies`/`creates`/`deletes` name, and the
/// dedicated `state`/`frame` node itself (the `state` family's own
/// representative node is a plain `state_clause`, so its lowering closure
/// stays the uniform single-`aaaa` shape every other plain family node has;
/// `checked_package_v2_frame_bodies.rs` locates the one real frame node by
/// its `(node_tag, semantic_form)` shape, not by position).
///
/// Every node is typed by node 0 (`aaaa`, a self-typed `scalar_type`/
/// `boolean`) for simplicity; the two nodes whose lowering closure must reach
/// a second node (`eeee`/`7070`, the `expression` and `correspondence`
/// families) also depend on node 3 (`dddd`). The function/temporal/protocol/
/// claim family nodes (`ffff`/`4040`/`5050`/`6060`) are the only nodes with a
/// real `application` body — the four catalogued operations
/// (`quire.op.function.call`/`.temporal.clause`/`.protocol.control`/
/// `.claim.clause`) this generator's own `plain_node` counterparts would
/// otherwise never exercise.
fn build_v2_all_families() -> Value {
    let aaaa = family_key("aaaa");
    let bbbb = family_key("bbbb");
    let cccc = family_key("cccc");
    let dddd = family_key("dddd");
    let eeee = family_key("eeee");
    let f1010 = family_key("1010");
    let f2020 = family_key("2020");
    let f3030 = family_key("3030");
    let f7070 = family_key("7070");
    let f8080 = family_key("8080");
    let f1414 = family_key("1414");
    let f1515 = family_key("1515");
    let f1616 = family_key("1616");
    let frame_key = "f4a0".repeat(16);

    let nodes = vec![
        plain_node(
            &aaaa,
            "scalar_type",
            "boolean",
            &aaaa,
            &[],
            json!({"term": "literal", "type": node_id(&aaaa), "value_kind": "boolean", "value": true}),
        ),
        // No wire `dependencies` of its own: its lowering closure already
        // reaches `aaaa` through `semantic_type` alone, and leaving this one
        // family node's `dependencies` empty in the base document is what lets
        // `complete_v1_checked_package.rs`'s own edges accounting test prove
        // the exact/one-over boundary by adding exactly one dependency to it.
        plain_node(
            &bbbb,
            "composite_type",
            "option",
            &aaaa,
            &[],
            empty_aggregate(),
        ),
        plain_node(
            &cccc,
            "bounded_domain",
            "integer_range",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        plain_node(
            &dddd,
            "value",
            "literal",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        plain_node(
            &eeee,
            "expression",
            "reference",
            &aaaa,
            &[aaaa.as_str(), dddd.as_str()],
            json!({"term": "reference", "target": node_id(&dddd)}),
        ),
        application_node(
            "function",
            "pure_function",
            "call",
            function_call_body()["operation"].clone(),
            &aaaa,
            vec![json!({"term": "reference", "target": node_id(&f8080)})],
            // FR-322's application-node join: exactly the body's reference
            // targets.
            &[f8080.as_str()],
        ),
        plain_node(
            &f1010,
            "model",
            "namespace",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        plain_node(
            &f2020,
            "relation",
            "relationship",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        plain_node(
            &f3030,
            "state",
            "state_clause",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        application_node(
            "temporal",
            "temporal_clause",
            "temporal",
            temporal_clause_body()["operation"].clone(),
            &aaaa,
            Vec::new(),
            &[],
        ),
        application_node(
            "protocol",
            "protocol_clause",
            "protocol_control",
            protocol_control_body()["operation"].clone(),
            &aaaa,
            Vec::new(),
            &[],
        ),
        application_node(
            "claim",
            "verification_claim",
            "claim",
            claim_clause_body()["operation"].clone(),
            &aaaa,
            Vec::new(),
            &[],
        ),
        plain_node(
            &f7070,
            "correspondence",
            "source_locus",
            &aaaa,
            &[aaaa.as_str(), dddd.as_str()],
            empty_aggregate(),
        ),
        plain_node(
            &f8080,
            "function",
            "pure_function",
            &aaaa,
            &[],
            empty_aggregate(),
        ),
        plain_node(
            &f1414,
            "model",
            "field_declaration",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        plain_node(
            &f1515,
            "model",
            "object_type",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        plain_node(
            &f1616,
            "model",
            "process",
            &aaaa,
            &[aaaa.as_str()],
            empty_aggregate(),
        ),
        plain_node(
            &frame_key,
            "state",
            "frame",
            &aaaa,
            &[
                f1414.as_str(),
                f1515.as_str(),
                f1616.as_str(),
                f2020.as_str(),
            ],
            json!({
                "term": "frame",
                "modifies": [node_id(&f1414), node_id(&f2020)],
                "creates": [node_id(&f1515)],
                "deletes": [node_id(&f1616)],
            }),
        ),
    ];

    let source = fixture_source();
    let profile_selections = vec![temporal_profile_law(), protocol_profile_law()];
    let mut package = json!({
        "contract_version": "quire.checked-package/v2",
        "identity_preimage": placeholder_identity_preimage(),
        "package_id": placeholder_package_id(),
        "lock": fixture_lock(profile_selections),
        "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": nodes},
        "source_map": Value::Array(Vec::new()),
        "capability_report": fixture_capability_report(),
        "diagnostics": {"catalog": fixture_diagnostics_catalog(), "entries": Value::Array(Vec::new())},
    });
    let nodes_array = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    package["source_map"] = source_map_for(&nodes_array, &source);
    refresh_identity(&mut package);
    package
}

/// Builds `positive_operation_identities()`: a small five-node document
/// carrying one node of each shape
/// `tc_048_deleting_a_declared_wire_member_refuses_before_the_projection_compare`
/// needs to delete a member from — a `declaration`, a `literal.type`, an
/// `application.operation` and an `application.result_type` — so it can
/// assert the closed-schema refusal each deletion produces.
fn build_operation_identities() -> Value {
    let root = "beef".repeat(16);
    let declaring = "cafe".repeat(16);
    let literal_key = "d00d".repeat(16);
    let func = "f00d".repeat(16);

    let root_node = plain_node(
        &root,
        "scalar_type",
        "boolean",
        &root,
        &[],
        empty_aggregate(),
    );

    let mut declaring_node = plain_node(
        &declaring,
        "model",
        "object_type",
        &root,
        &[root.as_str()],
        empty_aggregate(),
    );
    declaring_node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    declaring_node["declaration"] = json!({"qualified_name": ["Example", "Widget"]});

    let literal_node = plain_node(
        &literal_key,
        "value",
        "literal",
        &root,
        &[root.as_str()],
        json!({"term": "literal", "type": node_id(&root), "value_kind": "boolean", "value": true}),
    );

    let func_node = plain_node(
        &func,
        "function",
        "pure_function",
        &root,
        &[],
        empty_aggregate(),
    );

    let call_node = application_node(
        "expression",
        "call",
        "call",
        json!({
            "identity": "quire.op.function.call", "laws": Value::Array(Vec::new()),
            "mode": Value::Null, "member": Value::Null, "leaves": Value::Array(Vec::new()),
        }),
        &root,
        vec![json!({"term": "reference", "target": node_id(&func)})],
        &[func.as_str()],
    );

    let nodes = vec![
        root_node,
        declaring_node,
        literal_node,
        func_node,
        call_node,
    ];
    let source = fixture_source();
    let mut package = json!({
        "contract_version": "quire.checked-package/v2",
        "identity_preimage": placeholder_identity_preimage(),
        "package_id": placeholder_package_id(),
        "lock": fixture_lock(Vec::new()),
        "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": nodes},
        "source_map": Value::Array(Vec::new()),
        "capability_report": fixture_capability_report(),
        "diagnostics": {"catalog": fixture_diagnostics_catalog(), "entries": Value::Array(Vec::new())},
    });
    let nodes_array = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    package["source_map"] = source_map_for(&nodes_array, &source);
    refresh_identity(&mut package);
    package
}
