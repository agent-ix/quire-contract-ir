// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-035-AC-5/TC-047 and QSpec FR-330: the single six-field package value.

use crate::support::checked_package::{
    self, canonical, evidence_for, mint_ungrouped_structural_keys, settle, sha256_hex,
    structural_key, typed_node_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    ContractDependencies, ContractDispositionV1, ContractPackageReadResult, ContractPackageWireV1,
    CONTRACT_IR_SEMANTIC_DOMAIN, CONTRACT_PACKAGE_VERSION,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn id(prefix: &str) -> CheckedNodeId {
    if matches!(prefix, "1010" | "dddd") {
        let (tag, form) = if prefix == "1010" {
            ("model", "namespace")
        } else {
            ("value", "literal")
        };
        typed_node_id(&structural_key(
            tag,
            form,
            Some(&checked_package::family_key("a3a3")),
            &json!({"term": "aggregate", "members": []}),
        ))
    } else {
        typed_node_id(&checked_package::family_key(prefix))
    }
}

fn missing() -> CheckedNodeId {
    typed_node_id(&"0123456789abcdef".repeat(4))
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
    mint_ungrouped_structural_keys(&mut value);
    settle(&mut value);
    value
}

fn profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        reference: quire_contract_model::LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
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

fn projection(value: &Value) -> Value {
    json!({
        "contract_version": value["contract_version"],
        "package": value["package"],
        "semantic_graph": value["semantic_graph"],
        "feature_accounting": value["feature_accounting"],
    })
}

fn read(bytes: &[u8]) -> ContractPackageReadResult {
    ContractPackageWireV1::read(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    )
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn one_call_emits_one_six_field_package_with_total_accounting_and_source_map() {
    let input = mixed_fixture();
    let admitted = admit(&input);
    let result = admitted.lower(&request(), &profile());
    let package = result.package.wire().expect("wire").wire();
    assert_eq!(package.contract_version.as_ref(), CONTRACT_PACKAGE_VERSION);
    assert_eq!(package.package.source.package_id, *admitted.package_id());
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
    assert_eq!(
        read_package.identity(),
        result.package.package_id().expect("identity")
    );
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn identity_covers_exact_semantic_projection_and_excludes_valid_source_coordinates() {
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
            "identity",
            "package",
            "semantic_graph",
            "source_map"
        ]
    );
    let identity = first.package.wire().expect("wire").identity();
    assert_eq!(identity.domain.as_ref(), CONTRACT_IR_SEMANTIC_DOMAIN);
    assert_eq!(
        identity.digest.as_ref(),
        sha256_hex(&canonical(&projection(&wire)))
    );

    let mut reversed = request();
    reversed.reverse();
    let reordered = lower(&input, &reversed);
    assert_ne!(
        first.package.package_id(),
        reordered.package.package_id(),
        "request order is included accounting"
    );
    let mut relocated = input;
    relocated["source_map"][3]["regions"][0]["end"] = json!(5);
    let changed = lower(&relocated, &request());
    assert_ne!(bytes(&first), bytes(&changed));
    assert_eq!(first.package.package_id(), changed.package.package_id());
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn reader_refuses_malformed_outer_shape_and_stale_own_identity() {
    use quire_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    let output = lower(&mixed_fixture(), &request());
    let wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    let mut missing = wire.clone();
    missing
        .as_object_mut()
        .expect("object")
        .remove("source_map");
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&missing)) else {
        panic!("missing outer member must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::MissingMember))
    );
    assert_eq!(refusal.path.expect("root pointer").as_str(), "");
    let mut extra = wire.clone();
    extra["types"] = json!([]);
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&extra)) else {
        panic!("extra outer member must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::UnknownMember))
    );
    assert_eq!(refusal.path.expect("object pointer").as_str(), "");
    let mut stale = wire;
    stale["identity"]["digest"] = json!("0".repeat(64));
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&stale)) else {
        panic!("stale own identity must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue))
    );
    assert_eq!(
        refusal.path.expect("digest pointer").as_str(),
        "/identity/digest"
    );

    let mut wrong_domain: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    wrong_domain["identity"]["domain"] = json!("quire.package.semantic/v2");
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wrong_domain)) else {
        panic!("wrong identity domain must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue))
    );
    assert_eq!(
        refusal.path.expect("domain pointer").as_str(),
        "/identity/domain"
    );
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
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
        refusal.path.expect("object pointer").as_str(),
        "/package/lowering_profile"
    );
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn reader_locates_malformed_nested_source_value() {
    use quire_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    wire["package"]["source"]["lock"]["sources"][0]["digest"] = json!(4);
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wire)) else {
        panic!("malformed nested source value must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::WrongValueKind))
    );
    assert_eq!(
        refusal.path.expect("member pointer").as_str(),
        "/package/source/lock/sources/0/digest"
    );
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
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
            refusal.path.expect("role pointer").as_str(),
            "/source_map/nodes/0/occurrences/0/role"
        );
    }
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
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
    assert_eq!(refusal.path.expect("map pointer").as_str(), "/source_map");

    let mut wrong_request = wire;
    wrong_request["feature_accounting"][0]["node_id"] = json!(missing());
    let ContractPackageReadResult::Refused(refusal) = read(&canonical(&wrong_request)) else {
        panic!("request correspondence must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue))
    );
    assert_eq!(
        refusal.path.expect("request pointer").as_str(),
        "/feature_accounting"
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
        refusal.path.expect("region pointer").as_str(),
        "/source_map/nodes/0/occurrences/0/regions/0/start"
    );
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
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

fn rehashed(mut wire: Value) -> Vec<u8> {
    let digest = sha256_hex(&canonical(&projection(&wire)));
    wire["identity"]["digest"] = json!(digest);
    canonical(&wire)
}

fn failed_wire(limit: &str, consumed: &str) -> Value {
    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    wire["feature_accounting"][1] = json!({
        "node_id": id("1010"), "disposition": "failed", "limit_kind": "work",
        "limit": limit, "consumed": consumed,
    });
    wire["source_map"]["refusals"][0]["disposition"] = json!("failed");
    wire
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn canonical_decimal_counters_keep_the_full_u64_range() {
    use quire_contract_model::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
    for counter in ["0", "18446744073709551615"] {
        assert!(matches!(
            read(&rehashed(failed_wire(counter, counter))),
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
            refusal.path.expect("pointer").as_str(),
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

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn defensive_body_payloads_are_closed_and_omitted_pointers_stay_omitted() {
    use quire_contract_model::CheckedPackageRefusalCause;
    let mut wire = failed_wire("0", "0");
    wire["feature_accounting"][1] = json!({
        "node_id": id("1010"), "disposition": "body_incomplete", "body_node_id": id("1010"),
        "incomplete": {"limit_kind": "bytes", "limit": "8", "consumed": "8"},
    });
    wire["source_map"]["refusals"][0]["disposition"] = json!("body_incomplete");
    assert!(matches!(
        read(&rehashed(wire.clone())),
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
        read(&rehashed(wire.clone())),
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
        refusal.path.expect("kind pointer").as_str(),
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
    body["feature_accounting"][1] = json!({
        "node_id": id("1010"), "disposition": "invalid_body", "body_node_id": id("1010"),
        "refusal": {"code": "invalid_package", "path": ""},
    });
    body["source_map"]["refusals"][0]["disposition"] = json!("invalid_body");
    assert!(matches!(
        read(&rehashed(body.clone())),
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
        refusal.path.expect("cause pointer").as_str(),
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
        refusal.path.expect("code pointer").as_str(),
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

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn duplicate_member_and_noncanonical_bytes_refuse_before_identity() {
    use quire_contract_model::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
    let wire = rehashed(failed_wire("0", "0"));
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
    let spaced = format!(" {text}");
    let ContractPackageReadResult::Refused(refusal) = read(spaced.as_bytes()) else {
        panic!("whitespace must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::NoncanonicalWire);
}

/// Trace: FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn selected_dependencies_are_bound_by_library_identity_before_graph_checks() {
    use quire_contract_model::{
        CheckedPackageRefusalCause, CheckedPackageRefusalCode, ContractDependencyInput,
    };
    let dependency = admit(&mixed_fixture());
    let output = lower(&mixed_fixture(), &request());
    let mut wire: Value = serde_json::from_slice(bytes(&output)).expect("wire json");
    wire["package"]["source"]["lock"]["dependency_selections"] = json!([{
        "identity": "lib/a", "package_id": dependency.package_id(),
    }]);
    let bytes = rehashed(wire.clone());
    let mut supplied = ContractDependencies::new();
    supplied.insert(
        "lib/a",
        ContractDependencyInput {
            identity: "lib/a",
            package: &dependency,
        },
    );
    assert!(matches!(
        ContractPackageWireV1::read(&bytes, CheckedPackageReadLimits::bounded(), &supplied),
        ContractPackageReadResult::Admitted(_)
    ));
    let ContractPackageReadResult::Refused(missing) = read(&bytes) else {
        panic!("selection requires input");
    };
    assert_eq!(missing.code, CheckedPackageRefusalCode::MissingImport);
    assert_eq!(
        missing.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        missing.path.expect("path").as_str(),
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
    supplied.insert(
        "lib/a",
        ContractDependencyInput {
            identity: "lib/a",
            package: &dependency,
        },
    );
    wire["package"]["source"]["lock"]["dependency_selections"][0]["package_id"]["digest"] =
        json!("0".repeat(64));
    let stale = rehashed(wire);
    let ContractPackageReadResult::Refused(wrong_package) =
        ContractPackageWireV1::read(&stale, CheckedPackageReadLimits::bounded(), &supplied)
    else {
        panic!("stale package refuses");
    };
    assert_eq!(
        wrong_package.code,
        CheckedPackageRefusalCode::StaleDependency
    );
    assert_eq!(
        wrong_package.cause,
        Some(CheckedPackageRefusalCause::ContentMismatch)
    );
    assert_eq!(
        wrong_package
            .expected()
            .expect("selected identity")
            .digest
            .as_ref(),
        "0".repeat(64)
    );
    assert_eq!(wrong_package.actual(), Some(dependency.package_id()));
}

/// Trace: FR-035-AC-4
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
    let ContractPackageReadResult::Refused(refusal) = read(&rehashed(wire)) else {
        panic!("reference type cannot target a scalar key");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue))
    );
    assert_eq!(
        refusal.path.expect("body pointer").as_str(),
        format!("/semantic_graph/nodes/{position}/body")
    );
}
