// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Strict CheckedPackage V2 reader: locally authored adverse mutations, injected
//! refusals, resource limits, package identity, and nominal node identity.

use crate::support::checked_package::{
    self, all_families_read_work, canonical, domain_package_document, evidence_for,
    evidence_with_document, family_key, incomplete, node_id, nominal_fixture_members,
    nominal_package, pointer as support_pointer, positive_operation_identities, refresh_identity,
    refusal, refusal_at, refusal_bytes, refusal_cause, rekey, select_model, sha256_hex,
    unknown_version, v2_all_families, v2_nominal, ExpectedRefusal, COMPLETE_VALUE_FEATURE,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    read_checked_package, CheckedOccurrenceRole, CheckedPackageDispatchResult,
    CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
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

/// Reader-only FR-038-AC-4 subcase: the fixture supplies the ordinal and
/// source row, and the reader preserves them while checking their binding.
/// Compiler derivation from step order is covered separately by AC-87.
#[trace("TC-048", "FR-038-AC-4")]
#[test]
fn generated_occurrence_ordinal_and_source_region_survive_reading() {
    let mut package = v2_all_families();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["occurrences"][0]["role"] == "generated")
        .expect("generated node");
    let id = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    let map_position = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .position(|entry| entry["node_id"] == id && entry["role"] == "generated")
        .expect("generated source map entry");
    package["semantic_graph"]["nodes"][position]["occurrences"][0]["ordinal"] = json!(1);
    package["source_map"][map_position]["ordinal"] = json!(1);
    let first = admitted(&package);
    assert_eq!(
        first.graph().nodes[position].occurrences[0].role,
        CheckedOccurrenceRole::Generated
    );
    assert_eq!(first.graph().nodes[position].occurrences[0].ordinal, 1);
    assert_eq!(first.source_map()[map_position].ordinal, 1);

    // A different producer-supplied row region leaves the ordinal and
    // semantic node unchanged.
    let mut reordered_source = package.clone();
    reordered_source["source_map"][map_position]["regions"][0]["start"] = json!(1000);
    reordered_source["source_map"][map_position]["regions"][0]["end"] = json!(1001);
    let reordered = admitted(&reordered_source);
    assert_eq!(reordered.package_id(), first.package_id());
    assert_eq!(
        reordered.graph().nodes[position].node_id,
        first.graph().nodes[position].node_id
    );
    assert_eq!(reordered.graph().nodes[position].occurrences[0].ordinal, 1);
    assert_eq!(reordered.source_map()[map_position].ordinal, 1);
    assert_eq!(reordered.source_map()[map_position].regions[0].start, 1000);
    assert_eq!(reordered.source_map()[map_position].regions[0].end, 1001);

    let mut mismatched = package;
    mismatched["source_map"][map_position]["ordinal"] = json!(0);
    let refusal = refused(&mismatched, &evidence_for(&mismatched));
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidSourceMap);
    assert_eq!(
        refusal.path,
        Some(support_pointer(&format!("/source_map/{map_position}")))
    );
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
fn nominal(path: &str) -> ExpectedRefusal {
    refusal(CheckedPackageRefusalCode::InvalidSemanticGraph, path)
}

fn dispatch(bytes: &[u8], evidence: &CheckedPackageEvidence) -> CheckedPackageDispatchResult {
    read_checked_package(bytes, CheckedPackageReadLimits::bounded(), evidence)
}

fn assert_dispatch_refused(result: CheckedPackageDispatchResult, expected: ExpectedRefusal) {
    match result {
        CheckedPackageDispatchResult::Refused(actual) => assert_eq!(actual, expected),
        other => panic!("expected {expected:?}, got {other:?}"),
    }
}

