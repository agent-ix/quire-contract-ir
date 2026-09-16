// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Byte-for-byte golden for the frozen CheckedPackage V1 reader.
//!
//! This file uses only the V1 API that existed before V2 consumption, so the
//! same assertions pass against the pre-split reader and the split one.

use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedArtifactLocator, CheckedNodeId, CheckedPackage, CheckedPackageLimit,
    CheckedPackageReadContext, CheckedPackageReadLimits, CheckedPackageReadResult,
    CheckedPackageRefusalCode, CompleteLoweringRecord,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

type Mutation = Box<dyn Fn(&mut Value)>;

/// Package identity of the golden wire, recorded from the pre-split reader.
const GOLDEN_PACKAGE_DIGEST: &str =
    "6472b73bf6375ea07b6befc4ed22caee898e1bb062eecfb7d16a754b0224e348";

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

/// Tracing: TC-047, FR-038-AC-1
#[trace("TC-047", "FR-038-AC-1")]
#[test]
fn tc_047_frozen_v1_reader_admits_lowers_and_refuses_exactly_as_before() {
    let (value, context) = golden_value();
    let wire = canonical(&value);
    let package = match CheckedPackage::read(&wire, CheckedPackageReadLimits::bounded(), &context) {
        CheckedPackageReadResult::Admitted(package) => package,
        other => panic!("golden V1 wire must admit, got {other:?}"),
    };
    assert_eq!(
        package.package_id().domain.as_ref(),
        "quire.package.semantic/v1"
    );
    assert_eq!(package.package_id().digest.as_ref(), GOLDEN_PACKAGE_DIGEST);

    let result = package.lower(&[id(0), id(98), id(4), id(12)], 2);
    assert_eq!(result.package_id, *package.package_id());
    assert_eq!(result.records.len(), 4);
    match &result.records[0] {
        CompleteLoweringRecord::Lowered { node } => {
            assert_eq!(node.node_id, id(0));
            assert_eq!(node.node_tag.as_ref(), "scalar_type");
            assert_eq!(node.semantic_form.as_ref(), "boolean");
            assert_eq!(node.semantic_type, id(0));
            assert!(node.dependencies.is_empty());
            assert_eq!(
                node.body,
                json!({"term":"literal","value_kind":"boolean","value":true})
            );
            assert_eq!(node.source_map.len(), 1);
            assert_eq!(node.source_map[0].regions[0].start, 0);
            assert_eq!(node.source_map[0].regions[0].end, 1);
        }
        other => panic!("expected lowered scalar, got {other:?}"),
    }
    assert_eq!(
        result.records[1],
        CompleteLoweringRecord::InvalidInput { node_id: id(98) }
    );
    assert_eq!(
        result.records[2],
        CompleteLoweringRecord::Failed {
            node_id: id(4),
            limit: 2,
            consumed: 3
        }
    );
    assert_eq!(
        result.records[3],
        CompleteLoweringRecord::Failed {
            node_id: id(12),
            limit: 2,
            consumed: 4
        }
    );
    match &package.lower(&[id(4)], 3).records[0] {
        CompleteLoweringRecord::Lowered { node } => {
            assert_eq!(node.body, json!({"term":"reference","target":node_id(0)}));
        }
        other => panic!("expected lowered expression, got {other:?}"),
    }

    let cases: Vec<(&str, Mutation, CheckedPackageRefusalCode, &str)> = vec![
        (
            "unknown contract version",
            Box::new(|v| v["contract_version"] = json!("quire.checked-package/v2")),
            CheckedPackageRefusalCode::UnknownContractVersion,
            "contract_version",
        ),
        (
            "package id domain",
            Box::new(|v| v["package_id"]["domain"] = json!("quire.package.semantic/v2")),
            CheckedPackageRefusalCode::DigestDomainMismatch,
            "package_id.domain",
        ),
        (
            "source digest domain",
            Box::new(|v| {
                v["lock"]["sources"][0]["digest_domain"] = json!("quire.definition.bytes/v1");
            }),
            CheckedPackageRefusalCode::DigestDomainMismatch,
            "lock.sources",
        ),
        (
            "unavailable capability",
            Box::new(|v| v["capability_report"][0]["disposition"] = json!("unsupported")),
            CheckedPackageRefusalCode::UnknownRequiredCapability,
            "capability_report",
        ),
        (
            "unknown member",
            Box::new(|v| v["future_member"] = json!(true)),
            CheckedPackageRefusalCode::UnknownMember,
            "document",
        ),
        (
            "V2 nominal member on a V1 node",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][0]["nominal_identity_preimage"] = json!(null);
            }),
            CheckedPackageRefusalCode::UnknownMember,
            "document",
        ),
        (
            "unknown node tag",
            Box::new(|v| v["semantic_graph"]["nodes"][0]["node_tag"] = json!("future_node")),
            CheckedPackageRefusalCode::UnsupportedNodeTag,
            "semantic_graph.nodes.node_tag",
        ),
        (
            "wrong family form",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][0]["semantic_form"] = json!("protocol_clause");
            }),
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.semantic_form",
        ),
        (
            "stale package id",
            Box::new(|v| v["package_id"]["digest"] = json!("0".repeat(64))),
            CheckedPackageRefusalCode::StaleDependency,
            "package_id.digest",
        ),
        (
            "incomplete source map",
            Box::new(|v| {
                v["source_map"].as_array_mut().expect("source map").pop();
            }),
            CheckedPackageRefusalCode::InvalidSourceMap,
            "source_map",
        ),
    ];
    for (name, mutate, code, path) in cases {
        let mut mutated = value.clone();
        mutate(&mut mutated);
        assert_refused(
            CheckedPackage::read(
                &canonical(&mutated),
                CheckedPackageReadLimits::bounded(),
                &context,
            ),
            code,
            path,
            name,
        );
    }
    let mut spaced = b" ".to_vec();
    spaced.extend_from_slice(&wire);
    assert_refused(
        CheckedPackage::read(&spaced, CheckedPackageReadLimits::bounded(), &context),
        CheckedPackageRefusalCode::NoncanonicalWire,
        "document",
        "noncanonical wire",
    );
    assert_refused(
        CheckedPackage::read(
            &wire,
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageReadContext::new(),
        ),
        CheckedPackageRefusalCode::StaleDependency,
        "lock.sources",
        "absent artifact bytes",
    );

    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = 12;
    match CheckedPackage::read(&wire, limits, &context) {
        CheckedPackageReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!((incomplete.limit, incomplete.consumed), (12, 13));
        }
        other => panic!("expected work incompleteness, got {other:?}"),
    }
}

