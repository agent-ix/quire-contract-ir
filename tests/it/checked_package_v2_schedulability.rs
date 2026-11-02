// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Independent local task claims through the public authenticated reader.

use crate::support::checked_package as fixtures;
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedLiteralRead, CheckedLiteralValue, CheckedNodeId,
    CheckedPackageReadLimits, CheckedPackageSidecarsRef, CheckedPackageSourceCorrespondenceRef,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedSchedulabilityClaimRow,
    CheckedSchedulabilityKind, CheckedSchedulingPolicy, CheckedTaskArrival, CheckedTaskCriticality,
};
use serde_json::{json, Value};

fn aggregate(members: Vec<Value>) -> Value {
    json!({"term":"aggregate", "members":members})
}
fn binding(name: &str, value: Value) -> Value {
    json!({"term":"binding", "name":name, "value":value})
}
fn literal(type_id: &Value, kind: &str, value: &str) -> Value {
    json!({"term":"literal", "type":type_id, "value_kind":kind, "value":value})
}

fn package(policy: &str, response: bool, high: bool, occurrences: u64) -> Value {
    let mut package = fixtures::v2_all_families();
    let find = |form: &str| {
        package["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["semantic_form"] == form)
            .unwrap()["node_id"]
            .clone()
    };
    let boolean = find("boolean");
    let text = find("text");
    let integer = find("integer");
    let mut nominal = fixtures::v2_nominal();
    let old = fixtures::node_id("qualified:Example/Length");
    let time = fixtures::node_id("qualified:Example/Time");
    fixtures::reassign_node_id(&mut nominal, &old, &time);
    for node in nominal["semantic_graph"]["nodes"].as_array_mut().unwrap() {
        if node["semantic_form"] == "dimension" {
            node["declaration"]["qualified_name"] = json!(["Example", "Time"]);
        }
    }
    let unit = nominal["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["semantic_form"] == "unit")
        .unwrap()["node_id"]
        .clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .extend(
            nominal["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|node| matches!(node["semantic_form"].as_str(), Some("unit" | "dimension")))
                .cloned(),
        );
    let word = |s: &str| literal(&text, "text", s);
    let quantity = |s: &str| literal(&unit, "quantity", s);
    let body = aggregate(vec![
        binding(
            "kind",
            word(if response { "response" } else { "schedulable" }),
        ),
        binding("policy", word(policy)),
        binding(
            "task",
            if response {
                word("worker")
            } else {
                aggregate(vec![])
            },
        ),
        binding(
            "bound",
            if response {
                quantity("17/2")
            } else {
                aggregate(vec![])
            },
        ),
        binding(
            "tasks",
            aggregate(vec![aggregate(vec![
                binding("name", word("worker")),
                binding("arrival", word("sporadic")),
                binding("period", quantity("19/2")),
                binding("deadline", quantity("17/2")),
                binding("bcet", quantity("3/2")),
                binding("wcet", quantity("5/2")),
                binding(
                    "wcet_hi",
                    if high {
                        quantity("7/2")
                    } else {
                        aggregate(vec![])
                    },
                ),
                binding("criticality", word(if high { "hi" } else { "lo" })),
                binding(
                    "priority",
                    if policy == "edf" {
                        aggregate(vec![])
                    } else {
                        literal(&integer, "integer", "7")
                    },
                ),
                binding("jitter", quantity("0/1")),
                binding("blocking", quantity("0/1")),
            ])]),
        ),
    ]);
    let id = fixtures::node_id("fixture/schedulability/worker");
    package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "node_id":id, "schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"claim", "semantic_form":"schedulability", "semantic_type":boolean,
        "dependencies":[], "occurrences":(0..occurrences).map(|ordinal| json!({"role":"claim","ordinal":ordinal})).collect::<Vec<_>>(),
        "body":body
    }));
    package["schedulability_claims"] = json!((0..occurrences)
        .map(|occurrence| json!({"declaration":id,"occurrence":occurrence}))
        .collect::<Vec<_>>());
    fixtures::rebuild_source_map(&mut package);
    fixtures::index_fixture_node_ids(&mut package);
    package
}

