// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Strict CheckedPackage V2 reader: vendored adverse mutations, injected
//! refusals, resource limits, package identity, and nominal node identity.

#[path = "support/checked_package.rs"]
mod checked_package;

use checked_package::{
    apply_patch, canonical, evidence_for, fixture, incomplete, json_depth, locator,
    node_identity_vectors, nominal_package, refresh_identity, refusal, refusal_code, rekey,
    sha256_hex, v2_all_families, v2_nominal, ALL_FAMILIES_READ_WORK, COMPLETE_VALUE_FEATURE,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    read_checked_package, CheckedNodeTag, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
    NominalIdentityPreimage,
};
use serde_json::{json, Value};

type Mutation = Box<dyn Fn(&mut Value)>;

const NOMINAL_PATH: &str = "semantic_graph.nodes.nominal_identity_preimage";

fn read(value: &Value, evidence: &CheckedPackageEvidence) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        evidence,
    )
}

fn admitted(value: &Value) -> Box<CheckedPackageV2> {
    match read(value, &evidence_for(value)) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("expected V2 admission, got {other:?}"),
    }
}

fn refused(value: &Value, evidence: &CheckedPackageEvidence) -> CheckedPackageRefusal {
    refused_bytes(&canonical(value), evidence)
}

fn refused_bytes(bytes: &[u8], evidence: &CheckedPackageEvidence) -> CheckedPackageRefusal {
    match CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), evidence) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected V2 refusal, got {other:?}"),
    }
}

