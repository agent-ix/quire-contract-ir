// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-935 source-selection vectors from the authoritative QSpec checkout.

use crate::fixture_support as checked_package;
use checked_package::{canonical, evidence_for, refresh_identity, typed_node_id, v2_nominal};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
};
use serde_json::{json, Value};

const SELECTION_VECTORS: &str = "contract-package-selection-vectors.json";

fn vectors() -> Value {
    let path = super::proposal_dir_of(&super::fixtures_dir()).join(SELECTION_VECTORS);
    let value = super::read_json(&path).unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(
        value["version"],
        "quire.contract-package-selection-vectors/v1"
    );
    assert_eq!(
        value["compatibility_cases"]
            .as_array()
            .expect("compatibility cases")
            .len(),
        4
    );
    assert_eq!(
        value["repeat_controls"]
            .as_array()
            .expect("repeat controls")
            .len(),
        9
    );
    value
}

fn row(vectors: &Value, label: &str) -> Value {
    vectors["selections"][label]
        .as_object()
        .expect("published selection row");
    vectors["selections"][label].clone()
}

fn rows(vectors: &Value, labels: &Value) -> Vec<Value> {
    labels
        .as_array()
        .expect("selection labels")
        .iter()
        .map(|label| row(vectors, label.as_str().expect("selection label")))
        .collect()
}

fn profile(work_limit: u64) -> CompleteLoweringProfileV2 {
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
            CheckedNodeTag::Correspondence,
        ]
        .into_iter()
        .collect(),
        require_bounds: false,
        work_limit,
    }
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

fn lowered(record: &CompleteLoweringRecordV2) -> &quire_contract_model::CompleteContractNodeV2 {
    match record {
        CompleteLoweringRecordV2::Lowered { node } => node,
        other => panic!("expected lowered record, got {other:?}"),
    }
}

fn selected_nominal(rows: Vec<Value>) -> (Value, CheckedPackageV2, CheckedNodeId) {
    let mut value = v2_nominal();
    value["lock"]["profile_selections"] = Value::Array(rows);
    refresh_identity(&mut value);
    let package = admit(&value);
    let request = serde_json::from_value(value["semantic_graph"]["nodes"][0]["node_id"].clone())
        .expect("typed first node key");
    (value, package, request)
}

fn with_requirements(rows: &[Value], work_limit: u64) -> CompleteLoweringProfileV2 {
    let mut selected = profile(work_limit);
    selected.required_source_selections = rows
        .iter()
        .cloned()
        .map(|row| serde_json::from_value(row).expect("typed source selection"))
        .collect();
    selected
}

fn tc_935_rows() -> (Value, Value, Value) {
    let vectors = vectors();
    (row(&vectors, "T"), row(&vectors, "D"), row(&vectors, "U"))
}

fn assert_tc_935_failed_work(
    source: &Value,
    package: &CheckedPackageV2,
    request: &CheckedNodeId,
    requirements: &[Value],
    limit: u64,
    consumed: u64,
) {
    let result = package.lower(
        std::slice::from_ref(request),
        &with_requirements(requirements, limit),
    );
    assert_eq!(
        result.records,
        vec![CompleteLoweringRecordV2::Failed {
            node_id: request.clone(),
            limit_kind: CheckedPackageLimit::Work,
            limit,
            consumed,
        }]
    );
    assert!(result.package.lowered().is_empty());
    assert!(result.package.dependencies().is_empty());
    let wire = result.package.wire().expect("failed-work accounting wire");
    let emitted = serde_json::to_value(wire.wire()).expect("wire value");
    assert!(wire.wire().semantic_graph.nodes.is_empty());
    assert_eq!(emitted["package"]["source"]["lock"], source["lock"]);
    assert_eq!(
        emitted["package"]["lowering_profile"],
        json!({"authority":"quire","identity":"complete-v1"})
    );
    assert_eq!(
        emitted["feature_accounting"],
        json!([{
            "node_id":request, "disposition":"failed", "limit_kind":"work",
            "limit":limit.to_string(), "consumed":consumed.to_string()
        }])
    );
}

