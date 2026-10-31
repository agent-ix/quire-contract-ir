// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-035/TC-047 and QSpec FR-330: the single six-member package value.

use crate::support::checked_package::{
    self, canonical, evidence_for, refresh_identity, settle, typed_node_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeKind, CheckedNodeTag, CheckedOperationViewError, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm,
    CompleteLoweringProfileV2, CompleteLoweringRecordV2, ContractDependencies,
    ContractDispositionV1, ContractPackageReadResult, ContractPackageWireV1, ContractRootViewV1,
    ContractSourceHandoffError, ContractSourceKeyV1, ContractUnsupportedSourceKindError,
    TemporalForm, CONTRACT_PACKAGE_VERSION,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn id(prefix: &str) -> CheckedNodeId {
    typed_node_id(&checked_package::family_key(prefix))
}

fn missing() -> CheckedNodeId {
    typed_node_id(&"0123456789abcdef".repeat(4))
}

fn source_key(key: &CheckedNodeId) -> ContractSourceKeyV1 {
    ContractSourceKeyV1::from_structural(key).expect("fixture key is structural")
}

fn admit(value: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected V2 admission, got {other:?}"),
    }
}

fn mixed_fixture() -> Value {
    let mut value = v2_all_families();
    let integer = checked_package::node_id(&checked_package::family_key("a3a3"));
    for node in value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["node_id"] == checked_package::node_id(&checked_package::family_key("1010")) {
            node["semantic_type"] = integer.clone();
            node["dependencies"] = json!([integer]);
        } else if node["node_id"] == checked_package::node_id(&checked_package::family_key("dddd"))
        {
            node["semantic_type"] = integer.clone();
            node["dependencies"] = json!([checked_package::node_id(&checked_package::family_key(
                "cccc"
            ))]);
        }
    }
    settle(&mut value);
    value
}

fn mixed_fixture_with_two_dependencies() -> Value {
    let mut value = mixed_fixture();
    let node = value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["node_tag"] == "value")
        .expect("value node");
    node["dependencies"] = json!([
        node["dependencies"][0],
        checked_package::node_id(&checked_package::family_key("a3a3"))
    ]);
    settle(&mut value);
    value
}

fn profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        reference: quire_contract_model::LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: [
            CheckedNodeTag::ScalarType,
            CheckedNodeTag::CompositeType,
            CheckedNodeTag::BoundedDomain,
            CheckedNodeTag::Value,
            CheckedNodeTag::Expression,
            CheckedNodeTag::Function,
            CheckedNodeTag::Model,
            CheckedNodeTag::Relation,
            CheckedNodeTag::State,
            CheckedNodeTag::Temporal,
            CheckedNodeTag::Protocol,
            CheckedNodeTag::Claim,
        ]
        .into_iter()
        .collect(),
        require_bounds: true,
        work_limit: u64::MAX,
    }
}

fn request() -> Vec<CheckedNodeId> {
    vec![id("dddd"), id("1010"), id("7070"), missing(), id("dddd")]
}

fn lower(
    value: &Value,
    requested: &[CheckedNodeId],
) -> quire_contract_model::CompleteLoweringResultV2 {
    admit(value).lower(requested, &profile())
}

fn bytes(result: &quire_contract_model::CompleteLoweringResultV2) -> &[u8] {
    result.package.canonical_bytes().expect("package bytes")
}

fn read(bytes: &[u8]) -> ContractPackageReadResult {
    ContractPackageWireV1::read(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    )
}

fn read_with_limits(bytes: &[u8], limits: CheckedPackageReadLimits) -> ContractPackageReadResult {
    ContractPackageWireV1::read(bytes, limits, &ContractDependencies::new())
}

fn operation(term: &CheckedTerm) -> &quire_contract_model::CheckedApplicationOperation {
    let CheckedTerm::Application { operation, .. } = term else {
        panic!("represented operation term");
    };
    operation
}