/// The reader parses `contract_version` exactly once and admits only the
/// current contract; every other version, and every malformed document, is
/// refused before any version-specific decoding.
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
/// `lock.sources[0].authority`/`capability_report`, all checked
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
        "/lock/sources/0/authority",
        "malformed_wire",
        CheckedPackageRefusalCode::MalformedWire,
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
        "/lock/sources/0/authority" => json!(""),
        "/capability_report" => json!([]),
        other => panic!("no replacement authored for structural mutation pointer {other}"),
    }
}

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

    let cases: Vec<(&str, Mutation, ExpectedRefusal)> = vec![
        (
            "missing body term tag",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][4]["body"]
                    .as_object_mut()
                    .expect("body object")
                    .remove("term");
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/4/body",
            ),
        ),
        (
            "nonstring body term tag",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][4]["body"]["term"] = json!(7);
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/4/body/term",
            ),
        ),
        (
            "unknown body term tag",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][4]["body"]["term"] = json!("future_term");
                refresh_identity(v);
            }),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/4/body",
            ),
        ),
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
            "occurrence role spelled like a decoder message",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][0]["occurrences"][0]["role"] = json!("unknown field")
            }),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/semantic_graph/nodes/0/occurrences/0/role",
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
            // The sole graph node contains the explicit null the decoder dropped.
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/semantic_graph/nodes/1/recursion_group",
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
                checked_package::mint_ungrouped_structural_keys(v);
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
            "shipped projection",
            Box::new(|v| {
                v["identity_preimage"]["identity_projection"] = json!([]);
            }),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/identity_preimage/identity_projection",
            ),
        ),
        (
            "stale package id",
            Box::new(|v| v["package_id"]["digest"] = json!("0".repeat(64))),
            refusal_cause(
                CheckedPackageRefusalCode::StaleDependency,
                "/package_id/digest",
                CheckedPackageRefusalCause::ContentMismatch,
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
            // Nodes 3 and 6 form a dependency cycle without a group. The
            // separately keyed graph-cycle control below covers valid keys.
            Box::new(|v| {
                let first = v["semantic_graph"]["nodes"][3]["node_id"].clone();
                let second = v["semantic_graph"]["nodes"][6]["node_id"].clone();
                v["semantic_graph"]["nodes"][3]["dependencies"] = json!([second]);
                v["semantic_graph"]["nodes"][6]["dependencies"] = json!([first]);
                refresh_identity(v);
            }),
            // Neither node carries the member the cycle requires; the
            // lower-positioned one is named.
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/3",
            ),
        ),
        (
            "cycle split across recursion groups",
            Box::new(|v| {
                let first = v["semantic_graph"]["nodes"][3]["node_id"].clone();
                let second = v["semantic_graph"]["nodes"][6]["node_id"].clone();
                v["semantic_graph"]["nodes"][3]["dependencies"] = json!([second]);
                v["semantic_graph"]["nodes"][6]["dependencies"] = json!([first]);
                v["semantic_graph"]["nodes"][3]["recursion_group"] = json!("left");
                v["semantic_graph"]["nodes"][6]["recursion_group"] = json!("right");
                refresh_identity(v);
            }),
            // Each group's key would need to hash the other group's key, a
            // hash fixed point. Authentication therefore refuses the first
            // stale key before the graph can inspect the group mismatch.
            refusal_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/3/node_id",
                Some(CheckedPackageRefusalCause::StaleNodeKey),
                &digest_of(&base["semantic_graph"]["nodes"][3]),
            ),
        ),
        (
            "self dependency without a recursion group",
            Box::new(|v| {
                let own = v["semantic_graph"]["nodes"][3]["node_id"].clone();
                v["semantic_graph"]["nodes"][3]["dependencies"] = json!([own]);
                refresh_identity(v);
            }),
            // The node lacks the `recursion_group` member its cycle needs.
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/3",
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
    // integer tokens only: a fractional one refuses as a literal grammar
    // defect, and a whole one is not canonical bytes. The refusal is the same
    // whether or not serde_json's `arbitrary_precision` is unified into the
    // build.
    type Build<'a> = Box<dyn Fn(Value) -> Value + 'a>;
    let integer_type = node_id(&family_key("a3a3"));
    let body: Build = Box::new(|literal| {
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][3]["semantic_type"] = integer_type.clone();
        changed["semantic_graph"]["nodes"][3]["dependencies"] = json!([integer_type]);
        changed["semantic_graph"]["nodes"][3]["body"] =
            json!({"term":"literal","type":integer_type,"value_kind":"integer","value":literal});
        checked_package::mint_ungrouped_structural_keys(&mut changed);
        refresh_identity(&mut changed);
        changed
    });
    let detail: Build = Box::new(|literal| {
        let mut changed = base.clone();
        let target = integer_type.clone();
        // FR-208's `DiagnosticCausePairing` requires `cause_tag: "invalid-value"`
        // to pair with `code: "invalid_package"`.
        changed["diagnostics"]["entries"] = json!([{
            "stage":"type_checking","code":"invalid_package","cause_tag":"invalid-value",
            "details":[{"term":"literal","type":target,"value_kind":"integer","value":literal}],"loci":[]
        }]);
        changed
    });
    // Each refuses at the literal term itself.
    for (build, at) in [
        (body, "/semantic_graph/nodes/3/body"),
        (detail, "/diagnostics/entries/0/details/0"),
    ] {
        let integer_refusal = refusal(CheckedPackageRefusalCode::InvalidSemanticGraph, at);
        let integer = build(json!(-7));
        assert!(matches!(
            read(&integer, &evidence),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        let fractional = build(json!(1.5));
        assert_eq!(refused(&fractional, &evidence), integer_refusal);
        // A whole float is spelled `2` in RFC 8785, so its `2.0` spelling is
        // refused at intake as non-canonical bytes, before the literal
        // grammar is read (FR-038-AC-79).
        assert_eq!(
            refused(&build(json!(2.0)), &evidence),
            refusal_bytes(CheckedPackageRefusalCode::NoncanonicalWire)
        );
    }

    // A cycle is admitted once every member shares one explicit group.
    let mut grouped = base.clone();
    let first = grouped["semantic_graph"]["nodes"][3]["node_id"].clone();
    let second = grouped["semantic_graph"]["nodes"][6]["node_id"].clone();
    grouped["semantic_graph"]["nodes"][3]["dependencies"] = json!([second]);
    grouped["semantic_graph"]["nodes"][6]["dependencies"] = json!([first]);
    grouped["semantic_graph"]["nodes"][3]["recursion_group"] = json!("pair");
    grouped["semantic_graph"]["nodes"][6]["recursion_group"] = json!("pair");
    checked_package::mint_grouped_structural_keys(&mut grouped);
    checked_package::mint_ungrouped_structural_keys(&mut grouped);
    checked_package::settle(&mut grouped);
    admitted(&grouped);

    // Evidence the caller must supply: the required feature.
    let mut unsupported = CheckedPackageEvidence::new();
    let family_document = checked_package::family_model_document();
    unsupported
        .insert_domain_package_document("test/families", canonical(&family_document))
        .expect("domain evidence declarations agree on exact bytes");
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
    // is charged before any value exists; nodes, at the first node past the
    // ceiling; edges and occurrences, at the first dependency or source-map
    // region past it; diagnostics, at the first entry past it; work, at the
    // value whose validation took the meter over. There is no depth limit
    // (FR-038-AC-117).
    type Narrow = fn(&mut CheckedPackageReadLimits) -> u64;
    let narrowings: [(CheckedPackageLimit, Narrow, Option<&str>); 6] = [
        (
            CheckedPackageLimit::Bytes,
            |l| {
                l.bytes -= 1;
                l.bytes
            },
            None,
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
    let all_families_read_work = all_families_read_work();
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
            Some("/semantic_graph/nodes/16/dependencies/2")
        ))
    );
}

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

    // A secondary source's nominal identity is outside the preimage. Every
    // source-map reference moves with the locked source row.
    let mut source_identity = base.clone();
    let old_source = base["lock"]["sources"][1].clone();
    let mut new_source = old_source.clone();
    new_source["identity"] = json!("quire.fixture.source.secondary-renamed/v1");
    replace_everywhere(&mut source_identity, &old_source, &new_source);
    assert_ne!(source_identity["lock"], base["lock"]);
    assert_eq!(
        source_identity["identity_preimage"],
        base["identity_preimage"]
    );
    assert_eq!(*admitted(&source_identity).package_id(), recorded);

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
                v["lock"]["edition"]["definition"]["identity"] =
                    json!("quire.fixture.edition.next/v1");
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
                    sha256_hex(&canonical(
                        &crate::support::checked_package::identity_input(&changed)
                    )),
                    "{name}"
                );
            }
            other => panic!("{name}: re-derived identity must admit, got {other:?}"),
        }
    }

    // The nominal declaration's own key cannot be re-derived as a grouped
    // structural key while retaining its required nominal preimage. A group
    // mutation therefore reaches key authentication before package identity.
    let mut grouped_nominal = base.clone();
    grouped_nominal["semantic_graph"]["nodes"][1]["recursion_group"] = json!("solo");
    refresh_identity(&mut grouped_nominal);
    let mut expected = refusal_cause(
        CheckedPackageRefusalCode::InvalidPackage,
        "/semantic_graph/nodes/1/node_id",
        CheckedPackageRefusalCause::StaleNodeKey,
    );
    expected.locus = Some(checked_package::typed_node_id(&digest_of(
        &grouped_nominal["semantic_graph"]["nodes"][1],
    )));
    assert_eq!(
        refused(&grouped_nominal, &evidence_for(&grouped_nominal)),
        expected,
    );
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
            typed.digest(1 << 20).as_deref().ok(),
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