/// Replaces every occurrence of `from` inside `value` with `to`.
fn replace_everywhere(value: &mut Value, from: &Value, to: &Value) {
    if value == from {
        *value = to.clone();
        return;
    }
    match value {
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| replace_everywhere(item, from, to)),
        Value::Object(members) => members
            .values_mut()
            .for_each(|member| replace_everywhere(member, from, to)),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn nominal(code: CheckedPackageRefusalCode) -> CheckedPackageRefusal {
    refusal(code, NOMINAL_PATH)
}

fn dispatch(bytes: &[u8], evidence: &CheckedPackageEvidence) -> CheckedPackageDispatchResult {
    read_checked_package(bytes, CheckedPackageReadLimits::bounded(), evidence)
}

fn assert_dispatch_refused(
    result: CheckedPackageDispatchResult,
    code: CheckedPackageRefusalCode,
    path: &str,
) {
    match result {
        CheckedPackageDispatchResult::Refused(actual) => assert_eq!(actual, refusal(code, path)),
        other => panic!("expected {code:?} at {path}, got {other:?}"),
    }
}

/// The reader parses `contract_version` exactly once and admits only the
/// current contract; every other version, and every malformed document, is
/// refused before any version-specific decoding.
///
/// Tracing: TC-048, FR-038-AC-1
#[trace("TC-048", "FR-038-AC-1")]
#[test]
fn tc_048_reader_refuses_unknown_absent_and_malformed_versions() {
    let fixture = v2_all_families();
    let evidence = evidence_for(&fixture);
    let with_version = |version: Value| {
        let mut value = fixture.clone();
        value["contract_version"] = version;
        canonical(&value)
    };
    for unknown in ["quire.checked-package/v3", "quire.checked-package/v0", ""] {
        assert_dispatch_refused(
            dispatch(&with_version(json!(unknown)), &evidence),
            CheckedPackageRefusalCode::UnknownContractVersion,
            "contract_version",
        );
    }
    for malformed in [json!(2), json!(null), json!(["quire.checked-package/v2"])] {
        assert_dispatch_refused(
            dispatch(&with_version(malformed), &evidence),
            CheckedPackageRefusalCode::MalformedWire,
            "contract_version",
        );
    }
    let mut absent = fixture.clone();
    absent
        .as_object_mut()
        .expect("fixture object")
        .remove("contract_version");
    assert_dispatch_refused(
        dispatch(&canonical(&absent), &evidence),
        CheckedPackageRefusalCode::MalformedWire,
        "contract_version",
    );
    assert_dispatch_refused(
        dispatch(b"[]", &evidence),
        CheckedPackageRefusalCode::MalformedWire,
        "document",
    );
    assert_dispatch_refused(
        dispatch(b"{\"contract_version\":", &evidence),
        CheckedPackageRefusalCode::MalformedWire,
        "document",
    );

    // The strict parse runs once, before any version is selected.
    let bytes = canonical(&fixture);
    let body = std::str::from_utf8(&bytes).expect("UTF-8 fixture");
    let duplicate = format!(
        "{{\"contract_version\":\"quire.checked-package/v0\",{}",
        body.trim_start_matches('{')
    );
    assert!(matches!(
        dispatch(duplicate.as_bytes(), &evidence),
        CheckedPackageDispatchResult::Refused(ref refused)
            if refused.code == CheckedPackageRefusalCode::DuplicateMember
    ));
    let mut spaced = b" ".to_vec();
    spaced.extend_from_slice(&bytes);
    assert_dispatch_refused(
        dispatch(&spaced, &evidence),
        CheckedPackageRefusalCode::NoncanonicalWire,
        "document",
    );

    // A recognized version still admits through the same entry point.
    assert!(matches!(
        dispatch(&bytes, &evidence),
        CheckedPackageDispatchResult::AdmittedV2(_)
    ));
}

/// Tracing: TC-048, FR-038-AC-2
#[trace("TC-048", "FR-038-AC-2")]
#[test]
fn tc_048_v2_reader_refuses_every_vendored_structural_mutation() {
    let adverse = fixture("checked-package-v2/fixtures/adverse.json");
    let mutations = adverse["structural_mutations"]
        .as_array()
        .expect("structural mutations");
    assert_eq!(mutations.len(), 5);
    for base in [v2_all_families(), v2_nominal()] {
        let evidence = evidence_for(&base);
        for mutation in mutations {
            let pointer = mutation["pointer"].as_str().expect("pointer");
            let code = refusal_code(mutation["outcome"].as_str().expect("outcome"));
            let mut mutated = base.clone();
            *mutated.pointer_mut(pointer).expect("pointer target") =
                mutation["replacement"].clone();
            // Refused both as recorded and with the identity re-derived.
            let mut rederived = mutated.clone();
            refresh_identity(&mut rederived);
            for candidate in [&mutated, &rederived] {
                let actual = refused(candidate, &evidence);
                assert_eq!(actual.code, code, "{}", mutation["id"]);
            }
        }
    }
}

/// Tracing: TC-048, FR-038-AC-2
#[trace("TC-048", "FR-038-AC-2")]
#[test]
fn tc_048_v2_reader_refuses_injected_wire_evidence_and_graph_faults() {
    let base = v2_all_families();
    let evidence = evidence_for(&base);
    let bytes = canonical(&base);
    let text = std::str::from_utf8(&bytes).expect("UTF-8 fixture");
    assert_eq!(
        refused_bytes(
            format!(
                "{{\"contract_version\":\"quire.checked-package/v2\",{}",
                &text[1..]
            )
            .as_bytes(),
            &evidence
        )
        .code,
        CheckedPackageRefusalCode::DuplicateMember
    );
    let mut spaced = bytes.clone();
    spaced.push(b'\n');
    assert_eq!(
        refused_bytes(&spaced, &evidence),
        refusal(CheckedPackageRefusalCode::NoncanonicalWire, "document")
    );

    let cases: Vec<(&str, Mutation, CheckedPackageRefusal)> = vec![
        (
            "unknown top-level member",
            Box::new(|v| v["future"] = json!(1)),
            refusal(CheckedPackageRefusalCode::UnknownMember, "document"),
        ),
        (
            "unknown node member",
            Box::new(|v| v["semantic_graph"]["nodes"][0]["future"] = json!(1)),
            refusal(CheckedPackageRefusalCode::UnknownMember, "document"),
        ),
        (
            "wrong member kind",
            Box::new(|v| v["semantic_graph"]["nodes"][0]["dependencies"] = json!("none")),
            refusal(CheckedPackageRefusalCode::MalformedWire, "document"),
        ),
        (
            "missing member",
            Box::new(|v| {
                v["diagnostics"]
                    .as_object_mut()
                    .expect("diagnostics")
                    .remove("entries");
            }),
            refusal(CheckedPackageRefusalCode::MalformedWire, "document"),
        ),
        (
            "null recursion group",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][1]["recursion_group"] = Value::Null;
                refresh_identity(v);
            }),
            refusal(CheckedPackageRefusalCode::MalformedWire, "document"),
        ),
        (
            "empty recursion group",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][1]["recursion_group"] = json!("");
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "semantic_graph.nodes.recursion_group",
            ),
        ),
        (
            "unreported required feature",
            Box::new(|v| v["capability_report"] = json!([])),
            refusal(
                CheckedPackageRefusalCode::UnknownRequiredCapability,
                "capability_report",
            ),
        ),
        (
            "unavailable required feature",
            Box::new(|v| v["capability_report"][0]["disposition"] = json!("unsupported")),
            refusal(
                CheckedPackageRefusalCode::UnknownRequiredCapability,
                "capability_report",
            ),
        ),
        (
            "dangling body reference",
            Box::new(|v| {
                // Not "9".repeat(64): that digest now collides with the
                // fixture's own real node 23 (systems_interface/Flowable).
                v["semantic_graph"]["nodes"][4]["body"]["target"]["digest"] =
                    json!("0123456789abcdef".repeat(4));
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.body.target",
            ),
        ),
        (
            "dangling dependency",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][1]["dependencies"] = json!([{"domain":"quire.checked-semantic-node/v1","digest":"0123456789abcdef".repeat(4)}]);
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.dependencies",
            ),
        ),
        (
            "incomplete source map",
            Box::new(|v| {
                v["source_map"].as_array_mut().expect("source map").pop();
            }),
            refusal(CheckedPackageRefusalCode::InvalidSourceMap, "source_map"),
        ),
        (
            "unlocked source region",
            Box::new(|v| v["source_map"][0]["regions"][0]["source"]["identity"] = json!("other")),
            refusal(
                CheckedPackageRefusalCode::InvalidSourceMap,
                "source_map.regions.source",
            ),
        ),
        (
            "stale projection",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][0]["body"]["value"] = json!(false);
            }),
            refusal(
                CheckedPackageRefusalCode::StaleDependency,
                "identity_preimage.identity_projection",
            ),
        ),
        (
            "stale package id",
            Box::new(|v| v["package_id"]["digest"] = json!("0".repeat(64))),
            refusal(
                CheckedPackageRefusalCode::StaleDependency,
                "package_id.digest",
            ),
        ),
        (
            "unrecognised identity preimage version",
            Box::new(|v| {
                v["identity_preimage"]["version"] = json!("quire.checked-package-id/v1");
                v["package_id"]["digest"] = json!(sha256_hex(&canonical(&v["identity_preimage"])));
            }),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "identity_preimage.version",
            ),
        ),
        (
            "cycle without a recursion group",
            Box::new(|v| {
                let first = v["semantic_graph"]["nodes"][1]["node_id"].clone();
                let second = v["semantic_graph"]["nodes"][2]["node_id"].clone();
                v["semantic_graph"]["nodes"][1]["dependencies"] = json!([second]);
                v["semantic_graph"]["nodes"][2]["dependencies"] = json!([first]);
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.recursion_group",
            ),
        ),
        (
            "cycle split across recursion groups",
            Box::new(|v| {
                let first = v["semantic_graph"]["nodes"][1]["node_id"].clone();
                let second = v["semantic_graph"]["nodes"][2]["node_id"].clone();
                v["semantic_graph"]["nodes"][1]["dependencies"] = json!([second]);
                v["semantic_graph"]["nodes"][2]["dependencies"] = json!([first]);
                v["semantic_graph"]["nodes"][1]["recursion_group"] = json!("left");
                v["semantic_graph"]["nodes"][2]["recursion_group"] = json!("right");
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.recursion_group",
            ),
        ),
        (
            "self dependency without a recursion group",
            Box::new(|v| {
                let own = v["semantic_graph"]["nodes"][1]["node_id"].clone();
                v["semantic_graph"]["nodes"][1]["dependencies"] = json!([own]);
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.recursion_group",
            ),
        ),
    ];
    for (name, mutate, expected) in cases {
        let mut mutated = base.clone();
        mutate(&mut mutated);
        match read(&mutated, &evidence) {
            CheckedPackageV2ReadResult::Refused(actual) => assert_eq!(actual, expected, "{name}"),
            other => panic!("{name}: expected refusal, got {other:?}"),
        }
    }

    // Literal numbers are integers, in node bodies and diagnostic details. A
    // fraction is refused by the schema and the reader alike; a whole-valued
    // float spelling is refused too, as the reader admits integer tokens only.
    // The refusal is the same whether or not serde_json's
    // `arbitrary_precision` is unified into the build.
    let schema = jsonschema::JSONSchema::compile(&fixture("checked-package-v2/schema.json"))
        .expect("vendored schema compiles");
    type Build<'a> = Box<dyn Fn(Value) -> Value + 'a>;
    let self_type = base["semantic_graph"]["nodes"][0]["node_id"].clone();
    let body: Build = Box::new(|literal| {
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][0]["body"] = json!({"term":"literal","type":self_type.clone(),"value_kind":"integer","value":literal});
        refresh_identity(&mut changed);
        changed
    });
    let detail: Build = Box::new(|literal| {
        let mut changed = base.clone();
        let target = base["semantic_graph"]["nodes"][0]["node_id"].clone();
        // FR-208's new `DiagnosticCausePairing` (added by this re-pin) requires
        // `cause_tag: "invalid-value"` to pair with `code: "invalid_package"`,
        // not `"ill_typed"` as this case used before.
        changed["diagnostics"]["entries"] = json!([{
            "stage":"type_checking","code":"invalid_package","cause_tag":"invalid-value",
            "details":[{"term":"literal","type":target,"value_kind":"integer","value":literal}],"loci":[]
        }]);
        changed
    });
    let integer_refusal = refusal(
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        "semantic_graph.nodes.body",
    );
    for build in [body, detail] {
        let integer = build(json!(-7));
        assert!(schema.is_valid(&integer));
        assert!(matches!(
            read(&integer, &evidence),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        let fractional = build(json!(1.5));
        assert!(!schema.is_valid(&fractional));
        for candidate in [fractional, build(json!(2.0))] {
            assert_eq!(refused(&candidate, &evidence), integer_refusal);
        }
    }

    // A cycle is admitted once every member shares one explicit group.
    let mut grouped = base.clone();
    let first = grouped["semantic_graph"]["nodes"][1]["node_id"].clone();
    let second = grouped["semantic_graph"]["nodes"][2]["node_id"].clone();
    grouped["semantic_graph"]["nodes"][1]["dependencies"] = json!([second]);
    grouped["semantic_graph"]["nodes"][2]["dependencies"] = json!([first]);
    grouped["semantic_graph"]["nodes"][1]["recursion_group"] = json!("pair");
    grouped["semantic_graph"]["nodes"][2]["recursion_group"] = json!("pair");
    refresh_identity(&mut grouped);
    assert_eq!(admitted(&grouped).graph().nodes.len(), 26);

    // Evidence the caller must supply: every locked digest and the feature.
    assert_eq!(
        refused(&base, &CheckedPackageEvidence::new()),
        refusal(CheckedPackageRefusalCode::StaleDependency, "lock.sources")
    );
    let mut stale_catalog = evidence_for(&base);
    stale_catalog.insert_artifact_digest(locator(&base["diagnostics"]["catalog"]), "4".repeat(64));
    assert_eq!(
        refused(&base, &stale_catalog),
        refusal(
            CheckedPackageRefusalCode::StaleDependency,
            "diagnostics.catalog"
        )
    );
    let mut byte_evidence = evidence_for(&base);
    byte_evidence.insert_artifact_bytes(locator(&base["lock"]["sources"][0]), b"not the source");
    assert_eq!(
        refused(&base, &byte_evidence),
        refusal(CheckedPackageRefusalCode::StaleDependency, "lock.sources")
    );
    let mut unsupported = CheckedPackageEvidence::new();
    for artifact in checked_package::locked_artifacts(&base) {
        unsupported.insert_artifact_digest(
            locator(&artifact),
            artifact["digest"].as_str().expect("digest"),
        );
    }
    assert_eq!(
        refused(&base, &unsupported),
        refusal(
            CheckedPackageRefusalCode::UnknownRequiredCapability,
            "capability_report"
        )
    );
    unsupported.support_feature(COMPLETE_VALUE_FEATURE);
    assert!(matches!(
        read(&base, &unsupported),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Tracing: TC-048, FR-038-AC-3
#[trace("TC-048", "FR-038-AC-3")]
#[test]
fn tc_048_v2_reader_reports_exact_and_one_over_limits() {
    let mut value = v2_nominal();
    let member = value["semantic_graph"]["nodes"][0]["node_id"].clone();
    value["diagnostics"]["entries"] = json!([{
        "stage": "type_checking",
        "code": "ill_typed",
        "cause_tag": "invalid-value",
        "details": [{"term": "reference", "target": member}],
        "loci": [{"source": value["lock"]["sources"][0], "start": 0, "end": 1}],
    }]);
    let evidence = evidence_for(&value);
    let bytes = canonical(&value);
    let exact = CheckedPackageReadLimits {
        bytes: u64::try_from(bytes.len()).expect("length"),
        depth: json_depth(&value),
        nodes: 4,
        edges: 2,
        occurrences: 8,
        diagnostics: 1,
        // Terms 4 + nominal 11 + graph edges 5 + diagnostic detail 1.
        // Graph edges is 5, not 4: node 0's body now carries a required
        // `literal.type` that (like its `semantic_type`) self-references
        // node 0, adding one more body-target edge into the Tarjan walk.
        work: 21,
    };
    match CheckedPackageV2::read(&bytes, exact, &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => {
            assert_eq!(package.diagnostics().entries.len(), 1);
        }
        other => panic!("exact limits must admit, got {other:?}"),
    }
    type Narrow = fn(&mut CheckedPackageReadLimits) -> u64;
    let narrowings: [(CheckedPackageLimit, Narrow); 7] = [
        (CheckedPackageLimit::Bytes, |l| {
            l.bytes -= 1;
            l.bytes
        }),
        (CheckedPackageLimit::Depth, |l| {
            l.depth -= 1;
            l.depth
        }),
        (CheckedPackageLimit::Nodes, |l| {
            l.nodes -= 1;
            l.nodes
        }),
        (CheckedPackageLimit::Edges, |l| {
            l.edges -= 1;
            l.edges
        }),
        (CheckedPackageLimit::Occurrences, |l| {
            l.occurrences -= 1;
            l.occurrences
        }),
        (CheckedPackageLimit::Diagnostics, |l| {
            l.diagnostics -= 1;
            l.diagnostics
        }),
        (CheckedPackageLimit::Work, |l| {
            l.work -= 1;
            l.work
        }),
    ];
    for (kind, narrow) in narrowings {
        let mut limits = exact;
        let limit = narrow(&mut limits);
        match CheckedPackageV2::read(&bytes, limits, &evidence) {
            CheckedPackageV2ReadResult::Incomplete(actual) => {
                assert_eq!(actual, incomplete(kind, limit, limit + 1), "{kind:?}");
            }
            other => panic!("{kind:?} one over must be incomplete, got {other:?}"),
        }
    }

    let all = v2_all_families();
    let all_bytes = canonical(&all);
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = ALL_FAMILIES_READ_WORK;
    assert!(matches!(
        CheckedPackageV2::read(&all_bytes, limits, &evidence_for(&all)),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    limits.work = ALL_FAMILIES_READ_WORK - 1;
    assert_eq!(
        CheckedPackageV2::read(&all_bytes, limits, &evidence_for(&all)),
        CheckedPackageV2ReadResult::Incomplete(incomplete(
            CheckedPackageLimit::Work,
            ALL_FAMILIES_READ_WORK - 1,
            ALL_FAMILIES_READ_WORK
        ))
    );
}

/// Tracing: TC-048, FR-038-AC-4
#[trace("TC-048", "FR-038-AC-4")]
#[test]
fn tc_048_package_id_covers_exactly_the_identity_preimage() {
    let base = v2_nominal();
    let recorded = admitted(&base).package_id().clone();

    // Edits outside the preimage keep the recorded identity.
    let mut excluded = base.clone();
    excluded["source_map"][0]["regions"][0]["end"] = json!(9);
    excluded["capability_report"]
        .as_array_mut()
        .expect("report")
        .push(json!({"feature":"quire.future/v1","disposition":"unsupported"}));
    excluded["diagnostics"]["entries"] = json!([{
        "stage":"lowering","code":"unimplemented_capability","cause_tag":"unsupported-feature",
        "details":[],"loci":[]
    }]);
    excluded["semantic_graph"]["nodes"][0]["occurrences"] =
        json!([{"role":"declaration","ordinal":0},{"role":"expression","ordinal":1}]);
    excluded["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({
            "node_id": base["semantic_graph"]["nodes"][0]["node_id"],
            "role":"expression","ordinal":1,
            "regions":[{"source": base["lock"]["sources"][1], "start": 0, "end": 1}],
        }));
    assert_eq!(*admitted(&excluded).package_id(), recorded);

    // A capability disposition is outside the preimage.
    let mut disposition = excluded.clone();
    let report = disposition["capability_report"]
        .as_array_mut()
        .expect("report");
    let future = report.len() - 1;
    report[future]["disposition"] = json!("unimplemented");
    assert_eq!(*admitted(&disposition).package_id(), recorded);

    // A raw source digest is outside the preimage: every reference to the
    // source moves with it, and the caller attests the new bytes digest.
    let mut source_digest = base.clone();
    let old_source = base["lock"]["sources"][0].clone();
    let mut new_source = old_source.clone();
    new_source["digest"] = json!("5".repeat(64));
    replace_everywhere(&mut source_digest, &old_source, &new_source);
    assert_ne!(source_digest["lock"], base["lock"]);
    assert_eq!(
        source_digest["identity_preimage"],
        base["identity_preimage"]
    );
    assert_eq!(*admitted(&source_digest).package_id(), recorded);

    // Every preimage member changes the identity once re-derived, and is
    // refused as stale when it is not.
    let included: Vec<(&str, Mutation)> = vec![
        (
            "required feature",
            Box::new(|v| {
                v["lock"]["required_features"] = json!([COMPLETE_VALUE_FEATURE, "quire.extra/v1"]);
                v["capability_report"] = json!([
                    {"feature": COMPLETE_VALUE_FEATURE, "disposition":"available"},
                    {"feature":"quire.extra/v1","disposition":"available"}
                ]);
            }),
        ),
        (
            "edition",
            Box::new(|v| {
                v["lock"]["edition"]["definition"]["digest"] = json!("8".repeat(64));
            }),
        ),
        (
            "recursion group",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][1]["recursion_group"] = json!("solo");
            }),
        ),
        (
            "profile selection",
            Box::new(|v| {
                v["lock"]["profile_selections"] =
                    json!([{"role":"profile","definition": v["lock"]["definition_selections"][0]}]);
            }),
        ),
        (
            // `evidence_for` attests the selected domain package, so the
            // unmirrored refusal is the lock/preimage mismatch, not evidence.
            "model selection",
            Box::new(|v| {
                v["lock"]["model_selections"] = json!([{
                    "identity": "test/orders", "version": "1",
                    "digest_domain": "sha256-jcs", "digest": "5".repeat(64)
                }]);
            }),
        ),
    ];
    for (name, mutate) in included {
        let mut changed = base.clone();
        mutate(&mut changed);
        let stale = refused(&changed, &evidence_for(&changed));
        assert_eq!(
            stale.code,
            CheckedPackageRefusalCode::StaleDependency,
            "{name}"
        );
        if name == "model selection" {
            assert_eq!(
                stale,
                refusal(CheckedPackageRefusalCode::StaleDependency, "lock")
            );
        }
        refresh_identity(&mut changed);
        let mut evidence = evidence_for(&changed);
        evidence.support_feature("quire.extra/v1");
        match read(&changed, &evidence) {
            CheckedPackageV2ReadResult::Admitted(package) => {
                assert_ne!(*package.package_id(), recorded, "{name}");
                assert_eq!(
                    package.package_id().digest.as_ref(),
                    sha256_hex(&canonical(&changed["identity_preimage"])),
                    "{name}"
                );
            }
            other => panic!("{name}: re-derived identity must admit, got {other:?}"),
        }
    }
}

/// Tracing: TC-048, FR-038-AC-5
#[trace("TC-048", "FR-038-AC-5")]
#[test]
fn tc_048_nominal_vectors_rederive_and_admit_as_one_package() {
    let vectors = node_identity_vectors();
    let vectors = vectors["vectors"].as_array().expect("vectors");
    assert_eq!(vectors.len(), 14);
    let mut members = Vec::new();
    for vector in vectors {
        let recorded = vector["sha256"].as_str().expect("sha256");
        let typed: NominalIdentityPreimage =
            serde_json::from_value(vector["preimage"].clone()).expect("typed preimage");
        assert_eq!(
            serde_json::to_value(&typed).expect("round trip"),
            vector["preimage"]
        );
        assert_eq!(
            typed.digest().as_deref(),
            Some(recorded),
            "{}",
            vector["name"]
        );
        assert_eq!(sha256_hex(&canonical(&vector["preimage"])), recorded);
        members.push((vector["preimage"].clone(), recorded.to_owned()));
    }
    let package = nominal_package(&members);
    let admitted = admitted(&package);
    assert_eq!(admitted.graph().nodes.len(), 14);

    // The recorded nominal fixture is exactly this construction over its nodes,
    // so the builder cannot drift from the published shape.
    let recorded = v2_nominal();
    let fixture_members = recorded["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .map(|node| {
            (
                node["nominal_identity_preimage"].clone(),
                node["node_id"]["digest"].as_str().expect("key").to_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(nominal_package(&fixture_members), recorded);
}

/// Tracing: TC-048, FR-038-AC-5
#[trace("TC-048", "FR-038-AC-5")]
#[test]
fn tc_048_invalid_nominal_mutations_refuse_retained_and_rekeyed() {
    let document = node_identity_vectors();
    let vectors = document["vectors"].as_array().expect("vectors");
    let names = vectors
        .iter()
        .map(|vector| vector["name"].as_str().expect("name"))
        .collect::<Vec<_>>();
    let preimages = vectors
        .iter()
        .map(|vector| vector["preimage"].clone())
        .collect::<Vec<_>>();
    let keys = vectors
        .iter()
        .map(|vector| vector["sha256"].as_str().expect("sha256").to_owned())
        .collect::<Vec<_>>();
    let mutations = document["invalid_mutations"].as_array().expect("mutations");
    assert_eq!(mutations.len(), 12);
    for mutation in mutations {
        let name = mutation["name"].as_str().expect("name");
        let base = names
            .iter()
            .position(|candidate| *candidate == mutation["base"].as_str().expect("base"))
            .expect("mutation base vector");
        assert_eq!(
            mutation["retained_sha256"].as_str(),
            Some(keys[base].as_str())
        );
        assert_eq!(
            mutation["expected_code"].as_str(),
            Some("invalid_semantic_graph")
        );
        let mut patched = preimages.clone();
        apply_patch(&mut patched[base], &mutation["patch"]);

        // Retained key: the preimage no longer derives the node key.
        let retained = patched
            .iter()
            .cloned()
            .zip(keys.iter().cloned())
            .collect::<Vec<_>>();
        assert_eq!(
            refused(&nominal_package(&retained), &evidence_for(&v2_nominal())),
            nominal(CheckedPackageRefusalCode::InvalidSemanticGraph),
            "{name} retained"
        );

        // Rekeyed: the key derives, so only a semantic or owner rule refuses.
        if mutation["kind"] != json!("stale_key") {
            let mut rekeyed = patched.clone();
            let fresh = rekey(&mut rekeyed, &keys);
            assert_ne!(fresh[base], keys[base], "{name}");
            let members = rekeyed.into_iter().zip(fresh).collect::<Vec<_>>();
            assert_eq!(
                refused(&nominal_package(&members), &evidence_for(&v2_nominal())),
                nominal(CheckedPackageRefusalCode::InvalidSemanticGraph),
                "{name} rekeyed"
            );
        }
    }
}

/// Declaration checks (`validate_nominal_nodes`) must run, and refuse,
/// before the graph's `dependencies` edges are resolved — the vendored
/// README's normative reader order is "graph-shape, ..., declaration,
/// frame, ..., then nodes in ascending node-id digest order" (the edge
/// resolution `validate_graph`'s Loop 2 performs). A package carrying both
/// defects at once has one determined outcome, not a position-dependent
/// one: this pins that outcome to the declaration defect, at
/// `semantic_graph.nodes.nominal_identity_preimage`, never the generic
/// unresolved-edge refusal a dangling `dependencies` entry would otherwise
/// raise at `semantic_graph.nodes.dependencies`.
///
/// Both defects sit on the same node (the fixture's enum declaration,
/// node index 1): its `nominal_identity_preimage.members` gains an entry
/// without updating `node_id`, so the preimage no longer re-derives the
/// node's own key (`validate_nominal_nodes`'s own defect); its
/// `dependencies` gains an entry naming no real node
/// (`validate_enum_declaration` never inspects `node.dependencies`, unlike
/// the enum-member/dimension/unit preimages, so this entry is invisible to
/// the nominal check and reaches only the later edge-resolution loop, if
/// that loop is ever reached).
///
/// Tracing: TC-048, FR-038-AC-5
#[trace("TC-048", "FR-038-AC-5")]
#[test]
fn tc_048_declaration_defect_is_reported_before_a_dangling_dependency_reference() {
    let mut package = v2_nominal();
    package["semantic_graph"]["nodes"][1]["nominal_identity_preimage"]["members"]
        .as_array_mut()
        .expect("members")
        .push(json!("EXTRA"));
    package["semantic_graph"]["nodes"][1]["dependencies"] = json!([{
        "domain": "quire.checked-semantic-node/v1",
        "digest": "0123456789abcdef".repeat(4),
    }]);
    refresh_identity(&mut package);

    assert_eq!(
        refused(&package, &evidence_for(&package)),
        nominal(CheckedPackageRefusalCode::InvalidSemanticGraph),
        "declaration checks must refuse before the dangling dependency edge is resolved"
    );
}

/// Tracing: TC-048, FR-038-AC-5
#[trace("TC-048", "FR-038-AC-5")]
#[test]
fn tc_048_nominal_cross_field_contradictions_refuse() {
    let base = v2_nominal();
    let member = 0;
    let declaration = 1;
    let unit = 2;
    let dimension = 3;
    let cases: Vec<(&str, Mutation)> = vec![
        (
            "absent required preimage",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][declaration]
                    .as_object_mut()
                    .expect("node")
                    .remove("nominal_identity_preimage");
            }),
        ),
        (
            "swapped preimage kind",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][declaration]["nominal_identity_preimage"] =
                    v["semantic_graph"]["nodes"][member]["nominal_identity_preimage"].clone();
            }),
        ),
        (
            "preimage not deriving the node key",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][unit]["nominal_identity_preimage"]
                    ["qualified_declaration"] = json!(["Example", "centimetre"]);
            }),
        ),
        (
            "unexpected preimage",
            Box::new(move |v| {
                let preimage =
                    v["semantic_graph"]["nodes"][dimension]["nominal_identity_preimage"].clone();
                let nodes = v["semantic_graph"]["nodes"].as_array_mut().expect("nodes");
                nodes[member]["nominal_identity_preimage"] = preimage;
                nodes[member]["semantic_form"] = json!("literal");
                // "literal" is not "enum_value", so this node's retained
                // `declaration`-role occurrence now requires a `declaration`
                // member (FR-208's `DeclarationOccurrenceRule`); add one so
                // this case still isolates the nominal-preimage mismatch it
                // targets, rather than tripping that unrelated rule first.
                nodes[member]["declaration"] = json!({"qualified_name": ["Example", "Member"]});
            }),
        ),
        (
            "member body case",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][member]["body"]["value"] = json!("DONE");
            }),
        ),
        (
            "member dependencies",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][member]["dependencies"] = json!([]);
            }),
        ),
        (
            "unit semantic type",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][unit]["semantic_type"] =
                    v["semantic_graph"]["nodes"][unit]["node_id"].clone();
            }),
        ),
        (
            "dimension semantic type",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][dimension]["semantic_type"] =
                    v["semantic_graph"]["nodes"][declaration]["node_id"].clone();
            }),
        ),
        (
            "owner outside lock",
            Box::new(move |v| {
                v["lock"]["definition_selections"][0]["identity"] = json!("other-model");
                let replacement = v["lock"]["definition_selections"].clone();
                v["identity_preimage"]["definition_selections"] = replacement;
            }),
        ),
    ];
    for (name, mutate) in cases {
        let mut mutated = base.clone();
        mutate(&mut mutated);
        refresh_identity(&mut mutated);
        assert_eq!(
            refused(&mutated, &evidence_for(&mutated)),
            nominal(CheckedPackageRefusalCode::InvalidSemanticGraph),
            "{name}"
        );
    }
}