#[trace(
    "TC-047",
    "FR-035-AC-23",
    "FR-035-AC-24",
    "FR-035-AC-25",
    "FR-035-AC-26"
)]
#[test]
fn v1_operation_views_require_authenticated_source() {
    let source = admit(&checked_package::positive_operation_identities());
    let key = source
        .graph()
        .nodes
        .iter()
        .find(|node| matches!(&node.body, CheckedTerm::Application { .. }))
        .expect("operation node")
        .node_id
        .clone();
    let lowered = source.lower(std::slice::from_ref(&key), &profile());
    let producer = lowered.package.wire().expect("lowered wire");
    let source_term = source.term(&key).expect("admitted source term");
    let producer_term = producer.term(&key).expect("producer term");
    assert_eq!(producer_term, source_term);
    assert_eq!(operation(producer_term).mode(), Ok(None));
    assert_eq!(operation(producer_term).law_definitions(), Ok(&[][..]));

    let ContractPackageReadResult::Admitted(plain) = read(bytes(&lowered)) else {
        panic!("plain canonical V1 read");
    };
    let plain_term = plain.term(&key).expect("plain term");
    assert_eq!(plain_term, producer_term);
    assert_eq!(
        CheckedTerm::Aggregate {
            members: vec![plain_term.clone()],
        },
        CheckedTerm::Aggregate {
            members: vec![producer_term.clone()],
        }
    );
    assert_eq!(
        operation(plain_term).mode(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        operation(plain_term).law_definitions(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert_eq!(
        operation(plain_term).member_declaration(),
        Err(CheckedOperationViewError::NotAdmitted)
    );

    let mut lower_work = 0;
    let mut upper_work = CheckedPackageReadLimits::bounded().work;
    while lower_work < upper_work {
        let middle = lower_work + (upper_work - lower_work) / 2;
        let limits = CheckedPackageReadLimits {
            work: middle,
            ..CheckedPackageReadLimits::bounded()
        };
        match read_with_limits(bytes(&lowered), limits) {
            ContractPackageReadResult::Admitted(_) => upper_work = middle,
            ContractPackageReadResult::Incomplete(_) => lower_work = middle + 1,
            other => panic!("valid package at work {middle}: {other:?}"),
        }
    }
    assert!(upper_work > 0);
    let exact = CheckedPackageReadLimits {
        work: upper_work,
        ..CheckedPackageReadLimits::bounded()
    };
    let ContractPackageReadResult::Admitted(exact_package) =
        read_with_limits(bytes(&lowered), exact)
    else {
        panic!("exact work boundary must admit");
    };
    assert_eq!(exact_package.term(&key), Some(source_term));
    assert_eq!(
        operation(exact_package.term(&key).unwrap()).mode(),
        Err(CheckedOperationViewError::NotAdmitted)
    );
    assert!(matches!(
        read_with_limits(
            bytes(&lowered),
            CheckedPackageReadLimits {
                work: upper_work - 1,
                ..exact
            }
        ),
        ContractPackageReadResult::Incomplete(_)
    ));

    let authenticated = ContractPackageWireV1::read_with_source(
        bytes(&lowered),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &source,
    )
    .expect("source correspondence");
    let ContractPackageReadResult::Admitted(authenticated) = authenticated else {
        panic!("authenticated canonical V1 read");
    };
    assert_eq!(authenticated.wire(), plain.wire());
    assert_eq!(authenticated.term(&key), Some(source_term));
    assert_eq!(
        operation(authenticated.term(&key).unwrap()).mode(),
        Ok(None)
    );
    assert_eq!(
        operation(authenticated.term(&key).unwrap()).law_definitions(),
        Ok(&[][..])
    );
}

#[trace("TC-047", "FR-035-AC-23", "FR-035-AC-24", "FR-035-AC-26")]
#[test]
fn v1_handoff_preserves_populated_mode_laws_and_member_declaration() {
    for identity in [
        "quire.op.ieee.from_exact",
        "quire.op.quantity.convert",
        "quire.op.numeric.convert_rounding",
    ] {
        let fixture = crate::checked_package_v2_catalogue_differential::admitted_fixture(identity)
            .expect("catalog fixture");
        let source = admit(&fixture);
        let key = source
            .graph()
            .nodes
            .iter()
            .find_map(|node| match &node.body {
                CheckedTerm::Application { operation, .. }
                    if operation.identity() == Some(identity) =>
                {
                    Some(node.node_id.clone())
                }
                _ => None,
            })
            .expect("selected operation");
        let mut profile = profile();
        profile.require_bounds = false;
        let lowered = source.lower(std::slice::from_ref(&key), &profile);
        let producer = lowered.package.wire().expect("lowered wire");
        let ContractPackageReadResult::Admitted(authenticated) =
            ContractPackageWireV1::read_with_source(
                bytes(&lowered),
                CheckedPackageReadLimits::bounded(),
                &ContractDependencies::new(),
                &source,
            )
            .expect("corresponding source")
        else {
            panic!("canonical package must admit");
        };
        let source_term = source.term(&key).expect("source term");
        let produced_term = producer.term(&key).expect("producer term");
        let read_term = authenticated.term(&key).expect("authenticated term");
        assert_eq!(produced_term, source_term);
        assert_eq!(read_term, source_term);
        assert_eq!(operation(read_term).mode(), operation(source_term).mode());
        assert_eq!(
            operation(read_term).law_definitions(),
            operation(source_term).law_definitions()
        );
        assert_eq!(
            operation(read_term).member_declaration(),
            operation(source_term).member_declaration()
        );
        assert_eq!(authenticated.wire(), producer.wire());
    }
}

#[trace("TC-047", "FR-035-AC-23", "FR-035-AC-26")]
#[test]
fn v1_handoff_keeps_operation_views_after_source_node_reordering() {
    let mut fixture = checked_package::positive_operation_identities();
    fixture["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("source nodes")
        .reverse();
    settle(&mut fixture);
    let source = admit(&fixture);
    let key = source
        .graph()
        .nodes
        .iter()
        .find(|node| matches!(&node.body, CheckedTerm::Application { .. }))
        .expect("source operation")
        .node_id
        .clone();
    let lowered = source.lower(std::slice::from_ref(&key), &profile());
    let producer = lowered.package.wire().expect("lowered wire");
    let ContractPackageReadResult::Admitted(authenticated) =
        ContractPackageWireV1::read_with_source(
            bytes(&lowered),
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &source,
        )
        .expect("corresponding source")
    else {
        panic!("canonical package must admit");
    };
    assert_eq!(authenticated.term(&key), producer.term(&key));
    assert_eq!(authenticated.term(&key), source.term(&key));
    assert_eq!(
        operation(authenticated.term(&key).expect("keyed operation")).mode(),
        operation(source.term(&key).expect("source operation")).mode()
    );
}

#[trace("TC-047", "FR-035-AC-26")]
#[test]
fn v1_source_handoff_is_atomic_and_preserves_plain_refusal_precedence() {
    let source = admit(&checked_package::positive_operation_identities());
    let key = source
        .graph()
        .nodes
        .iter()
        .find(|node| matches!(&node.body, CheckedTerm::Application { .. }))
        .expect("operation node")
        .node_id
        .clone();
    let lowered = source.lower(std::slice::from_ref(&key), &profile());
    let mut different_name = mixed_fixture();
    different_name["name"] = json!("test/different");
    refresh_identity(&mut different_name);
    let wrong_source = admit(&different_name);
    assert_eq!(
        ContractPackageWireV1::read_with_source(
            bytes(&lowered),
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &wrong_source,
        ),
        Err(ContractSourceHandoffError::SourceIdentity)
    );

    let mut wrong_lock: Value = serde_json::from_slice(bytes(&lowered)).expect("wire json");
    wrong_lock["package"]["source"]["lock"] =
        serde_json::to_value(wrong_source.lock()).expect("lock wire");
    let wrong_lock = canonical(&wrong_lock);
    assert!(matches!(
        read(&wrong_lock),
        ContractPackageReadResult::Admitted(_)
    ));
    assert_eq!(
        ContractPackageWireV1::read_with_source(
            &wrong_lock,
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &source,
        ),
        Err(ContractSourceHandoffError::SourceLock)
    );

    let mut unrelated: Value = serde_json::from_slice(bytes(&lowered)).expect("wire json");
    unrelated["package"]["source"]["name"] = json!(wrong_source.name());
    unrelated["package"]["source"]["lock"] =
        serde_json::to_value(wrong_source.lock()).expect("source lock wire");
    let unrelated = canonical(&unrelated);
    assert!(matches!(
        read(&unrelated),
        ContractPackageReadResult::Admitted(_)
    ));
    assert!(matches!(
        ContractPackageWireV1::read_with_source(
            &unrelated,
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &wrong_source,
        ),
        Err(ContractSourceHandoffError::MissingNode(_))
    ));

    let original_operation = admit(
        &crate::checked_package_v2_catalogue_differential::admitted_fixture(
            "quire.op.numeric.convert_rounding",
        )
        .expect("rounding operation source"),
    );
    let changed_operation = admit(
        &crate::checked_package_v2_catalogue_differential::admitted_fixture(
            "quire.op.ieee.from_exact",
        )
        .expect("different admitted operation source"),
    );
    let changed_key = changed_operation
        .graph()
        .nodes
        .iter()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() == Some("quire.op.ieee.from_exact") =>
            {
                Some(node.node_id.clone())
            }
            _ => None,
        })
        .expect("changed operation key");
    assert!(original_operation.term(&changed_key).is_none());
    let mut no_bounds = profile();
    no_bounds.require_bounds = false;
    let changed_lowered = changed_operation.lower(std::slice::from_ref(&changed_key), &no_bounds);
    let mut changed: Value =
        serde_json::from_slice(bytes(&changed_lowered)).expect("changed operation V1 wire");
    changed["package"]["source"]["name"] = json!(original_operation.name());
    changed["package"]["source"]["lock"] =
        serde_json::to_value(original_operation.lock()).expect("original source lock");
    let changed = canonical(&changed);
    assert!(matches!(
        read(&changed),
        ContractPackageReadResult::Admitted(_)
    ));
    assert!(matches!(
        ContractPackageWireV1::read_with_source(
            &changed,
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &original_operation,
        ),
        Err(ContractSourceHandoffError::MissingNode(_))
    ));

    let original_reference = admit(&mixed_fixture());
    let mut changed_reference_wire = mixed_fixture();
    let reference = changed_reference_wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("source nodes")
        .iter_mut()
        .find(|node| node["body"]["term"] == "reference")
        .expect("reference source node");
    reference["body"]["target"] = checked_package::node_id(&checked_package::family_key("aaaa"));
    settle(&mut changed_reference_wire);
    let changed_reference = admit(&changed_reference_wire);
    let changed_key = changed_reference
        .graph()
        .nodes
        .iter()
        .find(|node| matches!(&node.body, CheckedTerm::Reference { .. }))
        .expect("changed reference node")
        .node_id
        .clone();
    assert_ne!(
        original_reference.term(&changed_key),
        changed_reference.term(&changed_key),
        "the same structural node key must not authenticate a changed body"
    );
    let changed_lowered = changed_reference.lower(std::slice::from_ref(&changed_key), &profile());
    let mut changed: Value =
        serde_json::from_slice(bytes(&changed_lowered)).expect("changed reference V1 wire");
    changed["package"]["source"]["name"] = json!(original_reference.name());
    changed["package"]["source"]["lock"] =
        serde_json::to_value(original_reference.lock()).expect("original source lock");
    let changed = canonical(&changed);
    assert!(matches!(
        read(&changed),
        ContractPackageReadResult::Admitted(_)
    ));
    assert!(matches!(
        ContractPackageWireV1::read_with_source(
            &changed,
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &original_reference,
        ),
        Err(ContractSourceHandoffError::NodeMismatch(key)) if key == changed_key
    ));
}

#[trace("TC-047", "FR-035-AC-22")]
#[test]
fn lowered_package_retains_typed_terms_by_admitted_node_key() {
    let source = admit(&mixed_fixture());
    let output = source.lower(&request(), &profile());
    let producer = output.package.wire().expect("complete package");
    let ContractPackageReadResult::Admitted(reader) = read(bytes(&output)) else {
        panic!("canonical package must admit");
    };
    assert_eq!(reader.wire(), producer.wire());

    let mut typed_terms = 0;
    for node in &producer.wire().semantic_graph.nodes {
        let key = &node.node_id;
        let source_term = source.term(key);
        assert_eq!(producer.term(key), source_term);
        assert_eq!(reader.term(key), source_term);
        assert_eq!(producer.node(key), Some(node));
        assert_eq!(reader.node(key), Some(node));
        if let Some(term) = source_term {
            typed_terms += 1;
            assert_eq!(serde_json::to_value(term).expect("term json"), node.body);
        }
    }
    assert!(typed_terms > 0, "fixture must exercise typed terms");
    for unrepresented in [id("1010"), id("7070")] {
        assert!(source
            .graph()
            .nodes
            .iter()
            .any(|node| node.node_id == unrepresented));
        assert_eq!(producer.term(&unrepresented), None);
        assert_eq!(reader.term(&unrepresented), None);
        assert_eq!(producer.node(&unrepresented), None);
        assert_eq!(reader.node(&unrepresented), None);
    }
    assert_eq!(producer.term(&missing()), None);
    assert_eq!(reader.term(&missing()), None);
    assert_eq!(producer.node(&missing()), None);
    assert_eq!(reader.node(&missing()), None);
}

#[trace("TC-047", "FR-035-AC-31")]
#[test]
fn reader_refuses_each_missing_required_top_level_member() {
    let output = lower(&mixed_fixture(), &request());
    let wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    for member in [
        "contract_version",
        "package",
        "semantic_graph",
        "source_keys",
        "source_map",
        "feature_accounting",
    ] {
        let mut changed = wire.clone();
        changed.as_object_mut().expect("object").remove(member);
        let ContractPackageReadResult::Refused(refusal) = read(&canonical(&changed)) else {
            panic!("missing {member} must refuse");
        };
        assert_eq!(refusal.code, Code::InvalidPackage, "missing {member}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some(""),
            "missing {member}"
        );
        assert_eq!(
            refusal.cause,
            Some(Cause::MissingMember),
            "missing {member}"
        );
    }
}

/// FR-035-AC-33 / QSpec FR-330-AC-15: the two index tables and the
/// source-key references have a fixed refusal order before semantic joins.
#[trace("TC-047", "FR-035-AC-33")]
#[test]
fn reader_checks_output_then_source_indexes_then_source_references() {
    let output = lower(&mixed_fixture(), &request());
    let wire: Value = serde_json::from_slice(bytes(&output)).expect("six-member wire");
    assert!(matches!(
        read(&canonical(&wire)),
        ContractPackageReadResult::Admitted(_)
    ));
    let graph_len = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph")
        .len();
    let source_len = wire["source_keys"].as_array().expect("source keys").len();
    assert!(graph_len >= 2 && source_len >= 2);

    let mut output_bad = wire.clone();
    output_bad["semantic_graph"]["nodes"][0]["node_id"]["path"][0] = json!({"node": graph_len});
    let mut source_bad = wire.clone();
    source_bad["source_keys"][0]["path"][0] = json!({"node": source_len});
    let mut reference_bad = wire.clone();
    reference_bad["feature_accounting"][0]["node_id"]["source_key"] = json!(source_len);
    let mut paired_output_source = output_bad.clone();
    paired_output_source["source_keys"][0]["path"][0] = json!({"node": source_len});
    let mut paired_source_reference = source_bad.clone();
    paired_source_reference["feature_accounting"][0]["node_id"]["source_key"] = json!(source_len);
    let mut output_map_bad = wire.clone();
    output_map_bad["source_map"]["nodes"][0]["node_id"]["path"][0] = json!({"node": graph_len});
    let cases = [
        (output_bad, "/semantic_graph/nodes/0/node_id/path/0/node"),
        (source_bad, "/source_keys/0/path/0/node"),
        (reference_bad, "/feature_accounting/0/node_id/source_key"),
        (
            paired_output_source,
            "/semantic_graph/nodes/0/node_id/path/0/node",
        ),
        (paired_source_reference, "/source_keys/0/path/0/node"),
        (output_map_bad, "/source_map/nodes/0/node_id/path/0/node"),
    ];
    for (changed, path) in cases {
        let ContractPackageReadResult::Refused(refusal) = read(&canonical(&changed)) else {
            panic!("indexed defect at {path} must refuse");
        };
        assert_eq!(refusal.code, Code::MalformedWire, "{path}");
        assert_eq!(refusal.cause, None, "{path}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|at| at.as_str()),
            Some(path)
        );
    }

    let mut output_cycle = wire.clone();
    output_cycle["semantic_graph"]["nodes"][0]["node_id"]["path"][0] = json!({"node": 0});
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&output_cycle)) else {
        panic!("output self-cycle must refuse");
    };
    assert_eq!(refusal.code, Code::MalformedWire);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|at| at.as_str()),
        Some("/semantic_graph/nodes/0/node_id/path/0/node")
    );
    let mut out_of_range_before_cycle = output_cycle;
    out_of_range_before_cycle["semantic_graph"]["nodes"][1]["node_id"]["path"][0] =
        json!({"node": graph_len});
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&out_of_range_before_cycle))
    else {
        panic!("out-of-range output index must precede output cycle");
    };
    assert_eq!(refusal.code, Code::MalformedWire);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|at| at.as_str()),
        Some("/semantic_graph/nodes/1/node_id/path/0/node")
    );

    let mut self_cycle = wire.clone();
    self_cycle["source_keys"][0]["path"][0] = json!({"node": 0});
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&self_cycle)) else {
        panic!("source-key self-cycle must refuse");
    };
    assert_eq!(refusal.code, Code::MalformedWire);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|at| at.as_str()),
        Some("/source_keys/0/path/0/node")
    );

    let mut two_cycle = wire.clone();
    two_cycle["source_keys"][0]["path"][0] = json!({"node": 1});
    two_cycle["source_keys"][1]["path"][0] = json!({"node": 0});
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&two_cycle)) else {
        panic!("two-key source cycle must refuse");
    };
    assert_eq!(refusal.code, Code::MalformedWire);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|at| at.as_str()),
        Some("/source_keys/0/path/0/node")
    );

    let mut duplicate = wire;
    let first = duplicate["source_keys"][0].clone();
    duplicate["source_keys"]
        .as_array_mut()
        .expect("table")
        .push(first);
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&duplicate)) else {
        panic!("duplicate structural source key must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue))
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|at| at.as_str()),
        Some(format!("/source_keys/{source_len}").as_str())
    );
}

