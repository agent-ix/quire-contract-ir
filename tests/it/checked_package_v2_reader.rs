// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Strict CheckedPackage V2 reader: vendored adverse mutations, injected
//! refusals, resource limits, package identity, and nominal node identity.

use crate::support::checked_package::{
    self, all_families_read_work, canonical, domain_package_digest, domain_package_document,
    evidence_for, incomplete, json_depth, locator, nominal_fixture_members, nominal_package,
    pointer as support_pointer, positive_operation_identities, refresh_identity, refusal,
    refusal_at, refusal_bytes, refusal_cause, rekey, sha256_hex, unknown_version, v2_all_families,
    v2_nominal, ALL_FAMILIES_READ_WORK, COMPLETE_VALUE_FEATURE,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    read_checked_package, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, ExpressionForm, NominalIdentityPreimage,
};
use serde_json::{json, Value};

type Mutation = Box<dyn Fn(&mut Value)>;

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

/// `invalid_semantic_graph`, the code of every nominal-stage refusal, at
/// `path`.
fn nominal(path: &str) -> CheckedPackageRefusal {
    refusal(CheckedPackageRefusalCode::InvalidSemanticGraph, path)
}

fn dispatch(bytes: &[u8], evidence: &CheckedPackageEvidence) -> CheckedPackageDispatchResult {
    read_checked_package(bytes, CheckedPackageReadLimits::bounded(), evidence)
}

fn assert_dispatch_refused(result: CheckedPackageDispatchResult, expected: CheckedPackageRefusal) {
    match result {
        CheckedPackageDispatchResult::Refused(actual) => assert_eq!(actual, expected),
        other => panic!("expected {expected:?}, got {other:?}"),
    }
}

/// The reader parses `contract_version` exactly once and admits only the
/// current contract; every other version, and every malformed document, is
/// refused before any version-specific decoding.
///
/// Tracing: TC-048, FR-038-AC-1, FR-038-AC-24, FR-038-AC-25
#[trace("TC-048", "FR-038-AC-1", "FR-038-AC-24", "FR-038-AC-25")]
#[test]
fn tc_048_reader_refuses_unknown_absent_and_malformed_versions() {
    let fixture = v2_all_families();
    let evidence = evidence_for(&fixture);
    let with_version = |version: Value| {
        let mut value = fixture.clone();
        value["contract_version"] = version;
        canonical(&value)
    };
    // An unknown version is refused at `/contract_version`, naming the
    // version string the reader read there.
    for unknown in ["quire.checked-package/v3", "quire.checked-package/v0", ""] {
        assert_dispatch_refused(
            dispatch(&with_version(json!(unknown)), &evidence),
            unknown_version(unknown),
        );
    }
    for malformed in [json!(2), json!(null), json!(["quire.checked-package/v2"])] {
        assert_dispatch_refused(
            dispatch(&with_version(malformed), &evidence),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/contract_version",
            ),
        );
    }
    let mut absent = fixture.clone();
    absent
        .as_object_mut()
        .expect("fixture object")
        .remove("contract_version");
    // Absent: the document object lacks the member, so the refusal points
    // at the document (the empty pointer), as it does for a non-object
    // document. Malformed JSON is about no value and carries no pointer.
    assert_dispatch_refused(
        dispatch(&canonical(&absent), &evidence),
        refusal(CheckedPackageRefusalCode::MalformedWire, ""),
    );
    assert_dispatch_refused(
        dispatch(b"[]", &evidence),
        refusal(CheckedPackageRefusalCode::MalformedWire, ""),
    );
    assert_dispatch_refused(
        dispatch(b"{\"contract_version\":", &evidence),
        refusal_bytes(CheckedPackageRefusalCode::MalformedWire),
    );

    // The strict parse runs once, before any version is selected.
    let bytes = canonical(&fixture);
    let body = std::str::from_utf8(&bytes).expect("UTF-8 fixture");
    let duplicate = format!(
        "{{\"contract_version\":\"quire.checked-package/v0\",{}",
        body.trim_start_matches('{')
    );
    assert_dispatch_refused(
        dispatch(duplicate.as_bytes(), &evidence),
        refusal(
            CheckedPackageRefusalCode::DuplicateMember,
            "/contract_version",
        ),
    );
    let mut spaced = b" ".to_vec();
    spaced.extend_from_slice(&bytes);
    assert_dispatch_refused(
        dispatch(&spaced, &evidence),
        refusal_bytes(CheckedPackageRefusalCode::NoncanonicalWire),
    );

    // A recognized version still admits through the same entry point.
    assert!(matches!(
        dispatch(&bytes, &evidence),
        CheckedPackageDispatchResult::AdmittedV2(_)
    ));
}

