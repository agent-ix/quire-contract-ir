// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Real-reader qualification for the frozen QSpec I04 CheckedPackage V1 wire.

use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedArtifactLocator, CheckedPackage, CheckedPackageLimit, CheckedPackageReadContext,
    CheckedPackageReadLimits, CheckedPackageReadResult, CheckedPackageRefusalCode,
    CompleteLoweringRecord,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const NODE_TAGS: [(&str, &str, &str); 13] = [
    ("scalar_type", "boolean", "literal"),
    ("composite_type", "record", "aggregate"),
    ("bounded_domain", "integer_range", "aggregate"),
    ("value", "literal", "literal"),
    ("expression", "reference", "reference"),
    ("function", "pure_function", "application"),
    ("model", "model_import", "aggregate"),
    ("relation", "relationship", "aggregate"),
    ("state", "state_clause", "aggregate"),
    ("temporal", "temporal_clause", "application"),
    ("protocol", "protocol_clause", "application"),
    ("claim", "verification_claim", "application"),
    ("correspondence", "source_locus", "reference"),
];

/// Tracing: TC-044, FR-035-AC-1, FR-035-AC-3, FR-035-AC-4, FR-322-AC-7
#[test]
#[trace("TC-044", "FR-035-AC-1", "FR-035-AC-3", "FR-035-AC-4")]
fn tc_044_i04_reader_admits_and_lowers_every_public_node_family() {
    let (wire, context, node_ids) = valid_wire();
    let admitted = admit(&wire, &context, CheckedPackageReadLimits::bounded());
    assert_eq!(admitted.graph().nodes.len(), NODE_TAGS.len());
    let result = admitted.lower(
        &[node_ids[0].clone(), missing_node(), node_ids[1].clone()],
        2,
    );
    assert!(matches!(
        &result.records[0],
        CompleteLoweringRecord::Lowered { node } if node.node_tag.as_ref() == "scalar_type" && node.source_map.len() == 1
    ));
    assert!(matches!(
        &result.records[1],
        CompleteLoweringRecord::InvalidInput { .. }
    ));
    assert!(matches!(
        &result.records[2],
        CompleteLoweringRecord::Failed {
            limit: 2,
            consumed: 3,
            ..
        }
    ));
}

/// Tracing: TC-044, FR-035-AC-2, FR-322-AC-4, FR-322-AC-5
#[test]
#[trace("TC-044", "FR-035-AC-2")]
fn tc_044_i04_reader_refuses_strict_wire_and_identity_mutations() {
    let (wire, context, _) = valid_wire();
    let duplicate = format!(
        "{{\"contract_version\":\"quire.checked-package/v1\",\"contract_version\":\"quire.checked-package/v1\",{}}}",
        wire.trim_start_matches('{')
    );
    assert_refusal(
        CheckedPackage::read(
            duplicate.as_bytes(),
            CheckedPackageReadLimits::bounded(),
            &context,
        ),
        CheckedPackageRefusalCode::DuplicateMember,
    );
    assert_refusal(
        CheckedPackage::read(
            b" {\"contract_version\":\"quire.checked-package/v1\"}",
            CheckedPackageReadLimits::bounded(),
            &context,
        ),
        CheckedPackageRefusalCode::NoncanonicalWire,
    );

    let mut unknown_version: Value = serde_json::from_str(&wire).expect("fixture JSON");
    unknown_version["contract_version"] = json!("quire.checked-package/v2");
    assert_refusal(
        CheckedPackage::read(
            &canonical(&unknown_version),
            CheckedPackageReadLimits::bounded(),
            &context,
        ),
        CheckedPackageRefusalCode::UnknownContractVersion,
    );

    let mut cross_domain: Value = serde_json::from_str(&wire).expect("fixture JSON");
    cross_domain["lock"]["sources"][0]["digest_domain"] = json!("quire.definition.bytes/v1");
    assert_refusal(
        CheckedPackage::read(
            &canonical(&cross_domain),
            CheckedPackageReadLimits::bounded(),
            &context,
        ),
        CheckedPackageRefusalCode::DigestDomainMismatch,
    );

    let mut unavailable: Value = serde_json::from_str(&wire).expect("fixture JSON");
    unavailable["capability_report"][0]["disposition"] = json!("unsupported");
    assert_refusal(
        CheckedPackage::read(
            &canonical(&unavailable),
            CheckedPackageReadLimits::bounded(),
            &context,
        ),
        CheckedPackageRefusalCode::UnknownRequiredCapability,
    );

    let mut unknown_member: Value = serde_json::from_str(&wire).expect("fixture JSON");
    unknown_member["future_member"] = json!(true);
    assert_refusal(
        CheckedPackage::read(
            &canonical(&unknown_member),
            CheckedPackageReadLimits::bounded(),
            &context,
        ),
        CheckedPackageRefusalCode::UnknownMember,
    );

    let mut unknown_tag: Value = serde_json::from_str(&wire).expect("fixture JSON");
    unknown_tag["semantic_graph"]["nodes"][0]["node_tag"] = json!("future_node");
    assert_refusal(
        CheckedPackage::read(
            &canonical(&unknown_tag),
            CheckedPackageReadLimits::bounded(),
            &context,
        ),
        CheckedPackageRefusalCode::UnsupportedNodeTag,
    );

    let stale = CheckedPackageReadContext::new();
    assert_refusal(
        CheckedPackage::read(wire.as_bytes(), CheckedPackageReadLimits::bounded(), &stale),
        CheckedPackageRefusalCode::StaleDependency,
    );
}

