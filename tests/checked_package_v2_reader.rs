// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Strict CheckedPackage V2 reader: vendored adverse mutations, injected
//! refusals, resource limits, package identity, and nominal node identity.

#[path = "support/checked_package.rs"]
mod checked_package;

use checked_package::{
    apply_patch, canonical, evidence_for, fixture, incomplete, json_depth, locator,
    node_identity_vectors, nominal_package, refresh_identity, refusal, refusal_code, rekey,
    sha256_hex, v2_all_families, v2_nominal, COMPLETE_VALUE_FEATURE,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
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
                v["semantic_graph"]["nodes"][4]["body"]["target"]["digest"] = json!("9".repeat(64));
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
                v["semantic_graph"]["nodes"][1]["dependencies"] =
                    json!([{"domain":"quire.checked-semantic-node/v1","digest":"9".repeat(64)}]);
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
    // Where serde_json's `arbitrary_precision` is unified into the build, a
    // non-integer token already fails the canonical-bytes check, so either
    // refusal is accepted.
    let schema = jsonschema::JSONSchema::compile(&fixture("checked-package-v2/schema.json"))
        .expect("vendored schema compiles");
    type Build<'a> = Box<dyn Fn(Value) -> Value + 'a>;
    let body: Build = Box::new(|literal| {
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][0]["body"] =
            json!({"term":"literal","value_kind":"integer","value":literal});
        refresh_identity(&mut changed);
        changed
    });
    let detail: Build = Box::new(|literal| {
        let mut changed = base.clone();
        changed["diagnostics"]["entries"] = json!([{
            "stage":"type_checking","code":"ill_typed","cause_tag":"invalid-value",
            "details":[{"term":"literal","value_kind":"integer","value":literal}],"loci":[]
        }]);
        changed
    });
    let integer_refusals = [
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.body",
        ),
        refusal(CheckedPackageRefusalCode::NoncanonicalWire, "document"),
    ];
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
            let actual = refused(&candidate, &evidence);
            assert!(integer_refusals.contains(&actual), "{actual:?}");
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
    assert_eq!(admitted(&grouped).graph().nodes.len(), 13);

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
        // Terms 4 + nominal 11 + graph edges 4 + diagnostic detail 1.
        work: 20,
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
    // Terms 13 + graph edges 14; no nominal or diagnostic work.
    limits.work = 27;
    assert!(matches!(
        CheckedPackageV2::read(&all_bytes, limits, &evidence_for(&all)),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    limits.work = 26;
    assert_eq!(
        CheckedPackageV2::read(&all_bytes, limits, &evidence_for(&all)),
        CheckedPackageV2ReadResult::Incomplete(incomplete(CheckedPackageLimit::Work, 26, 27))
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