/// A refusal's pointer is built from the reader's own position, escaping
/// `~` as `~0` and `/` as `~1` in every member name and naming array
/// elements by index, so it resolves to the exact value in the document the
/// reader was given — including a member the reader has never heard of.
///
/// Tracing: TC-048, FR-038-AC-24
#[trace("TC-048", "FR-038-AC-24")]
#[test]
fn tc_048_refusal_pointers_escape_member_names_and_resolve() {
    let base = v2_all_families();
    let evidence = evidence_for(&base);
    for (name, place) in [
        ("top level", "/a~1b~0c"),
        ("node member", "/semantic_graph/nodes/3/a~1b~0c"),
        ("lock source member", "/lock/sources/0/~1~0"),
    ] {
        let mut mutated = base.clone();
        let (parent, key) = place.rsplit_once('/').expect("member pointer");
        let key = key.replace("~1", "/").replace("~0", "~");
        mutated
            .pointer_mut(parent)
            .and_then(Value::as_object_mut)
            .expect("parent object")
            .insert(key, json!(1));
        let refusal_found = refused(&mutated, &evidence);
        assert_eq!(
            refusal_found,
            refusal(CheckedPackageRefusalCode::UnknownMember, place),
            "{name}"
        );
        assert_eq!(mutated.pointer(place), Some(&json!(1)), "{name}");
    }

    // An unknown member that happens to be named like the typed nominal
    // preimage is still just an unknown member at that key: only the two
    // positions the wire types a preimage at are ever looked inside.
    for place in [
        "/capability_report/0/nominal_identity_preimage",
        "/lock/nominal_identity_preimage",
    ] {
        let mut mutated = base.clone();
        let (parent, key) = place.rsplit_once('/').expect("member pointer");
        mutated
            .pointer_mut(parent)
            .and_then(Value::as_object_mut)
            .expect("parent object")
            .insert(key.to_owned(), json!({"version": 1}));
        assert_eq!(
            refused(&mutated, &evidence),
            refusal(CheckedPackageRefusalCode::UnknownMember, place),
            "{place}"
        );
    }

    // A repeated member is located by the strict parse itself, before any
    // decoding, with the same escaping.
    let bytes = canonical(&base);
    let text = std::str::from_utf8(&bytes).expect("UTF-8 fixture");
    let duplicate = format!(r#"{{"x/y~":1,"x/y~":2,{}"#, &text[1..]);
    assert_eq!(
        refused_bytes(duplicate.as_bytes(), &evidence),
        refusal(CheckedPackageRefusalCode::DuplicateMember, "/x~1y~0")
    );
}

/// Five structural mutations against document-level members every
/// `quire.checked-package/v2` document shares, each named by JSON pointer and
/// its expected refusal code. Authored here rather than vendored: unlike the
/// node-identity vectors, this is not an independent conformance oracle —
/// each entry asserts the reader's own documented structural rule
/// (`contract_version`/`lock.sources`/`semantic_graph.graph_version`/
/// `diagnostics.catalog.digest_domain`/`capability_report`, all checked
/// directly by `v2::mod::validate`) against itself, so authoring it in this
/// repository makes no assertion tautological.
const STRUCTURAL_MUTATIONS: [(&str, &str, CheckedPackageRefusalCode); 5] = [
    (
        "/contract_version",
        "unknown_contract_version",
        CheckedPackageRefusalCode::UnknownContractVersion,
    ),
    (
        "/lock/sources",
        "stale_dependency",
        CheckedPackageRefusalCode::StaleDependency,
    ),
    (
        "/semantic_graph/graph_version",
        "invalid_semantic_graph",
        CheckedPackageRefusalCode::InvalidSemanticGraph,
    ),
    (
        "/diagnostics/catalog/digest_domain",
        "digest_domain_mismatch",
        CheckedPackageRefusalCode::DigestDomainMismatch,
    ),
    (
        "/capability_report",
        "unknown_required_capability",
        CheckedPackageRefusalCode::UnknownRequiredCapability,
    ),
];

fn structural_mutation_replacement(pointer: &str) -> Value {
    match pointer {
        "/contract_version" => json!("quire.checked-package/v3"),
        "/lock/sources" => json!([]),
        "/semantic_graph/graph_version" => json!("quire.checked-semantic-graph/v3"),
        "/diagnostics/catalog/digest_domain" => json!("quire.fixture.wrong-domain/v1"),
        "/capability_report" => json!([]),
        other => panic!("no replacement authored for structural mutation pointer {other}"),
    }
}

/// Tracing: TC-048, FR-038-AC-2, FR-038-AC-24
#[trace("TC-048", "FR-038-AC-2", "FR-038-AC-24")]
#[test]
fn tc_048_v2_reader_refuses_every_structural_mutation() {
    for base in [v2_all_families(), v2_nominal()] {
        let evidence = evidence_for(&base);
        for (pointer, id, code) in STRUCTURAL_MUTATIONS {
            let mut mutated = base.clone();
            *mutated.pointer_mut(pointer).expect("pointer target") =
                structural_mutation_replacement(pointer);
            // Refused both as constructed and with the identity re-derived:
            // none of these five mutations touches a member
            // `refresh_identity` recomputes, so the outcome must not change
            // when the identity is rederived around it.
            let mut rederived = mutated.clone();
            refresh_identity(&mut rederived);
            for candidate in [&mutated, &rederived] {
                let actual = refused(candidate, &evidence);
                assert_eq!(actual.code, code, "{id}");
                // The refusal points at exactly the value the mutation
                // replaced.
                assert_eq!(actual.path, Some(support_pointer(pointer)), "{id}");
            }
        }
    }
}

/// Tracing: TC-048, FR-038-AC-2, FR-038-AC-24
#[trace("TC-048", "FR-038-AC-2", "FR-038-AC-24")]
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
        ),
        refusal(
            CheckedPackageRefusalCode::DuplicateMember,
            "/contract_version"
        )
    );
    let mut spaced = bytes.clone();
    spaced.push(b'\n');
    assert_eq!(
        refused_bytes(&spaced, &evidence),
        refusal_bytes(CheckedPackageRefusalCode::NoncanonicalWire)
    );

    let cases: Vec<(&str, Mutation, CheckedPackageRefusal)> = vec![
        (
            "unknown top-level member",
            Box::new(|v| v["future"] = json!(1)),
            refusal(CheckedPackageRefusalCode::UnknownMember, "/future"),
        ),
        (
            "unknown node member",
            Box::new(|v| v["semantic_graph"]["nodes"][0]["future"] = json!(1)),
            refusal(
                CheckedPackageRefusalCode::UnknownMember,
                "/semantic_graph/nodes/0/future",
            ),
        ),
        (
            "wrong member kind",
            Box::new(|v| v["semantic_graph"]["nodes"][0]["dependencies"] = json!("none")),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/semantic_graph/nodes/0/dependencies",
            ),
        ),
        (
            "missing member",
            Box::new(|v| {
                v["diagnostics"]
                    .as_object_mut()
                    .expect("diagnostics")
                    .remove("entries");
            }),
            // The object lacking the member is the value at fault.
            refusal(CheckedPackageRefusalCode::MalformedWire, "/diagnostics"),
        ),
        (
            "null recursion group",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][1]["recursion_group"] = Value::Null;
                refresh_identity(v);
            }),
            // The identity projection's mirror of the null member is the
            // first value the lossy decode dropped.
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/identity_preimage/identity_projection/1/recursion_group",
            ),
        ),
        (
            "empty recursion group",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][1]["recursion_group"] = json!("");
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/semantic_graph/nodes/1/recursion_group",
            ),
        ),
        (
            "unreported required feature",
            Box::new(|v| v["capability_report"] = json!([])),
            refusal(
                CheckedPackageRefusalCode::UnknownRequiredCapability,
                "/capability_report",
            ),
        ),
        (
            // Closed vocabularies decode at the wire edge: a value outside one
            // is a wire-shape refusal before any semantic check runs.
            "unknown selection role",
            Box::new(|v| v["lock"]["edition"]["role"] = json!("future_role")),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/lock/edition/role",
            ),
        ),
        (
            // The value is spelled like the serde message the reader
            // classifies by; it must still refuse as a bad value, not as an
            // unknown member.
            "selection role spelled like a decoder message",
            Box::new(|v| v["lock"]["edition"]["role"] = json!("unknown field")),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/lock/edition/role",
            ),
        ),
        (
            "unknown capability disposition",
            Box::new(|v| v["capability_report"][0]["disposition"] = json!("deferred")),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/capability_report/0/disposition",
            ),
        ),
        (
            "unavailable required feature",
            Box::new(|v| v["capability_report"][0]["disposition"] = json!("unsupported")),
            refusal(
                CheckedPackageRefusalCode::UnknownRequiredCapability,
                "/capability_report/0/disposition",
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
                "/semantic_graph/nodes/4/body/target",
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
                "/semantic_graph/nodes/1/dependencies/0",
            ),
        ),
        (
            "incomplete source map",
            Box::new(|v| {
                v["source_map"].as_array_mut().expect("source map").pop();
            }),
            refusal(CheckedPackageRefusalCode::InvalidSourceMap, "/source_map"),
        ),
        (
            "unlocked source region",
            Box::new(|v| v["source_map"][0]["regions"][0]["source"]["identity"] = json!("other")),
            refusal(
                CheckedPackageRefusalCode::InvalidSourceMap,
                "/source_map/0/regions/0/source",
            ),
        ),
        (
            "stale projection",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][0]["body"]["value"] = json!(false);
            }),
            refusal(
                CheckedPackageRefusalCode::StaleDependency,
                "/identity_preimage/identity_projection/0/body/value",
            ),
        ),
        (
            "stale package id",
            Box::new(|v| v["package_id"]["digest"] = json!("0".repeat(64))),
            refusal(
                CheckedPackageRefusalCode::StaleDependency,
                "/package_id/digest",
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
                "/identity_preimage/version",
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
            // Neither node carries the member the cycle requires; the
            // lower-positioned one is named.
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/1",
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
            // Node 1 sets the group; node 2's differs.
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/2/recursion_group",
            ),
        ),
        (
            "self dependency without a recursion group",
            Box::new(|v| {
                let own = v["semantic_graph"]["nodes"][1]["node_id"].clone();
                v["semantic_graph"]["nodes"][1]["dependencies"] = json!([own]);
                refresh_identity(v);
            }),
            // The node lacks the `recursion_group` member its cycle needs.
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/1",
            ),
        ),
    ];
    for (name, mutate, expected) in cases {
        let mut mutated = base.clone();
        mutate(&mut mutated);
        match read(&mutated, &evidence) {
            CheckedPackageV2ReadResult::Refused(actual) => {
                assert_eq!(actual, expected, "{name}");
                // Every pointer resolves in the document the reader was given.
                let path = actual.path.expect("a refusal about a value");
                assert!(mutated.pointer(path.as_str()).is_some(), "{name}: {path}");
            }
            other => panic!("{name}: expected refusal, got {other:?}"),
        }
    }

    // Literal numbers are integers, in node bodies and diagnostic details. A
    // whole-valued float spelling is refused too, as the reader admits
    // integer tokens only. The refusal is the same whether or not
    // serde_json's `arbitrary_precision` is unified into the build.
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
    // Each refuses at the literal term itself.
    for (build, at) in [
        (body, "/semantic_graph/nodes/0/body"),
        (detail, "/diagnostics/entries/0/details/0"),
    ] {
        let integer_refusal = refusal(CheckedPackageRefusalCode::InvalidSemanticGraph, at);
        let integer = build(json!(-7));
        assert!(matches!(
            read(&integer, &evidence),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        let fractional = build(json!(1.5));
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
    // 18 total: see `build_v2_all_families` in `tests/support/checked_package.rs`.
    assert_eq!(admitted(&grouped).graph().nodes.len(), 18);

    // Evidence the caller must supply: every locked digest and the feature.
    // With no evidence, the first locked source's locator is unattested.
    assert_eq!(
        refused(&base, &CheckedPackageEvidence::new()),
        refusal(
            CheckedPackageRefusalCode::StaleDependency,
            "/lock/sources/0"
        )
    );
    let mut stale_catalog = evidence_for(&base);
    stale_catalog.insert_artifact_digest(locator(&base["diagnostics"]["catalog"]), "4".repeat(64));
    assert_eq!(
        refused(&base, &stale_catalog),
        refusal(
            CheckedPackageRefusalCode::StaleDependency,
            "/diagnostics/catalog/digest"
        )
    );
    let mut byte_evidence = evidence_for(&base);
    byte_evidence.insert_artifact_bytes(locator(&base["lock"]["sources"][0]), b"not the source");
    assert_eq!(
        refused(&base, &byte_evidence),
        refusal(
            CheckedPackageRefusalCode::StaleDependency,
            "/lock/sources/0/digest"
        )
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
        // Reported available, but the reader does not support it.
        refusal(
            CheckedPackageRefusalCode::UnknownRequiredCapability,
            "/lock/required_features/0"
        )
    );
    unsupported.support_feature(COMPLETE_VALUE_FEATURE);
    assert!(matches!(
        read(&base, &unsupported),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Tracing: TC-048, FR-038-AC-3, FR-038-AC-26
#[trace("TC-048", "FR-038-AC-3", "FR-038-AC-26")]
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
    // Each one-over limit names the value whose charge failed: the byte limit
    // is charged before any value exists; depth, at the first value nested one
    // level too deep; nodes, at the first node past the ceiling; edges and
    // occurrences, at the first dependency or source-map region past it;
    // diagnostics, at the first entry past it; work, at the value whose
    // validation took the meter over.
    type Narrow = fn(&mut CheckedPackageReadLimits) -> u64;
    let narrowings: [(CheckedPackageLimit, Narrow, Option<&str>); 7] = [
        (
            CheckedPackageLimit::Bytes,
            |l| {
                l.bytes -= 1;
                l.bytes
            },
            None,
        ),
        (
            CheckedPackageLimit::Depth,
            |l| {
                l.depth -= 1;
                l.depth
            },
            Some("/diagnostics/entries/0/loci/0/source/revision/namespace"),
        ),
        (
            CheckedPackageLimit::Nodes,
            |l| {
                l.nodes -= 1;
                l.nodes
            },
            Some("/semantic_graph/nodes/3"),
        ),
        (
            CheckedPackageLimit::Edges,
            |l| {
                l.edges -= 1;
                l.edges
            },
            Some("/semantic_graph/nodes/2/dependencies/0"),
        ),
        (
            CheckedPackageLimit::Occurrences,
            |l| {
                l.occurrences -= 1;
                l.occurrences
            },
            Some("/source_map/3/regions/0"),
        ),
        (
            CheckedPackageLimit::Diagnostics,
            |l| {
                l.diagnostics -= 1;
                l.diagnostics
            },
            Some("/diagnostics/entries/0"),
        ),
        (
            CheckedPackageLimit::Work,
            |l| {
                l.work -= 1;
                l.work
            },
            Some("/diagnostics/entries/0/details/0"),
        ),
    ];
    for (kind, narrow, path) in narrowings {
        let mut limits = exact;
        let limit = narrow(&mut limits);
        match CheckedPackageV2::read(&bytes, limits, &evidence) {
            CheckedPackageV2ReadResult::Incomplete(actual) => {
                assert_eq!(actual, incomplete(kind, limit, limit + 1, path), "{kind:?}");
                // The pointer resolves in the package the reader was given.
                if let Some(path) = path {
                    assert!(value.pointer(path).is_some(), "{kind:?} {path}");
                }
            }
            other => panic!("{kind:?} one over must be incomplete, got {other:?}"),
        }
    }

    let all = v2_all_families();
    let all_bytes = canonical(&all);
    // Pinned so a future change to the reader's charging logic that shifts
    // the real boundary is caught here, rather than silently absorbed by a
    // binary search that measures whatever the reader under test now does.
    assert_eq!(all_families_read_work(), ALL_FAMILIES_READ_WORK);
    let all_families_read_work = ALL_FAMILIES_READ_WORK;
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = all_families_read_work;
    assert!(matches!(
        CheckedPackageV2::read(&all_bytes, limits, &evidence_for(&all)),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    limits.work = all_families_read_work - 1;
    assert_eq!(
        CheckedPackageV2::read(&all_bytes, limits, &evidence_for(&all)),
        CheckedPackageV2ReadResult::Incomplete(incomplete(
            CheckedPackageLimit::Work,
            all_families_read_work - 1,
            all_families_read_work,
            Some("/semantic_graph/nodes/17/dependencies/3")
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
                    "digest_domain": "sha256-jcs",
                    "digest": domain_package_digest_of("test/orders")
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
            // The lock's array differs from its preimage mirror in length, so
            // the array itself is the value at fault.
            assert_eq!(
                stale,
                refusal(
                    CheckedPackageRefusalCode::StaleDependency,
                    "/lock/model_selections"
                )
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

// `tc_048_nominal_vectors_rederive_and_admit_as_one_package` and
// `tc_048_invalid_nominal_mutations_refuse_retained_and_rekeyed` were removed
// here: both replayed `node_identity_vectors()`, the independent
// node-identity conformance oracle copied from a private upstream repository
// and deleted with the rest of the private-sourced fixture tree (the issue's
// own account names both by this description). Regenerating that oracle from
// this crate's own code would make every assertion checked against it a
// tautology, so it is not recoverable here; see AGE-1961.
// `v2_nominal()`'s own construction (`nominal_package` over
// `nominal_fixture_members()` in `tests/support/checked_package.rs`) is still
// exercised end-to-end by every other test in this crate that reads it.
//
// The first removed test's serde round-trip and digest re-derivation checks
// did not actually depend on the deleted oracle, though: they checked that a
// typed `NominalIdentityPreimage` serializes back to its own wire form and
// that `NominalIdentityPreimage::digest()` reproduces the node key it is
// keyed by. Both properties are re-checked below against
// `nominal_fixture_members()` — this module's own locally-authored preimages,
// never the deleted oracle — so `NominalIdentityPreimage`'s serde impl and
// `digest()` keep a real test rather than going untested.

/// Tracing: TC-048, FR-038-AC-5
#[trace("TC-048", "FR-038-AC-5")]
#[test]
fn tc_048_nominal_preimages_round_trip_and_digest_to_their_own_node_key() {
    for (preimage, key) in nominal_fixture_members() {
        let typed: NominalIdentityPreimage =
            serde_json::from_value(preimage.clone()).expect("typed preimage");
        assert_eq!(
            serde_json::to_value(&typed).expect("round trip"),
            preimage,
            "preimage did not round-trip through NominalIdentityPreimage"
        );
        assert_eq!(
            typed.digest().as_deref(),
            Some(key.as_str()),
            "digest() did not reproduce the node key this preimage is keyed by"
        );
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
        // The declaration's preimage no longer derives its own key.
        nominal("/semantic_graph/nodes/1/node_id"),
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
    // Each refuses at the member the failed check read. The member node
    // (position 0) is checked first, so a declaration it cannot resolve is
    // reported at its own `declaration_node_id`.
    let cases: Vec<(&str, Mutation, &str)> = vec![
        (
            "absent required preimage",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][declaration]
                    .as_object_mut()
                    .expect("node")
                    .remove("nominal_identity_preimage");
            }),
            "/semantic_graph/nodes/0/nominal_identity_preimage/declaration_node_id",
        ),
        (
            "swapped preimage kind",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][declaration]["nominal_identity_preimage"] =
                    v["semantic_graph"]["nodes"][member]["nominal_identity_preimage"].clone();
            }),
            "/semantic_graph/nodes/0/nominal_identity_preimage/declaration_node_id",
        ),
        (
            "preimage not deriving the node key",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][unit]["nominal_identity_preimage"]
                    ["qualified_declaration"] = json!(["Example", "centimetre"]);
            }),
            "/semantic_graph/nodes/2/node_id",
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
            "/semantic_graph/nodes/0/nominal_identity_preimage",
        ),
        (
            "member body case",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][member]["body"]["value"] = json!("DONE");
            }),
            "/semantic_graph/nodes/0/body",
        ),
        (
            "member dependencies",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][member]["dependencies"] = json!([]);
            }),
            "/semantic_graph/nodes/0/dependencies",
        ),
        (
            "unit semantic type",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][unit]["semantic_type"] =
                    v["semantic_graph"]["nodes"][unit]["node_id"].clone();
            }),
            "/semantic_graph/nodes/2/semantic_type",
        ),
        (
            "dimension semantic type",
            Box::new(move |v| {
                v["semantic_graph"]["nodes"][dimension]["semantic_type"] =
                    v["semantic_graph"]["nodes"][declaration]["node_id"].clone();
            }),
            "/semantic_graph/nodes/3/semantic_type",
        ),
        (
            "owner outside lock",
            Box::new(move |v| {
                v["lock"]["definition_selections"][0]["identity"] = json!("other-model");
                let replacement = v["lock"]["definition_selections"].clone();
                v["identity_preimage"]["definition_selections"] = replacement;
            }),
            "/semantic_graph/nodes/3/nominal_identity_preimage/owner",
        ),
    ];
    for (name, mutate, path) in cases {
        let mut mutated = base.clone();
        mutate(&mut mutated);
        refresh_identity(&mut mutated);
        assert_eq!(
            refused(&mutated, &evidence_for(&mutated)),
            nominal(path),
            "{name}"
        );
    }
}