/// An admitted source graph and the V1 source table may use different numeric
/// spellings for the same structural operand. The query names that structure.
#[trace("TC-047", "FR-035-AC-35")]
#[test]
fn indexed_reached_closure_uses_a_typed_source_structural_key() {
    fn replace(value: &mut Value, before: &Value, after: &Value) {
        if value == before {
            *value = after.clone();
            return;
        }
        match value {
            Value::Array(items) => {
                for item in items {
                    replace(item, before, after);
                }
            }
            Value::Object(members) => {
                for item in members.values_mut() {
                    replace(item, before, after);
                }
            }
            _ => {}
        }
    }

    fn remap_output_indexes(value: &mut Value, first: usize, second: usize) {
        match value {
            Value::Object(members)
                if members.len() == 2
                    && members.contains_key("package")
                    && members.contains_key("node") => {}
            Value::Object(members)
                if members.len() == 3
                    && members.contains_key("package")
                    && members.contains_key("path")
                    && members.contains_key("ordinal") =>
            {
                for segment in members["path"].as_array_mut().expect("output id path") {
                    if let Some(index) = segment
                        .as_object_mut()
                        .filter(|object| object.len() == 1)
                        .and_then(|object| object.get_mut("node"))
                    {
                        let old = usize::try_from(index.as_u64().expect("output index"))
                            .expect("fixture output index fits usize");
                        if old == first {
                            *index = json!(second);
                        } else if old == second {
                            *index = json!(first);
                        }
                    }
                }
            }
            Value::Object(members) => {
                for child in members.values_mut() {
                    remap_output_indexes(child, first, second);
                }
            }
            Value::Array(items) => {
                for child in items {
                    remap_output_indexes(child, first, second);
                }
            }
            _ => {}
        }
    }

    let mut input = mixed_fixture();
    let before = checked_package::node_id(&checked_package::family_key("dddd"));
    let operand = checked_package::node_id(&checked_package::family_key("a3a3"));
    let after = json!({
        "package": before["package"],
        "path": [before["path"][0].clone(), operand],
        "ordinal": before["ordinal"]
    });
    replace(&mut input, &before, &after);
    settle(&mut input);
    checked_package::index_fixture_node_ids(&mut input);
    settle(&mut input);

    let source = admit(&input);
    let position = source
        .graph()
        .nodes
        .iter()
        .position(|node| {
            node.node_id.path.iter().any(|segment| {
                matches!(
                    segment,
                    quire_contract_model::CheckedNodePathSegment::Index(_)
                )
            })
        })
        .expect("indexed request present");
    let root = source.graph().nodes[position].node_id.clone();
    let structural = source
        .structural_key_at(position)
        .expect("admitted structural key");
    let key = source_key(structural);
    assert!(ContractSourceKeyV1::from_structural(&root).is_none());

    let output = source.lower(std::slice::from_ref(&root), &profile());
    let producer = output.package.wire().expect("producer V1");
    let ContractPackageReadResult::Admitted(reader) = read(bytes(&output)) else {
        panic!("indexed V1 readback must admit");
    };
    let accounted = reader.wire().feature_accounting[0].node_id();
    assert_ne!(
        &root, accounted,
        "source graph and source table use distinct indexes"
    );
    assert!(ContractSourceKeyV1::from_structural(accounted).is_none());
    let table_index = reader
        .wire()
        .source_keys
        .iter()
        .position(|entry| entry == accounted)
        .expect("accounting key in source table");
    assert_eq!(reader.source_key_at(table_index), Some(key.clone()));
    let ContractRootViewV1::Lowered(produced) =
        producer.reached_closure(&key).expect("producer view")
    else {
        panic!("indexed request must lower");
    };
    let ContractRootViewV1::Lowered(read_back) = reader.reached_closure(&key).expect("read view")
    else {
        panic!("indexed readback request must remain lowered");
    };
    assert_eq!(read_back, produced);
    let first = read_back
        .root
        .node_id
        .path
        .iter()
        .find_map(|segment| match segment {
            quire_contract_model::CheckedNodePathSegment::Index(index) => {
                usize::try_from(*index).ok()
            }
            _ => None,
        })
        .expect("projected root has indexed operand");
    let graph_len = reader.wire().semantic_graph.nodes.len();
    assert!(graph_len >= 2 && first < graph_len);
    let second = if first == 0 { 1 } else { 0 };
    let mut reordered: Value = serde_json::from_slice(bytes(&output)).expect("V1 JSON");
    reordered["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("output nodes")
        .swap(first, second);
    remap_output_indexes(&mut reordered["semantic_graph"], first, second);
    let mapped = reordered["source_map"]["nodes"]
        .as_array_mut()
        .expect("source-map nodes");
    assert_eq!(mapped.len(), graph_len);
    mapped.swap(first, second);
    for entry in mapped {
        remap_output_indexes(&mut entry["node_id"], first, second);
    }
    let ContractPackageReadResult::Admitted(reordered) = read(&canonical(&reordered)) else {
        panic!("rewritten producer-chosen output graph must admit");
    };
    let ContractRootViewV1::Lowered(reordered_view) =
        reordered.reached_closure(&key).expect("reordered view")
    else {
        panic!("reordered indexed root must remain lowered");
    };
    assert_ne!(reordered_view.root.node_id, read_back.root.node_id);
    assert_eq!(
        reordered_view.root_structural_key,
        read_back.root_structural_key
    );
    assert_eq!(
        reordered_view
            .nodes
            .iter()
            .map(|entry| &entry.structural_key)
            .collect::<Vec<_>>(),
        read_back
            .nodes
            .iter()
            .map(|entry| &entry.structural_key)
            .collect::<Vec<_>>()
    );
    assert_eq!(reordered_view.bounds, read_back.bounds);
    assert_eq!(reordered_view.claims, read_back.claims);
    assert_eq!(
        reordered_view
            .nodes
            .iter()
            .map(|entry| &entry.sources)
            .collect::<Vec<_>>(),
        read_back
            .nodes
            .iter()
            .map(|entry| &entry.sources)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        reordered_view.selected_references,
        read_back.selected_references
    );
    let unrequested = source
        .graph()
        .nodes
        .iter()
        .enumerate()
        .find(|(candidate, _)| *candidate != position)
        .and_then(|(candidate, _)| source.structural_key_at(candidate))
        .expect("second admitted source node");
    assert!(matches!(
        reader
            .reached_closure(&source_key(unrequested))
            .expect("unrequested source view"),
        ContractRootViewV1::Absent
    ));
    assert!(matches!(
        reader
            .reached_closure(&source_key(&missing()))
            .expect("absent view"),
        ContractRootViewV1::Absent
    ));

    let mut unsupported = profile();
    unsupported.supported_tags.clear();
    let refused = source.lower(std::slice::from_ref(&root), &unsupported);
    let ContractPackageReadResult::Admitted(refused_package) = read(bytes(&refused)) else {
        panic!("non-lowered indexed request must read back");
    };
    assert!(matches!(
        refused_package
            .reached_closure(&key)
            .expect("non-lowered indexed view"),
        ContractRootViewV1::NonLowered(record)
            if record.disposition() == ContractDispositionV1::Unsupported
    ));
}

#[trace("TC-047", "FR-035-AC-33")]
#[test]
fn reader_refuses_closed_source_reference_shapes_before_index_checks() {
    let output = lower(&mixed_fixture(), &request());
    let wire: Value = serde_json::from_slice(bytes(&output)).expect("six-member wire");
    let cases = [
        (
            json!({"source_key": 0, "extra": true}),
            Cause::UnknownMember,
            "/feature_accounting/0/node_id",
        ),
        (
            json!({}),
            Cause::MissingMember,
            "/feature_accounting/0/node_id",
        ),
        (
            json!({"source_key": "0"}),
            Cause::WrongValueKind,
            "/feature_accounting/0/node_id/source_key",
        ),
        (
            json!({"source_key": -1}),
            Cause::InvalidValue,
            "/feature_accounting/0/node_id/source_key",
        ),
    ];
    for (reference, cause, path) in cases {
        let mut changed = wire.clone();
        changed["feature_accounting"][0]["node_id"] = reference;
        changed["semantic_graph"]["nodes"][0]["node_id"]["path"][0] = json!({"node": 999});
        let ContractPackageReadResult::Refused(refusal) = read(&canonical(&changed)) else {
            panic!("closed source-reference shape must refuse at {path}");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(cause))
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|at| at.as_str()),
            Some(path)
        );
    }
}

