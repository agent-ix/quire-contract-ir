// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-935 source-selection vectors from the authoritative QSpec checkout.

use crate::fixture_support as checked_package;
use checked_package::{canonical, evidence_for, v2_nominal};
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
    let package = admit(&value);
    let request = serde_json::from_value(value["semantic_graph"]["nodes"][0]["node_id"].clone())
        .expect("typed first node key");
    (value, package, request)
}

fn missing_node_id() -> CheckedNodeId {
    serde_json::from_value(json!({
        "package": "test/missing",
        "path": ["Absent"],
        "ordinal": 0
    }))
    .expect("valid absent structural id")
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

fn source_key_ref(wire: &Value, node_id: &CheckedNodeId) -> Value {
    let node_id = serde_json::to_value(node_id).expect("requested node id");
    let source_keys = wire["source_keys"].as_array().expect("source keys");
    let index = source_keys
        .iter()
        .position(|key| key == &node_id)
        .expect("requested node has a source key");
    json!({"source_key":index})
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
            "node_id":source_key_ref(&emitted, request), "disposition":"failed", "limit_kind":"work",
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
    let missing = missing_node_id();

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
                    &json!({"node_id": source_key_ref(&serialized, &requests[index]), "disposition": "invalid_input"})
                );
                let expected = source["source_map"]
                    .as_array()
                    .expect("source entries")
                    .iter()
                    .filter(|entry| entry["node_id"] == json!(requests[index]))
                    .map(|entry| {
                        let mut entry = entry.clone();
                        entry["node_id"] = source_key_ref(&serialized, &requests[index]);
                        entry
                    })
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
    let absent = missing_node_id();
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

    let (matching_source, admitted, _) = selected_nominal(vec![t.clone()]);
    let matching_request =
        serde_json::from_value(matching_source["semantic_graph"]["nodes"][1]["node_id"].clone())
            .expect("self-typed declaration key");
    let admits_at = |limit| {
        matches!(
            admitted
                .lower(
                    std::slice::from_ref(&matching_request),
                    &with_requirements(std::slice::from_ref(&t), limit),
                )
                .records
                .first(),
            Some(CompleteLoweringRecordV2::Lowered { .. })
        )
    };
    let mut exact_work = 1_u64;
    while !admits_at(exact_work) {
        exact_work = exact_work
            .checked_mul(2)
            .expect("finite fixture work bound");
    }
    let mut below = exact_work / 2;
    while below + 1 < exact_work {
        let middle = below + (exact_work - below) / 2;
        if admits_at(middle) {
            exact_work = middle;
        } else {
            below = middle;
        }
    }
    assert!(exact_work > 0);
    let exact = with_requirements(std::slice::from_ref(&t), exact_work);
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
    let failed = admitted
        .lower(
            std::slice::from_ref(&matching_request),
            &with_requirements(std::slice::from_ref(&t), exact_work - 1),
        )
        .records
        .remove(0);
    let CompleteLoweringRecordV2::Failed {
        limit_kind: CheckedPackageLimit::Work,
        limit,
        consumed,
        ..
    } = failed
    else {
        panic!("one less than the minimum admitting work must fail: {failed:?}");
    };
    assert_eq!(limit, exact_work - 1);
    assert!(consumed > limit, "the first unpaid charge is reported");
    assert_tc_935_failed_work(
        &matching_source,
        &admitted,
        &matching_request,
        std::slice::from_ref(&t),
        limit,
        consumed,
    );
}

const DEPENDENCY_VECTORS: &str = "dependency-selection-vectors.json";

fn dependency_vectors(path: &std::path::Path) -> Result<Value, String> {
    let value = super::read_json(path)?;
    validate_dependency_vectors(&value)?;
    Ok(value)
}