const DOMAIN_PACKAGE_DIGEST: &str =
    "5555555555555555555555555555555555555555555555555555555555555555";

/// The digest `evidence_for` supplies the document under, for `identity`
/// at version `1`.
fn domain_package_digest_of(identity: &str) -> String {
    domain_package_digest(&domain_package_document(identity, "1", Vec::new()))
}

fn domain_package(identity: &str) -> Value {
    json!({
        "identity": identity, "version": "1",
        "digest_domain": "sha256-jcs", "digest": domain_package_digest_of(identity)
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
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    let base = model_owned_package(owner.clone(), json!([domain_package("test/orders")]));
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
            nominal("/semantic_graph/nodes/0/nominal_identity_preimage/owner"),
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
    // Each points at the first unknown member, inside the internally tagged
    // preimage and owner as well as in the lock.
    let retired = [
        (
            "compiled-model owner with export",
            model_owned_package(
                json!({"kind": "model", "authority": "agent-ix",
                       "identity": "test/orders", "export": "Status"}),
                json!([domain_package("test/orders")]),
            ),
            // The decoder reads the identity projection's mirror of the node
            // first, and names the first unknown member of its owner.
            "/identity_preimage/identity_projection/0/nominal_identity_preimage/owner/authority",
        ),
        (
            "domain package owner with export",
            model_owned_package(
                json!({"kind": "model", "identity": "test/orders",
                       "node": "ix://test/orders/Status", "export": "Status"}),
                json!([domain_package("test/orders")]),
            ),
            "/identity_preimage/identity_projection/0/nominal_identity_preimage/owner/export",
        ),
        (
            "compiled-model lock reference",
            {
                let mut value = base.clone();
                value["lock"]["model_selections"] = json!([compiled_ref]);
                refresh_identity(&mut value);
                value
            },
            "/identity_preimage/model_selections/0/authority",
        ),
        (
            "domain package reference with export",
            {
                let mut value = base.clone();
                value["lock"]["model_selections"][0]["export"] = json!("Status");
                refresh_identity(&mut value);
                value
            },
            "/identity_preimage/model_selections/0/export",
        ),
    ];
    let evidence = evidence_for(&base);
    for (name, mutated, path) in retired {
        assert_eq!(
            refused(&mutated, &evidence),
            refusal(CheckedPackageRefusalCode::UnknownMember, path),
            "{name}"
        );
    }

    // Domain, shape and evidence are checked in the domain package domain,
    // each at the member of the one selection it is about.
    let model = |member: &str| format!("/lock/model_selections/0/{member}");
    let mut compiled_domain = base.clone();
    compiled_domain["lock"]["model_selections"][0]["digest_domain"] =
        json!("quire.compiled-model.bytes/v1");
    refresh_identity(&mut compiled_domain);
    assert_eq!(
        refused(&compiled_domain, &evidence),
        refusal(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            &model("digest_domain")
        )
    );
    let mut empty_version = base.clone();
    empty_version["lock"]["model_selections"][0]["version"] = json!("");
    refresh_identity(&mut empty_version);
    assert_eq!(
        refused(&empty_version, &evidence),
        refusal(CheckedPackageRefusalCode::MalformedWire, &model("version"))
    );
    let mut other_version = base.clone();
    other_version["lock"]["model_selections"][0]["version"] = json!("2");
    refresh_identity(&mut other_version);
    assert_eq!(
        refused(&other_version, &evidence),
        refusal_cause(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &model("version"),
            CheckedPackageRefusalCause::WrongModelSelection
        ),
        "the supplied document names version 1, not the selected 2"
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
        refusal_cause(
            CheckedPackageRefusalCode::MissingImport,
            &model("digest"),
            CheckedPackageRefusalCause::MissingSelection
        ),
        "equal digest bytes attested as a raw artifact never satisfy a domain package"
    );
}

/// Tracing: TC-048, FR-038-AC-10
#[trace("TC-048", "FR-038-AC-10")]
#[test]
fn tc_048_duplicate_model_selection_refuses_as_malformed_wire() {
    let owner = model_owner("test/orders", "ix://test/orders/Status");

    // A verbatim repeat (identity, version, digest_domain and digest all
    // equal) violates the closed schema's `uniqueItems` on `model_selections`;
    // `model_owned_package` mirrors the lock into the identity preimage via
    // `refresh_identity`, so the repeat is present in both members at once
    // and reaches the new uniqueness check.
    let duplicated = model_owned_package(
        owner.clone(),
        json!([domain_package("test/orders"), domain_package("test/orders")]),
    );
    assert_eq!(
        refused(&duplicated, &evidence_for(&duplicated)),
        // The second occurrence is the repeat.
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            "/lock/model_selections/1"
        )
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
    assert_ne!(
        lock_only["lock"]["model_selections"], lock_only["identity_preimage"]["model_selections"],
        "the repeat is confined to the lock, not mirrored into the preimage"
    );
    assert_eq!(
        refused(&lock_only, &evidence_for(&lock_only)),
        // The lock's array differs from its preimage mirror in length.
        refusal(
            CheckedPackageRefusalCode::StaleDependency,
            "/lock/model_selections"
        )
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
    assert_eq!(
        refused(&distinct_digest, &evidence_for(&distinct_digest)),
        refusal(
            CheckedPackageRefusalCode::StaleDependency,
            // The later row names a second digest for one locator.
            "/lock/model_selections/1/digest"
        ),
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
    // The code never depends on position; the pointer names the repeat's
    // second occurrence wherever it sits.
    for (name, package, repeat) in [
        (
            "duplicate before stale",
            &duplicate_before_stale,
            "/lock/model_selections/1",
        ),
        (
            "stale before duplicate",
            &stale_before_duplicate,
            "/lock/model_selections/2",
        ),
    ] {
        assert_eq!(
            refused(package, &evidence),
            refusal(CheckedPackageRefusalCode::MalformedWire, repeat),
            "{name}: the uniqueness check runs over the whole array before any digest is evaluated"
        );
    }
}

/// Each `model_selections` defect class is swept over the whole array before
/// the next class is evaluated over any of it, so an array carrying two
/// classes refuses for the earlier class whichever entry comes first. This
/// function pins the boundaries among FR-038's classes 1, 3, 4 and 5 --
/// repeated entry, then declared-domain mismatch, then shape defect, then a
/// digest the evidence does not attest. Class 2 (same identity, different
/// version) is pinned separately, against classes 3 and 5, by
/// `tc_048_model_selection_same_identity_different_version_refuses_as_malformed_wire`
/// below.
///
/// Three of the four adjacent boundaries among 1, 3, 4 and 5 are pinned
/// below: (3,4) and (4,5) through `boundaries`, (1,5) through
/// `tc_048_model_selection_duplicate_outranks_stale_digest_regardless_of_position`
/// above. The fourth,
/// repeated-entry (1) against declared-domain-mismatch (3), is pinned at the
/// end of this function -- separately, since a repeated entry needs two
/// physical array slots rather than the single entry each other class uses.
/// Unlike the other boundaries, this one does not regress under the
/// per-entry short-circuit this ticket replaces: #122 already made the
/// repeated-entry check a whole-array pass that runs before any per-entry
/// class check, so it was already position-independent against every other
/// class. Pinned anyway as a stated invariant, not a reproduction of a bug
/// that predates this fix.
///
/// Tracing: TC-048, FR-038-AC-19
#[trace("TC-048", "FR-038-AC-19")]
#[test]
fn tc_048_model_selection_refusal_is_decided_by_defect_class_not_array_position() {
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    // The owner joins this selection, so every array below carries it; the
    // evidence built from it attests this entry and no other.
    let joined = domain_package("test/orders");
    let evidence = evidence_for(&model_owned_package(owner.clone(), json!([joined.clone()])));
    // One entry per defect class, each naming its own identity so that no
    // array below also repeats an entry.
    let cross_domain = json!({
        "identity": "test/cross-domain", "version": "1",
        "digest_domain": "quire.compiled-model.bytes/v1", "digest": DOMAIN_PACKAGE_DIGEST
    });
    let malformed = json!({
        "identity": "", "version": "1",
        "digest_domain": "sha256-jcs", "digest": DOMAIN_PACKAGE_DIGEST
    });
    let unattested = json!({
        "identity": "test/other", "version": "1",
        "digest_domain": "sha256-jcs", "digest": DOMAIN_PACKAGE_DIGEST
    });
    // Each adjacent class boundary, pinned in both orderings against the same
    // expected code: an outcome decided by array position fails one ordering
    // of each pair rather than passing both. The pointer follows the
    // earlier-class entry to wherever it sits, at the member at fault.
    let boundaries = [
        (
            "cross-domain outranks malformed shape",
            CheckedPackageRefusalCode::DigestDomainMismatch,
            &cross_domain,
            &malformed,
            "digest_domain",
        ),
        (
            "malformed shape outranks unattested digest",
            CheckedPackageRefusalCode::MalformedWire,
            &malformed,
            &unattested,
            "identity",
        ),
    ];
    for (boundary, code, earlier, later, member) in boundaries {
        let earlier_first = model_owned_package(
            owner.clone(),
            json!([joined.clone(), earlier.clone(), later.clone()]),
        );
        let later_first = model_owned_package(
            owner.clone(),
            json!([joined.clone(), later.clone(), earlier.clone()]),
        );
        for (order, package, at) in [
            ("earlier class first", &earlier_first, 1),
            ("later class first", &later_first, 2),
        ] {
            assert_eq!(
                refused(package, &evidence),
                refusal(code, &format!("/lock/model_selections/{at}/{member}")),
                "{boundary}, {order}: the refusal is decided by defect class, not array position"
            );
        }
    }
    // The remaining boundary, class 1 (repeated entry, #122's whole-array uniqueness check)
    // against class 3 (declared-domain mismatch): a repeated entry needs two physical array
    // slots, so it doesn't fit the single-entry `boundaries` loop above. Both orderings still
    // refuse `MalformedWire`, the duplicate check's own code, regardless of whether the
    // repeated pair or the mismatched entry appears first.
    let duplicated = json!({
        "identity": "test/duplicated", "version": "1",
        "digest_domain": "sha256-jcs", "digest": DOMAIN_PACKAGE_DIGEST
    });
    let duplicate_first = model_owned_package(
        owner.clone(),
        json!([
            joined.clone(),
            duplicated.clone(),
            duplicated.clone(),
            cross_domain.clone()
        ]),
    );
    let mismatch_first = model_owned_package(
        owner.clone(),
        json!([
            joined.clone(),
            cross_domain.clone(),
            duplicated.clone(),
            duplicated.clone()
        ]),
    );
    for (order, package, repeat) in [
        (
            "duplicate pair first",
            &duplicate_first,
            "/lock/model_selections/2",
        ),
        (
            "mismatched entry first",
            &mismatch_first,
            "/lock/model_selections/3",
        ),
    ] {
        assert_eq!(
            refused(package, &evidence),
            refusal(CheckedPackageRefusalCode::MalformedWire, repeat),
            "repeated entry outranks declared-domain mismatch, {order}: the refusal is decided \
             by defect class, not array position"
        );
    }
}

/// A `lock.model_selections` array holding two selections of one identity at
/// different versions refuses as `malformed_wire` even when both selections
/// are individually well-formed and individually attested: the nominal
/// `model` owner joins by identity alone (`identity::validate_owner`), so
/// admitting two versions under one identity would leave that join ambiguous
/// -- which version an owner of the identity names is undecidable. This is a
/// distinct rule from FR-038-AC-10's whole-item `uniqueItems` refusal, which
/// requires every member (including `version`) to match verbatim and so
/// never reaches a pair that differs by version; it is also distinct from
/// AC-10's same-identity-*and*-same-version-different-digest case, which
/// shares one locator and so continues to refuse deterministically as
/// `stale_dependency` -- this rule is not widened to reach it. This rule is
/// class 2 of FR-038's five-class `model_selections` order and is swept
/// before `validate_domain_packages`, so it outranks class 3
/// (`digest_domain_mismatch`) and class 5 (`stale_dependency`): a
/// same-identity, different-version pair refuses `malformed_wire` even when
/// one of its entries also carries a domain-mismatch or an unattested-digest
/// defect.
///
/// Tracing: TC-048, FR-038-AC-20
#[trace("TC-048", "FR-038-AC-20")]
#[test]
fn tc_048_model_selection_same_identity_different_version_refuses_as_malformed_wire() {
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    // Whichever order the pair takes, the later entry's `version` is the one
    // that conflicts with the identity's first selection.
    let lock_path = "/lock/model_selections/1/version";

    // Two selections of one identity at different versions: each has its own
    // locator (identity, version), so each is individually well-formed and
    // individually attestable by the package evidence -- this rule refuses
    // the pair before either digest is ever checked against evidence.
    let version_one = domain_package("test/orders");
    let mut version_two = domain_package("test/orders");
    version_two["version"] = json!("2");
    version_two["digest"] = json!(domain_package_digest(&domain_package_document(
        "test/orders",
        "2",
        Vec::new()
    )));
    let both_versions = model_owned_package(
        owner.clone(),
        json!([version_one.clone(), version_two.clone()]),
    );
    assert_eq!(
        refused(&both_versions, &evidence_for(&both_versions)),
        refusal(CheckedPackageRefusalCode::MalformedWire, lock_path),
        "same identity, different versions, both individually attested"
    );

    // The same pair, reversed: the outcome is decided by defect class, not
    // by which entry the reader reaches first (mirroring FR-038-AC-11's
    // array-position independence for the pre-existing classes).
    let reversed = model_owned_package(
        owner.clone(),
        json!([version_two.clone(), version_one.clone()]),
    );
    assert_eq!(
        refused(&reversed, &evidence_for(&reversed)),
        refusal(CheckedPackageRefusalCode::MalformedWire, lock_path),
        "same pair, reversed array order"
    );

    // This class is swept before `validate_domain_packages` below, so it
    // outranks both classes 3 and 5 (FR-038-AC-20): a same-identity,
    // different-version pair still refuses `malformed_wire`, never
    // `digest_domain_mismatch` or `stale_dependency`, even when the later
    // entry also carries one of those defects.
    let mut version_two_wrong_domain = version_two.clone();
    version_two_wrong_domain["digest_domain"] = json!("quire.compiled-model.bytes/v1");
    let outranks_domain_mismatch = model_owned_package(
        owner.clone(),
        json!([version_one.clone(), version_two_wrong_domain.clone()]),
    );
    let outranks_domain_mismatch_reversed = model_owned_package(
        owner.clone(),
        json!([version_two_wrong_domain, version_one.clone()]),
    );
    for (name, package) in [
        (
            "version 1 then domain-mismatched version 2",
            &outranks_domain_mismatch,
        ),
        (
            "domain-mismatched version 2 then version 1",
            &outranks_domain_mismatch_reversed,
        ),
    ] {
        assert_eq!(
            refused(package, &evidence_for(package)),
            refusal(CheckedPackageRefusalCode::MalformedWire, lock_path),
            "{name}: class 2 outranks class 3 (digest_domain_mismatch)"
        );
    }

    // Evidence attests only the `@1` selection, so `@2` would otherwise be
    // `stale_dependency` -- class 2 must still be decided first.
    let single_version_evidence = evidence_for(&model_owned_package(
        owner.clone(),
        json!([version_one.clone()]),
    ));
    let outranks_stale = model_owned_package(
        owner.clone(),
        json!([version_one.clone(), version_two.clone()]),
    );
    let outranks_stale_reversed =
        model_owned_package(owner.clone(), json!([version_two, version_one.clone()]));
    for (name, package) in [
        ("version 1 then version 2", &outranks_stale),
        ("version 2 then version 1", &outranks_stale_reversed),
    ] {
        assert_eq!(
            refused(package, &single_version_evidence),
            refusal(CheckedPackageRefusalCode::MalformedWire, lock_path),
            "{name}: class 2 outranks class 5 (stale_dependency)"
        );
    }

    // Two different identities never trigger this rule and still admit.
    let other_identity = domain_package("test/other");
    let distinct_identities = model_owned_package(owner, json!([version_one, other_identity]));
    admitted(&distinct_identities);
}

/// Tracing: TC-048, FR-038-AC-2
#[trace("TC-048", "FR-038-AC-2")]
#[test]
fn tc_048_model_export_is_not_a_v2_model_form() {
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
        admitted(&value);
    }
    let mut export = base.clone();
    export["semantic_graph"]["nodes"][model]["semantic_form"] = json!("model_export");
    refresh_identity(&mut export);
    assert_eq!(
        refused(&export, &evidence_for(&export)),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{model}/semantic_form")
        )
    );
}

/// Tracing: TC-048, FR-038-AC-9, FR-038-AC-26
#[trace("TC-048", "FR-038-AC-9", "FR-038-AC-26")]
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
                // Every limit but the byte limit names the value it charged,
                // and that pointer resolves in the package read.
                let path = report.path.as_ref().map(|path| path.as_str().to_owned());
                assert_eq!(
                    path.is_some(),
                    kind != CheckedPackageLimit::Bytes,
                    "{kind:?} pointer presence"
                );
                if let Some(path) = &path {
                    assert!(value.pointer(path).is_some(), "{kind:?} {path}");
                }
                assert_eq!(
                    report,
                    incomplete(kind, hi - 1, hi, path.as_deref()),
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
/// though `validate_term`'s recursive walk reports every one of those as the
/// same `literal.type` member: only the body's own outermost
/// term is the node's `literal.type` in FR-322's sense (FR-038-AC-18).
///
/// Tracing: TC-048, FR-038-AC-18
#[trace("TC-048", "FR-038-AC-18")]
#[test]
fn tc_048_self_typed_carve_out_is_keyed_on_literal_type_member_and_body_root() {
    let base = v2_all_families();
    let own_id = base["semantic_graph"]["nodes"][1]["node_id"].clone();
    let other_id = base["semantic_graph"]["nodes"][1]["semantic_type"].clone();

    // The node lacks the `recursion_group` member its self-cycle requires,
    // so the refusal points at the node.
    let recursion_group_refusal = refusal(
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        "/semantic_graph/nodes/1",
    );
    // An `application`-termed body embedding a literal self-reference to its
    // own node_id is, once `validate_application_keys` exists, a
    // cryptographic impossibility to construct honestly: a node's digest is
    // derived from its body, so a body cannot legitimately embed that same
    // digest as one of its own values without a hash-preimage attack. The
    // stale-key check (run ahead of recursion, IR-216) therefore now catches
    // this shape before recursion detection ever sees it — a more
    // fundamental defect than a bare `recursion_group` refusal, not a
    // regression of the carve-out's own coverage (cases 1-3 below, which
    // never touch `validate_application_keys`, still exercise it directly).
    let stale_application_key_refusal = refusal_at(
        CheckedPackageRefusalCode::InvalidPackage,
        "/semantic_graph/nodes/1/node_id",
        Some(CheckedPackageRefusalCause::StaleNodeKey),
        own_id["digest"].as_str().expect("own_id digest"),
    );

    let assert_self_cycle_refused = |body: Value, expected: CheckedPackageRefusal| {
        let mut mutated = base.clone();
        mutated["semantic_graph"]["nodes"][1]["semantic_type"] = own_id.clone();
        mutated["semantic_graph"]["nodes"][1]["body"] = body;
        refresh_identity(&mut mutated);
        assert_eq!(refused(&mutated, &evidence_for(&mutated)), expected);
    };

    // Negative: a self-typed node whose body is a `reference` term naming
    // itself is a genuine 1-node cycle, not the carve-out's case. The carve-
    // out regressed this: it used to key on node identity alone
    // (`semantic_type == position`), which also swallowed this case with no
    // `recursion_group` on `origin/main`'s vendored fixture.
    assert_self_cycle_refused(
        json!({"term": "reference", "target": own_id}),
        recursion_group_refusal.clone(),
    );

    // Negative: the same self-typed `literal.type` self-reference, nested
    // one level inside an `aggregate` member instead of being the body's own
    // top-level term, is still a genuine 1-node cycle. `validate_term`
    // reports this nested literal's `type` as the same `literal.type` member
    // as the body-root case, so the carve-out must not key on the member
    // alone.
    assert_self_cycle_refused(
        json!({
            "term": "aggregate",
            "members": [
                {"term": "literal", "type": own_id, "value_kind": "integer", "value": 1}
            ]
        }),
        recursion_group_refusal.clone(),
    );

    // Negative: the same self-reference nested inside a `binding` value.
    assert_self_cycle_refused(
        json!({
            "term": "binding",
            "name": "x",
            "value": {"term": "literal", "type": own_id, "value_kind": "integer", "value": 1}
        }),
        recursion_group_refusal.clone(),
    );

    // Negative: the same self-reference nested inside an `application`
    // argument. The body's own top-level term is now `application`, so
    // `validate_application_keys` (IR-216) evaluates this node's key before
    // recursion detection runs, and refuses it as stale — see
    // `stale_application_key_refusal` above for why that is the correct,
    // more fundamental defect rather than a masked recursion assertion.
    assert_self_cycle_refused(
        json!({
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
        }),
        stale_application_key_refusal.clone(),
    );

    // Negative: a self-typed node whose body-root `application.result_type`
    // names itself is the same genuine 1-node cycle as the `reference` body
    // case above — `application.result_type` is never exempt, whether or not
    // the argument is itself self-referencing. Same stale-key reasoning as
    // the previous case applies here too.
    assert_self_cycle_refused(
        json!({
            "term": "application",
            "operator": "call",
            "operation": {
                "identity": "quire.op.function.call",
                "laws": [],
                "mode": null,
                "member": null,
                "leaves": []
            },
            "result_type": own_id,
            "arguments": [
                {"term": "literal", "type": other_id, "value_kind": "integer", "value": 1}
            ]
        }),
        stale_application_key_refusal,
    );

    // Positive: a self-typed node's own body-root `literal.type`
    // self-reference is the carve-out's intended case and admits with no
    // `recursion_group`; the same package also carries a sibling node whose
    // nested `literal.type` names a *different* node — one admission
    // exercising both the exempt body-root case and an ordinary nested
    // reference the narrowing must not disturb.
    let mut literal_type = base.clone();
    literal_type["semantic_graph"]["nodes"][1]["semantic_type"] = own_id.clone();
    literal_type["semantic_graph"]["nodes"][1]["body"] = json!({
        "term": "literal", "type": own_id, "value_kind": "integer", "value": 1
    });
    literal_type["semantic_graph"]["nodes"][2]["body"] = json!({
        "term": "aggregate",
        "members": [
            {"term": "literal", "type": other_id, "value_kind": "integer", "value": 1}
        ]
    });
    refresh_identity(&mut literal_type);
    let package = admitted(&literal_type);
    assert_eq!(
        package.graph().nodes[1].recursion_group,
        None,
        "a self-typed literal.type self-reference is not a cycle"
    );
    assert_eq!(
        package.graph().nodes[2].recursion_group,
        None,
        "a nested literal typed by a different node is not a cycle"
    );
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
    let base = positive_operation_identities();
    // The base fixture must itself be admissible before any of the deletion
    // cases below can say anything about which member's absence refuses it:
    // an already-refusing base document would make every case below pass
    // vacuously, for whatever reason the base already refuses rather than
    // the deletion under test.
    admitted(&base);
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
            // The node lacks the member it requires.
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                &format!("/semantic_graph/nodes/{declaring_node}"),
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
            // The term lacks a member its closed shape requires.
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                &format!("/semantic_graph/nodes/{literal_node}/body"),
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
            // The term lacks a member its closed shape requires.
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                &format!("/semantic_graph/nodes/{application_node}/body"),
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
            // The term lacks a member its closed shape requires.
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                &format!("/semantic_graph/nodes/{application_node}/body"),
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

/// FR-038-AC-17's closed `expression` form list: every one of the fifteen
/// `ExpressionForm::ALL` admits as a node form, and a
/// sixteenth, undeclared form refuses as `invalid_semantic_graph`.
/// `tc_048_model_export_is_not_a_v2_model_form` already covers the same
/// pattern for the eighteen `model` forms — this test is the `expression`
/// counterpart, not a repeat of it. (The schema-vs-`CheckedNodeTag::forms`
/// cross-check this test used to also run was dropped along with the
/// vendored schema it needed; see AGE-1961.)
///
/// Tracing: TC-048, FR-038-AC-17
#[trace("TC-048", "FR-038-AC-17")]
#[test]
fn tc_048_expression_forms_are_exactly_fifteen_and_bound_admission() {
    let base = v2_all_families();
    let expression = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == json!("expression"))
        .expect("all-families fixture carries an expression node");
    assert_eq!(ExpressionForm::ALL.len(), 15);
    for form in ExpressionForm::ALL {
        let mut value = base.clone();
        value["semantic_graph"]["nodes"][expression]["semantic_form"] = json!(form.as_wire());
        refresh_identity(&mut value);
        admitted(&value);
    }
    let mut sixteenth = base.clone();
    sixteenth["semantic_graph"]["nodes"][expression]["semantic_form"] = json!("future_expression");
    refresh_identity(&mut sixteenth);
    assert_eq!(
        refused(&sixteenth, &evidence_for(&sixteenth)),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{expression}/semantic_form")
        )
    );
}

fn digest_of(node: &Value) -> String {
    node["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned()
}

/// The pointer of the declared name of the node keyed `digest` in `package`:
/// a declaration-name refusal is located at that name.
fn declared_name_at(package: &Value, digest: &str) -> String {
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| digest_of(node) == digest)
        .expect("node with that key");
    format!("/semantic_graph/nodes/{position}/declaration/qualified_name")
}

/// Tracing: TC-048, FR-038-AC-21
#[trace("TC-048", "FR-038-AC-21")]
#[test]
fn tc_048_declared_name_must_equal_its_nominal_preimage_name() {
    // Node 1 of the nominal fixture is the enum declaration; its preimage
    // fixes `qualified_declaration`, so its declared name must equal it.
    let mut value = v2_nominal();
    let declaration = 1;
    value["semantic_graph"]["nodes"][declaration]["declaration"]["qualified_name"] =
        json!(["Example", "Renamed"]);
    refresh_identity(&mut value);
    assert_eq!(
        refused(&value, &evidence_for(&value)),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{declaration}/declaration/qualified_name"),
            Some(CheckedPackageRefusalCause::DeclarationNominalMismatch),
            &digest_of(&value["semantic_graph"]["nodes"][declaration]),
        )
    );
}

/// The nominal fixture with its dimension renamed to its unit's declared name,
/// re-keyed through every dependant so every nominal join still holds: the
/// only remaining defect is two nodes declaring one name. Returns the package
/// and the two nodes' key digests.
fn with_shared_name() -> (Value, [String; 2]) {
    let version = |preimage: &Value| preimage["version"].as_str().map(str::to_owned);
    let mut members = nominal_fixture_members();
    // Dependency order, so one rekey pass carries the rename to every
    // dependant: a unit names its dimension's key.
    let order = [
        "quire.enum-declaration-node/v1",
        "quire.enum-member-node/v1",
        "quire.dimension-node/v1",
        "quire.unit-node/v1",
    ];
    members.sort_by_key(|(preimage, _)| {
        order
            .iter()
            .position(|kind| version(preimage).as_deref() == Some(*kind))
            .expect("known nominal kind")
    });
    let unit_name = members[3].0["qualified_declaration"].clone();
    members[2].0["qualified_declaration"] = unit_name;
    let (mut preimages, keys): (Vec<_>, Vec<_>) = members.into_iter().unzip();
    let fresh = rekey(&mut preimages, &keys);
    let digests = [fresh[2].clone(), fresh[3].clone()];
    let mut value = nominal_package(&preimages.into_iter().zip(fresh).collect::<Vec<_>>());
    refresh_identity(&mut value);
    (value, digests)
}

/// Tracing: TC-048, FR-038-AC-21
#[trace("TC-048", "FR-038-AC-21")]
#[test]
fn tc_048_two_nodes_declaring_one_name_refuse_as_ambiguous() {
    let (value, digests) = with_shared_name();
    let least = digests.iter().min().expect("two digests");
    assert_eq!(
        refused(&value, &evidence_for(&value)),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &declared_name_at(&value, least),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            least,
        )
    );
}