#[trace("TC-047", "FR-035-AC-13")]
#[test]
fn reader_refuses_each_leading_json_whitespace_byte() {
    let output = lower(&mixed_fixture(), &request());
    for whitespace in *b" \t\n\r" {
        let mut changed = vec![whitespace];
        changed.extend_from_slice(bytes(&output));
        let ContractPackageReadResult::Refused(refusal) = read(&changed) else {
            panic!("leading {whitespace:?} must refuse");
        };
        assert_eq!(
            refusal.code,
            Code::NoncanonicalWire,
            "leading {whitespace:?}"
        );
        assert_eq!(refusal.loci[0].path.clone(), None, "leading {whitespace:?}");
        assert_eq!(refusal.cause, None, "leading {whitespace:?}");
    }
}

#[trace("TC-047", "FR-035-AC-14")]
#[test]
fn reader_refuses_first_accounting_record_for_later_requested_items() {
    let output = lower(&mixed_fixture(), &request());
    let wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    for index in 1..=3 {
        let mut changed = wire.clone();
        changed["feature_accounting"][0]["node_id"] =
            wire["source_map"]["requested_items"][index]["requested_item"].clone();
        let ContractPackageReadResult::Refused(refusal) = read(&canonical(&changed)) else {
            panic!("accounting record for requested item {index} must refuse");
        };
        assert_eq!(refusal.code, Code::InvalidPackage, "requested item {index}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some("/feature_accounting"),
            "requested item {index}"
        );
        assert_eq!(
            refusal.cause,
            Some(Cause::InvalidValue),
            "requested item {index}"
        );
    }

    // Both roots are lowered and represented. This isolates the ordered
    // correspondence check from the lowered-node membership check.
    let two_roots = lower(&mixed_fixture(), &[id("cccc"), id("dddd")]);
    let mut changed: Value = serde_json::from_slice(bytes(&two_roots)).expect("wire json");
    assert_eq!(changed["feature_accounting"][0]["disposition"], "lowered");
    assert_eq!(changed["feature_accounting"][1]["disposition"], "lowered");
    let other_root = changed["source_map"]["requested_items"][1]["requested_item"].clone();
    let other_source_index = other_root["source_key"].as_u64().expect("source reference") as usize;
    assert!(changed["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .any(|node| node["node_id"] == changed["source_keys"][other_source_index]));
    changed["feature_accounting"][0]["node_id"] = other_root;
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&changed)) else {
        panic!("first accounting record for another represented root must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some("/feature_accounting")
    );
    assert_eq!(refusal.cause, Some(Cause::InvalidValue));
}

#[trace("TC-047", "FR-035-AC-10")]
#[test]
fn reader_refuses_wrong_string_version_before_shape_identity_and_nodes() {
    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    let wrong = "quire.contract-package/v9";
    wire["contract_version"] = json!(wrong);
    let canonical_wrong = canonical(&wire);
    let ContractPackageReadResult::Refused(refusal) = read(&canonical_wrong) else {
        panic!("wrong string version must refuse");
    };
    assert_eq!(refusal.code, Code::UnknownContractVersion);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some("/contract_version")
    );
    assert_eq!(refusal.contract_version.as_deref(), Some(wrong));
    assert_eq!(refusal.cause, None);
    assert_eq!(refusal.locus, None);

    // Canonical shape, identity, and node defects cannot mask the version refusal.
    let mut no_source_map = wire.clone();
    no_source_map
        .as_object_mut()
        .expect("object")
        .remove("source_map");
    let mut extra_member = wire.clone();
    extra_member["extra"] = json!(0);
    let mut no_nodes = wire.clone();
    no_nodes["semantic_graph"]["nodes"] = json!(0);
    for defective in [no_source_map, extra_member, no_nodes] {
        let ContractPackageReadResult::Refused(refusal) = read(&canonical(&defective)) else {
            panic!("version must win over shape and node checks");
        };
        assert_eq!(refusal.code, Code::UnknownContractVersion);
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some("/contract_version")
        );
        assert_eq!(refusal.contract_version.as_deref(), Some(wrong));
        assert_eq!(refusal.cause, None);
        assert_eq!(refusal.locus, None);
    }

    let mut numeric: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    numeric["contract_version"] = json!(4);
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&numeric)) else {
        panic!("numeric version must refuse as wrong value kind");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::WrongValueKind));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some("/contract_version")
    );
    assert_eq!(refusal.contract_version, None);
}

#[trace("TC-047", "FR-035-AC-11")]
#[test]
fn reader_incomplete_reports_each_exhausted_meter_and_pointer() {
    let output = lower(&mixed_fixture(), &request());
    let wire = bytes(&output);
    let size = wire.len() as u64;
    assert!(matches!(read(wire), ContractPackageReadResult::Admitted(_)));
    let cases = [
        (CheckedPackageLimit::Bytes, 100, size, None),
        (
            CheckedPackageLimit::Nodes,
            1,
            3,
            Some("/semantic_graph/nodes"),
        ),
        // Each node has at most one dependency here, so this case cannot distinguish
        // the actual consumed count from limit + 1. The two-dependency case below does.
        (
            CheckedPackageLimit::Edges,
            1,
            2,
            Some("/semantic_graph/nodes/2/dependencies"),
        ),
        (
            CheckedPackageLimit::Occurrences,
            0,
            1,
            Some("/source_map/nodes/0/occurrences/0"),
        ),
        (
            CheckedPackageLimit::Work,
            2,
            6,
            Some("/semantic_graph/nodes/1/body"),
        ),
    ];
    for (kind, limit, consumed, path) in cases {
        let mut limits = CheckedPackageReadLimits::bounded();
        match kind {
            CheckedPackageLimit::Bytes => limits.bytes = limit,
            CheckedPackageLimit::Nodes => limits.nodes = limit,
            CheckedPackageLimit::Edges => limits.edges = limit,
            CheckedPackageLimit::Occurrences => limits.occurrences = limit,
            CheckedPackageLimit::Work => limits.work = limit,
            CheckedPackageLimit::Diagnostics => unreachable!(),
        }
        let ContractPackageReadResult::Incomplete(incomplete) = read_with_limits(wire, limits)
        else {
            panic!("{kind:?} limit must return Incomplete");
        };
        assert_eq!(incomplete.limit_kind, kind);
        assert_eq!(incomplete.limit, limit);
        assert_eq!(incomplete.consumed, consumed, "{kind:?}");
        assert_eq!(
            incomplete.path.as_ref().map(|path| path.as_str()),
            path,
            "{kind:?}"
        );
    }

    let two_edge_output = lower(&mixed_fixture_with_two_dependencies(), &request());
    let two_edge_nodes = &two_edge_output
        .package
        .wire()
        .expect("two-dependency package")
        .wire()
        .semantic_graph
        .nodes;
    let mut preceding_edges = 0_u64;
    let (two_edge_position, two_edge_count) = two_edge_nodes
        .iter()
        .enumerate()
        .find_map(|(position, node)| {
            let count =
                u64::try_from(node.dependencies.len()).expect("fixture edge count fits u64");
            if count >= 2 {
                Some((position, count))
            } else {
                preceding_edges += count;
                None
            }
        })
        .expect("one represented node has two dependencies");
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.edges = preceding_edges;
    let ContractPackageReadResult::Incomplete(incomplete) =
        read_with_limits(bytes(&two_edge_output), limits)
    else {
        panic!("two dependencies on one node must exhaust the edge limit");
    };
    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Edges);
    assert_eq!(incomplete.limit, preceding_edges);
    assert_eq!(incomplete.consumed, preceding_edges + two_edge_count);
    let expected_path = format!("/semantic_graph/nodes/{two_edge_position}/dependencies");
    assert_eq!(
        incomplete.path.as_ref().map(|path| path.as_str()),
        Some(expected_path.as_str())
    );

    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = 7;
    let ContractPackageReadResult::Incomplete(incomplete) = read_with_limits(wire, limits) else {
        panic!("reference validation must exhaust the work limit");
    };
    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
    assert_eq!(incomplete.limit, 7);
    assert_eq!(incomplete.consumed, 8);
    assert_eq!(
        incomplete.path.as_ref().map(|path| path.as_str()),
        Some("/semantic_graph/nodes/1/semantic_type")
    );
}

#[trace("TC-047", "FR-035-AC-15")]
#[test]
fn reader_uses_one_work_ceiling_across_value_and_reference_validation() {
    let output = lower(&mixed_fixture(), &request());
    let wire = bytes(&output);
    // Independent body count: node 0 has one unit, node 1 five, node 2 one.
    // Reference validation then visits six targets, for a total of thirteen.
    for (limit, consumed, path) in [
        (2, 6, "/semantic_graph/nodes/1/body"),
        (6, 7, "/semantic_graph/nodes/2/body"),
        (7, 8, "/semantic_graph/nodes/1/semantic_type"),
        (10, 11, "/semantic_graph/nodes/1/body/members/1/value/type"),
        (12, 13, "/semantic_graph/nodes/2/dependencies/0"),
    ] {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = limit;
        let ContractPackageReadResult::Incomplete(incomplete) = read_with_limits(wire, limits)
        else {
            panic!("work limit {limit} must stop validation");
        };
        assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
        assert_eq!(incomplete.limit, limit);
        assert_eq!(incomplete.consumed, consumed);
        assert_eq!(
            incomplete.path.as_ref().map(|path| path.as_str()),
            Some(path)
        );
    }
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = 13;
    assert!(matches!(
        read_with_limits(wire, limits),
        ContractPackageReadResult::Admitted(_)
    ));
}

