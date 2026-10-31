// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public V2 structural identity and canonical-byte controls over an authored
//! single-node fixture. The published QSpec readback runs in conformance_qspec.

use ix_trace_rs::trace;
use quire_canonical::{Error, LimitKind, Limits};
use quire_contract_model::{
    encode_checked_package_v2, CheckedPackageEvidence, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    LoweringProfileRef,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use crate::support::checked_package::{index_fixture_node_ids, v2_all_families};

/// Resolve a path inside this checkout for the V1 source-scan controls.
pub(crate) fn repository_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// Omit test-only items when a source inventory asserts production behavior.
pub(crate) fn production_source(text: &str) -> String {
    let mut kept = String::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim_start() != "#[cfg(test)]" {
            kept.push_str(line);
            kept.push('\n');
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let closing = format!("{}}}", " ".repeat(indent));
        let mut first = true;
        for skipped in lines.by_ref() {
            let item = skipped.trim_start();
            if item.starts_with("#[") || item.starts_with("//") {
                continue;
            }
            if first && item.ends_with(';') {
                break;
            }
            first = false;
            if skipped.starts_with(&closing) {
                break;
            }
        }
    }
    kept
}

fn local_package() -> Value {
    let mut package = v2_all_families();
    let scalar = package["semantic_graph"]["nodes"][0].clone();
    let scalar_id = scalar["node_id"].clone();
    package["semantic_graph"]["nodes"] = json!([scalar]);
    package["source_map"] = json!(package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .filter(|entry| entry["node_id"] == scalar_id)
        .cloned()
        .collect::<Vec<_>>());
    package
}

fn derived_range_package() -> Value {
    let mut package = v2_all_families();
    let boolean = package["semantic_graph"]["nodes"][0].clone();
    let integer = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
        .expect("Integer node")
        .clone();
    let mut range = package["semantic_graph"]["nodes"][2].clone();
    let old_range_id = range["node_id"].clone();
    range["node_id"] = json!({
        "package": "test/families",
        "path": ["bounded_domain", "integer_range", integer["node_id"], "0", "9"],
        "ordinal": 0,
    });
    range["semantic_type"] = integer["node_id"].clone();
    range["dependencies"] = json!([integer["node_id"]]);
    range["body"] = json!({"term":"aggregate","members":[
        {"term":"binding","name":"min","value":
            {"term":"literal","type":integer["node_id"],"value_kind":"integer","value":"0"}},
        {"term":"binding","name":"max","value":
            {"term":"literal","type":integer["node_id"],"value_kind":"integer","value":"9"}}
    ]});
    let selected = [
        boolean["node_id"].clone(),
        integer["node_id"].clone(),
        old_range_id,
    ];
    let mut maps = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .filter(|entry| selected.contains(&entry["node_id"]))
        .cloned()
        .collect::<Vec<_>>();
    let range_map = maps
        .iter_mut()
        .find(|entry| entry["node_id"] == selected[2])
        .expect("range source");
    range_map["node_id"] = range["node_id"].clone();
    package["semantic_graph"]["nodes"] = json!([boolean, integer, range]);
    package["source_map"] = json!(maps);
    index_fixture_node_ids(&mut package);
    package
}

#[trace("TC-226", "FR-038-AC-124")]
#[test]
fn derived_range_binding_name_is_validated_independently_of_its_id() {
    let mut package = derived_range_package();
    let _admitted = admit(&package);
    let retained_id = package["semantic_graph"]["nodes"][2]["node_id"].clone();
    package["semantic_graph"]["nodes"][2]["body"]["members"][0]["name"] = json!("oops");
    assert_eq!(
        package["semantic_graph"]["nodes"][2]["node_id"],
        retained_id
    );
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let outcome = CheckedPackageV2::read(
        &quire_canonical::to_vec(&package, Limits::new(u64::MAX)).expect("canonical bytes"),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    let refusal = match outcome {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        CheckedPackageV2ReadResult::Admitted(_) => panic!("malformed derived body was admitted"),
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            panic!("malformed derived body exhausted a limit: {incomplete:?}")
        }
    };
    assert_eq!(
        refusal.code,
        quire_contract_model::CheckedPackageRefusalCode::InvalidPackage
    );
    assert_eq!(
        refusal.cause,
        Some(quire_contract_model::CheckedPackageRefusalCause::InvalidDerivedShape)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/2/node_id".to_owned())
    );
    assert_eq!(
        refusal.locus,
        Some(serde_json::from_value(retained_id).expect("authored structural node id"))
    );
}

#[trace("TC-226", "FR-038-AC-125")]
#[test]
fn derived_range_type_and_literal_type_are_checked_independently_of_id() {
    let package = derived_range_package();
    let _admitted = admit(&package);
    let boolean_id = package["semantic_graph"]["nodes"][0]["node_id"].clone();
    for edit in ["semantic_type", "bound_literal_type"] {
        let mut changed = package.clone();
        let retained_id = changed["semantic_graph"]["nodes"][2]["node_id"].clone();
        match edit {
            "semantic_type" => {
                changed["semantic_graph"]["nodes"][2]["semantic_type"] = boolean_id.clone()
            }
            "bound_literal_type" => {
                changed["semantic_graph"]["nodes"][2]["body"]["members"][0]["value"]["type"] =
                    boolean_id.clone()
            }
            _ => unreachable!(),
        }
        assert_eq!(
            changed["semantic_graph"]["nodes"][2]["node_id"],
            retained_id
        );
        let mut evidence = CheckedPackageEvidence::new();
        evidence.support_feature("quire.value.complete/v1");
        let outcome = CheckedPackageV2::read(
            &quire_canonical::to_vec(&changed, Limits::new(u64::MAX)).expect("canonical bytes"),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        let refusal = match outcome {
            CheckedPackageV2ReadResult::Refused(refusal) => refusal,
            other => panic!("{edit}: expected derived-shape refusal, got {other:?}"),
        };
        assert_eq!(
            refusal.code,
            quire_contract_model::CheckedPackageRefusalCode::InvalidPackage,
            "{edit}"
        );
        assert_eq!(
            refusal.cause,
            Some(quire_contract_model::CheckedPackageRefusalCause::InvalidDerivedShape),
            "{edit}"
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some("/semantic_graph/nodes/2/node_id".to_owned()),
            "{edit}"
        );
    }
}

fn package_with_independent_requests() -> Value {
    let mut package = local_package();
    let boolean = package["semantic_graph"]["nodes"][0].clone();
    let boolean_id = boolean["node_id"].clone();
    let mut integer = boolean.clone();
    integer["node_id"] = json!({"package":"test/families","path":["i"],"ordinal":0});
    integer["semantic_type"] = integer["node_id"].clone();
    integer["semantic_form"] = json!("integer");
    let mut reference = boolean;
    reference["node_id"] = json!({"package":"test/families","path":["reference-long"],"ordinal":0});
    reference["semantic_type"] = boolean_id.clone();
    reference["node_tag"] = json!("expression");
    reference["semantic_form"] = json!("reference");
    reference["body"] = json!({"term":"reference","target":boolean_id});
    reference["dependencies"] = json!([boolean_id]);
    package["semantic_graph"]["nodes"] =
        json!([package["semantic_graph"]["nodes"][0], integer, reference]);
    let original = package["source_map"][0].clone();
    let mut integer_map = original.clone();
    integer_map["node_id"] = package["semantic_graph"]["nodes"][1]["node_id"].clone();
    let mut reference_map = original.clone();
    reference_map["node_id"] = package["semantic_graph"]["nodes"][2]["node_id"].clone();
    package["source_map"] = json!([original, integer_map, reference_map]);
    package
}

fn admit(value: &Value) -> Box<CheckedPackageV2> {
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let bytes = quire_canonical::to_vec(value, Limits::new(u64::MAX)).expect("fixture bytes");
    match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("structural package must admit, got {other:?}"),
    }
}

fn encode(package: &CheckedPackageV2, limit: u64) -> Result<Vec<u8>, Error> {
    encode_checked_package_v2(
        package.name(),
        package.lock(),
        package.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: package.source_map(),
            refinement_step_rows: package.refinement_step_rows(),
        },
        package.capability_report(),
        package.diagnostics(),
        Limits::new(limit),
    )
}

