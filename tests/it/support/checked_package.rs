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

use quire_contract_model::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageIncomplete, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult, JsonPointer,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub const COMPLETE_VALUE_FEATURE: &str = "quire.value.complete/v1";
pub const NODE_DOMAIN: &str = "quire.checked-semantic-node/v1";
const FAMILY_MODEL_IDENTITY: &str = "test/families";
const FAMILY_OBJECT: &str = "ix://test/families/Account";
const FAMILY_RELATIONSHIP: &str = "ix://test/families/relationship/Account-balance-Account";

pub fn family_model_document() -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": FAMILY_MODEL_IDENTITY, "version": "1"},
        "constructs": [{
            "kind": {"module": FAMILY_MODEL_IDENTITY, "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }],
        "types": [{
            "identity": FAMILY_OBJECT, "displayName": "Account",
            "kind": {"module": FAMILY_MODEL_IDENTITY, "name": "entity"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "supertypes": [],
            "fields": [], "operations": [],
            "relationships": [{
                "identity": FAMILY_RELATIONSHIP,
                "category": "structural", "composite": false, "direction": "bidirectional",
                "sourceEnd": {"type": FAMILY_OBJECT, "role": "balanceRel",
                    "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}},
                "targetEnd": {"type": FAMILY_OBJECT,
                    "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}},
                "origin": {"source": {"sourceIdentity": FAMILY_OBJECT,
                    "path": "models/Account.md", "startLine": 1, "startColumn": 1}},
            }],
        }],
    })
}

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

pub fn v2_all_families() -> Value {
    build_v2_all_families()
}

pub fn v2_nominal() -> Value {
    nominal_package(&nominal_fixture_members())
}

/// [`v2_all_families`] with a chain of `length` further `expression`/`reference`
/// nodes, each referencing the one before it: the first the family's own
/// reference node, and the last, whose key is returned, the end of an
/// expression `length` nodes deep. Each node costs one node, one edge and one
/// occurrence, so a limit sized for the node count decides nothing else, and the
/// body grammar keeps the document's JSON depth at that of a one-node chain
/// (FR-038-AC-117, merged QSpec FR-322-AC-41).
///
/// Returned as the canonical document's bytes, built as text one node at a time
/// and spliced into the family's document at three markers, with `package_id`
/// derived over the spliced identity preimage: no tree of the whole document is
/// ever held, which would be many times the size of its text.
pub fn v2_reference_chain(length: usize) -> (Vec<u8>, String) {
    const NODES: &str = "\u{1}nodes";
    const PROJECTION: &str = "\u{1}projection";
    const SOURCE_MAP: &str = "\u{1}source_map";
    let boolean = family_key("aaaa");
    let mut previous = family_key("eeee");
    let mut package = v2_all_families();
    let base = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let source = fixture_source();
    let (mut nodes, mut projection, mut source_map) = (Vec::new(), Vec::new(), Vec::new());
    for index in 0..length {
        let body = json!({"term": "reference", "target": node_id(&previous)});
        let key = structural_key("expression", "reference", Some(&boolean), &body);
        let mut node = plain_node(
            &key,
            "expression",
            "reference",
            &boolean,
            &[previous.as_str()],
            body,
        );
        nodes.push(serde_json::to_string(&node).expect("node"));
        node.as_object_mut()
            .expect("node object")
            .remove("occurrences");
        projection.push(serde_json::to_string(&node).expect("projection"));
        let position = base + index;
        source_map.push(
            serde_json::to_string(&json!({
                "node_id": node_id(&key), "role": "generated", "ordinal": 0,
                "regions": [{"source": source, "start": position, "end": position + 1}],
            }))
            .expect("source map entry"),
        );
        previous = key;
    }
    let marker = |package: &mut Value, pointer: &str, text: &str| {
        package
            .pointer_mut(pointer)
            .and_then(Value::as_array_mut)
            .expect("an array")
            .push(json!(text));
    };
    marker(&mut package, "/semantic_graph/nodes", NODES);
    marker(
        &mut package,
        "/identity_preimage/identity_projection",
        PROJECTION,
    );
    marker(&mut package, "/source_map", SOURCE_MAP);
    let splice = |text: String, marker: &str, entries: &[String]| {
        let quoted = serde_json::to_string(marker).expect("marker");
        text.replace(&quoted, &entries.join(","))
    };
    let preimage = String::from_utf8(canonical(&package["identity_preimage"])).expect("utf-8");
    let preimage = splice(preimage, PROJECTION, &projection);
    package["package_id"]["digest"] = json!(sha256_hex(preimage.as_bytes()));
    let text = String::from_utf8(canonical(&package)).expect("utf-8");
    let text = splice(text, NODES, &nodes);
    let text = splice(text, PROJECTION, &projection);
    let text = splice(text, SOURCE_MAP, &source_map);
    (text.into_bytes(), previous)
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

/// The source owner attached to a declared structural fixture node.
pub fn source_owner(source: &Value) -> Value {
    json!({
        "kind": "source", "authority": source["authority"],
        "identity": source["identity"],
    })
}

/// The model owner attached to an undeclared structural fixture node.
pub fn model_owner(identity: &str, node: &str) -> Value {
    json!({"kind": "model", "identity": identity, "node": node})
}

/// Attaches the owner before a fixture projects or derives the node's key.
/// The caller keeps its existing key-derivation oracle at this seam.
pub fn owned_structural_node(mut node: Value, owner: Value) -> Value {
    node["owner"] = owner;
    node
}

pub fn typed_node_id(digest: &str) -> quire_contract_model::CheckedNodeId {
    serde_json::from_value(node_id(digest)).expect("node id")
}

/// A minimal Semantic IR 2.0.0 document for a domain package, declaring the
/// given object types.
pub fn domain_package_document(identity: &str, version: &str, types: Vec<Value>) -> Value {
    serde_json::json!({
        "contractVersion": "2.0.0",
        "package": {"identity": identity, "version": version},
        "constructs": [],
        "types": types,
    })
}

/// The `sha256-jcs` digest a lock names [`domain_package_document`] under.
pub fn domain_package_digest(document: &Value) -> String {
    sha256_hex(&canonical(document))
}

/// Evidence for a package: its domain package documents plus the complete-value feature.
pub fn evidence_for(package: &Value) -> CheckedPackageEvidence {
    let mut evidence = CheckedPackageEvidence::new();
    // A lock's compiled-model selections are `sha256-jcs` domain packages,
    // supplied as documents. A selection whose digest names no document this
    // helper builds is left unsupplied, so the reader refuses it.
    for model in package["lock"]["model_selections"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        // The row names no version; the document it is supplied as carries
        // `1`, which nothing reads.
        let identity = model["identity"].as_str().expect("model identity");
        let document = if identity == FAMILY_MODEL_IDENTITY {
            family_model_document()
        } else {
            domain_package_document(identity, "1", Vec::new())
        };
        let digest = domain_package_digest(&document);
        if model["digest"].as_str() == Some(digest.as_str()) {
            evidence.insert_domain_package_document(digest, canonical(&document));
        }
    }
    evidence.support_feature(COMPLETE_VALUE_FEATURE);
    evidence
}

/// Reads `package` with only its own evidence.
fn read_admitting(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

/// A dependency package the reader admitted: the digest its `package_id`
/// carries, and the package itself.
pub fn admitted_dependency(package: &Value) -> (String, CheckedPackageV2) {
    match read_admitting(package) {
        CheckedPackageV2ReadResult::Admitted(admitted) => (
            package["package_id"]["digest"]
                .as_str()
                .expect("package id digest")
                .to_owned(),
            *admitted,
        ),
        other => panic!("the dependency package admits, read {other:?}"),
    }
}

/// `read`, with `dependencies` supplied as `(identity, package)`.
pub fn read_with_dependencies(
    package: &Value,
    dependencies: &[(&str, &CheckedPackageV2)],
) -> CheckedPackageV2ReadResult {
    let mut evidence: CheckedPackageEvidence = evidence_for(package);
    for (identity, dependency) in dependencies {
        evidence.insert_dependency_package(*identity, (*dependency).clone());
    }
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
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

/// Renames the node `stale` to `fresh` everywhere it is named in `package`:
/// its own `node_id`, every `semantic_type`, dependency and body reference.
pub fn rename_node(package: &mut Value, stale: &str, fresh: &str) {
    replace_digest(package, stale, fresh);
}

/// Mint QSL FR-092 keys for a builder's owner-free, ungrouped structural
/// nodes. Each rename reaches every reference in the test package; repeated
/// passes let a node built before a type it names pick up the type's real key.
/// Nominal, application, model-owned, grouped and FR-451 relation keys have
/// their separate constructors or owning stages.
pub fn mint_ungrouped_structural_keys(package: &mut Value) {
    let count = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    for _ in 0..=count {
        let mut changed = false;
        for position in 0..count {
            let node = package["semantic_graph"]["nodes"][position].clone();
            let tag = node["node_tag"].as_str().expect("node tag");
            let form = node["semantic_form"].as_str().expect("semantic form");
            if node.get("nominal_identity_preimage").is_some()
                || node["body"]["term"] == "application"
                || node.get("declaration").is_some()
                || node.get("recursion_group").is_some()
                || matches!(tag, "model" | "relation")
                || (tag == "correspondence" && form == "abstraction_relation")
            {
                continue;
            }
            let stale = node["node_id"]["digest"].as_str().expect("node digest");
            let semantic_type = node["semantic_type"]["digest"]
                .as_str()
                .expect("semantic type digest");
            let fresh = structural_key(
                tag,
                form,
                (semantic_type != stale).then_some(semantic_type),
                &node["body"],
            );
            if fresh != stale {
                rename_node(package, stale, &fresh);
                changed = true;
            }
        }
        if !changed {
            return;
        }
    }
    panic!("ungrouped fixture nodes did not reach stable structural keys");
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

/// Independently authored expected reader fields. Tests build this value from
/// their chosen input and compare each public field with the real refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpectedNodeId {
    Absent,
    Present,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedRefusal {
    pub code: CheckedPackageRefusalCode,
    pub path: Option<JsonPointer>,
    pub cause: Option<CheckedPackageRefusalCause>,
    pub locus: Option<CheckedNodeId>,
    pub contract_version: Option<Box<str>>,
    pub document_pointer: Option<JsonPointer>,
    pub expected_node_id: ExpectedNodeId,
}

impl PartialEq<ExpectedRefusal> for CheckedPackageRefusal {
    fn eq(&self, expected: &ExpectedRefusal) -> bool {
        self.code == expected.code
            && self.path == expected.path
            && self.cause == expected.cause
            && self.locus == expected.locus
            && self.contract_version == expected.contract_version
            && self.document_pointer == expected.document_pointer
            && match expected.expected_node_id {
                ExpectedNodeId::Absent => self.expected_node_id().is_none(),
                ExpectedNodeId::Present => self.expected_node_id().is_some(),
            }
    }
}

impl PartialEq<CheckedPackageRefusal> for ExpectedRefusal {
    fn eq(&self, actual: &CheckedPackageRefusal) -> bool {
        actual == self
    }
}

impl PartialEq<ExpectedRefusal> for CheckedPackageV2ReadResult {
    fn eq(&self, expected: &ExpectedRefusal) -> bool {
        match self {
            Self::Refused(actual) => actual == expected,
            Self::Admitted(_) | Self::Incomplete(_) => false,
        }
    }
}

/// A refusal about the value at the RFC 6901 pointer `path`.
pub fn refusal(code: CheckedPackageRefusalCode, path: &str) -> ExpectedRefusal {
    ExpectedRefusal {
        code,
        path: Some(pointer(path)),
        cause: None,
        locus: None,
        contract_version: None,
        document_pointer: None,
        expected_node_id: ExpectedNodeId::Absent,
    }
}

/// A refusal at `path` carrying a machine `cause`.
pub fn refusal_cause(
    code: CheckedPackageRefusalCode,
    path: &str,
    cause: CheckedPackageRefusalCause,
) -> ExpectedRefusal {
    ExpectedRefusal {
        cause: Some(cause),
        ..refusal(code, path)
    }
}

/// A refusal about the byte stream rather than a value: it carries no path.
pub fn refusal_bytes(code: CheckedPackageRefusalCode) -> ExpectedRefusal {
    ExpectedRefusal {
        code,
        path: None,
        cause: None,
        locus: None,
        contract_version: None,
        document_pointer: None,
        expected_node_id: ExpectedNodeId::Absent,
    }
}

/// `unknown_contract_version` at `/contract_version`, carrying the version
/// string the reader read there.
pub fn unknown_version(version: &str) -> ExpectedRefusal {
    ExpectedRefusal {
        code: CheckedPackageRefusalCode::UnknownContractVersion,
        path: Some(pointer("/contract_version")),
        cause: None,
        locus: None,
        contract_version: Some(version.into()),
        document_pointer: None,
        expected_node_id: ExpectedNodeId::Absent,
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
) -> ExpectedRefusal {
    ExpectedRefusal {
        code,
        path: Some(pointer(path)),
        cause,
        locus: Some(typed_node_id(locus_digest)),
        contract_version: None,
        document_pointer: None,
        expected_node_id: if code == CheckedPackageRefusalCode::InvalidPackage
            && cause == Some(CheckedPackageRefusalCause::StaleNodeKey)
        {
            ExpectedNodeId::Present
        } else {
            ExpectedNodeId::Absent
        },
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
const FIXTURE_SOURCE_DOMAIN: &str = "quire.source.bytes/v1";
const APPLICATION_NODE_VERSION: &str = "quire.application-node/v1";

/// A `CheckedArtifactRef`-shaped definition reference: exactly `{authority,
/// identity}` (QSpec `DefinitionRef`).
pub fn definition_ref(authority: &str, identity: &str) -> Value {
    json!({
        "authority": authority,
        "identity": identity,
    })
}

/// A `CheckedSourceRef`-shaped locked source: exactly `{authority, identity,
/// digest_domain, digest}` (QSpec `RawSourceRef`). `digest` need not be a real
/// content hash: no evidence is attested for a source's bytes, so any
/// syntactically valid 64-hex-digit string is a current lock row.
pub fn source_ref(authority: &str, identity: &str, domain: &str, digest: &str) -> Value {
    json!({
        "authority": authority,
        "identity": identity,
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

/// The key QSL FR-092 and FR-094 give an anonymous structural node
/// (`quire.structural-node/v1`: no `declaration`, no `recursion`, no `owner`;
/// `semantic_type` `null` for a node typed by itself). Written here from that
/// preimage, independently of the reader, so a fixture's derived keys are not
/// the reader's own output.
pub fn structural_key(
    node_tag: &str,
    semantic_form: &str,
    semantic_type: Option<&str>,
    body: &Value,
) -> String {
    let preimage = json!({
        "version": "quire.structural-node/v1",
        "node_tag": node_tag,
        "semantic_form": semantic_form,
        "semantic_type": semantic_type.map_or(Value::Null, node_id),
        "declaration": Value::Null,
        "recursion": Value::Null,
        "body": body,
    });
    sha256_hex(&canonical(&preimage))
}

/// The derived key of the anonymous `scalar_type`/`boolean` node, as a
/// literal; `structural_key` recomputes it in `tc_226_the_recorded_scalar_keys_are_the_derived_ones`.
pub const BOOLEAN_KEY: &str = "9964390677844ad66b781babdbfa95933bc2b16ef1e86f67005966b77e6db3aa";

/// The derived key of the anonymous `scalar_type`/`integer` node, as a literal.
pub const INTEGER_KEY: &str = "07f6dca966d22bde13d3bb198f12610e57d8e1e04d0476bbab03f405d2b04e32";

/// The derived key of the anonymous `scalar_type`/`boolean` node.
pub fn boolean_key() -> String {
    BOOLEAN_KEY.to_owned()
}

/// The derived key of the anonymous `scalar_type`/`integer` node.
pub fn integer_key() -> String {
    INTEGER_KEY.to_owned()
}

/// An `aggregate` of one `reference` to `target`: the closed body of a
/// `reference`, `option` and collection node.
pub fn over_body(target: &str) -> Value {
    json!({"term": "aggregate", "members": [
        {"term": "reference", "target": node_id(target)},
    ]})
}

/// An `aggregate` of the `min` then `max` bindings, each an `integer` literal
/// typed at the derived `Integer` key: the closed body of an `integer_range`
/// and a `collection_bounds` node.
pub fn bounds_body(min: &str, max: &str) -> Value {
    let integer = integer_key();
    let bound = |name: &str, value: &str| {
        json!({"term": "binding", "name": name, "value": {
            "term": "literal", "type": node_id(&integer),
            "value_kind": "integer", "value": value}})
    };
    json!({"term": "aggregate", "members": [bound("min", min), bound("max", max)]})
}

/// The derived key of an anonymous `integer_range` node over `[min, max]`.
pub fn integer_range_key(min: &str, max: &str) -> String {
    structural_key(
        "bounded_domain",
        "integer_range",
        Some(&integer_key()),
        &bounds_body(min, max),
    )
}

/// The `quire.application-node/v1` key of an undeclared application node of
/// `node_tag`/`semantic_form` typed at the node `semantic_type`, over `body`.
pub fn application_node_key(
    node_tag: &str,
    semantic_form: &str,
    semantic_type: &str,
    body: &Value,
) -> String {
    application_key(node_tag, semantic_form, &node_id(semantic_type), body)
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
        "a2a2" => application_key(
            "temporal",
            "formula",
            &node_id(&family_key("aaaa")),
            &eventually_formula_body(),
        ),
        "a5a5" => application_key(
            "temporal",
            "formula",
            &node_id(&family_key("aaaa")),
            &holds_formula_body(),
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
        // The undeclared nodes of the ten derived shapes carry the key the
        // reader derives (FR-038-AC-134), not a placeholder. `a6a6` is the
        // same anonymous Boolean node as `aaaa`: one key names one node.
        "aaaa" | "a6a6" => boolean_key(),
        "a3a3" => integer_key(),
        "bbbb" => structural_key(
            "composite_type",
            "option",
            None,
            &over_body(&family_key("aaaa")),
        ),
        "cccc" => integer_range_key("0", "1000"),
        "3030" => structural_key(
            "state",
            "snapshot",
            Some(&boolean_key()),
            &empty_aggregate(),
        ),
        "7070" => structural_key(
            "correspondence",
            "source_locus",
            Some(&boolean_key()),
            &empty_aggregate(),
        ),
        "8080" => structural_key(
            "function",
            "pure_function",
            Some(&boolean_key()),
            &empty_aggregate(),
        ),
        "a1a1" => structural_key("scalar_type", "text", None, &empty_aggregate()),
        "dddd" => structural_key("value", "literal", Some(&boolean_key()), &empty_aggregate()),
        "eeee" => structural_key(
            "expression",
            "reference",
            Some(&boolean_key()),
            &json!({"term": "reference", "target": node_id(&family_key("dddd"))}),
        ),
        "a4a4" => structural_key(
            "value",
            "parameter",
            Some(&boolean_key()),
            &json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "name", "value": {
                        "term": "literal", "type": node_id(&family_key("a1a1")),
                        "value_kind": "text", "value": "over"}},
                    {"term": "binding", "name": "level", "value": {
                        "term": "literal", "type": node_id(&family_key("a3a3")),
                        "value_kind": "integer", "value": "0"}},
                ],
            }),
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
/// construction rather than by two independently-typed copies. The profile is
/// one of QSpec FR-250's five, a bounded one, which fits the fixture's closed
/// interval `{0, 3}` (FR-038-AC-104, FR-038-AC-108).
fn temporal_profile_law() -> Value {
    json!({
        "role": "temporal_profile",
        "definition": definition_ref(
            FIXTURE_SOURCE_AUTHORITY,
            "quire.temporal.event-position.false-extension/v1",
        ),
    })
}

/// The lock's own `protocol_profile` selection; see [`temporal_profile_law`].
fn protocol_profile_law() -> Value {
    json!({
        "role": "protocol_profile",
        "definition": definition_ref(
            FIXTURE_SOURCE_AUTHORITY,
            "quire.fixture.protocol-profile/v1",
        ),
    })
}

/// The shared fixture's `quire.op.temporal.clause` body, in the shape the
/// catalog fixes (FR-038-AC-68): six arguments and no member. The first is a
/// `reference` to the `over` `value`/`parameter` node (`a4a4`), the second a
/// `text` literal, the next three empty aggregates, and the sixth a
/// `reference` to the `temporal`/`formula` node (`a2a2`) applying
/// `quire.op.temporal.eventually` with the interval `{0, 3}` over a second
/// formula node (`a5a5`) applying `quire.op.temporal.holds` (FR-038-AC-96).
fn temporal_clause_body() -> Value {
    json!({
        "term": "application",
        "operator": "temporal",
        "operation": {
            "identity": "quire.op.temporal.clause",
            "laws": [temporal_profile_law()],
            "mode": Value::Null,
            "member": Value::Null,
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [
            {"term": "reference", "target": node_id(&family_key("a4a4"))},
            {"term": "literal", "type": node_id(&family_key("a1a1")),
             "value_kind": "text", "value": "clause"},
            empty_aggregate(),
            empty_aggregate(),
            empty_aggregate(),
            {"term": "reference", "target": node_id(&family_key("a2a2"))},
        ],
    })
}

/// The reference targets of [`temporal_clause_body`], digest-ascending: the
/// node's wire `dependencies` (FR-322's application-node join).
fn temporal_clause_dependencies() -> Vec<String> {
    let mut targets = vec![family_key("a4a4"), family_key("a2a2")];
    targets.sort();
    targets
}

/// A `temporal_formula` application of `identity` over `arguments`, with
/// `member`, as the body root of a `temporal`/`formula` node.
fn formula_body(identity: &str, member: Value, arguments: Vec<Value>) -> Value {
    json!({
        "term": "application",
        "operator": "temporal_formula",
        "operation": {
            "identity": identity,
            "laws": Value::Array(Vec::new()),
            "mode": Value::Null,
            "member": member,
            "leaves": Value::Array(Vec::new()),
        },
        "result_type": node_id(&family_key("aaaa")),
        "arguments": arguments,
    })
}

/// The fixture's inner formula (`a5a5`): `quire.op.temporal.holds` over a
/// Boolean literal typed at the fixture's own Boolean node (`a6a6`), which
/// no test mutates the way `aaaa`, the family nodes' shared type, is.
fn holds_formula_body() -> Value {
    formula_body(
        "quire.op.temporal.holds",
        Value::Null,
        vec![
            json!({"term": "literal", "type": node_id(&family_key("a6a6")),
                    "value_kind": "boolean", "value": true}),
        ],
    )
}

/// The fixture's root formula (`a2a2`): `quire.op.temporal.eventually` with
/// the interval `{0, 3}` over the `holds` node.
fn eventually_formula_body() -> Value {
    formula_body(
        "quire.op.temporal.eventually",
        json!({"kind": "temporal_interval", "interval": {"lower": "0", "upper": "3"}}),
        vec![json!({"term": "reference", "target": node_id(&family_key("a5a5"))})],
    )
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
/// `occurrences` is a single `generated`-role occurrence. It carries no
/// `declaration` and no nominal preimage; [`declared`] makes one a source
/// declaration.
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

/// `node` as a source declaration named `qualified_name`: its one
/// occurrence has the `declaration` role and it carries the name.
fn declared(mut node: Value, qualified_name: &[&str]) -> Value {
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    node["declaration"] = json!({ "qualified_name": qualified_name });
    let source = fixture_source();
    node["owner"] = json!({
        "kind": "source", "authority": source["authority"], "identity": source["identity"],
    });
    node
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

pub fn fixture_source() -> Value {
    source_ref(
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
    source_ref(
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
    definition_ref(
        FIXTURE_SOURCE_AUTHORITY,
        "quire.fixture.definition.selection/v1",
    )
}

fn fixture_edition_definition() -> Value {
    definition_ref(FIXTURE_SOURCE_AUTHORITY, "quire.fixture.edition/v1")
}

fn fixture_diagnostics_catalog() -> Value {
    definition_ref(
        FIXTURE_SOURCE_AUTHORITY,
        "quire.fixture.diagnostics-catalog/v1",
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

/// Collects the targets FR-322's application-node dependency join names in
/// `term` (its `reference` targets and the `declaration` of each application's
/// operation member) into `out`; whether `term` holds an application.
fn join_targets(term: &Value, out: &mut std::collections::BTreeSet<String>) -> bool {
    let mut holds_application = false;
    match term["term"].as_str() {
        Some("reference") => {
            out.insert(
                term["target"]["digest"]
                    .as_str()
                    .expect("digest")
                    .to_owned(),
            );
        }
        Some("application") => {
            holds_application = true;
            if let Some(declaration) = term["operation"]["member"]["declaration"]["digest"].as_str()
            {
                out.insert(declaration.to_owned());
            }
            for argument in term["arguments"].as_array().into_iter().flatten() {
                join_targets(argument, out);
            }
        }
        Some("aggregate") => {
            for member in term["members"].as_array().into_iter().flatten() {
                holds_application |= join_targets(member, out);
            }
        }
        Some("binding") => holds_application |= join_targets(&term["value"], out),
        _ => {}
    }
    holds_application
}

/// Gives every node whose body holds an `application` exactly the
/// `dependencies` FR-322's join names, digest-ascending.
fn join_application_dependencies(package: &mut Value) {
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        let mut targets = std::collections::BTreeSet::new();
        if join_targets(&node["body"], &mut targets) {
            node["dependencies"] =
                Value::Array(targets.iter().map(|digest| node_id(digest)).collect());
        }
    }
}

/// After a test edited node bodies: re-derives each application node's
/// dependencies and key, innermost first, renaming each key wherever it is
/// referenced, then the source map and the package identity. A node keyed by
/// something other than its application preimage keeps its key.
pub fn settle(package: &mut Value) {
    let count = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    for _ in 0..=count {
        join_application_dependencies(package);
        let mut changed = false;
        for position in 0..count {
            let node = package["semantic_graph"]["nodes"][position].clone();
            if node["body"]["term"] != "application" {
                continue;
            }
            let fresh = application_key(
                node["node_tag"].as_str().expect("tag"),
                node["semantic_form"].as_str().expect("form"),
                &node["semantic_type"],
                &node["body"],
            );
            if node["node_id"]["digest"] != fresh.as_str() {
                rekey_application_node(package, position);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    join_application_dependencies(package);
    rebuild_source_map(package);
    refresh_identity(package);
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
/// four supporting nodes the family nodes' own required members reference —
/// a second `pure_function` the `function` family's real `application`
/// argument names, the source-declared `object_type` (`Example::Account`,
/// self-typed) and the `process` the `state` family's frame body names, and
/// the dedicated `state`/`frame` node itself, typed by that object type and
/// modifying its `balance` field and the relationship node (the `state`
/// family's own representative node is a plain `snapshot`, so its lowering closure
/// stays the uniform single-`aaaa` shape every other plain family node has;
/// `checked_package_v2_frame_bodies.rs` locates the one real frame node by
/// its `(node_tag, semantic_form)` shape, not by position).
///
/// Every other node is typed by node 0 (`aaaa`, a self-typed `scalar_type`/
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
    let f1515 = family_key("1515");
    let f1616 = family_key("1616");
    let frame_body = json!({
        "term": "frame",
        "modifies": [
            {"kind": "field", "declaration": node_id(&f1515), "name": "balance"},
            {"kind": "relationship", "declaration": node_id(&f2020)},
        ],
        "creates": [node_id(&f1515)],
        "deletes": [node_id(&f1616)],
    });
    let frame_key = structural_key("state", "frame", Some(&f1515), &frame_body);

    let nodes = vec![
        plain_node(
            &aaaa,
            "scalar_type",
            "boolean",
            &aaaa,
            &[],
            empty_aggregate(),
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
            &bbbb,
            &[],
            over_body(&aaaa),
        ),
        plain_node(
            &cccc,
            "bounded_domain",
            "integer_range",
            &family_key("a3a3"),
            &[family_key("a3a3").as_str()],
            bounds_body("0", "1000"),
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
        {
            let mut relation = plain_node(
                &f2020,
                "relation",
                "relationship",
                &aaaa,
                &[aaaa.as_str()],
                empty_aggregate(),
            );
            relation["owner"] = json!({
                "kind": "model", "identity": FAMILY_MODEL_IDENTITY,
                "node": FAMILY_RELATIONSHIP,
            });
            relation
        },
        plain_node(
            &f3030,
            "state",
            "snapshot",
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
            temporal_clause_body()["arguments"]
                .as_array()
                .expect("arguments")
                .clone(),
            &temporal_clause_dependencies()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
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
        declared(
            plain_node(
                &f8080,
                "function",
                "pure_function",
                &aaaa,
                &[],
                empty_aggregate(),
            ),
            &["Example", "Function"],
        ),
        declared(
            plain_node(
                &f1515,
                "model",
                "object_type",
                &f1515,
                &[aaaa.as_str()],
                empty_aggregate(),
            ),
            &["Example", "Account"],
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
            &f1515,
            &[f1515.as_str(), f1616.as_str(), f2020.as_str()],
            frame_body,
        ),
        // The nodes the temporal clause's arguments name: the `text` type of
        // its name literal, the `temporal`/`formula` node it applies
        // (`eventually` over `holds`) and its `over` parameter.
        plain_node(
            &family_key("a1a1"),
            "scalar_type",
            "text",
            &family_key("a1a1"),
            &[],
            empty_aggregate(),
        ),
        application_node(
            "temporal",
            "formula",
            "temporal_formula",
            eventually_formula_body()["operation"].clone(),
            &aaaa,
            eventually_formula_body()["arguments"]
                .as_array()
                .expect("arguments")
                .clone(),
            &[family_key("a5a5").as_str()],
        ),
        application_node(
            "temporal",
            "formula",
            "temporal_formula",
            holds_formula_body()["operation"].clone(),
            &aaaa,
            holds_formula_body()["arguments"]
                .as_array()
                .expect("arguments")
                .clone(),
            // A literal's `type` is no dependency (FR-322).
            &[],
        ),
        plain_node(
            &family_key("a3a3"),
            "scalar_type",
            "integer",
            &family_key("a3a3"),
            &[],
            empty_aggregate(),
        ),
        // The clause's `over` parameter: QSL FR-092's `{name, level}` body,
        // typed at the Boolean node, carrying the `expression` role.
        {
            let mut parameter = plain_node(
                &family_key("a4a4"),
                "value",
                "parameter",
                &aaaa,
                &[],
                json!({
                    "term": "aggregate",
                    "members": [
                        {"term": "binding", "name": "name", "value": {
                            "term": "literal", "type": node_id(&family_key("a1a1")),
                            "value_kind": "text", "value": "over"}},
                        {"term": "binding", "name": "level", "value": {
                            "term": "literal", "type": node_id(&family_key("a3a3")),
                            "value_kind": "integer", "value": "0"}},
                    ],
                }),
            );
            parameter["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
            parameter
        },
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
    let family_document = family_model_document();
    package["lock"]["model_selections"] = json!([{
        "identity": FAMILY_MODEL_IDENTITY,
        "digest_domain": "sha256-jcs",
        "digest": domain_package_digest(&family_document),
    }]);
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
    let root = boolean_key();
    let declaring = "cafe".repeat(16);
    let literal_body =
        json!({"term": "literal", "type": node_id(&root), "value_kind": "boolean", "value": true});
    let literal_key = structural_key("value", "literal", Some(&root), &literal_body);
    let func = structural_key("function", "pure_function", Some(&root), &empty_aggregate());

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
    let source = fixture_source();
    declaring_node["owner"] = json!({
        "kind": "source", "authority": source["authority"], "identity": source["identity"],
    });

    let literal_node = plain_node(
        &literal_key,
        "value",
        "literal",
        &root,
        &[root.as_str()],
        literal_body,
    );

    let func_node = declared(
        plain_node(
            &func,
            "function",
            "pure_function",
            &root,
            &[],
            empty_aggregate(),
        ),
        &["Example", "Function"],
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