fn admitted(package: &Value) -> Box<CheckedPackageV2> {
    let result = CheckedPackageV2::read(
        &fixtures::canonical(package),
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(package),
    );
    match result {
        CheckedPackageV2ReadResult::Admitted(read) => read,
        other => panic!("local task claim must admit: {other:?}"),
    }
}

fn exact(read: CheckedLiteralRead<'_>, unit: &CheckedNodeId, numerator: i32, denominator: i32) {
    assert_eq!(read.type_id, unit);
    assert_eq!(
        read.value,
        CheckedLiteralValue::Rational {
            numerator: numerator.into(),
            denominator: denominator.into()
        }
    );
}

fn encode(read: &CheckedPackageV2) -> Vec<u8> {
    encode_checked_package_v2(
        read.name(),
        read.lock(),
        read.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: read.source_map(),
            refinement_step_rows: read.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            checked_term_data: read.checked_term_data(),
            temporal_bindings: read.temporal_bindings(),
            schedulability_claims: read.schedulability_claim_rows(),
        },
        read.capability_report(),
        read.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("complete encoding")
}

#[trace("TC-451", "FR-019-AC-19", "FR-038-AC-396", "FR-038-AC-398")]
#[test]
fn authenticated_views_preserve_exact_premises_and_shared_occurrences() {
    for (policy, expected_policy, response, high) in [
        (
            "fixed-priority",
            CheckedSchedulingPolicy::FixedPriority,
            true,
            false,
        ),
        ("edf", CheckedSchedulingPolicy::Edf, false, false),
        ("amc", CheckedSchedulingPolicy::Amc, false, true),
    ] {
        let wire = package(policy, response, high, 2);
        let read = admitted(&wire);
        let mut claims = read.schedulability_claims();
        assert_eq!(claims.len(), 2);
        let first = claims.next().expect("first occurrence");
        assert_eq!(claims.len(), 1);
        let second = claims.next().expect("second occurrence");
        assert_eq!(claims.len(), 0);
        assert!(claims.next().is_none());
        assert_eq!(first.declaration(), second.declaration());
        assert!(
            std::ptr::eq(first.body(), second.body()),
            "shared node has one borrowed body"
        );
        assert_eq!(first.occurrence().ordinal, 0);
        assert_eq!(second.occurrence().ordinal, 1);
        assert_eq!(first.policy(), expected_policy);
        assert_eq!(
            first.kind(),
            if response {
                CheckedSchedulabilityKind::Response
            } else {
                CheckedSchedulabilityKind::Schedulable
            }
        );
        assert_eq!(first.response_task(), response.then_some("worker"));
        assert_eq!(first.tasks().len(), 1);
        let task = first.tasks().next().expect("worker");
        assert_eq!(task.name(), "worker");
        assert_eq!(task.arrival(), CheckedTaskArrival::Sporadic);
        assert_eq!(
            task.criticality(),
            if high {
                CheckedTaskCriticality::Hi
            } else {
                CheckedTaskCriticality::Lo
            }
        );
        let unit = task.period().type_id;
        for (value, numerator, denominator) in [
            (task.period(), 19, 2),
            (task.deadline(), 17, 2),
            (task.bcet(), 3, 2),
            (task.wcet(), 5, 2),
            (task.jitter(), 0, 1),
            (task.blocking(), 0, 1),
        ] {
            exact(value, unit, numerator, denominator);
        }
        if high {
            exact(task.wcet_hi().expect("high work"), unit, 7, 2);
        } else {
            assert!(task.wcet_hi().is_none());
        }
        if policy == "edf" {
            assert!(task.priority().is_none());
        } else {
            assert_eq!(
                task.priority().expect("priority").value,
                CheckedLiteralValue::Integer(7.into())
            );
        }
        if response {
            exact(first.response_bound().expect("response bound"), unit, 17, 2);
        } else {
            assert!(first.response_bound().is_none());
        }
        let bytes = encode(&read);
        let encoded: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            encoded["schedulability_claims"],
            wire["schedulability_claims"]
        );
        assert_eq!(encoded["semantic_graph"], wire["semantic_graph"]);
        let again = admitted(&encoded);
        assert_eq!(encode(&again), bytes);
        assert_eq!(again.schedulability_claims().len(), 2);
    }
}