/// Tracing: TC-048, FR-038-AC-21
#[trace("TC-048", "FR-038-AC-21")]
#[test]
fn tc_048_a_nominal_name_mismatch_is_reported_before_an_ambiguous_name() {
    // Both defects at once: the shared name, and the enum declaration's
    // declared name moved off its preimage's. The nominal join runs first.
    let (mut value, _) = with_shared_name();
    let declaration = value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == json!("enum"))
        .expect("enum declaration");
    value["semantic_graph"]["nodes"][declaration]["declaration"]["qualified_name"] =
        json!(["Example", "Renamed"]);
    refresh_identity(&mut value);
    assert_eq!(
        refused(&value, &evidence_for(&value)),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{declaration}/declaration/qualified_name"),
            Some(CheckedPackageRefusalCause::DeclarationNominalMismatch),
            &digest_of(&value["semantic_graph"]["nodes"][declaration]),
        )
    );
}

/// Gives the node at `position` a `declaration`-role occurrence and the
/// declared name `name`.
fn declare(package: &mut Value, position: usize, name: &[&str]) {
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    node["declaration"] = json!({"qualified_name": name});
}

/// Tracing: TC-048, FR-038-AC-21
#[trace("TC-048", "FR-038-AC-21")]
#[test]
fn tc_048_a_declaration_refusal_precedes_an_operation_refusal() {
    // Nodes: 0 root, 1 declaring model ("Example::Widget"), 2 literal,
    // 3 plain function, 4 application call.
    let with_operation_defect = || {
        let mut value = positive_operation_identities();
        value["semantic_graph"]["nodes"][4]["body"]["operation"]["identity"] =
            json!("quire.op.unknown");
        checked_package::rekey_application_node(&mut value, 4);
        value
    };
    // Control: the operation defect alone is refused at the operation stage.
    let mut operation_only = with_operation_defect();
    refresh_identity(&mut operation_only);
    let control = refused(&operation_only, &evidence_for(&operation_only));
    assert_eq!(
        control.cause,
        Some(CheckedPackageRefusalCause::UnknownOperation)
    );
    // With a second node declaring the same name, the declaration refusal wins.
    let mut both = with_operation_defect();
    declare(&mut both, 3, &["Example", "Widget"]);
    checked_package::rebuild_source_map(&mut both);
    refresh_identity(&mut both);
    let declaring = digest_of(&both["semantic_graph"]["nodes"][1]);
    let function = digest_of(&both["semantic_graph"]["nodes"][3]);
    assert_eq!(
        refused(&both, &evidence_for(&both)),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &declared_name_at(&both, declaring.clone().min(function.clone()).as_str()),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            declaring.min(function).as_str(),
        )
    );
}