/// Tracing: TC-044, FR-035-AC-2, FR-322-AC-6, FR-322-AC-9
#[test]
#[trace("TC-044", "FR-035-AC-2")]
fn tc_044_i04_reader_reports_exact_and_one_over_resource_accounting() {
    let (wire, context, _) = valid_wire();
    let value: Value = serde_json::from_str(&wire).expect("fixture JSON");
    let mut exact = CheckedPackageReadLimits {
        bytes: u64::try_from(wire.len()).expect("fixture length"),
        depth: json_depth(&value),
        nodes: u64::try_from(NODE_TAGS.len()).expect("node count"),
        edges: 0,
        occurrences: u64::try_from(NODE_TAGS.len() * 2).expect("occurrence count"),
        diagnostics: 0,
        work: u64::try_from(NODE_TAGS.len()).expect("work count"),
    };
    assert!(matches!(
        CheckedPackage::read(wire.as_bytes(), exact, &context),
        CheckedPackageReadResult::Admitted(_)
    ));

    exact.bytes -= 1;
    assert_incomplete(
        CheckedPackage::read(wire.as_bytes(), exact, &context),
        CheckedPackageLimit::Bytes,
        exact.bytes,
        exact.bytes + 1,
    );
    exact.bytes = u64::try_from(wire.len()).expect("fixture length");
    exact.nodes -= 1;
    assert_incomplete(
        CheckedPackage::read(wire.as_bytes(), exact, &context),
        CheckedPackageLimit::Nodes,
        exact.nodes,
        exact.nodes + 1,
    );

    exact.nodes += 1;
    exact.occurrences -= 1;
    assert_incomplete(
        CheckedPackage::read(wire.as_bytes(), exact, &context),
        CheckedPackageLimit::Occurrences,
        exact.occurrences,
        exact.occurrences + 1,
    );
    exact.occurrences += 1;
    exact.work -= 1;
    assert_incomplete(
        CheckedPackage::read(wire.as_bytes(), exact, &context),
        CheckedPackageLimit::Work,
        exact.work,
        exact.work + 1,
    );

    let mut edge_value = value.clone();
    edge_value["semantic_graph"]["nodes"][1]["dependencies"] = json!([node_id(0)]);
    refresh_identity(&mut edge_value);
    let edge_wire = canonical(&edge_value);
    let mut edge_limits = CheckedPackageReadLimits::bounded();
    edge_limits.bytes = u64::try_from(edge_wire.len()).expect("fixture length");
    edge_limits.edges = 0;
    assert_incomplete(
        CheckedPackage::read(&edge_wire, edge_limits, &context),
        CheckedPackageLimit::Edges,
        0,
        1,
    );

    let mut diagnostic_value = value;
    diagnostic_value["diagnostics"]["entries"] = json!([{"stage":"package_read"}]);
    let diagnostic_wire = canonical(&diagnostic_value);
    let mut diagnostic_limits = CheckedPackageReadLimits::bounded();
    diagnostic_limits.bytes = u64::try_from(diagnostic_wire.len()).expect("fixture length");
    diagnostic_limits.diagnostics = 0;
    assert_incomplete(
        CheckedPackage::read(&diagnostic_wire, diagnostic_limits, &context),
        CheckedPackageLimit::Diagnostics,
        0,
        1,
    );
}