#[trace("TC-048", "FR-038-AC-397", "FR-038-AC-399", "FR-051-AC-7")]
#[test]
fn absent_index_stays_absent_and_raw_rows_do_not_authenticate_claims() {
    let wire = fixtures::v2_all_families();
    let read = admitted(&wire);
    assert!(read.schedulability_claim_rows().is_none());
    assert_eq!(read.schedulability_claims().len(), 0);
    let encoded: Value = serde_json::from_slice(&encode(&read)).unwrap();
    assert!(encoded.get("schedulability_claims").is_none());
    assert_eq!(encoded, wire, "absent-index complete package is lossless");
    let mut missing = package("fixed-priority", false, false, 1);
    let row: CheckedSchedulabilityClaimRow =
        serde_json::from_value(missing["schedulability_claims"][0].clone())
            .expect("untrusted row is parseable");
    assert_eq!(row.occurrence, 0);
    missing
        .as_object_mut()
        .unwrap()
        .remove("schedulability_claims");
    fixtures::refresh_identity(&mut missing);
    let result = CheckedPackageV2::read(
        &fixtures::canonical(&missing),
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(&missing),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("raw row cannot replace the missing authenticated index")
    };
    assert_eq!(
        refusal.code,
        quire_contract_model::CheckedPackageRefusalCode::InvalidPackage
    );
    assert_eq!(
        refusal.cause,
        Some(quire_contract_model::CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/schedulability_claims"
    );
}

#[trace("TC-048", "FR-038-AC-399", "FR-051-AC-7")]
#[test]
fn complete_writer_preserves_supplied_index_order_without_repair() {
    let wire = package("fixed-priority", false, false, 2);
    let read = admitted(&wire);
    let mut rows = read
        .schedulability_claim_rows()
        .expect("identity rows")
        .to_vec();
    rows.reverse();
    let bytes = encode_checked_package_v2(
        read.name(),
        read.lock(),
        read.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: read.source_map(),
            refinement_step_rows: read.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            checked_term_data: read.checked_term_data(),
            temporal_bindings: read.temporal_bindings(),
            schedulability_claims: Some(&rows),
        },
        read.capability_report(),
        read.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("writer retains supplied order");
    let encoded: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(encoded["schedulability_claims"][0]["occurrence"], 1);
    assert_eq!(encoded["schedulability_claims"][1]["occurrence"], 0);
    let result = CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(&encoded),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("writer must not silently canonicalize identity rows")
    };
    assert_eq!(
        refusal.code,
        quire_contract_model::CheckedPackageRefusalCode::InvalidPackage
    );
    assert_eq!(
        refusal.cause,
        Some(quire_contract_model::CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/schedulability_claims/1"
    );
}

#[trace("TC-048", "FR-038-AC-397")]
#[test]
fn explicit_empty_identity_index_is_malformed_even_without_claim_nodes() {
    let mut wire = fixtures::v2_all_families();
    wire["schedulability_claims"] = json!([]);
    fixtures::refresh_identity(&mut wire);
    let result = CheckedPackageV2::read(
        &fixtures::canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(&wire),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("an explicitly present empty carrier is malformed")
    };
    assert_eq!(
        refusal.code,
        quire_contract_model::CheckedPackageRefusalCode::MalformedWire
    );
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        "/schedulability_claims"
    );
}

#[trace("TC-048", "FR-038-AC-397", "FR-038-AC-400")]
#[test]
fn carrier_work_is_charged_before_a_row_is_resolved() {
    let mut wire = package("fixed-priority", false, false, 1);
    wire["schedulability_claims"][0]["declaration"]["package"] = json!("foreign");
    fixtures::refresh_identity(&mut wire);
    let bytes = fixtures::canonical(&wire);
    let evidence = fixtures::evidence_for(&wire);
    let read = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        CheckedPackageV2::read(&bytes, limits, &evidence)
    };
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    while low + 1 < high {
        let mid = low + (high - low) / 2;
        match read(mid) {
            CheckedPackageV2ReadResult::Incomplete(_) => low = mid,
            CheckedPackageV2ReadResult::Refused(refusal) => {
                assert_eq!(
                    refusal.code,
                    quire_contract_model::CheckedPackageRefusalCode::InvalidPackage
                );
                assert_eq!(
                    refusal.cause,
                    Some(quire_contract_model::CheckedPackageRefusalCause::InvalidValue)
                );
                assert_eq!(
                    refusal.loci[0].path.as_ref().unwrap().as_str(),
                    "/schedulability_claims/0/declaration"
                );
                high = mid;
            }
            other => panic!("invalid row must refuse once its charge succeeds: {other:?}"),
        }
    }
    let CheckedPackageV2ReadResult::Incomplete(report) = read(high - 1) else {
        panic!("row resolution needs its precharge")
    };
    assert_eq!(
        report.limit_kind,
        quire_contract_model::CheckedPackageLimit::Work
    );
    assert_eq!(
        report.path.as_ref().unwrap().as_str(),
        "/schedulability_claims/0"
    );
    assert_eq!(report.limit, high - 1);
}