/// Tracing: TC-048, FR-038-AC-21
#[trace("TC-048", "FR-038-AC-21")]
#[test]
fn tc_048_a_declaration_refusal_precedes_a_frame_refusal() {
    let with_frame_defect = || {
        let mut value = v2_all_families();
        let frame = value["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_tag"] == "state" && node["semantic_form"] == "frame")
            .expect("frame node");
        value["semantic_graph"]["nodes"][frame]["body"]["modifies"] =
            json!([checked_package::node_id(&"0123456789abcdef".repeat(4))]);
        value
    };
    // Control: the frame defect alone is refused at the frame stage.
    let mut frame_only = with_frame_defect();
    refresh_identity(&mut frame_only);
    let control = refused(&frame_only, &evidence_for(&frame_only));
    assert_eq!(control.code, CheckedPackageRefusalCode::MissingDeclaration);
    // Two plain nodes declaring one name: the declaration refusal wins.
    let mut both = with_frame_defect();
    declare(&mut both, 0, &["Example", "Dup"]);
    declare(&mut both, 1, &["Example", "Dup"]);
    checked_package::rebuild_source_map(&mut both);
    refresh_identity(&mut both);
    let first = digest_of(&both["semantic_graph"]["nodes"][0]);
    let second = digest_of(&both["semantic_graph"]["nodes"][1]);
    assert_eq!(
        refused(&both, &evidence_for(&both)),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &declared_name_at(&both, first.clone().min(second.clone()).as_str()),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            first.min(second).as_str(),
        )
    );
}