const DOMAIN_PACKAGE_DIGEST: &str =
    "5555555555555555555555555555555555555555555555555555555555555555";

fn domain_package(identity: &str) -> Value {
    json!({
        "identity": identity, "version": "1",
        "digest_domain": "sha256-jcs", "digest": DOMAIN_PACKAGE_DIGEST
    })
}

/// The recorded nominal package with its enum declaration owned by `owner`,
/// re-keyed through every dependant, and `models` as the locked domain
/// package selections.
fn model_owned_package(owner: Value, models: Value) -> Value {
    let recorded = v2_nominal();
    let nodes = recorded["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    // Dependency order (declaration before member, dimension before unit), so
    // one rekey pass carries the owner change through every dependant.
    let order = [1, 0, 3, 2];
    let mut preimages = order
        .iter()
        .map(|position| nodes[*position]["nominal_identity_preimage"].clone())
        .collect::<Vec<_>>();
    let keys = order
        .iter()
        .map(|position| {
            nodes[*position]["node_id"]["digest"]
                .as_str()
                .expect("key")
                .to_owned()
        })
        .collect::<Vec<_>>();
    preimages[0]["owner"] = owner;
    let fresh = rekey(&mut preimages, &keys);
    let members = preimages.into_iter().zip(fresh).collect::<Vec<_>>();
    let mut package = nominal_package(&members);
    package["lock"]["model_selections"] = models;
    refresh_identity(&mut package);
    package
}

fn model_owner(identity: &str, node: &str) -> Value {
    json!({"kind": "model", "identity": identity, "node": node})
}

/// Tracing: TC-048, FR-038-AC-2, FR-038-AC-5
#[trace("TC-048", "FR-038-AC-2", "FR-038-AC-5")]
#[test]
fn tc_048_model_owners_join_sha256_jcs_domain_package_selections() {
    let schema = fixture("checked-package-v2/schema.json");
    let schema = jsonschema::JSONSchema::compile(&schema).expect("vendored schema compiles");
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    let base = model_owned_package(owner.clone(), json!([domain_package("test/orders")]));
    assert!(
        schema.is_valid(&base),
        "published V2 schema admits the base"
    );
    let package = admitted(&base);
    let typed: Value = serde_json::to_value(package.lock()).expect("typed lock");
    assert_eq!(typed, base["lock"]);
    assert_eq!(
        serde_json::to_value(&package.identity_preimage().model_selections)
            .expect("preimage models"),
        json!([domain_package("test/orders")])
    );

    // Owner-to-lock join: by domain package identity, with a present node.
    let joins = [
        (
            "owner identity outside lock",
            model_owned_package(
                model_owner("test/other", "ix://test/orders/Status"),
                json!([domain_package("test/orders")]),
            ),
        ),
        (
            "no domain package selected",
            model_owned_package(owner.clone(), json!([])),
        ),
        (
            "empty owner node",
            model_owned_package(
                model_owner("test/orders", ""),
                json!([domain_package("test/orders")]),
            ),
        ),
    ];
    for (name, mutated) in joins {
        assert_eq!(
            refused(&mutated, &evidence_for(&mutated)),
            nominal(CheckedPackageRefusalCode::InvalidSemanticGraph),
            "{name}"
        );
    }

    // A lock reference or owner carrying `authority`, `revision` or `export`
    // is an unknown member.
    let compiled_ref = json!({
        "authority": "agent-ix", "identity": "test/orders",
        "revision": {"namespace": "git", "value": "1"},
        "digest_domain": "quire.compiled-model.bytes/v1",
        "digest": DOMAIN_PACKAGE_DIGEST, "export": "Status"
    });
    let retired = [
        (
            "compiled-model owner with export",
            model_owned_package(
                json!({"kind": "model", "authority": "agent-ix",
                       "identity": "test/orders", "export": "Status"}),
                json!([domain_package("test/orders")]),
            ),
        ),
        (
            "domain package owner with export",
            model_owned_package(
                json!({"kind": "model", "identity": "test/orders",
                       "node": "ix://test/orders/Status", "export": "Status"}),
                json!([domain_package("test/orders")]),
            ),
        ),
        ("compiled-model lock reference", {
            let mut value = base.clone();
            value["lock"]["model_selections"] = json!([compiled_ref]);
            refresh_identity(&mut value);
            value
        }),
        ("domain package reference with export", {
            let mut value = base.clone();
            value["lock"]["model_selections"][0]["export"] = json!("Status");
            refresh_identity(&mut value);
            value
        }),
    ];
    let evidence = evidence_for(&base);
    for (name, mutated) in retired {
        assert!(!schema.is_valid(&mutated), "{name} schema");
        assert_eq!(
            refused(&mutated, &evidence),
            refusal(CheckedPackageRefusalCode::UnknownMember, "document"),
            "{name}"
        );
    }

    // Domain, shape and evidence are checked in the domain package domain.
    let lock_path = "lock.model_selections";
    let mut compiled_domain = base.clone();
    compiled_domain["lock"]["model_selections"][0]["digest_domain"] =
        json!("quire.compiled-model.bytes/v1");
    refresh_identity(&mut compiled_domain);
    assert!(!schema.is_valid(&compiled_domain));
    assert_eq!(
        refused(&compiled_domain, &evidence),
        refusal(CheckedPackageRefusalCode::DigestDomainMismatch, lock_path)
    );
    let mut empty_version = base.clone();
    empty_version["lock"]["model_selections"][0]["version"] = json!("");
    refresh_identity(&mut empty_version);
    assert!(!schema.is_valid(&empty_version));
    assert_eq!(
        refused(&empty_version, &evidence),
        refusal(CheckedPackageRefusalCode::MalformedWire, lock_path)
    );
    let mut other_version = base.clone();
    other_version["lock"]["model_selections"][0]["version"] = json!("2");
    refresh_identity(&mut other_version);
    assert_eq!(
        refused(&other_version, &evidence),
        refusal(CheckedPackageRefusalCode::StaleDependency, lock_path)
    );
    let mut raw_only = evidence_for(&v2_nominal());
    raw_only.insert_artifact_digest(
        locator(&json!({
            "authority": "agent-ix", "identity": "test/orders",
            "revision": {"namespace": "git", "value": "1"},
            "digest_domain": "sha256-jcs"
        })),
        DOMAIN_PACKAGE_DIGEST,
    );
    assert_eq!(
        refused(&base, &raw_only),
        refusal(CheckedPackageRefusalCode::StaleDependency, lock_path),
        "equal digest bytes attested as a raw artifact never satisfy a domain package"
    );
}

/// Tracing: TC-048, FR-038-AC-10
#[trace("TC-048", "FR-038-AC-10")]
#[test]
fn tc_048_duplicate_model_selection_refuses_as_malformed_wire() {
    let schema = fixture("checked-package-v2/schema.json");
    let schema = jsonschema::JSONSchema::compile(&schema).expect("vendored schema compiles");
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    let lock_path = "lock.model_selections";

    // A verbatim repeat (identity, version, digest_domain and digest all
    // equal) violates the schema's `uniqueItems` on `model_selections`;
    // `model_owned_package` mirrors the lock into the identity preimage via
    // `refresh_identity`, so the repeat is present in both members at once
    // and reaches the new uniqueness check.
    let duplicated = model_owned_package(
        owner.clone(),
        json!([domain_package("test/orders"), domain_package("test/orders")]),
    );
    assert!(
        !schema.is_valid(&duplicated),
        "uniqueItems rejects the verbatim repeat"
    );
    assert_eq!(
        refused(&duplicated, &evidence_for(&duplicated)),
        refusal(CheckedPackageRefusalCode::MalformedWire, lock_path)
    );

    // The same repeat, confined to the lock and left unmirrored in the
    // identity preimage, never reaches that check: `same_non_graph_lock`
    // requires `identity_preimage.model_selections` to equal
    // `lock.model_selections` element-for-element before either is examined
    // further, and a two-entry lock against a one-entry preimage fails that
    // equality first.
    let single = model_owned_package(owner.clone(), json!([domain_package("test/orders")]));
    let mut lock_only = single.clone();
    lock_only["lock"]["model_selections"] =
        json!([domain_package("test/orders"), domain_package("test/orders")]);
    assert!(
        !schema.is_valid(&lock_only),
        "uniqueItems still rejects the lock's own repeat"
    );
    assert_ne!(
        lock_only["lock"]["model_selections"], lock_only["identity_preimage"]["model_selections"],
        "the repeat is confined to the lock, not mirrored into the preimage"
    );
    assert_eq!(
        refused(&lock_only, &evidence_for(&lock_only)),
        refusal(CheckedPackageRefusalCode::StaleDependency, "lock")
    );

    // Two selections sharing identity and version but differing in digest
    // are distinct JSON items under whole-value `uniqueItems` equality, so
    // this criterion never refuses them. The package evidence can attest
    // only one digest per identity/version locator, so the input is still
    // refused, but by the pre-existing per-item digest check as
    // `stale_dependency`, not by this uniqueness check as `malformed_wire`.
    let mut other_digest = domain_package("test/orders");
    other_digest["digest"] = json!("9".repeat(64));
    let distinct_digest =
        model_owned_package(owner, json!([domain_package("test/orders"), other_digest]));
    assert!(
        schema.is_valid(&distinct_digest),
        "distinct whole-value items satisfy uniqueItems"
    );
    assert_eq!(
        refused(&distinct_digest, &evidence_for(&distinct_digest)),
        refusal(CheckedPackageRefusalCode::StaleDependency, lock_path),
        "same identity/version but differing digest is not a duplicate under this criterion"
    );
}

/// The uniqueness check runs over the whole `model_selections` array before
/// any entry's digest is evaluated against evidence, so an array carrying
/// both a repeated entry and an entry the evidence does not attest refuses
/// as `malformed_wire` regardless of which defect appears first.
///
/// Tracing: TC-048, FR-038-AC-11
#[trace("TC-048", "FR-038-AC-11")]
#[test]
fn tc_048_model_selection_duplicate_outranks_stale_digest_regardless_of_position() {
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    let single = model_owned_package(owner.clone(), json!([domain_package("test/orders")]));
    // Evidence attests only the single "test/orders" selection, so a third
    // entry naming a different identity is never attested — it is stale
    // wherever it appears in the array.
    let evidence = evidence_for(&single);
    let stale_entry = json!({
        "identity": "test/other", "version": "1",
        "digest_domain": "sha256-jcs", "digest": DOMAIN_PACKAGE_DIGEST
    });
    let duplicate_before_stale = model_owned_package(
        owner.clone(),
        json!([
            domain_package("test/orders"),
            domain_package("test/orders"),
            stale_entry.clone()
        ]),
    );
    let stale_before_duplicate = model_owned_package(
        owner,
        json!([
            stale_entry,
            domain_package("test/orders"),
            domain_package("test/orders")
        ]),
    );
    for (name, package) in [
        ("duplicate before stale", &duplicate_before_stale),
        ("stale before duplicate", &stale_before_duplicate),
    ] {
        assert_eq!(
            refused(package, &evidence),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "lock.model_selections"
            ),
            "{name}: the uniqueness check runs over the whole array before any digest is evaluated"
        );
    }
}