fn validate_dependency_vectors(value: &Value) -> Result<(), String> {
    if value["base"] != "fixtures/positive-all-families.json" {
        return Err("dependency vector base is missing or unsupported".into());
    }
    let selections = value["dependency_selections"]
        .as_array()
        .filter(|rows| !rows.is_empty())
        .ok_or("missing dependency selections")?;
    for selection in selections {
        if selection.as_object().is_none_or(|entry| entry.len() != 1)
            || selection["identity"].as_str().is_none_or(str::is_empty)
        {
            return Err("published dependency selections must be closed identity entries".into());
        }
    }
    for family in ["entry_mutations", "order_vectors", "supply_vectors"] {
        let rows = value[family]
            .as_array()
            .filter(|rows| !rows.is_empty())
            .ok_or_else(|| format!("missing or empty {family}"))?;
        let mut ids = std::collections::BTreeSet::new();
        for row in rows {
            let id = row["id"]
                .as_str()
                .filter(|id| !id.is_empty())
                .ok_or("missing vector id")?;
            if !ids.insert(id) {
                return Err(format!("duplicate {family} vector {id}"));
            }
            let expected = row["outcome"]
                .as_str()
                .ok_or_else(|| format!("{id}: missing outcome"))?;
            if expected != "admitted" && !expected.starts_with("refused:") {
                return Err(format!("{id}: missing expected refusal or admission"));
            }
            match family {
                "entry_mutations" if !row["entry"].is_object() => {
                    return Err(format!("{id}: missing entry mutation"))
                }
                "order_vectors"
                    if row["dependency_selections"]
                        .as_array()
                        .is_none_or(Vec::is_empty) =>
                {
                    return Err(format!("{id}: missing ordered selections"))
                }
                "supply_vectors" if row["entries"].as_array().is_none_or(Vec::is_empty) => {
                    return Err(format!("{id}: missing supplied selections"))
                }
                _ => {}
            }
            if family != "entry_mutations"
                && expected.starts_with("refused:")
                && row["locus"].as_str().is_none()
                && row["loci"].as_array().is_none_or(Vec::is_empty)
            {
                return Err(format!("{id}: missing expected refusal loci"));
            }
        }
    }
    for (family, required) in [
        (
            "entry_mutations",
            &[
                "definition-selection-shape",
                "definition-ref-shape",
                "selection-carries-version",
            ][..],
        ),
        (
            "order_vectors",
            &["utf8-byte-order", "duplicate-identity", "misordered"][..],
        ),
        (
            "supply_vectors",
            &[
                "both-entries-unsupplied",
                "first-supplied-second-absent",
                "all-bound",
            ][..],
        ),
    ] {
        for id in required {
            if !value[family]
                .as_array()
                .ok_or("missing family")?
                .iter()
                .any(|row| row["id"] == *id)
            {
                return Err(format!("missing published vector {id}"));
            }
        }
    }
    Ok(())
}

fn dependency_read(
    wire: &Value,
    dependencies: &[(&str, &CheckedPackageV2)],
) -> Result<CheckedPackageV2ReadResult, String> {
    let mut evidence = quire_contract_model::CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .ok_or("missing features")?
    {
        evidence.support_feature(feature.as_str().ok_or("feature is not text")?);
    }
    if wire["lock"]["definition_selections"]
        .as_array()
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row["authority"] == "example" && row["identity"] == "retention")
        })
    {
        evidence
            .insert_extension_definition(
                quire_contract_model::CheckedArtifactRef {
                    authority: "example".into(),
                    identity: "retention".into(),
                },
                super::retention_definition(),
            )
            .map_err(|error| error.to_string())?;
    }
    for root in wire["semantic_graph"]["nodes"]
        .as_array()
        .ok_or("missing nodes")?
        .iter()
        .filter(|node| node["semantic_form"] == "model_root")
    {
        let identity = root["owner"]["identity"]
            .as_str()
            .ok_or("model root owner")?;
        let document = super::proposal_dir_of(&super::fixtures_dir()).join(super::DOMAIN_DOCUMENT);
        let document = super::read_json(&document)?;
        evidence
            .insert_domain_package_document(identity, canonical(&document))
            .map_err(|error| error.to_string())?;
    }
    for (identity, package) in dependencies {
        evidence
            .insert_dependency_package(*identity, (*package).clone())
            .map_err(|error| error.to_string())?;
    }
    Ok(CheckedPackageV2::read(
        &canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ))
}