/// The first `reference` target anywhere in a body term.
fn first_reference(term: &Value) -> Option<Value> {
    if term["term"] == "reference" {
        return Some(term["target"].clone());
    }
    ["arguments", "members"]
        .iter()
        .filter_map(|key| term.get(*key).and_then(Value::as_array))
        .flatten()
        .chain(term.get("value"))
        .find_map(first_reference)
}

/// Tracing: TC-048, FR-038-AC-17
#[trace("TC-048", "FR-038-AC-17")]
#[test]
fn tc_048_an_application_node_in_a_recursion_group_keys_by_fr322_ordinals() {
    let base = v2_all_families();
    let position_of = |value: &Value, test: &dyn Fn(&Value) -> bool| {
        value["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(test)
            .expect("node")
    };
    let function_key = json!(checked_package::family_key("ffff"));
    let function = position_of(&base, &|node| node["node_id"]["digest"] == function_key);
    let target = first_reference(&base["semantic_graph"]["nodes"][function]["body"])
        .expect("the call references a node");
    // Groups the application node with the node it calls, placing the
    // application first or second in graph order, and re-keys it by FR-322.
    // Returns the package and the group's key digests in graph order.
    let grouped = |application_first: bool| {
        let mut value = base.clone();
        let nodes = value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        let function = nodes
            .iter()
            .position(|node| node["node_id"]["digest"] == function_key)
            .expect("function");
        let referenced = nodes
            .iter()
            .position(|node| node["node_id"] == target)
            .expect("referenced");
        if (function < referenced) != application_first {
            nodes.swap(function, referenced);
        }
        let (function, referenced) = (function.min(referenced), function.max(referenced));
        let (function, referenced) = if application_first {
            (function, referenced)
        } else {
            (referenced, function)
        };
        for position in [function, referenced] {
            nodes[position]["recursion_group"] = json!("g");
        }
        let group = [function.min(referenced), function.max(referenced)]
            .map(|position| nodes[position]["node_id"].clone());
        let stale = nodes[function]["node_id"].clone();
        let fresh_digest = checked_package::application_key_in_group(&nodes[function], &group);
        let mut fresh = stale.clone();
        fresh["digest"] = json!(fresh_digest);
        replace_everywhere(&mut value, &stale, &fresh);
        refresh_identity(&mut value);
        let nodes = value["semantic_graph"]["nodes"].as_array().expect("nodes");
        let in_graph_order = [function.min(referenced), function.max(referenced)]
            .map(|position| digest_of(&nodes[position]));
        (value, in_graph_order)
    };
    // FR-322 orders a group by graph position, not by key. Use the placement
    // whose graph order disagrees with digest order, so ordering the group by
    // key instead would derive a different key and refuse.
    let (value, order) = [true, false]
        .into_iter()
        .map(grouped)
        .find(|(_, order)| order[0] > order[1])
        .expect("one placement puts the higher digest first in graph order");
    assert!(order[0] > order[1]);
    admitted(&value);

    // A key derived as if the node were outside any group is stale once it
    // joins one.
    let mut bare = base.clone();
    for position in [
        function,
        position_of(&base, &|node| node["node_id"] == target),
    ] {
        bare["semantic_graph"]["nodes"][position]["recursion_group"] = json!("g");
    }
    refresh_identity(&mut bare);
    assert_eq!(
        refused(&bare, &evidence_for(&bare)),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!(
                "/semantic_graph/nodes/{}/node_id",
                position_of(&bare, &|node| {
                    node["node_id"]["digest"] == checked_package::family_key("ffff")
                })
            ),
            Some(CheckedPackageRefusalCause::StaleNodeKey),
            &checked_package::family_key("ffff"),
        )
    );
}