/// Tracing: TC-048, FR-038-AC-2
#[trace("TC-048", "FR-038-AC-2")]
#[test]
fn tc_048_model_export_is_not_a_v2_model_form() {
    let schema = fixture("checked-package-v2/schema.json");
    let schema = jsonschema::JSONSchema::compile(&schema).expect("vendored schema compiles");
    let base = v2_all_families();
    let model = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == json!("model"))
        .expect("all-families fixture carries a model node");
    for form in [
        "model_import",
        "object_type",
        "value_type",
        "variant_type",
        "record_value_type",
        "event_type",
        "state_machine",
        "process",
        "persistence_interface",
        "namespace",
        "field_declaration",
        "operation_declaration",
        "clause_member_declaration",
        "systems_interface",
        "systems_part",
        "systems_port",
        "systems_connection",
        "systems_allocation",
    ] {
        let mut value = base.clone();
        value["semantic_graph"]["nodes"][model]["semantic_form"] = json!(form);
        refresh_identity(&mut value);
        assert!(schema.is_valid(&value), "{form} schema");
        admitted(&value);
    }
    let mut export = base.clone();
    export["semantic_graph"]["nodes"][model]["semantic_form"] = json!("model_export");
    refresh_identity(&mut export);
    assert!(!schema.is_valid(&export));
    assert_eq!(
        refused(&export, &evidence_for(&export)),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.semantic_form"
        )
    );
}

