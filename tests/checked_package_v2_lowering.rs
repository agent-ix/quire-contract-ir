// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Exact, independent per-item lowering of admitted CheckedPackage V2 nodes.

#[path = "support/checked_package.rs"]
mod checked_package;

use checked_package::{
    canonical, evidence_for, refresh_identity, sha256_hex, typed_node_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    CONTRACT_IR_SEMANTIC_DOMAIN,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// Every family in the all-families fixture, in graph order.
const FAMILIES: [(&str, CheckedNodeTag); 13] = [
    ("aaaa", CheckedNodeTag::ScalarType),
    ("bbbb", CheckedNodeTag::CompositeType),
    ("cccc", CheckedNodeTag::BoundedDomain),
    ("dddd", CheckedNodeTag::Value),
    ("eeee", CheckedNodeTag::Expression),
    ("ffff", CheckedNodeTag::Function),
    ("1010", CheckedNodeTag::Model),
    ("2020", CheckedNodeTag::Relation),
    ("3030", CheckedNodeTag::State),
    ("4040", CheckedNodeTag::Temporal),
    ("5050", CheckedNodeTag::Protocol),
    ("6060", CheckedNodeTag::Claim),
    ("7070", CheckedNodeTag::Correspondence),
];

fn key(prefix: &str) -> String {
    prefix.repeat(16)
}

fn id(prefix: &str) -> CheckedNodeId {
    typed_node_id(&key(prefix))
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

fn profile(work_limit: u64) -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        supported_tags: FAMILIES.iter().map(|(_, tag)| *tag).collect(),
        require_bounds: false,
        work_limit,
    }
}

fn lowered(record: &CompleteLoweringRecordV2) -> &quire_contract_ir::CompleteContractNodeV2 {
    match record {
        CompleteLoweringRecordV2::Lowered { node } => node,
        other => panic!("expected lowered record, got {other:?}"),
    }
}

/// Tracing: TC-050, FR-038-AC-7
#[trace("TC-050", "FR-038-AC-7")]
#[test]
fn tc_050_every_family_lowers_with_its_exact_closure_and_identity() {
    let value = v2_all_families();
    let package = admit(&value);
    let requested = FAMILIES
        .iter()
        .map(|(prefix, _)| id(prefix))
        .collect::<Vec<_>>();
    let result = package.lower(&requested, &profile(u64::MAX));
    assert_eq!(result.package_id, *package.package_id());
    assert_eq!(result.records.len(), FAMILIES.len());
    for (position, ((prefix, tag), record)) in FAMILIES.iter().zip(&result.records).enumerate() {
        let node = lowered(record);
        let wire = &value["semantic_graph"]["nodes"][position];
        assert_eq!(node.node_tag, *tag, "{prefix}");
        assert_eq!(node.node.node_id, id(prefix));
        assert_eq!(serde_json::to_value(&node.node).expect("node"), *wire);
        assert_eq!(node.semantic_type, id("aaaa"));
        let expected_dependencies = match *prefix {
            "aaaa" => vec![],
            "eeee" | "7070" => vec![id("aaaa"), id("dddd")],
            _ => vec![id("aaaa")],
        };
        assert_eq!(node.dependencies, expected_dependencies, "{prefix}");
        let only_self = |family: CheckedNodeTag| {
            if *tag == family {
                vec![id(prefix)]
            } else {
                vec![]
            }
        };
        assert_eq!(
            node.bounds,
            only_self(CheckedNodeTag::BoundedDomain),
            "{prefix}"
        );
        assert_eq!(node.claims, only_self(CheckedNodeTag::Claim), "{prefix}");
        let source_map = value["source_map"]
            .as_array()
            .expect("source map")
            .iter()
            .filter(|entry| entry["node_id"] == wire["node_id"])
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::to_value(&node.source_map).expect("source map"),
            Value::Array(source_map)
        );

        // The lowered identity is re-derivable from the wire alone.
        let mut projection = wire.clone();
        projection
            .as_object_mut()
            .expect("node")
            .remove("occurrences");
        let preimage = json!({
            "version": "quire.contract-ir.lowered-node/v1",
            "node": projection,
            "dependencies": node.dependencies,
            "bounds": node.bounds,
            "claims": node.claims,
        });
        assert_eq!(node.ir_id.domain.as_ref(), CONTRACT_IR_SEMANTIC_DOMAIN);
        assert_eq!(node.ir_id.algorithm.as_ref(), "sha256");
        assert_eq!(
            node.ir_id.digest.as_ref(),
            sha256_hex(&canonical(&preimage))
        );
    }
    let identities = result
        .records
        .iter()
        .map(|record| lowered(record).ir_id.digest.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(identities.len(), FAMILIES.len());

    // Nominal nodes lower with their preimages and nominal closures intact.
    let nominal_value = checked_package::v2_nominal();
    let nominal = admit(&nominal_value);
    let keys = nominal_value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .map(|node| typed_node_id(node["node_id"]["digest"].as_str().expect("key")))
        .collect::<Vec<_>>();
    let result = nominal.lower(&keys, &profile(u64::MAX));
    let [member, declaration, unit, dimension] = [0, 1, 2, 3].map(|i| lowered(&result.records[i]));
    assert_eq!(member.dependencies, vec![keys[1].clone()]);
    assert_eq!(member.semantic_type, keys[1]);
    assert!(declaration.dependencies.is_empty());
    assert_eq!(unit.dependencies, vec![keys[3].clone()]);
    assert!(dimension.dependencies.is_empty());
    for (position, node) in [member, declaration, unit, dimension].iter().enumerate() {
        assert_eq!(
            serde_json::to_value(&node.node).expect("node"),
            nominal_value["semantic_graph"]["nodes"][position]
        );
        assert!(node.node.nominal_identity_preimage.is_some());
    }
}