#[trace("TC-047", "FR-035-AC-16")]
#[test]
fn reader_charges_each_occurrence_and_region_at_its_own_pointer() {
    let output = lower(&mixed_fixture(), &request());
    let wire = bytes(&output);
    let source: Value = serde_json::from_slice(wire).expect("canonical package");
    let preceding = source["source_map"]["nodes"]
        .as_array()
        .expect("mapped nodes")
        .iter()
        .flat_map(|node| node["occurrences"].as_array().expect("occurrences"))
        .map(|entry| 1 + entry["regions"].as_array().expect("regions").len())
        .sum::<usize>();
    let preceding = u64::try_from(preceding).expect("fixture charge count fits u64");
    for (limit, consumed, path) in [
        (0, 1, "/source_map/nodes/0/occurrences/0"),
        (
            preceding,
            preceding + 1,
            "/source_map/requested_items/0/occurrences/0",
        ),
        (17, 18, "/source_map/refusals/1/occurrences/0/regions/0"),
    ] {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.occurrences = limit;
        let ContractPackageReadResult::Incomplete(incomplete) = read_with_limits(wire, limits)
        else {
            panic!("occurrence limit {limit} must stop validation");
        };
        assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Occurrences);
        assert_eq!(incomplete.limit, limit);
        assert_eq!(incomplete.consumed, consumed);
        assert_eq!(
            incomplete.path.as_ref().map(|path| path.as_str()),
            Some(path)
        );
    }
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.occurrences = 18;
    assert!(matches!(
        read_with_limits(wire, limits),
        ContractPackageReadResult::Admitted(_)
    ));
}

fn body_keys(value: &Value, keys: &mut Vec<CheckedNodeId>) {
    match value {
        Value::Object(object) => {
            if object.get("term").and_then(Value::as_str) == Some("reference") {
                keys.push(
                    serde_json::from_value(object["target"].clone()).expect("reference target"),
                );
            }
            for child in object.values() {
                body_keys(child, keys);
            }
        }
        Value::Array(array) => {
            for child in array {
                body_keys(child, keys);
            }
        }
        _ => {}
    }
}

fn independent_closure(
    admitted: &CheckedPackageV2,
    root: &CheckedNodeId,
) -> BTreeSet<CheckedNodeId> {
    let by_key = admitted
        .graph()
        .nodes
        .iter()
        .map(|node| (&node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    let mut reached = BTreeSet::new();
    let mut pending = VecDeque::from([root.clone()]);
    while let Some(key) = pending.pop_front() {
        if !reached.insert(key.clone()) {
            continue;
        }
        let node = by_key.get(&key).expect("admitted edge target");
        pending.push_back(node.semantic_type.clone());
        pending.extend(node.dependencies.iter().cloned());
        let mut body = Vec::new();
        body_keys(
            &serde_json::to_value(&node.body).expect("typed body"),
            &mut body,
        );
        pending.extend(body);
    }
    reached
}

#[trace("TC-047", "FR-035-AC-6", "FR-035-AC-7", "FR-035-AC-8")]
#[test]
fn reached_view_uses_all_three_edges_and_preserves_sources_and_wire() {
    let mut input = mixed_fixture();
    let expression = input["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "reference")
        .expect("reference expression");
    // The body target is deliberately absent from the declared dependencies.
    expression["dependencies"] = json!([
        checked_package::node_id(&checked_package::family_key("cccc")),
        checked_package::node_id(&checked_package::family_key("6060")),
    ]);
    settle(&mut input);
    let admitted = admit(&input);
    let root = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "expression" && node.semantic_form.as_ref() == "reference"
        })
        .expect("root")
        .node_id
        .clone();
    let result = admitted.lower(&[root.clone(), id("7070"), missing()], &profile());
    let package = result.package.wire().expect("wire");
    let before = package.wire().clone();
    let ContractRootViewV1::Lowered(view) =
        package.reached_closure(&source_key(&root)).expect("view")
    else {
        panic!("lowered root must have a view");
    };
    assert_eq!(view.root.node_id, root);
    let keys = view
        .nodes
        .iter()
        .map(|entry| entry.node.node_id.clone())
        .collect::<Vec<_>>();
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        keys.iter().cloned().collect::<BTreeSet<_>>(),
        independent_closure(&admitted, &root)
    );
    for entry in &view.nodes {
        let original = admitted
            .graph()
            .nodes
            .iter()
            .find(|node| node.node_id == entry.node.node_id)
            .expect("source node");
        assert_eq!(
            serde_json::to_value(entry.node).expect("contract node"),
            serde_json::to_value(original).expect("source node")
        );
        let sources = admitted
            .source_map()
            .iter()
            .filter(|source| source.node_id == entry.node.node_id)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(entry.sources, sources);
    }
    assert!(view.selected_references.is_empty());
    assert_eq!(view.bounds, vec![id("cccc")]);
    assert_eq!(view.claims, vec![id("6060")]);
    let CompleteLoweringRecordV2::Lowered { node: lowered } = &result.records[0] else {
        panic!("root must lower");
    };
    assert_eq!(view.bounds, lowered.bounds);
    assert_eq!(view.claims, lowered.claims);
    assert!(
        matches!(package.reached_closure(&source_key(&id("7070"))).expect("refused"), ContractRootViewV1::NonLowered(record) if record.disposition() == ContractDispositionV1::Unsupported)
    );
    assert!(
        matches!(package.reached_closure(&source_key(&missing())).expect("missing"), ContractRootViewV1::NonLowered(record) if record.disposition() == ContractDispositionV1::InvalidInput)
    );
    assert!(matches!(
        package
            .reached_closure(&source_key(&id("dddd")))
            .expect("reached only"),
        ContractRootViewV1::Absent
    ));
    assert_eq!(package.wire(), &before);
}

#[trace("TC-047", "FR-035-AC-7", "FR-035-AC-8")]
#[test]
fn reached_view_exposes_empty_subsets_and_absent_keys() {
    let admitted = admit(&v2_all_families());
    let root = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_tag.as_ref() == "value" && node.semantic_form.as_ref() == "literal")
        .expect("literal")
        .node_id
        .clone();
    let result = admitted.lower(std::slice::from_ref(&root), &profile());
    let package = result.package.wire().expect("wire");
    let ContractRootViewV1::Lowered(view) =
        package.reached_closure(&source_key(&root)).expect("view")
    else {
        panic!("lowered root must have a view");
    };
    assert!(view.bounds.is_empty());
    assert!(view.claims.is_empty());
    let CompleteLoweringRecordV2::Lowered { node: lowered } = &result.records[0] else {
        panic!("root must lower");
    };
    assert_eq!(view.bounds, lowered.bounds);
    assert_eq!(view.claims, lowered.claims);
    assert!(matches!(
        package
            .reached_closure(&source_key(&missing()))
            .expect("absent"),
        ContractRootViewV1::Absent
    ));
}

#[trace("TC-047", "FR-035-AC-3", "FR-035-AC-4")]
#[test]
fn one_call_emits_one_six_member_package_with_total_accounting_and_source_map() {
    let input = mixed_fixture();
    let admitted = admit(&input);
    let result = admitted.lower(&request(), &profile());
    let package = result.package.wire().expect("wire").wire();
    assert_eq!(package.contract_version.as_ref(), CONTRACT_PACKAGE_VERSION);
    assert_eq!(package.package.source.name.as_ref(), admitted.name());
    assert_eq!(package.package.source.lock, *admitted.lock());
    assert_eq!(package.package.lowering_profile, profile().reference);
    assert_eq!(package.semantic_graph.nodes.len(), 3);
    let keys = package
        .semantic_graph
        .nodes
        .iter()
        .map(|node| &node.node_id)
        .collect::<Vec<_>>();
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(package.source_map.nodes.len(), keys.len());
    assert_eq!(package.feature_accounting.len(), request().len());
    assert_eq!(package.source_map.requested_items.len(), request().len());
    assert_eq!(package.source_map.refusals.len(), 3);
    let dispositions = package
        .feature_accounting
        .iter()
        .map(|record| record.disposition())
        .collect::<Vec<_>>();
    assert_eq!(
        dispositions,
        [
            ContractDispositionV1::Lowered,
            ContractDispositionV1::RequiresBound,
            ContractDispositionV1::Unsupported,
            ContractDispositionV1::InvalidInput,
            ContractDispositionV1::Lowered
        ]
    );
    assert_eq!(
        package.source_map.requested_items[3].requested_item,
        missing()
    );
    assert!(package.source_map.requested_items[3].occurrences.is_empty());
    assert!(!keys.contains(&&id("1010")) && !keys.contains(&&id("7070")));
    assert!(matches!(
        result.records[1],
        CompleteLoweringRecordV2::RequiresBound { .. }
    ));
    let ContractPackageReadResult::Admitted(read_package) = read(bytes(&result)) else {
        panic!("emitted package must admit");
    };
    assert_eq!(read_package.wire(), package);
}

#[trace("TC-047", "FR-035-AC-19", "FR-035-AC-20", "FR-035-AC-21")]
#[test]
fn retained_typed_views_match_source_and_strict_reader_without_changing_wire() {
    let admitted = admit(&mixed_fixture());
    let result = admitted.lower(&request(), &profile());
    let producer = result.package.wire().expect("producer package");
    let ContractPackageReadResult::Admitted(reader) = read(bytes(&result)) else {
        panic!("emitted package must admit");
    };
    assert_eq!(producer.wire(), reader.wire());
    for node in &producer.wire().semantic_graph.nodes {
        let source_position = admitted
            .graph()
            .nodes
            .iter()
            .position(|source| source.node_id == node.node_id)
            .expect("source node");
        let expected = admitted.node_kinds()[source_position];
        assert_eq!(producer.node_kind(&node.node_id), Some(expected));
        assert_eq!(reader.node_kind(&node.node_id), Some(expected));
    }
    for package in [producer, reader.as_ref()] {
        let ContractRootViewV1::Lowered(view) = package
            .reached_closure(&source_key(&id("dddd")))
            .expect("closure")
        else {
            panic!("lowered root");
        };
        assert_eq!(
            view.root_kind,
            package.node_kind(&view.root.node_id).expect("root kind")
        );
        for entry in &view.nodes {
            assert_eq!(
                entry.kind,
                package
                    .node_kind(&entry.node.node_id)
                    .expect("reached kind")
            );
        }
    }
    assert_eq!(producer.node_kind(&id("1010")), None);
    assert_eq!(producer.node_kind(&id("7070")), None);
    assert_eq!(producer.node_kind(&missing()), None);
    for (index, record) in result.records.iter().enumerate() {
        let expected = match record {
            CompleteLoweringRecordV2::Unsupported { node_tag, .. } => Some(*node_tag),
            _ => None,
        };
        assert_eq!(producer.unsupported_tag_at(index), expected);
        assert_eq!(reader.unsupported_tag_at(index), expected);
    }
    assert_eq!(producer.unsupported_tag_at(result.records.len()), None);
    assert_eq!(reader.unsupported_tag_at(result.records.len()), None);
    assert_eq!(
        producer.unsupported_tag_at(2),
        Some(CheckedNodeTag::Correspondence)
    );
    assert!(!producer
        .wire()
        .semantic_graph
        .nodes
        .iter()
        .any(|node| node.node_id == id("7070")));
    assert!(matches!(
        producer.node_kind(&id("dddd")),
        Some(CheckedNodeKind::Value(_))
    ));
}