fn refuses(
    wire: &Value,
    code: quire_contract_model::CheckedPackageRefusalCode,
    cause: Option<quire_contract_model::CheckedPackageRefusalCause>,
    path: &str,
) {
    let result = CheckedPackageV2::read(
        &fixtures::canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &fixtures::evidence_for(wire),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("{path}: expected refusal, got {result:?}")
    };
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(refusal.loci.len(), 1);
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        path,
        "{refusal:?}"
    );
}

#[trace("TC-048", "FR-038-AC-397", "FR-038-AC-400")]
#[test]
fn carrier_join_checks_have_deterministic_precedence() {
    use quire_contract_model::{
        CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    };
    for case in [
        "absent-local",
        "wrong-kind",
        "wrong-role",
        "wire-before-graph",
        "graph-before-carrier",
        "aggregate-before-carrier",
        "declaration-before-occurrence",
        "duplicate-before-order",
        "every-row-before-coverage",
        "carrier-before-body",
    ] {
        let mut wire = package("fixed-priority", false, false, 2);
        let n = wire["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
        let mut code = Code::InvalidPackage;
        let mut cause = Some(Cause::InvalidValue);
        let path = match case {
            "absent-local" => {
                wire["schedulability_claims"][0]["declaration"]["path"] = json!(["missing-local"]);
                "/schedulability_claims/0/declaration".to_owned()
            }
            "wrong-kind" => {
                wire["schedulability_claims"][0]["declaration"] =
                    wire["semantic_graph"]["nodes"][0]["node_id"].clone();
                "/schedulability_claims/0/declaration".to_owned()
            }
            "wrong-role" => {
                wire["semantic_graph"]["nodes"][n]["occurrences"][0]["role"] = json!("expression");
                "/schedulability_claims/0/occurrence".to_owned()
            }
            "wire-before-graph" => {
                wire["schedulability_claims"][0]["occurrence"] = json!(-1);
                wire["semantic_graph"]["nodes"][n]["dependencies"] =
                    json!([wire["semantic_graph"]["nodes"][0]["node_id"].clone()]);
                code = Code::MalformedWire;
                cause = None;
                "/schedulability_claims/0/occurrence".to_owned()
            }
            "graph-before-carrier" => {
                wire["schedulability_claims"][0]["occurrence"] = json!(99);
                wire["semantic_graph"]["nodes"][n]["dependencies"] =
                    json!([wire["semantic_graph"]["nodes"][0]["node_id"].clone()]);
                code = Code::InvalidSemanticGraph;
                cause = None;
                format!("/semantic_graph/nodes/{n}/dependencies")
            }
            "aggregate-before-carrier" => {
                wire["schedulability_claims"][0]["occurrence"] = json!(99);
                wire["semantic_graph"]["nodes"][n]["body"]["members"] = json!("not-an-array");
                code = Code::MalformedWire;
                cause = None;
                format!("/semantic_graph/nodes/{n}/body/members")
            }
            "declaration-before-occurrence" => {
                wire["schedulability_claims"][0]["declaration"]["package"] = json!("foreign");
                wire["schedulability_claims"][0]["occurrence"] = json!(99);
                "/schedulability_claims/0/declaration".to_owned()
            }
            "duplicate-before-order" => {
                let first = wire["schedulability_claims"][1].clone();
                let second = wire["schedulability_claims"][0].clone();
                wire["schedulability_claims"] = json!([first.clone(), first, second]);
                cause = Some(Cause::ConflictingDefinition);
                "/schedulability_claims/1".to_owned()
            }
            "every-row-before-coverage" => {
                wire["schedulability_claims"][1]["occurrence"] = json!(99);
                "/schedulability_claims/1/occurrence".to_owned()
            }
            "carrier-before-body" => {
                wire["schedulability_claims"][0]["occurrence"] = json!(99);
                wire["semantic_graph"]["nodes"][n]["body"]["members"][1]["value"]["value"] =
                    json!("unknown-policy");
                "/schedulability_claims/0/occurrence".to_owned()
            }
            _ => unreachable!(),
        };
        fixtures::rebuild_source_map(&mut wire);
        fixtures::refresh_identity(&mut wire);
        refuses(&wire, code, cause, &path);
    }
}

#[trace("TC-048", "FR-038-AC-400")]
#[test]
fn exact_measured_work_admits_and_one_less_has_no_admitted_view() {
    let wire = package("amc", false, true, 2);
    let bytes = fixtures::canonical(&wire);
    let evidence = fixtures::evidence_for(&wire);
    let read = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        CheckedPackageV2::read(&bytes, limits, &evidence)
    };
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        match read(middle) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle,
            other => panic!("valid claim changed outcome: {other:?}"),
        }
    }
    let CheckedPackageV2ReadResult::Admitted(exact) = read(high) else {
        panic!("exact measured work must admit")
    };
    assert_eq!(exact.schedulability_claims().len(), 2);
    let CheckedPackageV2ReadResult::Incomplete(report) = read(high - 1) else {
        panic!("one less work returns no authenticated view")
    };
    assert_eq!(
        report.limit_kind,
        quire_contract_model::CheckedPackageLimit::Work
    );
    assert_eq!(report.limit, high - 1);
}