/// Tracing: TC-050, FR-038-AC-7
#[trace("TC-050", "FR-038-AC-7")]
#[test]
fn tc_050_non_lowered_records_are_terminal_and_independent() {
    let value = v2_all_families();
    let package = admit(&value);
    let missing = typed_node_id(&"9".repeat(64));

    // invalid_input for a key outside the admitted graph.
    let result = package.lower(&[missing.clone(), id("aaaa")], &profile(u64::MAX));
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::InvalidInput { node_id: missing }
    );
    assert_eq!(lowered(&result.records[1]).node.node_id, id("aaaa"));

    // unsupported names the first unsupported reachable key.
    let mut narrow = profile(u64::MAX);
    narrow.supported_tags.remove(&CheckedNodeTag::Value);
    let result = package.lower(&[id("7070"), id("aaaa"), id("dddd")], &narrow);
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::Unsupported {
            node_id: id("7070"),
            unsupported_node_id: id("dddd"),
            node_tag: CheckedNodeTag::Value,
        }
    );
    assert_eq!(lowered(&result.records[1]).node.node_id, id("aaaa"));
    assert_eq!(
        result.records[2],
        CompleteLoweringRecordV2::Unsupported {
            node_id: id("dddd"),
            unsupported_node_id: id("dddd"),
            node_tag: CheckedNodeTag::Value,
        }
    );

    // failed at exactly one over the request's own work; siblings unaffected.
    // eeee visits eeee, aaaa and dddd: one for the request plus three.
    let exact = package.lower(&[id("eeee")], &profile(4));
    assert_eq!(
        lowered(&exact.records[0]).dependencies,
        vec![id("aaaa"), id("dddd")]
    );
    let result = package.lower(&[id("eeee"), id("aaaa"), id("eeee")], &profile(3));
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("eeee"),
            limit: 3,
            consumed: 4,
        }
    );
    assert_eq!(
        result.records[1],
        package.lower(&[id("aaaa")], &profile(3)).records[0]
    );
    assert_eq!(result.records[2], result.records[0]);
    assert_eq!(
        package.lower(&[id("aaaa")], &profile(1)).records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("aaaa"),
            limit: 1,
            consumed: 2,
        }
    );
    assert_eq!(
        package.lower(&[id("aaaa")], &profile(0)).records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: id("aaaa"),
            limit: 0,
            consumed: 1,
        }
    );
}

/// Tracing: TC-050, FR-038-AC-7
#[trace("TC-050", "FR-038-AC-7")]
#[test]
fn tc_050_unbounded_types_require_a_reachable_bounding_domain() {
    let mut value = v2_all_families();
    value["semantic_graph"]["nodes"][0]["semantic_form"] = json!("integer");
    value["semantic_graph"]["nodes"][3]["dependencies"] =
        json!([value["semantic_graph"]["nodes"][2]["node_id"]]);
    refresh_identity(&mut value);
    let package = admit(&value);
    let mut bounded = profile(u64::MAX);
    bounded.require_bounds = true;

    let result = package.lower(&[id("bbbb"), id("dddd"), id("cccc")], &bounded);
    assert_eq!(
        result.records[0],
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id("bbbb"),
            unbounded_type: id("aaaa"),
        }
    );
    let value_node = lowered(&result.records[1]);
    assert_eq!(value_node.dependencies, vec![id("aaaa"), id("cccc")]);
    assert_eq!(value_node.bounds, vec![id("cccc")]);
    assert_eq!(lowered(&result.records[2]).bounds, vec![id("cccc")]);

    // Without the bound requirement the same request lowers.
    let unbounded = package.lower(&[id("bbbb")], &profile(u64::MAX));
    assert!(unbounded.records[0] != result.records[0]);
    assert!(lowered(&unbounded.records[0]).bounds.is_empty());
}