#[trace(
    "TC-047",
    "FR-035-AC-27",
    "FR-035-AC-28",
    "FR-035-AC-29",
    "FR-035-AC-30"
)]
#[test]
fn unsupported_source_kind_tracks_requested_root_across_mixed_accounting() {
    let source = crate::checked_package_v2_temporal::admitted_fairness_source_for_contract();
    let by_kind = |kind| {
        source
            .graph()
            .nodes
            .iter()
            .zip(source.node_kinds())
            .find(|(_, candidate)| **candidate == kind)
            .map(|(node, _)| node.node_id.clone())
            .expect("admitted kind")
    };
    let clause = by_kind(CheckedNodeKind::Temporal(TemporalForm::TemporalClause));
    let fairness = by_kind(CheckedNodeKind::Temporal(TemporalForm::Fairness));
    let formula = by_kind(CheckedNodeKind::Temporal(TemporalForm::Formula));
    let healthy = source
        .graph()
        .nodes
        .iter()
        .zip(source.node_kinds())
        .find(|(_, kind)| kind.tag() == CheckedNodeTag::Value)
        .map(|(node, _)| node.node_id.clone())
        .expect("healthy value");
    let requests = [
        clause.clone(),
        healthy.clone(),
        fairness.clone(),
        formula.clone(),
    ];
    let mut lowering_profile = profile();
    lowering_profile
        .supported_tags
        .remove(&CheckedNodeTag::Temporal);
    lowering_profile.require_bounds = false;
    let result = source.lower(&requests, &lowering_profile);
    let without_sibling = source.lower(
        &[clause.clone(), fairness.clone(), formula.clone()],
        &lowering_profile,
    );
    let without_sibling_package = without_sibling.package.wire().expect("comparison package");
    for (original_index, comparison_index) in [(0, 0), (2, 1), (3, 2)] {
        assert_eq!(
            result.records[original_index],
            without_sibling.records[comparison_index]
        );
        assert_eq!(
            result
                .package
                .wire()
                .expect("producer package")
                .unsupported_source_kind_at(original_index),
            without_sibling_package.unsupported_source_kind_at(comparison_index)
        );
    }
    let CompleteLoweringRecordV2::Unsupported {
        node_id,
        unsupported_node_id,
        node_tag,
    } = &result.records[0]
    else {
        panic!("clause request is unsupported");
    };
    assert_eq!(node_id, &clause);
    assert_eq!(unsupported_node_id, &fairness);
    assert_eq!(*node_tag, CheckedNodeTag::Temporal);
    assert_ne!(node_id, unsupported_node_id);
    let producer = result.package.wire().expect("producer package");
    let wire_bytes = bytes(&result);
    let ContractPackageReadResult::Admitted(plain) = read(wire_bytes) else {
        panic!("canonical plain read");
    };
    let ContractPackageReadResult::Admitted(authenticated) =
        ContractPackageWireV1::read_with_source(
            wire_bytes,
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &source,
        )
        .expect("source handoff")
    else {
        panic!("canonical authenticated read");
    };
    assert_eq!(producer.wire(), plain.wire());
    assert_eq!(producer.wire(), authenticated.wire());
    assert_eq!(
        wire_bytes,
        canonical(&serde_json::to_value(producer.wire()).expect("wire value"))
    );
    for (index, expected) in [
        Some(CheckedNodeKind::Temporal(TemporalForm::TemporalClause)),
        None,
        Some(CheckedNodeKind::Temporal(TemporalForm::Fairness)),
        Some(CheckedNodeKind::Temporal(TemporalForm::Formula)),
        None,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(producer.unsupported_source_kind_at(index), Ok(expected));
        assert_eq!(
            authenticated.unsupported_source_kind_at(index),
            Ok(expected)
        );
        assert_eq!(
            plain.unsupported_source_kind_at(index),
            if expected.is_some() {
                Err(ContractUnsupportedSourceKindError::NotAdmitted)
            } else {
                Ok(None)
            }
        );
    }
    assert!(matches!(
        result.records[1],
        CompleteLoweringRecordV2::Lowered { .. }
    ));
    assert_eq!(producer.node_kind(&clause), None);
    assert_eq!(producer.node_kind(&fairness), None);
    assert_eq!(producer.node_kind(&formula), None);
    assert!(producer.node_kind(&healthy).is_some());
    let mut represented_request: Value = serde_json::from_slice(wire_bytes).expect("wire json");
    let represented =
        represented_request["source_map"]["requested_items"][1]["requested_item"].clone();
    let healthy_sources =
        represented_request["source_map"]["requested_items"][1]["occurrences"].clone();
    represented_request["feature_accounting"][0]["node_id"] = represented.clone();
    represented_request["source_map"]["requested_items"][0]["requested_item"] = represented.clone();
    represented_request["source_map"]["requested_items"][0]["occurrences"] =
        healthy_sources.clone();
    represented_request["source_map"]["refusals"][0]["requested_item"] = represented;
    represented_request["source_map"]["refusals"][0]["occurrences"] = healthy_sources;
    let represented_request = canonical(&represented_request);
    let ContractPackageReadResult::Admitted(represented_plain) = read(&represented_request) else {
        panic!("canonical V1 with a represented unsupported request must admit");
    };
    assert!(represented_plain.node(&healthy).is_some());
    assert_eq!(
        represented_plain.unsupported_source_kind_at(0),
        Err(ContractUnsupportedSourceKindError::NotAdmitted)
    );
    let mut missing_root: Value = serde_json::from_slice(wire_bytes).expect("wire json");
    let absent_key = serde_json::to_value(missing()).expect("missing node id");
    let absent_index = missing_root["source_keys"]
        .as_array()
        .expect("source keys")
        .len();
    missing_root["source_keys"]
        .as_array_mut()
        .expect("source keys")
        .push(absent_key);
    let absent = json!({"source_key": absent_index});
    missing_root["feature_accounting"][0]["node_id"] = absent.clone();
    missing_root["source_map"]["requested_items"][0]["requested_item"] = absent.clone();
    missing_root["source_map"]["requested_items"][0]["occurrences"] = json!([]);
    missing_root["source_map"]["refusals"][0]["requested_item"] = absent;
    missing_root["source_map"]["refusals"][0]["occurrences"] = json!([]);
    let missing_root = canonical(&missing_root);
    let missing_root_read = read(&missing_root);
    assert!(
        matches!(missing_root_read, ContractPackageReadResult::Admitted(_)),
        "{missing_root_read:?}"
    );
    assert_eq!(
        ContractPackageWireV1::read_with_source(
            &missing_root,
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &source,
        ),
        Err(ContractSourceHandoffError::MissingNode(missing()))
    );
    let mut wrong_source_wire = mixed_fixture();
    wrong_source_wire["name"] = json!("test/different");
    refresh_identity(&mut wrong_source_wire);
    let wrong_source = admit(&wrong_source_wire);
    assert_ne!(wrong_source.name(), source.name());
    assert_eq!(
        ContractPackageWireV1::read_with_source(
            wire_bytes,
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &wrong_source,
        ),
        Err(ContractSourceHandoffError::SourceIdentity)
    );
    let mut wrong_lock: Value = serde_json::from_slice(wire_bytes).expect("wire json");
    let temporal_profile = wrong_lock["package"]["source"]["lock"]["profile_selections"]
        .as_array_mut()
        .expect("profile selections")
        .iter_mut()
        .find(|selection| selection["role"] == "temporal_profile")
        .expect("temporal profile selection");
    temporal_profile["definition"]["identity"] =
        json!("quire.temporal.event-position.false-extension/v1");
    let wrong_lock = canonical(&wrong_lock);
    assert!(matches!(
        read(&wrong_lock),
        ContractPackageReadResult::Admitted(_)
    ));
    assert_eq!(
        ContractPackageWireV1::read_with_source(
            &wrong_lock,
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
            &source,
        ),
        Err(ContractSourceHandoffError::SourceLock)
    );
    let short_limit = CheckedPackageReadLimits {
        bytes: u64::try_from(wrong_lock.len()).expect("wire length") - 1,
        ..CheckedPackageReadLimits::bounded()
    };
    let Ok(ContractPackageReadResult::Incomplete(incomplete)) =
        ContractPackageWireV1::read_with_source(
            &wrong_lock,
            short_limit,
            &ContractDependencies::new(),
            &source,
        )
    else {
        panic!("V1 byte limit must precede unsupported source handoff");
    };
    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Bytes);
    assert_eq!(incomplete.limit, short_limit.bytes);
}

#[trace("TC-047", "FR-035-AC-21")]
#[test]
fn retained_views_preserve_kind_and_tag_refusal_before_identity_and_limit() {
    let result = lower(&mixed_fixture(), &request());
    let original = bytes(&result);
    let mut bad_kind: Value = serde_json::from_slice(original).expect("package json");
    bad_kind["semantic_graph"]["nodes"][0]["semantic_form"] = json!("not_a_form");
    let ContractPackageReadResult::Refused(kind_refusal) = read(&canonical(&bad_kind)) else {
        panic!("invalid kind must refuse");
    };
    assert_eq!(kind_refusal.code, Code::InvalidSemanticGraph);
    assert_eq!(
        kind_refusal.loci[0]
            .path
            .clone()
            .expect("kind path")
            .as_str(),
        "/semantic_graph/nodes/0/semantic_form"
    );
    let mut bad_tag: Value = serde_json::from_slice(original).expect("package json");
    bad_tag["feature_accounting"][2]["node_tag"] = json!("not_a_tag");
    let ContractPackageReadResult::Refused(tag_refusal) = read(&canonical(&bad_tag)) else {
        panic!("invalid unsupported tag must refuse");
    };
    assert_eq!(tag_refusal.code, Code::InvalidPackage);
    assert_eq!(
        tag_refusal.loci[0].path.clone().expect("tag path").as_str(),
        "/feature_accounting/2/node_tag"
    );
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.nodes = 0;
    assert!(matches!(
        read_with_limits(original, limits),
        ContractPackageReadResult::Incomplete(_)
    ));
}