fn valid_wire() -> (
    String,
    CheckedPackageReadContext,
    Vec<quire_contract_ir::CheckedNodeId>,
) {
    let source = artifact(
        "agent-ix",
        "source",
        "git",
        "1",
        "quire.source.bytes/v1",
        b"source",
    );
    let definition = artifact(
        "agent-ix",
        "edition",
        "semver",
        "1",
        "quire.definition.bytes/v1",
        b"definition",
    );
    let catalog = artifact(
        "agent-ix",
        "catalog",
        "draft",
        "1",
        "quire.definition.bytes/v1",
        b"catalog",
    );
    let edition = json!({"role":"edition", "definition": definition});
    let node_ids = NODE_TAGS
        .iter()
        .enumerate()
        .map(|(index, _)| node_id(index))
        .collect::<Vec<_>>();
    let nodes = NODE_TAGS
        .iter()
        .enumerate()
        .map(|(index, (tag, form, term))| {
            let body = match *term {
                "literal" => json!({"term":"literal", "value_kind":"boolean", "value":true}),
                "aggregate" => json!({"term":"aggregate", "members":[]}),
                "application" => json!({"term":"application", "operator":"call", "arguments":[]}),
                "reference" => json!({"term":"reference", "target": node_ids[0]}),
                _ => unreachable!("fixed vector terms"),
            };
            json!({
                "node_id": node_ids[index],
                "schema_version":"quire.checked-semantic-graph/v1",
                "node_tag":tag,
                "semantic_form":form,
                "semantic_type":node_ids[0],
                "dependencies":[],
                "occurrences":[{"role":"declaration", "ordinal":0}],
                "body":body,
            })
        })
        .collect::<Vec<_>>();
    let identity_projection = nodes
        .iter()
        .cloned()
        .map(|mut node| {
            node.as_object_mut().expect("object").remove("occurrences");
            node
        })
        .collect::<Vec<_>>();
    let identity_preimage = json!({
        "version":"quire.checked-package-id/v1",
        "edition":edition,
        "profile_selections":[],
        "definition_selections":[],
        "model_selections":[],
        "required_features":["quire.value.complete/v1"],
        "dependency_selections":[],
        "identity_projection":identity_projection,
    });
    let package_digest = digest(&canonical(&identity_preimage));
    let source_map = node_ids
        .iter()
        .enumerate()
        .map(|(index, id)| {
            json!({"node_id":id, "role":"declaration", "ordinal":0, "regions":[{"source":source, "start":index, "end":index + 1}]})
        })
        .collect::<Vec<_>>();
    let value = json!({
        "contract_version":"quire.checked-package/v1",
        "identity_preimage":identity_preimage,
        "package_id":{"domain":"quire.package.semantic/v1", "algorithm":"sha256", "digest":package_digest},
        "lock":{"sources":[source], "edition":edition, "profile_selections":[], "definition_selections":[], "model_selections":[], "required_features":["quire.value.complete/v1"], "dependency_selections":[]},
        "semantic_graph":{"graph_version":"quire.checked-semantic-graph/v1", "nodes":nodes},
        "source_map":source_map,
        "capability_report":[{"feature":"quire.value.complete/v1", "disposition":"available"}],
        "diagnostics":{"catalog":catalog, "entries":[]},
    });
    let mut context = CheckedPackageReadContext::new();
    insert(
        &mut context,
        "agent-ix",
        "source",
        "git",
        "1",
        "quire.source.bytes/v1",
        b"source",
    );
    insert(
        &mut context,
        "agent-ix",
        "edition",
        "semver",
        "1",
        "quire.definition.bytes/v1",
        b"definition",
    );
    insert(
        &mut context,
        "agent-ix",
        "catalog",
        "draft",
        "1",
        "quire.definition.bytes/v1",
        b"catalog",
    );
    let ids = node_ids
        .into_iter()
        .map(|value| serde_json::from_value(value).expect("node id"))
        .collect();
    (
        String::from_utf8(canonical(&value)).expect("UTF-8 JSON"),
        context,
        ids,
    )
}