/// Tracing: TC-048, FR-038-AC-9
#[trace("TC-048", "FR-038-AC-9")]
#[test]
fn tc_048_shipped_default_read_limits_are_exact_and_finite() {
    // FR-038-AC-9: the shipped default policy is exactly these seven values.
    let bounded = CheckedPackageReadLimits::bounded();
    assert_eq!(bounded.bytes, 1_048_576);
    assert_eq!(bounded.depth, 128);
    assert_eq!(bounded.nodes, 10_000);
    assert_eq!(bounded.edges, 100_000);
    assert_eq!(bounded.occurrences, 100_000);
    assert_eq!(bounded.diagnostics, 10_000);
    assert_eq!(bounded.work, 1_000_000);

    // FR-038-AC-9: every member is strictly positive and finite, so the
    // default admits no unbounded read on any axis.
    for (name, value) in [
        ("bytes", bounded.bytes),
        ("depth", bounded.depth),
        ("nodes", bounded.nodes),
        ("edges", bounded.edges),
        ("occurrences", bounded.occurrences),
        ("diagnostics", bounded.diagnostics),
        ("work", bounded.work),
    ] {
        assert!(value > 0, "{name} must be positive");
        assert!(value < u64::MAX, "{name} must be finite");
    }

    // FR-038-AC-9: each meter is enforced at its own true measured boundary.
    // The shipped default (up to 1,048,576 bytes / 10,000 nodes / 100,000
    // edges) is far larger than any vendored fixture (the largest is 19,992
    // bytes / 13 nodes), so a boundary fixed at the default is unreachable by
    // any plausibly sized package and would prove nothing; instead this test
    // discovers each meter's real measured cost against an actual package by
    // binary search, then proves that exact value admits and one below it
    // refuses as `incomplete`, naming that meter and reporting the true
    // consumption.
    let mut value = v2_all_families();
    // Give the graph a real dependency edge so every one of the seven meters
    // is charged by this package and the axis check below is not vacuous.
    let bound_key = value["semantic_graph"]["nodes"][2]["node_id"].clone();
    value["semantic_graph"]["nodes"][3]["dependencies"] = json!([bound_key]);
    value["diagnostics"]["entries"] = json!([{
        "stage": "type_checking",
        "code": "ill_typed",
        "cause_tag": "invalid-value",
        "details": [{"term": "reference", "target": value["semantic_graph"]["nodes"][0]["node_id"]}],
        "loci": [{"source": value["lock"]["sources"][0], "start": 0, "end": 1}],
    }]);
    refresh_identity(&mut value);
    let evidence = evidence_for(&value);
    let bytes = canonical(&value);
    assert!(matches!(
        CheckedPackageV2::read(&bytes, bounded, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    type NarrowLimit = fn(&mut CheckedPackageReadLimits, u64);
    let narrowed: [(CheckedPackageLimit, NarrowLimit, u64); 7] = [
        (
            CheckedPackageLimit::Bytes,
            |limits, value| limits.bytes = value,
            bounded.bytes,
        ),
        (
            CheckedPackageLimit::Depth,
            |limits, value| limits.depth = value,
            bounded.depth,
        ),
        (
            CheckedPackageLimit::Nodes,
            |limits, value| limits.nodes = value,
            bounded.nodes,
        ),
        (
            CheckedPackageLimit::Edges,
            |limits, value| limits.edges = value,
            bounded.edges,
        ),
        (
            CheckedPackageLimit::Occurrences,
            |limits, value| limits.occurrences = value,
            bounded.occurrences,
        ),
        (
            CheckedPackageLimit::Diagnostics,
            |limits, value| limits.diagnostics = value,
            bounded.diagnostics,
        ),
        (
            CheckedPackageLimit::Work,
            |limits, value| limits.work = value,
            bounded.work,
        ),
    ];
    let read_with = |set: NarrowLimit, value: u64| -> CheckedPackageV2ReadResult {
        let mut limits = bounded;
        set(&mut limits, value);
        CheckedPackageV2::read(&bytes, limits, &evidence)
    };
    for (kind, set, default) in narrowed {
        // 0 must refuse (the meter is genuinely charged) and `default` must
        // admit (established above); binary search the smallest value in
        // between that admits, which is exactly this package's true measured
        // consumption for this meter.
        assert!(
            matches!(read_with(set, 0), CheckedPackageV2ReadResult::Incomplete(_)),
            "{kind:?} must refuse at zero"
        );
        let (mut lo, mut hi) = (0_u64, default);
        while lo + 1 < hi {
            let mid = lo + (hi - lo) / 2;
            match read_with(set, mid) {
                CheckedPackageV2ReadResult::Admitted(_) => hi = mid,
                CheckedPackageV2ReadResult::Incomplete(_) => lo = mid,
                other => panic!("unexpected outcome searching {kind:?} boundary: {other:?}"),
            }
        }
        // `hi` is now the true measured consumption: the smallest value that
        // admits, and `hi - 1` therefore names it exactly as the reported
        // `consumed` counter.
        assert!(
            matches!(read_with(set, hi), CheckedPackageV2ReadResult::Admitted(_)),
            "{kind:?} exact measured consumption must admit"
        );
        match read_with(set, hi - 1) {
            CheckedPackageV2ReadResult::Incomplete(report) => {
                assert_eq!(
                    report,
                    incomplete(kind, hi - 1, hi),
                    "{kind:?} one below its true measured consumption"
                );
            }
            other => panic!("expected incomplete for {kind:?}, got {other:?}"),
        }
    }
}

/// The self-typed carve-out in `validate_graph`'s adjacency construction is
/// keyed on the member *and* on the term being the node body's own top-level
/// term, not on node identity: only a self-typed node's own body-root
/// `literal.type` may name itself from its body without counting as a
/// reference cycle requiring `recursion_group` (a self-typed scalar's
/// `literal.type` states the same fact its `semantic_type` already does). A
/// self-typed node whose body is instead a `reference` term naming itself is
/// a genuine 1-node cycle and must still resolve through `recursion_group` or
/// refuse — nothing restricts which node may declare itself its own
/// `semantic_type`, so the carve-out must not be reachable through any body
/// member but `literal.type`. Nor is it reachable through a `literal.type`
/// that names itself from *inside* another term nested in the body — an
/// `aggregate` member, a `binding` value or an `application` argument — even
/// though `validate_term`'s recursive walk reports every one of those with
/// the same `BODY_TYPE_PATH` structural path: only the body's own outermost
/// term is the node's `literal.type` in FR-322's sense (FR-038-AC-18).
///
/// Tracing: TC-048, FR-038-AC-18
#[trace("TC-048", "FR-038-AC-18")]
#[test]
fn tc_048_self_typed_carve_out_is_keyed_on_literal_type_not_node_identity() {
    let base = v2_all_families();
    let own_id = base["semantic_graph"]["nodes"][1]["node_id"].clone();
    let other_id = base["semantic_graph"]["nodes"][1]["semantic_type"].clone();

    let assert_self_cycle_refused = |body: Value| {
        let mut mutated = base.clone();
        mutated["semantic_graph"]["nodes"][1]["semantic_type"] = own_id.clone();
        mutated["semantic_graph"]["nodes"][1]["body"] = body;
        refresh_identity(&mut mutated);
        assert_eq!(
            refused(&mutated, &evidence_for(&mutated)),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.recursion_group"
            )
        );
    };

    // Positive: a self-typed node's own body-root `literal.type`
    // self-reference is the carve-out's intended case and admits with no
    // `recursion_group`.
    let mut literal_type = base.clone();
    literal_type["semantic_graph"]["nodes"][1]["semantic_type"] = own_id.clone();
    literal_type["semantic_graph"]["nodes"][1]["body"] = json!({
        "term": "literal", "type": own_id, "value_kind": "integer", "value": 1
    });
    refresh_identity(&mut literal_type);
    let package = admitted(&literal_type);
    assert_eq!(
        package.graph().nodes[1].recursion_group,
        None,
        "a self-typed literal.type self-reference is not a cycle"
    );

    // Negative: a self-typed node whose body is a `reference` term naming
    // itself is a genuine 1-node cycle, not the carve-out's case. The carve-
    // out regressed this: it used to key on node identity alone
    // (`semantic_type == position`), which also swallowed this case with no
    // `recursion_group` on `origin/main`'s vendored fixture.
    assert_self_cycle_refused(json!({"term": "reference", "target": own_id}));

    // Negative: the same self-typed `literal.type` self-reference, nested
    // one level inside an `aggregate` member instead of being the body's own
    // top-level term, is still a genuine 1-node cycle. `validate_term`
    // reports this nested literal's `type` at the same `BODY_TYPE_PATH` path
    // as the body-root case, so the carve-out must not key on the path
    // alone.
    assert_self_cycle_refused(json!({
        "term": "aggregate",
        "members": [
            {"term": "literal", "type": own_id, "value_kind": "integer", "value": 1}
        ]
    }));

    // Negative: the same self-reference nested inside a `binding` value.
    assert_self_cycle_refused(json!({
        "term": "binding",
        "name": "x",
        "value": {"term": "literal", "type": own_id, "value_kind": "integer", "value": 1}
    }));

    // Negative: the same self-reference nested inside an `application`
    // argument.
    assert_self_cycle_refused(json!({
        "term": "application",
        "operator": "call",
        "operation": {
            "identity": "quire.op.function.call",
            "laws": [],
            "mode": null,
            "member": null,
            "leaves": []
        },
        "result_type": other_id,
        "arguments": [
            {"term": "literal", "type": own_id, "value_kind": "integer", "value": 1}
        ]
    }));

    // Positive: a nested `literal.type` typed by some *other* node, not
    // self, inside an `aggregate` member is an ordinary resolvable
    // reference — proof the narrowing to `is_body_root` did not start
    // refusing ordinary nested literals.
    let mut nested_other_typed = base.clone();
    nested_other_typed["semantic_graph"]["nodes"][1]["body"] = json!({
        "term": "aggregate",
        "members": [
            {"term": "literal", "type": other_id, "value_kind": "integer", "value": 1}
        ]
    });
    refresh_identity(&mut nested_other_typed);
    let package = admitted(&nested_other_typed);
    assert_eq!(
        package.graph().nodes[1].recursion_group,
        None,
        "a nested literal typed by a different node is not a cycle"
    );
}

/// The vendored tree pinned by `PROVENANCE` is what this reader actually
/// re-derives: each of the five positive fixtures admits and re-derives its
/// recorded `package_id` exactly. The digests are pinned as literals — not
/// read back from the fixture — so a silent identity change in the reader,
/// or an edit to a vendored fixture that is not re-pinned here, fails this
/// test rather than passing silently.
///
/// Tracing: TC-048, FR-038-AC-16
#[trace("TC-048", "FR-038-AC-16")]
#[test]
fn tc_048_vendored_positive_fixtures_rederive_their_recorded_package_id() {
    let cases = [
        (
            "checked-package-v2/fixtures/positive-all-families.json",
            "b0b40569b19f00bd06ae08e218f0f77d114ce97cf42d2b6fa7c868a96a18bdad",
        ),
        (
            "checked-package-v2/fixtures/positive-nominal-identities.json",
            "b70a9f27c9ef49711fb603d56014aa5ce092379cd820c5e62a0154c89877e7b4",
        ),
        (
            "checked-package-v2/fixtures/positive-operation-identities.json",
            "dca508e418e70d99bcaf49384ea48c7f909a389d8c5b15541e5fb1bddd426168",
        ),
        (
            "checked-package-v2/fixtures/positive-clause-operations.json",
            "d011de207a1fe5578b89d185f9394ef6a995c16244b72d63ba2775c6518b5952",
        ),
        (
            "checked-package-v2/fixtures/positive-control-operations.json",
            "c76a26bf468ae66a74ea3f79dde881657b5fc9c0c555535fcc12d59b4cc2b69f",
        ),
    ];
    for (path, recorded_digest) in cases {
        let value = fixture(path);
        let package = admitted(&value);
        assert_eq!(
            package.package_id().digest.as_ref(),
            recorded_digest,
            "{path}"
        );
    }
}

/// FR-038-AC-17: deleting `declaration`, `literal.type`,
/// `application.operation` or `application.result_type` from a single node
/// refuses as `invalid_semantic_graph`, whether the deletion is left on the
/// graph alone or mirrored into `identity_preimage.identity_projection`. Both
/// scenarios hit the identical outcome, because the graph node's own
/// member-level check — `validate_declaration` for `declaration`, the term
/// grammar's closed member set for the other three — refuses unconditionally
/// inside the per-node walk, before `identity_preimage.identity_projection`
/// is ever compared against the derived projection; mirroring the deletion
/// into the preimage changes nothing about which check fires first.
///
/// Tracing: TC-048, FR-038-AC-17
#[trace("TC-048", "FR-038-AC-17")]
#[test]
fn tc_048_deleting_a_declared_wire_member_refuses_before_the_projection_compare() {
    let base = fixture("checked-package-v2/fixtures/positive-operation-identities.json");
    let nodes = base["semantic_graph"]["nodes"].as_array().expect("nodes");
    let declaring_node = nodes
        .iter()
        .position(|node| node.get("declaration").is_some())
        .expect("a declaring node");
    let literal_node = nodes
        .iter()
        .position(|node| node["body"]["term"] == json!("literal"))
        .expect("a literal node");
    let application_node = nodes
        .iter()
        .position(|node| node["body"]["term"] == json!("application"))
        .expect("an application node");

    type Delete = fn(&mut Value, usize);
    let cases: [(&str, usize, Delete, CheckedPackageRefusal); 4] = [
        (
            "declaration",
            declaring_node,
            |v, i| {
                v["semantic_graph"]["nodes"][i]
                    .as_object_mut()
                    .expect("node object")
                    .remove("declaration");
            },
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.declaration",
            ),
        ),
        (
            "literal.type",
            literal_node,
            |v, i| {
                v["semantic_graph"]["nodes"][i]["body"]
                    .as_object_mut()
                    .expect("body object")
                    .remove("type");
            },
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.body",
            ),
        ),
        (
            "application.operation",
            application_node,
            |v, i| {
                v["semantic_graph"]["nodes"][i]["body"]
                    .as_object_mut()
                    .expect("body object")
                    .remove("operation");
            },
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.body",
            ),
        ),
        (
            "application.result_type",
            application_node,
            |v, i| {
                v["semantic_graph"]["nodes"][i]["body"]
                    .as_object_mut()
                    .expect("body object")
                    .remove("result_type");
            },
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_graph.nodes.body",
            ),
        ),
    ];

    for (name, index, delete, expected) in cases {
        // Left on the graph alone: the preimage still carries the member, so
        // the package_id/preimage staleness check (which runs before the
        // graph is walked) still passes unchanged, and it is the member-level
        // check inside the graph walk that actually refuses.
        let mut graph_only = base.clone();
        delete(&mut graph_only, index);
        assert_eq!(
            refused(&graph_only, &evidence_for(&graph_only)),
            expected,
            "{name}: graph alone"
        );

        // Mirrored into identity_preimage.identity_projection: refreshing
        // the identity after the same graph deletion re-derives a projection
        // and package_id consistent with the mutated graph, so the member is
        // absent from both. The outcome is identical, because the
        // member-level check never consults the preimage.
        let mut mirrored = base.clone();
        delete(&mut mirrored, index);
        refresh_identity(&mut mirrored);
        assert_eq!(
            refused(&mirrored, &evidence_for(&mirrored)),
            expected,
            "{name}: mirrored"
        );
    }
}