#[trace("TC-047", "FR-035-AC-3", "FR-035-AC-4")]
#[test]
fn canonical_package_retains_request_order_and_source_coordinates() {
    let input = mixed_fixture();
    let first = lower(&input, &request());
    let same = lower(&input, &request());
    assert_eq!(bytes(&first), bytes(&same));
    let wire: Value = serde_json::from_slice(bytes(&first)).expect("wire json");
    let keys = wire
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        [
            "contract_version",
            "feature_accounting",
            "package",
            "semantic_graph",
            "source_keys",
            "source_map"
        ]
    );
    let mut reversed = request();
    reversed.reverse();
    let reordered = lower(&input, &reversed);
    assert_ne!(
        bytes(&first),
        bytes(&reordered),
        "request order is retained in accounting"
    );
    let mut relocated = input;
    relocated["source_map"][3]["regions"][0]["end"] = json!(5);
    let changed = lower(&relocated, &request());
    assert_ne!(bytes(&first), bytes(&changed));
    let first_wire = first.package.wire().expect("first wire").wire();
    let changed_wire = changed.package.wire().expect("changed wire").wire();
    assert_eq!(first_wire.semantic_graph, changed_wire.semantic_graph);
    assert_eq!(
        first_wire.feature_accounting,
        changed_wire.feature_accounting
    );
    assert_ne!(first_wire.source_map, changed_wire.source_map);
}

#[trace("TC-047", "FR-035-AC-31")]
#[test]
fn reader_refuses_malformed_outer_shape_and_identity_member() {
    use quire_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    let output = lower(&mixed_fixture(), &request());
    let wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    for member in [
        "contract_version",
        "package",
        "semantic_graph",
        "source_keys",
        "source_map",
        "feature_accounting",
    ] {
        let mut missing = wire.clone();
        missing.as_object_mut().expect("object").remove(member);
        let ContractPackageReadResult::Refused(refusal) = read(&canonical(&missing)) else {
            panic!("missing {member} must refuse");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(Cause::MissingMember)),
            "{member}"
        );
        assert_eq!(
            refusal.loci[0].path.clone().expect("root pointer").as_str(),
            "",
            "{member}"
        );
    }
    let mut extra = wire.clone();
    extra["types"] = json!([]);
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&extra)) else {
        panic!("extra outer member must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::UnknownMember))
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .clone()
            .expect("object pointer")
            .as_str(),
        ""
    );
    let mut old_identity = wire;
    old_identity["identity"] = json!(null);
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&old_identity)) else {
        panic!("removed identity member must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::UnknownMember))
    );
    assert_eq!(
        refusal.loci[0].path.clone().expect("root pointer").as_str(),
        ""
    );
}

#[trace("TC-047", "FR-035-AC-9")]
#[test]
fn reader_keeps_lowering_profile_reference_closed() {
    use quire_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    wire["package"]["lowering_profile"]["settings"] = json!({});
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wire)) else {
        panic!("a lowering profile reference has only authority and identity");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::UnknownMember))
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .clone()
            .expect("object pointer")
            .as_str(),
        "/package/lowering_profile"
    );
}

#[trace("TC-047", "FR-035-AC-2")]
#[test]
fn reader_locates_malformed_nested_source_value() {
    use quire_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    wire["package"]["source"]["lock"]["sources"][0]["identity"] = json!(4);
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wire)) else {
        panic!("malformed nested source identity must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::WrongValueKind))
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .clone()
            .expect("member pointer")
            .as_str(),
        "/package/source/lock/sources/0/identity"
    );
}

#[trace("TC-047", "FR-035-AC-4")]
#[test]
fn reader_distinguishes_unknown_occurrence_role_from_wrong_value_kind() {
    use quire_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    let output = lower(&mixed_fixture(), &request());
    let wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    for (role, cause) in [
        (json!("bogus"), Cause::InvalidValue),
        (json!(4), Cause::WrongValueKind),
    ] {
        let mut changed = wire.clone();
        changed["source_map"]["nodes"][0]["occurrences"][0]["role"] = role;
        let ContractPackageReadResult::Refused(refusal) = read(&canonical(&changed)) else {
            panic!("invalid occurrence role must refuse");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(cause))
        );
        assert_eq!(
            refusal.loci[0].path.clone().expect("role pointer").as_str(),
            "/source_map/nodes/0/occurrences/0/role"
        );
    }
}

#[trace("TC-047", "FR-035-AC-4")]
#[test]
fn reader_requires_total_source_correspondence_and_request_order() {
    use quire_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    let output = lower(&mixed_fixture(), &request());
    let wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    let mut missing_map = wire.clone();
    missing_map["source_map"]["nodes"]
        .as_array_mut()
        .expect("node maps")
        .pop();
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&missing_map)) else {
        panic!("incomplete node map must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue))
    );
    assert_eq!(
        refusal.loci[0].path.clone().expect("map pointer").as_str(),
        "/source_map"
    );

    let mut bad_region: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    let end = bad_region["source_map"]["nodes"][0]["occurrences"][0]["regions"][0]["end"].clone();
    bad_region["source_map"]["nodes"][0]["occurrences"][0]["regions"][0]["start"] = end;
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&bad_region)) else {
        panic!("empty source region must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue))
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .clone()
            .expect("region pointer")
            .as_str(),
        "/source_map/nodes/0/occurrences/0/regions/0/start"
    );
}

#[trace("TC-047", "FR-035-AC-3")]
#[test]
fn refused_request_appears_only_if_reached_by_a_lowered_root() {
    let result = lower(&mixed_fixture(), &request());
    let represented = result
        .package
        .wire()
        .expect("wire")
        .wire()
        .semantic_graph
        .nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect::<BTreeSet<_>>();
    assert!(represented.contains(&id("dddd")));
    assert!(!represented.contains(&id("1010")));
    assert!(!represented.contains(&id("7070")));
    assert!(!represented.contains(&missing()));
}

fn failed_wire(limit: &str, consumed: &str) -> Value {
    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    let source_ref = wire["feature_accounting"][1]["node_id"].clone();
    wire["feature_accounting"][1] = json!({
        "node_id": source_ref, "disposition": "failed", "limit_kind": "work",
        "limit": limit, "consumed": consumed,
    });
    wire["source_map"]["refusals"][0]["disposition"] = json!("failed");
    wire
}

#[trace("TC-047", "FR-035-AC-11")]
#[test]
fn canonical_decimal_counters_keep_the_full_u64_range() {
    use quire_contract_model::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
    for counter in ["0", "18446744073709551615"] {
        assert!(matches!(
            read(&canonical(&failed_wire(counter, counter))),
            ContractPackageReadResult::Admitted(_)
        ));
    }
    for invalid in ["08", "+1", "-1", "", "١", "18446744073709551616"] {
        let bytes = canonical(&failed_wire(invalid, "8"));
        let ContractPackageReadResult::Refused(refusal) = read(&bytes) else {
            panic!("counter {invalid} must refuse");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::InvalidValue)
        );
        assert_eq!(
            refusal.loci[0].path.clone().expect("pointer").as_str(),
            "/feature_accounting/1/limit"
        );
    }
    let mut wrong_kind = failed_wire("8", "8");
    wrong_kind["feature_accounting"][1]["limit"] = json!(8);
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wrong_kind)) else {
        panic!("number must refuse");
    };
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::WrongValueKind)
    );
}

#[trace("TC-047", "FR-035-AC-17")]
#[test]
fn defensive_body_payloads_are_closed_and_omitted_pointers_stay_omitted() {
    use quire_contract_model::CheckedPackageRefusalCause;
    let mut wire = failed_wire("0", "0");
    let source_ref = wire["feature_accounting"][1]["node_id"].clone();
    wire["feature_accounting"][1] = json!({
        "node_id": source_ref, "disposition": "body_incomplete", "body_node_id": source_ref,
        "incomplete": {"limit_kind": "bytes", "limit": "8", "consumed": "8"},
    });
    wire["source_map"]["refusals"][0]["disposition"] = json!("body_incomplete");
    assert!(matches!(
        read(&canonical(&wire)),
        ContractPackageReadResult::Admitted(_)
    ));
    wire["feature_accounting"][1]["incomplete"]["path"] = json!("");
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wire)) else {
        panic!("bytes path must refuse");
    };
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::UnknownMember)
    );
    wire["feature_accounting"][1]["incomplete"]["limit_kind"] = json!("work");
    assert!(matches!(
        read(&canonical(&wire)),
        ContractPackageReadResult::Admitted(_)
    ));
    let mut unknown_kind = wire.clone();
    unknown_kind["feature_accounting"][1]["incomplete"]["limit_kind"] = json!("depth");
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&unknown_kind)) else {
        panic!("unknown limit kind must refuse");
    };
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path.clone().expect("kind pointer").as_str(),
        "/feature_accounting/1/incomplete/limit_kind"
    );
    wire["feature_accounting"][1]["incomplete"]
        .as_object_mut()
        .expect("object")
        .remove("path");
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wire)) else {
        panic!("work path required");
    };
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingMember)
    );

    let mut body = failed_wire("0", "0");
    let source_ref = body["feature_accounting"][1]["node_id"].clone();
    body["feature_accounting"][1] = json!({
        "node_id": source_ref, "disposition": "invalid_body", "body_node_id": source_ref,
        "refusal": {"code": "invalid_package", "path": ""},
    });
    body["source_map"]["refusals"][0]["disposition"] = json!("invalid_body");
    assert!(matches!(
        read(&canonical(&body)),
        ContractPackageReadResult::Admitted(_)
    ));
    let mut unpaired = body.clone();
    unpaired["feature_accounting"][1]["refusal"]["cause"] = json!("missing-selection");
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&unpaired)) else {
        panic!("unpaired cause must refuse");
    };
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .clone()
            .expect("cause pointer")
            .as_str(),
        "/feature_accounting/1/refusal/cause"
    );
    let mut unknown_code = body.clone();
    unknown_code["feature_accounting"][1]["refusal"]["code"] = json!("unknown_code");
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&unknown_code)) else {
        panic!("unknown code must refuse");
    };
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path.clone().expect("code pointer").as_str(),
        "/feature_accounting/1/refusal/code"
    );
    body["feature_accounting"][1]["refusal"]["path"] = Value::Null;
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&body)) else {
        panic!("null optional must refuse");
    };
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::WrongValueKind)
    );
}