// Retain one published scalar and its source occurrence. This is a test
// transformation of the live positive, not a local copy of that fixture.
fn scalar_dependency(base: &Value, name: &str) -> Value {
    let mut wire = base.clone();
    let scalar = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("published Boolean")
        .clone();
    let id = scalar["node_id"].clone();
    wire["semantic_graph"]["nodes"] = json!([scalar]);
    wire["source_map"] = json!(wire["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .filter(|source| source["node_id"] == id)
        .cloned()
        .collect::<Vec<_>>());
    let renamed = json!({"package":name,"path":id["path"],"ordinal":id["ordinal"]});
    checked_package::reassign_node_id(&mut wire, &id, &renamed);
    wire["name"] = json!(name);
    wire["lock"]["definition_selections"] = json!([]);
    wire["lock"]["dependency_selections"] = json!([]);
    checked_package::refresh_temporal_bindings(&mut wire);
    wire
}

fn admitted_dependency(wire: &Value) -> CheckedPackageV2 {
    match dependency_read(wire, &[]).expect("dependency evidence") {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        outcome => panic!("scalar dependency must admit: {outcome:?}"),
    }
}

fn dependency_outcome(
    wire: &Value,
    supplied: &[(&str, &CheckedPackageV2)],
    row: &Value,
) -> Result<(), String> {
    let expected = row["outcome"].as_str().ok_or("missing expected outcome")?;
    let id = row["id"].as_str().ok_or("missing vector id")?;
    let outcome = dependency_read(wire, supplied)?;
    match outcome {
        CheckedPackageV2ReadResult::Admitted(package) => {
            if expected != "admitted" {
                return Err(format!("{id}: expected {expected}, unexpectedly admitted"));
            }
            if serde_json::to_value(&package.lock().dependency_selections)
                .map_err(|error| error.to_string())?
                != wire["lock"]["dependency_selections"]
            {
                return Err(format!("{id}: selected names changed"));
            }
            let expected_dependencies = wire["lock"]["dependency_selections"]
                .as_array()
                .ok_or("selections")?
                .iter()
                .map(|entry| {
                    let name = entry["identity"].as_str().ok_or("selection identity")?;
                    supplied
                        .iter()
                        .find(|(identity, _)| *identity == name)
                        .map(|(_, package)| *package)
                        .ok_or("admitted missing supply")
                })
                .collect::<Result<Vec<_>, &str>>()?;
            if package.direct_dependencies().collect::<Vec<_>>() != expected_dependencies {
                return Err(format!(
                    "{id}: complete supplied values were not retained in selected order"
                ));
            }
        }
        CheckedPackageV2ReadResult::Refused(refusal) => {
            let actual = match refusal.cause {
                Some(cause) => format!(
                    "refused:{}/{}",
                    super::code_word(refusal.code),
                    super::cause_word(cause)
                ),
                None => format!("refused:{}", super::code_word(refusal.code)),
            };
            if actual != expected {
                return Err(format!(
                    "{id}: expected {expected}, got {actual}: {refusal:?}"
                ));
            }
            let expected_loci = if let Some(paths) = row["loci"].as_array() {
                Some(
                    paths
                        .iter()
                        .map(|path| path.as_str().ok_or("locus is not text").map(str::to_owned))
                        .collect::<Result<Vec<_>, _>>()?,
                )
            } else {
                row["locus"].as_str().map(|path| vec![path.to_owned()])
            };
            let actual_loci = refusal
                .loci
                .iter()
                .map(|locus| {
                    locus
                        .path
                        .as_ref()
                        .map(ToString::to_string)
                        .ok_or("refusal has no path")
                })
                .collect::<Result<Vec<_>, _>>()?;
            if actual_loci.is_empty() {
                return Err(format!("{id}: refusal has no locus"));
            }
            if expected_loci
                .as_ref()
                .is_some_and(|paths| paths != &actual_loci)
            {
                return Err(format!(
                    "{id}: expected {expected_loci:?}, got {actual_loci:?}"
                ));
            }
        }
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            return Err(format!("{id}: unexpected incomplete {incomplete:?}"))
        }
        CheckedPackageV2ReadResult::CatalogUnavailable(error) => {
            return Err(format!("{id}: production operation catalog unavailable: {error:?}"));
        }
    }
    Ok(())
}