#[trace("TC-451", "FR-038-AC-399", "FR-051-AC-7")]
#[test]
fn complete_claim_writer_has_exact_canonical_byte_boundary() {
    let wire = package("fixed-priority", true, false, 2);
    let read = admitted(&wire);
    let expected = fixtures::canonical(&wire);
    let length = u64::try_from(expected.len()).unwrap();
    let write = |limit| {
        encode_checked_package_v2(
            read.name(),
            read.lock(),
            read.graph(),
            CheckedPackageSourceCorrespondenceRef {
                source_map: read.source_map(),
                refinement_step_rows: read.refinement_step_rows(),
            },
            CheckedPackageSidecarsRef {
                checked_term_data: read.checked_term_data(),
                temporal_bindings: read.temporal_bindings(),
                schedulability_claims: read.schedulability_claim_rows(),
            },
            read.capability_report(),
            read.diagnostics(),
            quire_canonical::Limits::new(limit),
        )
    };
    assert_eq!(write(length).expect("exact canonical limit"), expected);
    match write(length - 1) {
        Err(quire_canonical::Error::Limit(limit)) => {
            assert_eq!(limit.kind, quire_canonical::LimitKind::CanonicalBytes);
            assert_eq!(limit.bound, length - 1);
        }
        other => panic!("one less byte returns typed limit without output: {other:?}"),
    }
}

#[trace("TC-451", "FR-019-AC-19", "FR-038-AC-398")]
#[test]
fn sparse_high_occurrence_ordinals_resolve_independently_of_array_position() {
    let mut wire = package("fixed-priority", false, false, 2);
    let n = wire["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
    wire["semantic_graph"]["nodes"][n]["occurrences"] = json!([
        {"role":"claim","ordinal":9_000_001},
        {"role":"claim","ordinal":47},
    ]);
    wire["schedulability_claims"][0]["occurrence"] = json!(47);
    wire["schedulability_claims"][1]["occurrence"] = json!(9_000_001);
    fixtures::rebuild_source_map(&mut wire);
    fixtures::refresh_identity(&mut wire);
    let read = admitted(&wire);
    let claims = read.schedulability_claims().collect::<Vec<_>>();
    assert_eq!(claims.len(), 2);
    assert_eq!(claims[0].occurrence().ordinal, 47);
    assert_eq!(claims[1].occurrence().ordinal, 9_000_001);
    assert!(std::ptr::eq(claims[0].body(), claims[1].body()));
    assert_eq!(
        serde_json::from_slice::<Value>(&encode(&read)).unwrap(),
        wire
    );
}