#[trace("TC-047", "FR-035-AC-17")]
#[test]
fn reader_refuses_causes_on_causeless_nested_reader_codes() {
    for (code, contract_version) in [
        ("unknown_contract_version", Some("unknown/v1")),
        ("unknown_required_capability", None),
    ] {
        let mut wire = failed_wire("0", "0");
        let source_ref = wire["feature_accounting"][1]["node_id"].clone();
        let mut nested = json!({"code": code, "path": ""});
        if let Some(version) = contract_version {
            nested["contract_version"] = json!(version);
        }
        wire["feature_accounting"][1] = json!({
            "node_id": source_ref, "disposition": "invalid_body", "body_node_id": source_ref,
            "refusal": nested,
        });
        wire["source_map"]["refusals"][0]["disposition"] = json!("invalid_body");
        assert!(matches!(
            read(&canonical(&wire)),
            ContractPackageReadResult::Admitted(_)
        ));

        // A recognized cause reaches code/cause pairing before digest checking.
        wire["feature_accounting"][1]["refusal"]["cause"] = json!("invalid-value");
        let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wire)) else {
            panic!("{code} with a cause must refuse");
        };
        assert_eq!(refusal.code, Code::InvalidPackage);
        assert_eq!(refusal.cause, Some(Cause::InvalidValue));
        assert_eq!(
            refusal.loci[0]
                .path
                .clone()
                .expect("cause pointer")
                .as_str(),
            "/feature_accounting/1/refusal/cause"
        );
    }

    for (code, cause, contract_version) in [
        (
            "unknown_contract_version",
            "unsupported-wire",
            Some("unknown/v1"),
        ),
        ("unknown_required_capability", "unsupported-feature", None),
    ] {
        let mut wire = failed_wire("0", "0");
        let mut nested = json!({"code": code, "path": "", "cause": cause});
        if let Some(version) = contract_version {
            nested["contract_version"] = json!(version);
        }
        wire["feature_accounting"][1] = json!({
            "node_id": id("1010"), "disposition": "invalid_body", "body_node_id": id("1010"),
            "refusal": nested,
        });
        wire["source_map"]["refusals"][0]["disposition"] = json!("invalid_body");
        let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wire)) else {
            panic!("{code} with {cause} must refuse");
        };
        assert_eq!(refusal.code, Code::InvalidPackage);
        assert_eq!(refusal.cause, Some(Cause::InvalidValue));
        assert_eq!(
            refusal.loci[0]
                .path
                .clone()
                .expect("cause pointer")
                .as_str(),
            "/feature_accounting/1/refusal/cause"
        );
    }
}

#[trace("TC-047", "FR-035-AC-13")]
#[test]
fn duplicate_member_and_noncanonical_bytes_refuse() {
    use quire_contract_model::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
    let wire = canonical(&failed_wire("0", "0"));
    let text = std::str::from_utf8(&wire).expect("utf8");
    let duplicate = text.replacen("\"limit\":\"0\"", "\"limit\":\"0\",\"limit\":\"0\"", 1);
    let ContractPackageReadResult::Refused(refusal) = read(duplicate.as_bytes()) else {
        panic!("duplicate must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DuplicateMember)
    );
}

#[trace("TC-047", "FR-035-AC-2", "FR-035-AC-19")]
#[test]
fn selected_dependencies_are_bound_by_library_identity_before_graph_checks() {
    use quire_contract_model::{
        CheckedPackageRefusalCause, CheckedPackageRefusalCode, ContractDependencyInput,
    };
    let (dependency, wire) = wire_selecting_lib_a();
    let bytes = canonical(&wire);
    let mut supplied = ContractDependencies::new();
    supplied.insert(
        "lib/a",
        ContractDependencyInput {
            identity: "lib/a",
            package: &dependency,
        },
    );
    let ContractPackageReadResult::Admitted(package) =
        ContractPackageWireV1::read(&bytes, CheckedPackageReadLimits::bounded(), &supplied)
    else {
        panic!("selected dependency must admit");
    };
    let represented_keys = package
        .wire()
        .semantic_graph
        .nodes
        .iter()
        .map(|node| &node.node_id)
        .collect::<BTreeSet<_>>();
    let local_source = admit(&mixed_fixture());
    let local_keys = local_source
        .graph()
        .nodes
        .iter()
        .map(|node| &node.node_id)
        .collect::<BTreeSet<_>>();
    let external_only_key = dependency
        .graph()
        .nodes
        .iter()
        .map(|node| &node.node_id)
        .find(|key| !represented_keys.contains(key) && local_keys.contains(key))
        .expect("an identically keyed local node cannot satisfy a selected dependency");
    assert_eq!(package.node_kind(external_only_key), None);
    let ContractPackageReadResult::Refused(missing) = read(&bytes) else {
        panic!("selection requires input");
    };
    assert_eq!(missing.code, CheckedPackageRefusalCode::MissingImport);
    assert_eq!(
        missing.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        missing.loci[0].path.clone().expect("path").as_str(),
        "/package/source/lock/dependency_selections/0"
    );
    supplied.insert(
        "lib/a",
        ContractDependencyInput {
            identity: "lib/b",
            package: &dependency,
        },
    );
    let ContractPackageReadResult::Refused(wrong_identity) =
        ContractPackageWireV1::read(&bytes, CheckedPackageReadLimits::bounded(), &supplied)
    else {
        panic!("wrong identity refuses");
    };
    assert_eq!(
        wrong_identity.code,
        CheckedPackageRefusalCode::MissingImport
    );
}

fn wire_selecting_lib_a() -> (CheckedPackageV2, Value) {
    let mut dependency_wire = mixed_fixture();
    dependency_wire["name"] = json!("lib/a");
    refresh_identity(&mut dependency_wire);
    let dependency = admit(&dependency_wire);
    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    wire["package"]["source"]["lock"]["dependency_selections"] = json!([{"identity": "lib/a"}]);
    (dependency, wire)
}

#[trace("TC-047", "FR-035-AC-18")]
#[test]
fn contract_reader_refuses_unequal_complete_values_in_dependency_diamond() {
    use crate::support::checked_package::read_with_dependencies;
    use quire_contract_model::ContractDependencyInput;

    let mut first_wire = mixed_fixture();
    first_wire["name"] = json!("lib/leaf");
    refresh_identity(&mut first_wire);
    let first_leaf = admit(&first_wire);
    let mut second_wire = first_wire.clone();
    second_wire["source_map"][0]["regions"][0]["end"] = json!(2);
    let second_leaf = admit(&second_wire);
    assert_eq!(first_leaf.name(), second_leaf.name());
    assert_ne!(canonical(&first_wire), canonical(&second_wire));

    let intermediary = |name: &str, leaf: &CheckedPackageV2| {
        let mut wire = mixed_fixture();
        wire["name"] = json!(name);
        wire["lock"]["dependency_selections"] = json!([{"identity": "lib/leaf"}]);
        refresh_identity(&mut wire);
        let CheckedPackageV2ReadResult::Admitted(package) =
            read_with_dependencies(&wire, &[("lib/leaf", leaf)])
        else {
            panic!("intermediary dependency admits");
        };
        package
    };
    let left = intermediary("lib/left", &first_leaf);
    let right = intermediary("lib/right", &second_leaf);

    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    wire["package"]["source"]["lock"]["dependency_selections"] = json!([
        {"identity": "lib/left"}, {"identity": "lib/right"}
    ]);
    let bytes = canonical(&wire);
    let mut supplied = ContractDependencies::new();
    supplied.insert(
        "lib/left",
        ContractDependencyInput {
            identity: "lib/left",
            package: &left,
        },
    );
    supplied.insert(
        "lib/right",
        ContractDependencyInput {
            identity: "lib/right",
            package: &right,
        },
    );
    let ContractPackageReadResult::Refused(refusal) =
        ContractPackageWireV1::read(&bytes, CheckedPackageReadLimits::bounded(), &supplied)
    else {
        panic!("ContractPackage must refuse the unequal transitive diamond");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::ConflictingDefinition));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some("/package/source/lock/dependency_selections/0/lock/dependency_selections/0")
    );
    assert_eq!(
        refusal.loci[1].path.as_ref().map(|path| path.as_str()),
        Some("/package/source/lock/dependency_selections/1/lock/dependency_selections/0")
    );
}

#[trace("TC-047", "FR-035-AC-2")]
#[test]
fn reader_refuses_wrong_named_selected_dependency_at_selection() {
    use quire_contract_model::{
        CheckedPackageRefusalCause, CheckedPackageRefusalCode, ContractDependencyInput,
    };
    let (dependency, mut wire) = wire_selecting_lib_a();
    let mut supplied = ContractDependencies::new();
    supplied.insert(
        "lib/a",
        ContractDependencyInput {
            identity: "lib/a",
            package: &dependency,
        },
    );
    wire["package"]["source"]["lock"]["dependency_selections"][0]["identity"] =
        json!("lib/missing");
    let ContractPackageReadResult::Refused(refusal) = ContractPackageWireV1::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &supplied,
    ) else {
        panic!("wrong-named selected dependency must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingImport);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some("/package/source/lock/dependency_selections/0")
    );
}

#[trace("TC-047", "FR-035-AC-4")]
#[test]
fn reader_refuses_reference_type_targeting_an_existing_scalar_node() {
    use quire_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let scalar = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type")
        .expect("represented scalar")["node_id"]
        .clone();
    let position = nodes
        .iter()
        .position(|node| node["node_tag"] != "scalar_type")
        .expect("represented non-scalar");
    let node = &mut nodes[position];
    node["node_tag"] = json!("composite_type");
    node["semantic_form"] = json!("reference");
    node["semantic_type"] = node["node_id"].clone();
    node["body"] = json!({
        "term": "aggregate",
        "members": [{"term": "reference", "target": scalar}],
    });
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wire)) else {
        panic!("reference type cannot target a scalar key");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue))
    );
    assert_eq!(
        refusal.loci[0].path.clone().expect("body pointer").as_str(),
        format!("/semantic_graph/nodes/{position}/body")
    );
}