#[trace("TC-935", "FR-035-AC-9")]
#[test]
fn tc_935_required_source_rows_match_exactly_without_changing_provenance() {
    let vectors = vectors();
    let (t, d, u) = (row(&vectors, "T"), row(&vectors, "D"), row(&vectors, "U"));
    let mut other_authority = t.clone();
    other_authority["definition"]["authority"] = json!("other-authority");
    let missing = typed_node_id(&"0123456789abcdef".repeat(4));

    let mut cases: Vec<(Vec<Value>, Vec<Value>, bool)> = vectors["compatibility_cases"]
        .as_array()
        .expect("compatibility cases")
        .iter()
        .map(|case| {
            let expected = case["expected"]
                .as_str()
                .expect("published expected result");
            assert!(!expected.is_empty());
            (
                rows(&vectors, &case["lock_profile_selections"]),
                rows(&vectors, &case["required_source_selections"]),
                !expected.contains("invalid_input"),
            )
        })
        .collect();
    cases.extend([
        (vec![t.clone(), u.clone()], vec![t.clone()], true),
        (vec![t.clone(), u.clone()], vec![t.clone(), t.clone()], true),
        (vec![t.clone(), u.clone()], vec![u.clone(), t.clone()], true),
        (vec![t.clone(), d.clone()], vec![t.clone(), d.clone()], true),
        (vec![d.clone()], vec![], true),
        (vec![u.clone()], vec![t.clone()], false),
        (vec![t.clone()], vec![other_authority], false),
        (vec![t.clone()], vec![t.clone(), d.clone()], false),
    ]);
    for (lock, required, succeeds) in cases {
        let (source, package, request) = selected_nominal(lock);
        let second =
            serde_json::from_value(source["semantic_graph"]["nodes"][1]["node_id"].clone())
                .expect("typed second node key");
        let requests = [second, request.clone()];
        let result = package.lower(&requests, &with_requirements(&required, u64::MAX));
        let wire = result
            .package
            .wire()
            .expect("refusal package is still encoded");
        let serialized = serde_json::to_value(wire.wire()).expect("wire");
        assert_eq!(serialized["package"]["source"]["lock"], source["lock"]);
        assert_eq!(
            serialized["package"]["lowering_profile"],
            json!({"authority":"quire","identity":"complete-v1"})
        );
        assert!(serialized.get("required_source_selections").is_none());
        assert_eq!(wire.wire().feature_accounting.len(), 2);
        if succeeds {
            for (record, requested) in result.records.iter().zip(&requests) {
                let node = lowered(record);
                let source_node = source["semantic_graph"]["nodes"]
                    .as_array()
                    .expect("source nodes")
                    .iter()
                    .find(|entry| entry["node_id"] == json!(requested))
                    .expect("requested node in source");
                assert_eq!(
                    serde_json::to_value(&node.node).expect("typed node"),
                    *source_node
                );
            }
        } else {
            assert_eq!(
                result.records,
                requests
                    .iter()
                    .cloned()
                    .map(|node_id| CompleteLoweringRecordV2::InvalidInput { node_id })
                    .collect::<Vec<_>>()
            );
            assert!(wire.wire().semantic_graph.nodes.is_empty());
            for (index, feature) in serialized["feature_accounting"]
                .as_array()
                .expect("records")
                .iter()
                .enumerate()
            {
                assert_eq!(
                    feature,
                    &json!({"node_id": requests[index], "disposition": "invalid_input"})
                );
                let expected = source["source_map"]
                    .as_array()
                    .expect("source entries")
                    .iter()
                    .filter(|entry| entry["node_id"] == json!(requests[index]))
                    .cloned()
                    .collect::<Vec<_>>();
                assert_eq!(
                    serialized["source_map"]["requested_items"][index]["occurrences"],
                    json!(expected)
                );
            }
        }
        assert_eq!(
            package
                .lower(
                    std::slice::from_ref(&missing),
                    &with_requirements(&required, u64::MAX)
                )
                .records,
            vec![CompleteLoweringRecordV2::InvalidInput {
                node_id: missing.clone()
            }]
        );
    }
}