fn assert_refused(
    result: CheckedPackageReadResult,
    code: CheckedPackageRefusalCode,
    path: &str,
    name: &str,
) {
    match result {
        CheckedPackageReadResult::Refused(refusal) => {
            assert_eq!(
                (refusal.code, refusal.path.as_ref()),
                (code, path),
                "{name}"
            );
        }
        other => panic!("{name}: expected refusal, got {other:?}"),
    }
}

fn golden_value() -> (Value, CheckedPackageReadContext) {
    let source = artifact("source", "git", "quire.source.bytes/v1");
    let definition = artifact("edition", "semver", "quire.definition.bytes/v1");
    let catalog = artifact("catalog", "draft", "quire.definition.bytes/v1");
    let edition = json!({"role":"edition", "definition": definition});
    let nodes = NODE_TAGS
        .iter()
        .enumerate()
        .map(|(index, (tag, form, term))| {
            let body = match *term {
                "literal" => json!({"term":"literal", "value_kind":"boolean", "value":true}),
                "aggregate" => json!({"term":"aggregate", "members":[]}),
                "application" => json!({"term":"application", "operator":"call", "arguments":[]}),
                _ => json!({"term":"reference", "target": node_id(0)}),
            };
            json!({
                "node_id": node_id(index),
                "schema_version":"quire.checked-semantic-graph/v1",
                "node_tag":tag,
                "semantic_form":form,
                "semantic_type":node_id(0),
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
    let package_digest = sha256(&canonical(&identity_preimage));
    let source_map = (0..NODE_TAGS.len())
        .map(|index| {
            json!({"node_id":node_id(index), "role":"declaration", "ordinal":0, "regions":[{"source":source, "start":index, "end":index + 1}]})
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
    for (identity, namespace, domain) in [
        ("source", "git", "quire.source.bytes/v1"),
        ("edition", "semver", "quire.definition.bytes/v1"),
        ("catalog", "draft", "quire.definition.bytes/v1"),
    ] {
        context.insert(
            CheckedArtifactLocator {
                authority: "agent-ix".into(),
                identity: identity.into(),
                revision_namespace: namespace.into(),
                revision_value: "1".into(),
                domain: domain.into(),
            },
            identity.as_bytes().to_vec(),
        );
    }
    (value, context)
}

fn artifact(identity: &str, namespace: &str, domain: &str) -> Value {
    json!({"authority":"agent-ix", "identity":identity, "revision":{"namespace":namespace, "value":"1"}, "digest_domain":domain, "digest":sha256(identity.as_bytes())})
}

fn node_id(index: usize) -> Value {
    json!({"domain":"quire.checked-semantic-node/v1", "digest":format!("{:064x}", index + 1)})
}

fn id(index: usize) -> CheckedNodeId {
    serde_json::from_value(node_id(index)).expect("node id")
}

fn canonical(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("canonical test JSON")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