/// A nominal preimage the closed preimage schema refuses for a missing
/// member or a value of the wrong kind is an invalid graph node, refused as
/// `invalid_semantic_graph` at the member at fault rather than as
/// `malformed_wire`; an unknown member keeps its own code.
#[trace("TC-048", "FR-038-AC-5")]
#[test]
fn tc_048_nominal_preimage_schema_defects_refuse_as_invalid_semantic_graph() {
    let edited = |edit: &dyn Fn(&mut serde_json::Map<String, Value>)| {
        let mut package = v2_nominal();
        edit(
            package["semantic_graph"]["nodes"][1]["nominal_identity_preimage"]
                .as_object_mut()
                .expect("preimage"),
        );
        refresh_identity(&mut package);
        refused(&package, &evidence_for(&package))
    };
    // The nominal preimage occurs only on its graph node.
    let preimage = "/semantic_graph/nodes/1/nominal_identity_preimage";
    assert_eq!(
        edited(&|preimage| {
            preimage.remove("members");
        }),
        nominal(preimage),
        "a missing member"
    );
    assert_eq!(
        edited(&|preimage| {
            preimage.insert("members".into(), json!("RED"));
        }),
        nominal(&format!("{preimage}/members")),
        "a member of the wrong kind"
    );
    assert_eq!(
        edited(&|preimage| {
            preimage.insert("extra".into(), json!(1));
        }),
        refusal(
            CheckedPackageRefusalCode::UnknownMember,
            &format!("{preimage}/extra")
        ),
        "an unknown member"
    );
}

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
                let source = v["lock"]["sources"][0].clone();
                v["semantic_graph"]["nodes"][declaration]["owner"] = json!({
                    "kind": "source", "authority": source["authority"],
                    "identity": source["identity"],
                });
                checked_package::mint_ungrouped_structural_keys(v);
                let member = &v["semantic_graph"]["nodes"][member];
                let stale = digest_of(member);
                let fresh = sha256_hex(&canonical(&member["nominal_identity_preimage"]));
                checked_package::rename_node(v, &stale, &fresh);
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