#[trace("TC-935", "FR-035-AC-9")]
#[test]
fn tc_935_compatibility_uses_existing_per_request_work_meter() {
    let (t, d, _) = tc_935_rows();
    let (empty_source, empty, request) = selected_nominal(vec![]);
    let (differing_source, differing, _) = selected_nominal(vec![d]);
    let failed = |limit, consumed| CompleteLoweringRecordV2::Failed {
        node_id: request.clone(),
        limit_kind: CheckedPackageLimit::Work,
        limit,
        consumed,
    };
    assert_eq!(
        empty
            .lower(
                std::slice::from_ref(&request),
                &with_requirements(std::slice::from_ref(&t), 2)
            )
            .records[0],
        CompleteLoweringRecordV2::InvalidInput {
            node_id: request.clone()
        }
    );
    assert_eq!(
        empty
            .lower(
                std::slice::from_ref(&request),
                &with_requirements(std::slice::from_ref(&t), 1)
            )
            .records[0],
        failed(1, 2)
    );
    assert_tc_935_failed_work(
        &empty_source,
        &empty,
        &request,
        std::slice::from_ref(&t),
        1,
        2,
    );
    assert_eq!(
        differing
            .lower(
                std::slice::from_ref(&request),
                &with_requirements(std::slice::from_ref(&t), 3)
            )
            .records[0],
        CompleteLoweringRecordV2::InvalidInput {
            node_id: request.clone()
        }
    );
    assert_eq!(
        differing
            .lower(
                std::slice::from_ref(&request),
                &with_requirements(std::slice::from_ref(&t), 2)
            )
            .records[0],
        failed(2, 3)
    );
    assert_tc_935_failed_work(
        &differing_source,
        &differing,
        &request,
        std::slice::from_ref(&t),
        2,
        3,
    );
    assert_eq!(
        empty
            .lower(
                &[request.clone(), request.clone()],
                &with_requirements(std::slice::from_ref(&t), 0)
            )
            .records,
        vec![failed(0, 1); 2]
    );
    assert_tc_935_failed_work(
        &empty_source,
        &empty,
        &request,
        std::slice::from_ref(&t),
        0,
        1,
    );
    let absent = typed_node_id(&"0123456789abcdef".repeat(4));
    assert_eq!(
        empty
            .lower(
                std::slice::from_ref(&absent),
                &with_requirements(std::slice::from_ref(&t), 0)
            )
            .records[0],
        CompleteLoweringRecordV2::Failed {
            node_id: absent.clone(),
            limit_kind: CheckedPackageLimit::Work,
            limit: 0,
            consumed: 1,
        }
    );
    assert_eq!(
        empty
            .lower(
                std::slice::from_ref(&absent),
                &with_requirements(std::slice::from_ref(&t), 1)
            )
            .records[0],
        CompleteLoweringRecordV2::InvalidInput { node_id: absent }
    );

    // The scalar root costs one node, one aggregate body term and one self edge.
    // Adding request, requirement and first candidate charges gives 6.
    let (matching_source, admitted, _) = selected_nominal(vec![t.clone()]);
    let matching_request =
        serde_json::from_value(matching_source["semantic_graph"]["nodes"][1]["node_id"].clone())
            .expect("self-typed declaration key");
    let exact = with_requirements(std::slice::from_ref(&t), 6);
    assert_eq!(
        lowered(
            &admitted
                .lower(std::slice::from_ref(&matching_request), &exact)
                .records[0]
        )
        .node
        .node_id,
        matching_request
    );
    assert_tc_935_failed_work(
        &matching_source,
        &admitted,
        &matching_request,
        std::slice::from_ref(&t),
        5,
        6,
    );
}