fn artifact(
    authority: &str,
    identity: &str,
    namespace: &str,
    revision: &str,
    domain: &str,
    bytes: &[u8],
) -> Value {
    json!({"authority":authority, "identity":identity, "revision":{"namespace":namespace, "value":revision}, "digest_domain":domain, "digest":digest(bytes)})
}

fn insert(
    context: &mut CheckedPackageReadContext,
    authority: &str,
    identity: &str,
    namespace: &str,
    revision: &str,
    domain: &str,
    bytes: &[u8],
) {
    context.insert(
        CheckedArtifactLocator {
            authority: authority.into(),
            identity: identity.into(),
            revision_namespace: namespace.into(),
            revision_value: revision.into(),
            domain: domain.into(),
        },
        bytes.to_vec(),
    );
}

fn node_id(index: usize) -> Value {
    json!({"domain":"quire.checked-semantic-node/v1", "digest":format!("{:064x}", index + 1)})
}

fn missing_node() -> quire_contract_ir::CheckedNodeId {
    serde_json::from_value(
        json!({"domain":"quire.checked-semantic-node/v1", "digest":format!("{:064x}", 99)}),
    )
    .expect("node id")
}

fn admit(
    wire: &str,
    context: &CheckedPackageReadContext,
    limits: CheckedPackageReadLimits,
) -> CheckedPackage {
    match CheckedPackage::read(wire.as_bytes(), limits, context) {
        CheckedPackageReadResult::Admitted(package) => *package,
        result => panic!("expected admission, got {result:?}"),
    }
}

fn assert_refusal(result: CheckedPackageReadResult, code: CheckedPackageRefusalCode) {
    assert!(
        matches!(result, CheckedPackageReadResult::Refused(ref refusal) if refusal.code == code),
        "expected {code:?}, got {result:?}"
    );
}

fn assert_incomplete(
    result: CheckedPackageReadResult,
    kind: CheckedPackageLimit,
    limit: u64,
    consumed: u64,
) {
    assert!(
        matches!(result, CheckedPackageReadResult::Incomplete(ref incomplete) if incomplete.limit_kind == kind && incomplete.limit == limit && incomplete.consumed == consumed),
        "expected {kind:?} {limit}/{consumed}, got {result:?}"
    );
}

fn canonical(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("canonical test JSON")
}

fn refresh_identity(value: &mut Value) {
    let projection = value["semantic_graph"]["nodes"]
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
    value["identity_preimage"]["identity_projection"] = Value::Array(projection);
    value["package_id"]["digest"] = json!(digest(&canonical(&value["identity_preimage"])));
}

fn json_depth(value: &Value) -> u64 {
    match value {
        Value::Array(values) => values
            .iter()
            .map(json_depth)
            .max()
            .unwrap_or(0)
            .saturating_add(1),
        Value::Object(values) => values
            .values()
            .map(json_depth)
            .max()
            .unwrap_or(0)
            .saturating_add(1),
        _ => 1,
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