fn run_dependency_vectors(base: &Value, vectors: &Value) -> Result<(), String> {
    validate_dependency_vectors(vectors)?;
    let mut positive = base.clone();
    positive["lock"]["dependency_selections"] = vectors["dependency_selections"].clone();
    let published_supplies = vectors["dependency_selections"]
        .as_array()
        .ok_or("published selections")?
        .iter()
        .map(|entry| {
            let identity = entry["identity"].as_str().ok_or("selected identity")?;
            Ok((
                identity.to_owned(),
                admitted_dependency(&scalar_dependency(base, identity)),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let supplied = published_supplies
        .iter()
        .map(|(identity, package)| (identity.as_str(), package))
        .collect::<Vec<_>>();
    dependency_outcome(
        &positive,
        &supplied,
        &json!({"id":"published-selection-positive", "outcome":"admitted"}),
    )?;
    for family in ["entry_mutations", "order_vectors", "supply_vectors"] {
        for row in vectors[family].as_array().ok_or("vector family")? {
            let mut wire = base.clone();
            let (entries, bound) = match family {
                "entry_mutations" => (vec![row["entry"].clone()], Vec::new()),
                "order_vectors" => {
                    let entries = row["dependency_selections"]
                        .as_array()
                        .ok_or("ordered entries")?
                        .clone();
                    let bound = entries
                        .iter()
                        .map(|entry| {
                            entry["identity"]
                                .as_str()
                                .ok_or("ordered identity")
                                .map(str::to_owned)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    (entries, bound)
                }
                "supply_vectors" => {
                    let mut entries = Vec::new();
                    let mut bound = Vec::new();
                    for entry in row["entries"].as_array().ok_or("supply entries")? {
                        let identity = entry["identity"].as_str().ok_or("supplied identity")?;
                        entries.push(json!({"identity":identity}));
                        match entry["supply"].as_str().ok_or("missing supply mode")? {
                            "bound" => bound.push(identity.to_owned()),
                            "absent" => {}
                            other => return Err(format!("unknown supply mode {other}")),
                        }
                    }
                    (entries, bound)
                }
                _ => return Err("unknown vector family".into()),
            };
            wire["lock"]["dependency_selections"] = json!(entries);
            let mut admitted = std::collections::BTreeMap::new();
            for identity in bound {
                admitted
                    .entry(identity.clone())
                    .or_insert_with(|| admitted_dependency(&scalar_dependency(base, &identity)));
            }
            let supplied = admitted
                .iter()
                .map(|(identity, package)| (identity.as_str(), package))
                .collect::<Vec<_>>();
            dependency_outcome(&wire, &supplied, row)?;
        }
    }
    Ok(())
}

#[trace("TC-226", "FR-038-AC-113")]
#[test]
fn published_dependency_selections_supply_and_diamonds_fail_closed() {
    let fixtures = super::fixtures_dir();
    let vectors = dependency_vectors(&super::proposal_dir_of(&fixtures).join(DEPENDENCY_VECTORS))
        .expect("current published dependency selection vectors");
    let base = super::read_json(&fixtures.join("positive-all-families.json"))
        .expect("current published base package");
    run_dependency_vectors(&base, &vectors).unwrap_or_else(|why| panic!("{why}"));
    // Both branches must agree on the complete retained leaf, not just its name.
    let leaf_wire = scalar_dependency(&base, "test/leaf");
    let leaf = admitted_dependency(&leaf_wire);
    let mut different_wire = leaf_wire.clone();
    different_wire["source_map"][0]["regions"][0]["end"] = json!(2);
    let different = admitted_dependency(&different_wire);
    assert_eq!(leaf.name(), different.name());
    assert_eq!(leaf.graph(), different.graph());
    assert_ne!(leaf.source_map(), different.source_map());
    let mut left_wire = scalar_dependency(&base, "test/left");
    left_wire["lock"]["dependency_selections"] = json!([{"identity":leaf.name()}]);
    let CheckedPackageV2ReadResult::Admitted(left) =
        dependency_read(&left_wire, &[(leaf.name(), &leaf)]).expect("left read")
    else {
        panic!("left branch must admit");
    };
    for (right_leaf, expected) in [
        (&leaf, "admitted"),
        (&different, "refused:invalid_package/conflicting-definition"),
    ] {
        let mut right_wire = scalar_dependency(&base, "test/right");
        right_wire["lock"]["dependency_selections"] = json!([{"identity":right_leaf.name()}]);
        let CheckedPackageV2ReadResult::Admitted(right) =
            dependency_read(&right_wire, &[(right_leaf.name(), right_leaf)]).expect("right read")
        else {
            panic!("right branch must admit before closure composition");
        };
        let mut root = scalar_dependency(&base, "test/root");
        root["lock"]["dependency_selections"] =
            json!([{"identity":left.name()},{"identity":right.name()}]);
        let mut row = json!({"id":"complete-value-diamond", "outcome":expected});
        if expected != "admitted" {
            row["loci"] = json!([
                "/lock/dependency_selections/0/lock/dependency_selections/0",
                "/lock/dependency_selections/1/lock/dependency_selections/0"
            ]);
        }
        dependency_outcome(&root, &[(left.name(), &left), (right.name(), &right)], &row)
            .unwrap_or_else(|why| panic!("{why}"));
    }

    let scratch = std::env::temp_dir().join(format!(
        "ir-dependency-vector-inputs-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&scratch).expect("scratch directory");
    assert!(
        dependency_vectors(&scratch.join(DEPENDENCY_VECTORS)).is_err(),
        "missing vectors cannot pass"
    );
    std::fs::write(scratch.join(DEPENDENCY_VECTORS), b"not JSON")
        .expect("malformed vector control");
    assert!(
        dependency_vectors(&scratch.join(DEPENDENCY_VECTORS)).is_err(),
        "malformed vectors cannot pass"
    );
    std::fs::remove_dir_all(scratch).expect("remove scratch inputs");
    for family in ["entry_mutations", "order_vectors", "supply_vectors"] {
        let mut missing = vectors.clone();
        missing.as_object_mut().expect("vectors").remove(family);
        assert!(
            validate_dependency_vectors(&missing).is_err(),
            "missing {family} cannot pass"
        );
        let mut missing_outcome = vectors.clone();
        missing_outcome[family][0]
            .as_object_mut()
            .expect("row")
            .remove("outcome");
        assert!(
            validate_dependency_vectors(&missing_outcome).is_err(),
            "missing expected refusal cannot pass"
        );
    }
    let mut wrong_refusal = vectors.clone();
    wrong_refusal["entry_mutations"][0]["outcome"] = json!("admitted");
    let error = run_dependency_vectors(&base, &wrong_refusal)
        .expect_err("unexpected actual refusal cannot pass");
    assert!(
        error.contains("expected admitted, got refused:"),
        "actual refusal must trigger the guard: {error}"
    );
    let mut unexpected_admission = vectors;
    let all_bound = unexpected_admission["supply_vectors"]
        .as_array_mut()
        .expect("supply rows")
        .iter_mut()
        .find(|row| row["id"] == "all-bound")
        .expect("positive supply control");
    all_bound["outcome"] = json!("refused:missing_import/missing-selection");
    all_bound["locus"] = json!("/lock/dependency_selections/0");
    let error = run_dependency_vectors(&base, &unexpected_admission)
        .expect_err("unexpected admission cannot pass");
    assert!(
        error.contains("unexpectedly admitted"),
        "the actual admission must trigger the guard: {error}"
    );
}