/// FR-038-AC-17's closed model/expression form lists: `CheckedNodeTag::forms`
/// for every family equals the vendored schema's own `semantic_form` enum for
/// that family's node definition, so the schema stays the single source for
/// the closed lists rather than two hand-maintained copies that can diverge
/// silently. The eighteen `model` forms and the fifteen `expression` forms
/// each admit as a node form; a nineteenth `model` form and a sixteenth
/// `expression` form each refuse as `invalid_semantic_graph`.
///
/// Tracing: TC-048, FR-038-AC-17
#[trace("TC-048", "FR-038-AC-17")]
#[test]
fn tc_048_node_tag_forms_match_the_vendored_schema_and_bound_admission() {
    let schema = fixture("checked-package-v2/schema.json");
    let defs = schema["$defs"].as_object().expect("schema $defs");
    for tag in CheckedNodeTag::ALL {
        let def = defs
            .values()
            .find(|def| {
                def["allOf"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|part| part["properties"]["node_tag"]["const"] == json!(tag.as_wire()))
            })
            .unwrap_or_else(|| panic!("schema defines a node for {}", tag.as_wire()));
        let forms = def["allOf"]
            .as_array()
            .expect("allOf")
            .iter()
            .find_map(|part| part["properties"]["semantic_form"]["enum"].as_array())
            .unwrap_or_else(|| panic!("{} declares a semantic_form enum", tag.as_wire()));
        let schema_forms = forms
            .iter()
            .map(|form| form.as_str().expect("form string"))
            .collect::<Vec<_>>();
        assert_eq!(schema_forms, tag.forms(), "{}", tag.as_wire());
    }

    let schema_doc = jsonschema::JSONSchema::compile(&schema).expect("vendored schema compiles");
    let base = v2_all_families();
    let model = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == json!("model"))
        .expect("all-families fixture carries a model node");
    for form in CheckedNodeTag::Model.forms() {
        let mut value = base.clone();
        value["semantic_graph"]["nodes"][model]["semantic_form"] = json!(form);
        refresh_identity(&mut value);
        assert!(schema_doc.is_valid(&value), "model/{form} schema");
        admitted(&value);
    }
    let mut nineteenth = base.clone();
    nineteenth["semantic_graph"]["nodes"][model]["semantic_form"] = json!("model_export");
    refresh_identity(&mut nineteenth);
    assert!(!schema_doc.is_valid(&nineteenth));
    assert_eq!(
        refused(&nineteenth, &evidence_for(&nineteenth)),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.semantic_form"
        )
    );

    let expression = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == json!("expression"))
        .expect("all-families fixture carries an expression node");
    for form in CheckedNodeTag::Expression.forms() {
        let mut value = base.clone();
        value["semantic_graph"]["nodes"][expression]["semantic_form"] = json!(form);
        refresh_identity(&mut value);
        assert!(schema_doc.is_valid(&value), "expression/{form} schema");
        admitted(&value);
    }
    let mut sixteenth = base.clone();
    sixteenth["semantic_graph"]["nodes"][expression]["semantic_form"] = json!("future_expression");
    refresh_identity(&mut sixteenth);
    assert!(!schema_doc.is_valid(&sixteenth));
    assert_eq!(
        refused(&sixteenth, &evidence_for(&sixteenth)),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.semantic_form"
        )
    );
}