/// The recorded nominal package with its enum declaration owned by `owner`,
/// re-keyed through every dependant, and `models` as the selected domain
/// package identities (one `model_root` each).
fn model_owned_package(owner: Value, models: &[&str]) -> Value {
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
    for identity in models {
        select_model(&mut package, identity);
    }
    refresh_identity(&mut package);
    package
}

fn model_owner(identity: &str, node: &str) -> Value {
    json!({"kind": "model", "identity": identity, "node": node})
}

#[trace("TC-048", "FR-038-AC-45", "FR-063-AC-5")]
#[test]
fn tc_048_a_content_only_model_owner_admits_and_its_keys_ignore_the_selected_document() {
    // A content-only package (owner `{kind, identity, node}`, no `version`)
    // admits against the selected domain package.
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    let at_one = model_owned_package(owner.clone(), &["test/orders"]);
    admitted(&at_one);

    // The model declaration owner is the whole preimage input: a different
    // document supplied under the identity leaves the keyed declaration
    // nodes and the selected root unchanged.
    let other_document = domain_package_document("test/orders", "2", Vec::new());
    let other_evidence = evidence_with_document(&at_one, "test/orders", &other_document);
    assert!(matches!(
        read(&at_one, &other_evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    // A `version` member on a model owner violates its closed wire shape.
    let mut versioned = owner.clone();
    versioned["version"] = json!("1.0.0");
    let versioned = model_owned_package(versioned, &["test/orders"]);
    assert_eq!(
        refused(&versioned, &evidence_for(&versioned)),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            "/semantic_graph/nodes/0"
        )
    );

    // An empty identity or node is not a valid owner.
    for (name, empty) in [
        ("identity", model_owner("", "ix://test/orders/Status")),
        ("node", model_owner("test/orders", "")),
    ] {
        let package = model_owned_package(empty, &["test/orders"]);
        assert_eq!(
            refused(&package, &evidence_for(&package)),
            nominal("/semantic_graph/nodes/0/nominal_identity_preimage/owner"),
            "empty {name}"
        );
    }
}

#[trace("TC-048", "FR-038-AC-2", "FR-038-AC-5", "FR-038-AC-64")]
#[test]
fn tc_048_model_owners_join_model_roots_by_identity() {
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    let base = model_owned_package(owner.clone(), &["test/orders"]);
    admitted(&base);

    // Owner-to-root join: by domain package identity, with a present node.
    let joins = [
        (
            "owner identity names no root",
            model_owned_package(
                model_owner("test/other", "ix://test/orders/Status"),
                &["test/orders"],
            ),
        ),
        (
            "no domain package selected",
            model_owned_package(owner.clone(), &[]),
        ),
        (
            "empty owner node",
            model_owned_package(model_owner("test/orders", ""), &["test/orders"]),
        ),
    ];
    for (name, mutated) in joins {
        assert_eq!(
            refused(&mutated, &evidence_for(&mutated)),
            nominal("/semantic_graph/nodes/0/nominal_identity_preimage/owner"),
            "{name}"
        );
    }

    // An owner carrying `authority`, `revision` or `export` is an unknown member.
    let retired = [
        (
            "compiled-model owner with export",
            model_owned_package(
                json!({"kind": "model", "authority": "agent-ix",
                       "identity": "test/orders", "export": "Status"}),
                &["test/orders"],
            ),
            // The decoder names the first unknown owner member on the graph node.
            "/semantic_graph/nodes/0/nominal_identity_preimage/owner/authority",
        ),
        (
            "domain package owner with export",
            model_owned_package(
                json!({"kind": "model", "identity": "test/orders",
                       "node": "ix://test/orders/Status", "export": "Status"}),
                &["test/orders"],
            ),
            "/semantic_graph/nodes/0/nominal_identity_preimage/owner/export",
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

    // The selected document must name the root's identity.
    let wrong_document = domain_package_document("test/other", "1", Vec::new());
    let wrong = evidence_with_document(&base, "test/orders", &wrong_document);
    let root = base["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["semantic_form"] == "model_root")
        .expect("root");
    assert_eq!(
        refused(&base, &wrong),
        refusal_cause(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/owner/identity"),
            CheckedPackageRefusalCause::WrongModelSelection
        ),
        "the supplied document names test/other, not the selected test/orders"
    );
}

#[trace("TC-048", "FR-038-AC-2", "FR-038-AC-62")]
#[test]
fn tc_048_a_model_root_owner_is_exactly_kind_and_identity() {
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    let base = model_owned_package(owner, &["test/orders"]);
    let root = base["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["semantic_form"] == "model_root")
        .expect("root");
    let at = format!("/semantic_graph/nodes/{root}/owner");
    let evidence = evidence_for(&base);
    for member in ["digest", "version"] {
        let mut extra = base.clone();
        extra["semantic_graph"]["nodes"][root]["owner"][member] = json!("1");
        refresh_identity(&mut extra);
        assert_eq!(
            refused(&extra, &evidence),
            refusal(CheckedPackageRefusalCode::MalformedWire, &at),
            "{member}"
        );
    }
    let mut lock_member = base.clone();
    lock_member["lock"]["model_selections"] = json!([]);
    assert_eq!(
        refused(&lock_member, &evidence),
        refusal(
            CheckedPackageRefusalCode::UnknownMember,
            "/lock/model_selections"
        )
    );
    let mut preimage_member = base;
    preimage_member["identity_preimage"]["model_selections"] = json!([]);
    assert_eq!(
        refused(&preimage_member, &evidence),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            "/identity_preimage/model_selections"
        )
    );
}

#[trace("TC-048", "FR-038-AC-2")]
#[test]
fn tc_048_domain_package_selection_with_no_supplied_document_refuses_as_missing_import() {
    let owner = model_owner("test/orders", "ix://test/orders/Status");
    let base = model_owned_package(owner, &["test/orders"]);
    let root = base["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["semantic_form"] == "model_root")
        .expect("root");
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature(COMPLETE_VALUE_FEATURE);
    assert_eq!(
        refused(&base, &evidence),
        refusal_cause(
            CheckedPackageRefusalCode::MissingImport,
            &format!("/semantic_graph/nodes/{root}/owner/identity"),
            CheckedPackageRefusalCause::MissingSelection
        )
    );
    admitted(&base);
}

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
        "process",
        "namespace",
        "systems_interface",
        "systems_part",
        "systems_port",
        "systems_connection",
        "systems_allocation",
    ] {
        // Both forms already have valid witnesses in the shared fixture;
        // changing the namespace into `process` would duplicate its key.
        if matches!(form, "object_type" | "process") {
            assert!(base["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .any(|node| node["node_tag"] == "model" && node["semantic_form"] == form));
            admitted(&base);
            continue;
        }
        let mut value = base.clone();
        if form == "systems_interface" {
            // FR-094 model-owned interfaces are self typed and named by the
            // selected model document, so add one without disturbing the
            // object type the all-family frame already references.
            let mut interface = value["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .find(|node| node["node_tag"] == "model" && node["semantic_form"] == "object_type")
                .expect("model-owned object type")
                .clone();
            let placeholder = "ab".repeat(32);
            interface["node_id"] = checked_package::node_id(&placeholder);
            interface["semantic_type"] = checked_package::node_id(&placeholder);
            interface["semantic_form"] = json!(form);
            interface["owner"] =
                checked_package::model_owner("test/families", "ix://test/families/Interface");
            value["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .push(interface);
            checked_package::rebuild_source_map(&mut value);
        } else {
            value["semantic_graph"]["nodes"][model]["semantic_form"] = json!(form);
        }
        rekey_declared_all_families(&mut value);
        admitted(&value);
    }
    // This form needs a selected model declaration and owner, unlike the
    // anonymous node used for the other vocabulary controls.
    let (state_machine, evidence) = crate::checked_package_v2_refinement::machine_package();
    assert!(matches!(
        read(&state_machine, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let (persistence, evidence) = crate::checked_package_v2_refinement::selected_forms_package();
    let result = read(&persistence, &evidence);
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("selected persistence interface must admit: {result:?}");
    };
    assert!(admitted.graph().nodes.iter().any(|node| {
        node.node_tag.as_ref() == "model" && node.semantic_form.as_ref() == "persistence_interface"
    }));
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

/// A source-declared interface carries its external type and SourceOwner in
/// the structural preimage. The fixed self-typed shape still applies to an
/// undeclared, model-owned object type.
#[trace("TC-226", "FR-038-AC-150")]
#[test]
fn tc_226_a_source_declared_interface_authenticates_but_a_model_owned_one_keeps_its_shape() {
    let mut value = v2_all_families();
    let nodes = value["semantic_graph"]["nodes"].as_array().expect("nodes");
    let object = nodes
        .iter()
        .find(|node| node["node_tag"] == "model" && node["semantic_form"] == "object_type")
        .expect("model-owned declaration")
        .clone();
    let model_owner = object["owner"].clone();
    let external_type = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("external Boolean type")["node_id"]
        .clone();
    let source = &value["lock"]["sources"][0];
    let source_owner = json!({
        "kind":"source", "authority":source["authority"], "identity":source["identity"]
    });
    let position = nodes.len();
    let mut interface = object;
    interface["node_id"] = checked_package::node_id(&"ab".repeat(32));
    interface["semantic_type"] = external_type.clone();
    interface["semantic_form"] = json!("systems_interface");
    interface["owner"] = source_owner.clone();
    interface["declaration"] = json!({"qualified_name":["Example","Flowable"]});
    interface["occurrences"] = json!([{"role":"declaration","ordinal":0}]);
    value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(interface);
    checked_package::rebuild_source_map(&mut value);
    rekey_declared_all_families(&mut value);

    let declared_key = sha256_hex(&canonical(&json!({
        "version":"quire.structural-node/v1", "node_tag":"model",
        "semantic_form":"systems_interface", "semantic_type":external_type,
        "declaration":{"qualified_name":["Example","Flowable"]},
        "recursion":null, "owner":source_owner,
        "body":{"term":"aggregate","members":[]}
    })));
    assert_eq!(
        digest_of(&value["semantic_graph"]["nodes"][position]),
        declared_key
    );
    admitted(&value);

    let mut model_owned = v2_all_families();
    let model_position = model_owned["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "model" && node["semantic_form"] == "object_type")
        .expect("model-owned object type");
    model_owned["semantic_graph"]["nodes"][model_position]["semantic_type"] = external_type.clone();
    rekey_declared_all_families(&mut model_owned);
    let model_key = sha256_hex(&canonical(&json!({
        "version":"quire.structural-node/v1", "node_tag":"model",
        "semantic_form":"object_type", "semantic_type":external_type,
        "declaration":null, "recursion":null, "owner":model_owner,
        "body":{"term":"aggregate","members":[]}
    })));
    assert_eq!(
        digest_of(&model_owned["semantic_graph"]["nodes"][model_position]),
        model_key
    );
    let refusal = refused(&model_owned, &evidence_for(&model_owned));
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidPackage,
        "{refusal:?}"
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );
    assert_eq!(
        refusal.path,
        Some(support_pointer(&format!(
            "/semantic_graph/nodes/{model_position}/node_id"
        )))
    );
    assert_eq!(
        refusal.locus.as_ref().map(|id| id.digest.as_ref()),
        Some(model_key.as_str())
    );
    assert!(refusal.expected_node_id().is_none());
}

#[trace("TC-048", "FR-038-AC-9", "FR-038-AC-26")]
#[test]
fn tc_048_shipped_default_read_limits_are_exact_and_finite() {
    // FR-038-AC-9: the shipped default policy is exactly these six values.
    // There is no depth member (FR-038-AC-117).
    let bounded = CheckedPackageReadLimits::bounded();
    assert_eq!(bounded.bytes, 1_048_576);
    assert_eq!(bounded.nodes, 10_000);
    assert_eq!(bounded.edges, 100_000);
    assert_eq!(bounded.occurrences, 100_000);
    assert_eq!(bounded.diagnostics, 10_000);
    assert_eq!(bounded.work, 1_000_000);

    // FR-038-AC-9: every member is strictly positive and finite, so the
    // default admits no unbounded read on any axis.
    for (name, value) in [
        ("bytes", bounded.bytes),
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
    // Give the graph a real dependency edge so every one of the six meters
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
    let narrowed: [(CheckedPackageLimit, NarrowLimit, u64); 6] = [
        (
            CheckedPackageLimit::Bytes,
            |limits, value| limits.bytes = value,
            bounded.bytes,
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

/// Public self-reference mutations are refused by authenticated node keys
/// before the conditional graph-stage body-root carve-out is reached. The
/// graph-stage distinction is exercised at the private validation seam.
#[trace("TC-048", "FR-038-AC-18")]
#[test]
fn tc_048_self_typed_body_references_refuse_at_authenticated_keys() {
    let base = v2_all_families();
    // Public reads authenticate these structural and application keys before
    // the conditional graph-stage carve-out can be evaluated.
    let node_at = |key: &str| {
        base["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"]["digest"] == checked_package::family_key(key))
            .expect("fixture node")
    };
    let own = node_at("8080");
    let own_id = base["semantic_graph"]["nodes"][own]["node_id"].clone();
    let other_id = base["semantic_graph"]["nodes"][own]["semantic_type"].clone();
    let application_position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "expression" && node["semantic_form"] == "reference")
        .expect("reference expression");
    let application_id = base["semantic_graph"]["nodes"][application_position]["node_id"].clone();

    let stale_structural_key_refusal = refusal_at(
        CheckedPackageRefusalCode::InvalidPackage,
        &format!("/semantic_graph/nodes/{own}/node_id"),
        Some(CheckedPackageRefusalCause::StaleNodeKey),
        own_id["digest"].as_str().expect("structural digest"),
    );
    // Application preimages include their bodies as well, so the same
    // first-refusal rule applies to a self-reference inside an application.
    let stale_application_key_refusal = refusal_at(
        CheckedPackageRefusalCode::InvalidPackage,
        &format!("/semantic_graph/nodes/{application_position}/node_id"),
        Some(CheckedPackageRefusalCause::StaleNodeKey),
        application_id["digest"]
            .as_str()
            .expect("application digest"),
    );

    let assert_self_cycle_refused = |position: usize, body: Value, expected: ExpectedRefusal| {
        let mut mutated = base.clone();
        mutated["semantic_graph"]["nodes"][position]["semantic_type"] =
            mutated["semantic_graph"]["nodes"][position]["node_id"].clone();
        mutated["semantic_graph"]["nodes"][position]["body"] = body;
        if position == application_position {
            mutated["semantic_graph"]["nodes"][position]["dependencies"] = json!([]);
        }
        refresh_identity(&mut mutated);
        assert_eq!(refused(&mutated, &evidence_for(&mutated)), expected);
    };

    // A body-root reference to the node's own key.
    assert_self_cycle_refused(
        own,
        json!({"term": "reference", "target": own_id}),
        stale_structural_key_refusal.clone(),
    );

    // A literal self-reference nested inside an aggregate.
    assert_self_cycle_refused(
        own,
        json!({
            "term": "aggregate",
            "members": [
                {"term": "literal", "type": own_id, "value_kind": "integer", "value": 1}
            ]
        }),
        stale_structural_key_refusal.clone(),
    );

    // The same literal self-reference inside a binding value.
    assert_self_cycle_refused(
        own,
        json!({
            "term": "aggregate",
            "members": [{
                "term": "binding",
                "name": "x",
                "value": {"term": "literal", "type": own_id, "value_kind": "integer", "value": 1}
            }]
        }),
        stale_structural_key_refusal.clone(),
    );

    // A literal self-reference nested inside an application argument.
    assert_self_cycle_refused(
        application_position,
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
                {"term": "literal", "type": application_id, "value_kind": "integer", "value": 1}
            ]
        }),
        stale_application_key_refusal.clone(),
    );

    // A self-reference through application.result_type.
    assert_self_cycle_refused(
        application_position,
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
            "result_type": application_id,
            "arguments": [
                {"term": "literal", "type": other_id, "value_kind": "integer", "value": 1}
            ]
        }),
        stale_application_key_refusal,
    );

    // A self-typed body-root literal would also fail key authentication on
    // this public wire: its structural preimage embeds its own final digest.
    let mut literal_type = base.clone();
    literal_type["semantic_graph"]["nodes"][own]["semantic_type"] = own_id.clone();
    literal_type["semantic_graph"]["nodes"][own]["body"] = json!({
        "term": "literal", "type": own_id, "value_kind": "integer", "value": 1
    });
    refresh_identity(&mut literal_type);
    assert_eq!(
        refused(&literal_type, &evidence_for(&literal_type)),
        stale_structural_key_refusal,
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
#[trace("TC-048", "FR-038-AC-17")]
#[test]
fn deleting_a_declared_wire_member_refuses_after_graph_identity_is_refreshed() {
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
    let cases: [(&str, usize, Delete, ExpectedRefusal); 4] = [
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
                CheckedPackageRefusalCode::MalformedWire,
                &format!("/semantic_graph/nodes/{declaring_node}/owner"),
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
        // Recompute the graph-derived package identity so this control reaches
        // the required-member check rather than failing earlier authentication.
        let mut changed = base.clone();
        delete(&mut changed, index);
        refresh_identity(&mut changed);
        assert_eq!(
            refused(&changed, &evidence_for(&changed)),
            expected,
            "{name}"
        );
    }
}

/// FR-038-AC-17's closed `expression` form list: every
/// `ExpressionForm::ALL` member admits as a node form, and an
/// undeclared form refuses as `invalid_semantic_graph`.
/// `tc_048_model_export_is_not_a_v2_model_form` already covers the same
/// pattern for the eighteen `model` forms — this test is the `expression`
/// counterpart, not a repeat of it. (The schema-vs-`CheckedNodeTag::forms`
/// cross-check this test used to also run was dropped along with the
/// vendored schema it needed; see AGE-1961.)
#[trace("TC-048", "FR-038-AC-17")]
#[test]
fn tc_048_expression_forms_bound_admission() {
    let base = v2_all_families();
    let expression = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == json!("expression"))
        .expect("all-families fixture carries an expression node");
    for form in ExpressionForm::ALL {
        let mut value = base.clone();
        value["semantic_graph"]["nodes"][expression]["semantic_form"] = json!(form.as_wire());
        checked_package::mint_ungrouped_structural_keys(&mut value);
        refresh_identity(&mut value);
        if *form == ExpressionForm::Case {
            // An `expression`/`case` node stands only with a `case`
            // application as its body (FR-038-AC-100), which this node's
            // `reference` body is not: its form contradicts its root, so it is
            // `invalid_semantic_graph` at the body. The form admits with its
            // own body in `checked_package_v2_union.rs`.
            assert_eq!(
                refused(&value, &evidence_for(&value)),
                refusal_at(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    &format!("/semantic_graph/nodes/{expression}/body"),
                    None,
                    &digest_of(&value["semantic_graph"]["nodes"][expression]),
                )
            );
            continue;
        }
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

#[trace("TC-230", "FR-038-AC-301")]
#[test]
fn tc_230_unit_and_dimension_can_share_one_name() {
    let (value, _) = with_shared_name();
    assert!(matches!(
        CheckedPackageV2::read(
            &canonical(&value),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&value),
        ),
        CheckedPackageV2ReadResult::Admitted(_),
    ));
}

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
    let source = package["lock"]["sources"][0].clone();
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    node["declaration"] = json!({"qualified_name": name});
    node["owner"] = json!({
        "kind": "source", "authority": source["authority"], "identity": source["identity"],
    });
}

/// Rekey the all-family fixture after a declaration changes a structural
/// preimage. Every rename reaches downstream type, body, frame and package
/// references; `settle` then rekeys application nodes over those references.
fn rekey_declared_all_families(package: &mut Value) {
    let count = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    for _ in 0..=count {
        let previous: Vec<Value> = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .map(|node| node["node_id"].clone())
            .collect();
        checked_package::mint_ungrouped_structural_keys(package);
        checked_package::settle(package);
        let current: Vec<Value> = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .map(|node| node["node_id"].clone())
            .collect();
        if current == previous {
            return;
        }
    }
    panic!("declared all-family fixture did not reach stable keys");
}

#[trace("TC-048", "TC-230", "FR-038-AC-21", "FR-038-AC-301")]
#[test]
fn tc_048_a_declaration_refusal_precedes_an_operation_refusal() {
    // Nodes: 0 root, 1 declaring namespace ("Example::Widget"), 2 literal,
    // 3 declaring function, 4 application call.
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
    let stale = digest_of(&both["semantic_graph"]["nodes"][3]);
    declare(&mut both, 3, &["Example", "Widget"]);
    let function = &both["semantic_graph"]["nodes"][3];
    let fresh = sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1",
        "node_tag": function["node_tag"],
        "semantic_form": function["semantic_form"],
        "semantic_type": function["semantic_type"],
        "owner": function["owner"],
        "declaration": function["declaration"],
        "recursion": null,
        "body": function["body"],
    })));
    checked_package::rename_node(&mut both, &stale, &fresh);
    checked_package::settle(&mut both);
    let declaring = digest_of(&both["semantic_graph"]["nodes"][1]);
    let function = digest_of(&both["semantic_graph"]["nodes"][3]);
    assert_eq!(
        refused(&both, &evidence_for(&both)),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &declared_name_at(&both, declaring.clone().max(function.clone()).as_str()),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            declaring.max(function).as_str(),
        )
    );
}

#[trace("TC-048", "TC-230", "FR-038-AC-21", "FR-038-AC-301")]
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
        value["semantic_graph"]["nodes"][frame]["body"]["modifies"] = json!([{
            "kind": "relationship",
            "declaration": checked_package::node_id(&"0123456789abcdef".repeat(4)),
        }]);
        checked_package::mint_ungrouped_structural_keys(&mut value);
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
    rekey_declared_all_families(&mut both);
    let first = digest_of(&both["semantic_graph"]["nodes"][0]);
    let second = digest_of(&both["semantic_graph"]["nodes"][1]);
    assert_eq!(
        refused(&both, &evidence_for(&both)),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &declared_name_at(&both, first.clone().max(second.clone()).as_str()),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            first.max(second).as_str(),
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
    let function = position_of(&base, &|node| {
        node["node_id"]["digest"] == checked_package::family_key("ffff")
    });
    // The call and its target are both independently keyed by their closed
    // preimages. Their graph placement determines the group ordinals.
    let original_target = first_reference(&base["semantic_graph"]["nodes"][function]["body"])
        .expect("the call references a node");
    let target = original_target;
    let function_key = base["semantic_graph"]["nodes"][function]["node_id"]["digest"].clone();
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
        checked_package::mint_grouped_structural_keys(&mut value);
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
                position_of(&bare, &|node| node["node_id"]["digest"] == function_key)
            ),
            Some(CheckedPackageRefusalCause::StaleNodeKey),
            function_key.as_str().expect("digest"),
        )
    );
}