/// The independently authored path is not computed from a body or digest.
#[trace("TC-048", "FR-038-AC-89")]
#[test]
fn structural_id_and_complete_body_survive_reading() {
    let wire = local_package();
    let package = admit(&wire);
    let expected = json!({"package":"test/families","path":["builtin/boolean"],"ordinal":0});
    let nodes = wire["semantic_graph"]["nodes"].as_array().expect("nodes");
    assert_eq!(package.name(), "test/families");
    assert_eq!(package.graph().nodes.len(), 1);
    assert_eq!(nodes[0]["node_id"], expected);
    let retained = serde_json::to_value(&package.graph().nodes[0]).expect("retained node");
    assert_eq!(retained["node_id"], expected);
    assert_eq!(retained["body"], nodes[0]["body"]);
    assert_eq!(retained["owner"], nodes[0]["owner"]);
}

#[trace("TC-048", "FR-038-AC-4", "FR-038-AC-367")]
#[test]
fn package_name_is_stable_while_complete_admitted_value_changes() {
    let mut original = local_package();
    let mut second_node = original["semantic_graph"]["nodes"][0].clone();
    second_node["node_id"] =
        json!({"package":"test/families","path":["boolean_second"],"ordinal":0});
    second_node["semantic_type"] = second_node["node_id"].clone();
    let mut second_map = original["source_map"][0].clone();
    second_map["node_id"] = second_node["node_id"].clone();
    original["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second_node);
    original["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second_map);
    let first_id = original["semantic_graph"]["nodes"][0]["node_id"].clone();
    let second_id = original["semantic_graph"]["nodes"][1]["node_id"].clone();
    let mut expression = original["semantic_graph"]["nodes"][0].clone();
    expression["node_id"] =
        json!({"package":"test/families","path":["selected_boolean"],"ordinal":0});
    expression["node_tag"] = json!("expression");
    expression["semantic_form"] = json!("reference");
    expression["semantic_type"] = first_id.clone();
    expression["dependencies"] = json!([first_id.clone()]);
    expression["body"] = json!({"term":"reference","target":first_id});
    let mut expression_map = original["source_map"][0].clone();
    expression_map["node_id"] = expression["node_id"].clone();
    original["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(expression);
    original["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(expression_map);
    let first = admit(&original);
    let mut reordered = original.clone();
    reordered["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .swap(0, 1);
    let second = admit(&reordered);
    assert_eq!(first.name(), second.name());
    assert_ne!(
        first, second,
        "node order is part of the complete package value"
    );

    let mut changed_body = original.clone();
    changed_body["semantic_graph"]["nodes"][2]["body"]["target"] = second_id.clone();
    changed_body["semantic_graph"]["nodes"][2]["dependencies"] = json!([second_id]);
    let fourth = admit(&changed_body);
    assert_eq!(first.name(), fourth.name());
    assert_ne!(
        first, fourth,
        "a valid body edit changes the complete package value"
    );

    let mut changed_source = original;
    changed_source["source_map"][0]["regions"][0]["end"] = json!(42);
    let third = admit(&changed_source);
    assert_eq!(first.name(), third.name());
    assert_ne!(
        first, third,
        "source correspondence is part of the package value"
    );
}
/// Canonical envelope bytes are checked through the public writer against
/// an independently constructed JSON value from the local fixture.
#[trace("TC-048", "FR-038-AC-91")]
#[test]
fn canonical_envelope_bytes_follow_semantic_and_source_edits() {
    let wire = local_package();
    let original = admit(&wire);
    let expected = quire_canonical::to_vec(&wire, Limits::new(u64::MAX)).expect("expected bytes");
    assert_eq!(encode(&original, u64::MAX).expect("writer bytes"), expected);

    let mut semantic = wire.clone();
    semantic["semantic_graph"]["nodes"][0]["semantic_form"] = json!("integer");
    let semantic_package = admit(&semantic);
    let semantic_bytes = encode(&semantic_package, u64::MAX).expect("semantic bytes");
    assert_ne!(semantic_bytes, expected);
    assert_eq!(semantic_package.source_map(), original.source_map());
    assert_eq!(semantic_package.name(), original.name());

    let mut source = wire;
    source["source_map"][0]["regions"][0]["start"] = json!(100);
    source["source_map"][0]["regions"][0]["end"] = json!(101);
    let source_package = admit(&source);
    let source_bytes = encode(&source_package, u64::MAX).expect("source bytes");
    assert_ne!(source_bytes, expected);
    assert_ne!(source_bytes, semantic_bytes);
    assert_eq!(source_package.graph().nodes, original.graph().nodes);
}

/// The typed node and complete envelope both admit at their exact canonical
/// byte ceilings and fail one byte below without yielding partial bytes.
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn typed_node_and_envelope_encoding_have_exact_byte_limits() {
    let package = admit(&local_package());
    let node = &package.graph().nodes[0];
    let node_bytes = quire_canonical::to_vec(node, Limits::new(u64::MAX)).expect("node bytes");
    assert_eq!(
        quire_canonical::to_vec(node, Limits::new(node_bytes.len() as u64))
            .expect("exact node ceiling"),
        node_bytes
    );
    assert!(matches!(
        quire_canonical::to_vec(node, Limits::new(node_bytes.len() as u64 - 1)),
        Err(Error::Limit(limit)) if limit.kind == LimitKind::CanonicalBytes
    ));

    let bytes = encode(&package, u64::MAX).expect("envelope bytes");
    assert_eq!(
        encode(&package, bytes.len() as u64).expect("exact ceiling"),
        bytes
    );
    assert!(matches!(
        encode(&package, bytes.len() as u64 - 1),
        Err(Error::Limit(limit)) if limit.kind == LimitKind::CanonicalBytes
    ));
}

/// One request remains lowered when another independent request exceeds its
/// own package bytes; the aggregate call yields no partial package.
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn multi_request_lowering_preserves_independent_sibling_on_local_byte_failure() {
    let wire = package_with_independent_requests();
    let bytes = quire_canonical::to_vec(&wire, Limits::new(u64::MAX)).expect("input bytes");
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.bytes = bytes.len() as u64;
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let CheckedPackageV2ReadResult::Admitted(package) =
        CheckedPackageV2::read(&bytes, limits, &evidence)
    else {
        panic!("input admits at its exact byte limit");
    };
    let requests = [
        package.graph().nodes[1].node_id.clone(),
        package.graph().nodes[2].node_id.clone(),
    ];
    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: package.node_kinds().iter().map(|kind| kind.tag()).collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let local = package.lower_independent_records(&requests, &profile);
    assert!(matches!(local[0], CompleteLoweringRecordV2::Lowered { .. }));
    assert!(matches!(
        local[1],
        CompleteLoweringRecordV2::Failed {
            limit_kind: CheckedPackageLimit::Bytes,
            limit,
            consumed,
            ..
        } if limit == limits.bytes && consumed > limit
    ));

    let whole = package.lower(&requests, &profile);
    assert!(whole.records.iter().all(|record| matches!(
        record,
        CompleteLoweringRecordV2::Failed {
            limit_kind: CheckedPackageLimit::Bytes,
            ..
        }
    )));
    assert!(whole.package.lowered().is_empty());
    assert!(whole.package.dependencies().is_empty());
    assert!(whole.package.canonical_bytes().is_none());
}
