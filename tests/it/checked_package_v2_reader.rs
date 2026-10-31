// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Strict CheckedPackage V2 reader: locally authored adverse mutations, injected
//! refusals, resource limits, structural package values, and nominal bodies.

use crate::support::checked_package::{
    self, all_families_read_work, canonical, domain_package_document, evidence_for,
    evidence_with_document, family_key, incomplete, node_id, pointer as support_pointer,
    positive_operation_identities, refresh_identity, refusal, refusal_bytes, refusal_cause,
    select_model, unknown_version, v2_all_families, v2_nominal, ExpectedRefusal,
    COMPLETE_VALUE_FEATURE,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    read_checked_package, CheckedOccurrenceRole, CheckedPackageDispatchResult,
    CheckedPackageEvidence, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, ExactTypeProjectionErrorCode, ExpressionForm,
    SelectedExactIdentities,
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

#[test]
fn exact_scalar_projection_uses_publicly_admitted_package() {
    let mut document = v2_all_families();
    let range = node_id(&family_key("cccc"));
    let parameter = node_id("projection/range-parameter");
    let mut range_parameter = document["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .expect("admitted parameter shape")
        .clone();
    range_parameter["node_id"] = parameter.clone();
    range_parameter["semantic_type"] = range;
    range_parameter["body"]["members"][0]["value"]["value"] = json!("range_parameter");
    document["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(range_parameter);
    checked_package::rebuild_source_map(&mut document);
    refresh_identity(&mut document);

    let package = admitted(&document);
    let identities = SelectedExactIdentities::from_stage_rows(&package, [], [])
        .expect("empty selected rows for scalar reads");
    for (id, expected) in [
        (node_id(&family_key("aaaa")), "Boolean"),
        (node_id(&family_key("a3a3")), "Integer"),
        (
            parameter,
            "Int(IntegerInterval { lower: Integer(0), upper: Integer(1000) })",
        ),
    ] {
        let id = serde_json::from_value(id).expect("structural ID");
        let projected = package
            .project_exact_type(&id, &identities, 12)
            .expect("Exact type");
        assert_eq!(projected.node(), &id);
        assert_eq!(format!("{:?}", projected.value_type()), expected);
        assert_eq!(
            package.clone().project_exact_type(&id, &identities, 12),
            Ok(projected),
        );
    }

    let id = serde_json::from_value(node_id(&family_key("aaaa"))).expect("Boolean ID");
    let exhausted = package
        .project_exact_type(&id, &identities, 1)
        .expect_err("finite limit");
    assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);

    let mut invalid = document;
    invalid["semantic_graph"]["nodes"][0]["semantic_type"] = node_id("missing-type");
    assert!(matches!(
        read(&invalid, &evidence_for(&invalid)),
        CheckedPackageV2ReadResult::Refused(_)
    ));
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
    assert_eq!(reordered.name(), first.name());
    assert_eq!(reordered.graph(), first.graph());
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
        refusal.loci[0].path.clone(),
        Some(support_pointer(&format!("/source_map/{map_position}")))
    );
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

    // Unknown members in both closed wire objects refuse at their own key.
    for place in [
        "/capability_report/0/unrecognized_claim",
        "/lock/unrecognized_claim",
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
        "malformed_wire",
        CheckedPackageRefusalCode::MalformedWire,
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
                assert_eq!(
                    actual.loci[0].path.clone(),
                    Some(support_pointer(pointer)),
                    "{id}"
                );
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
                v["semantic_graph"]["nodes"][4]["body"]["target"] = node_id("missing/body-target");
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
                v["semantic_graph"]["nodes"][1]["dependencies"] =
                    json!([node_id("missing/dependency")]);
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
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/6/recursion_group",
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
                let path = actual.loci[0]
                    .path
                    .clone()
                    .expect("a refusal about a value");
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
    // Find this fixture's exact work boundary through the public reader. A
    // changed semantic body or edge must move both the admitted and one-over
    // checks together, while still exercising the actual meter.
    let (mut below, mut minimum_work) = (0_u64, CheckedPackageReadLimits::bounded().work);
    while below + 1 < minimum_work {
        let middle = below + (minimum_work - below) / 2;
        let mut probe = CheckedPackageReadLimits::bounded();
        probe.work = middle;
        match CheckedPackageV2::read(&bytes, probe, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => minimum_work = middle,
            CheckedPackageV2ReadResult::Incomplete(limit) => {
                assert_eq!(limit.limit_kind, CheckedPackageLimit::Work);
                below = middle;
            }
            other => panic!("work boundary probe failed for another reason: {other:?}"),
        }
    }
    let exact = CheckedPackageReadLimits {
        bytes: u64::try_from(bytes.len()).expect("length"),
        nodes: 4,
        edges: 2,
        occurrences: 8,
        diagnostics: 1,
        work: minimum_work,
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

/// A malformed nominal body refuses at the wire stage before a dangling
/// graph dependency can be followed. The two faults are independent.
#[trace("TC-048", "FR-038-AC-5", "FR-038-AC-375")]
#[test]
fn tc_048_malformed_nominal_body_precedes_dangling_dependency() {
    let mut package = v2_nominal();
    package["semantic_graph"]["nodes"][1]["body"]["members"] = json!([]);
    package["semantic_graph"]["nodes"][1]["dependencies"] = json!([{
        "package": "test/families", "path": ["absent"], "ordinal": 0,
    }]);
    assert_eq!(
        refused(&package, &evidence_for(&package)),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            "/semantic_graph/nodes/1/body/members",
        ),
    );
}

/// The typed nominal bodies use the closed V2 wire grammar before semantic
/// owner, declaration and dependency checks run.
#[trace("TC-048", "FR-038-AC-5")]
#[test]
fn tc_048_nominal_body_schema_defects_refuse_as_malformed_wire() {
    let base = v2_nominal();
    let mut ordered = base.clone();
    ordered["semantic_graph"]["nodes"][1]["body"]["ordered"] = json!(true);
    let admitted_ordered = admitted(&ordered);
    assert_eq!(
        serde_json::to_value(&admitted_ordered.graph().nodes[1].body).expect("typed enum body")
            ["ordered"],
        true
    );

    let cases: Vec<(&str, Mutation, &str)> = vec![
        (
            "empty enum",
            Box::new(|v| v["semantic_graph"]["nodes"][1]["body"]["members"] = json!([])),
            "/semantic_graph/nodes/1/body/members",
        ),
        (
            "missing ordered",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][1]["body"]
                    .as_object_mut()
                    .expect("enum body")
                    .remove("ordered");
            }),
            "/semantic_graph/nodes/1/body",
        ),
        (
            "missing unit scale",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][2]["body"]
                    .as_object_mut()
                    .expect("unit body")
                    .remove("scale");
            }),
            "/semantic_graph/nodes/2/body",
        ),
        (
            "extra unit member",
            Box::new(|v| v["semantic_graph"]["nodes"][2]["body"]["extra"] = json!(1)),
            "/semantic_graph/nodes/2/body",
        ),
        (
            "noninteger dimension exponent",
            Box::new(|v| {
                let id = v["semantic_graph"]["nodes"][3]["node_id"].clone();
                v["semantic_graph"]["nodes"][3]["body"]["terms"] =
                    json!([{"dimension_node_id": id, "exponent": "oops"}]);
            }),
            "/semantic_graph/nodes/3/body/terms/0/exponent",
        ),
    ];
    for (name, edit, at) in cases {
        let mut changed = base.clone();
        edit(&mut changed);
        assert_eq!(
            refused(&changed, &evidence_for(&changed)),
            refusal(CheckedPackageRefusalCode::MalformedWire, at),
            "{name}"
        );
    }
}

#[trace("TC-048", "FR-038-AC-5")]
#[test]
fn tc_048_nominal_cross_field_contradictions_refuse() {
    let base = v2_nominal();
    let cases: Vec<(&str, Mutation, &str)> = vec![
        (
            "enum member dependency omitted",
            Box::new(|v| v["semantic_graph"]["nodes"][0]["dependencies"] = json!([])),
            "/semantic_graph/nodes/0/dependencies",
        ),
        (
            "unit semantic type is its own id",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][2]["semantic_type"] =
                    v["semantic_graph"]["nodes"][2]["node_id"].clone()
            }),
            "/semantic_graph/nodes/2/semantic_type",
        ),
        (
            "dimension semantic type names enum",
            Box::new(|v| {
                v["semantic_graph"]["nodes"][3]["semantic_type"] =
                    v["semantic_graph"]["nodes"][1]["node_id"].clone()
            }),
            "/semantic_graph/nodes/3/semantic_type",
        ),
    ];
    for (name, edit, at) in cases {
        let mut changed = base.clone();
        edit(&mut changed);
        assert_eq!(
            refused(&changed, &evidence_for(&changed)),
            refusal(CheckedPackageRefusalCode::InvalidSemanticGraph, at),
            "{name}",
        );
    }
}

/// The enum member's case must remain one of the selected declaration's cases.
#[trace("TC-048", "FR-038-AC-5", "FR-038-AC-375")]
#[test]
fn tc_048_enum_member_case_must_match_its_typed_declaration() {
    let mut package = v2_nominal();
    package["semantic_graph"]["nodes"][0]["body"]["value"] = json!("UNKNOWN");
    let refusal = refused(&package, &evidence_for(&package));
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/0/body/value".to_owned()),
    );
}

/// A locally authored selected-model dimension and its model root.
pub(super) fn model_owned_package(owner: Value, models: &[&str]) -> Value {
    const LENGTH: &str = "ix://test/orders/Length";
    let nominal = v2_nominal();
    let mut package = nominal.clone();
    let mut dimension = nominal["semantic_graph"]["nodes"][3].clone();
    let owner_identity = owner["identity"]
        .as_str()
        .filter(|part| !part.is_empty())
        .unwrap_or("test/orders");
    let owner_node = owner["node"]
        .as_str()
        .filter(|part| !part.is_empty())
        .unwrap_or(LENGTH);
    let id = checked_package::structural_id(
        owner_identity,
        vec![json!("scalar_type"), json!("dimension"), json!(owner_node)],
        0,
    );
    dimension["node_id"] = id.clone();
    dimension["semantic_type"] = id.clone();
    dimension["owner"] = owner;
    dimension["body"] = json!({"term":"aggregate","members":[]});
    dimension
        .as_object_mut()
        .expect("node")
        .remove("declaration");
    dimension["occurrences"] = json!([{"role":"generated","ordinal":0}]);
    let mut map = nominal["source_map"][3].clone();
    map["node_id"] = id;
    map["role"] = json!("generated");
    package["semantic_graph"]["nodes"] = json!([dimension]);
    package["source_map"] = json!([map]);
    for identity in models {
        select_model(&mut package, identity);
    }
    package
}

fn model_document(version: &str) -> Value {
    let mut document = domain_package_document("test/orders", version, Vec::new());
    document["dimensions"] = json!([{
        "identity":"ix://test/orders/Length", "name":"Length", "terms":[]
    }]);
    document
}

pub(super) fn model_evidence(package: &Value) -> CheckedPackageEvidence {
    evidence_with_document(package, "test/orders", &model_document("1"))
}

fn admitted_model(package: &Value) -> Box<CheckedPackageV2> {
    match read(package, &model_evidence(package)) {
        CheckedPackageV2ReadResult::Admitted(value) => value,
        other => panic!("locally authored model-owned dimension refused: {other:?}"),
    }
}

fn model_owner(identity: &str, node: &str) -> Value {
    json!({"kind": "model", "identity": identity, "node": node})
}

#[trace("TC-048", "FR-038-AC-45", "FR-063-AC-5", "FR-038-AC-367")]
#[test]
fn tc_048_a_content_only_model_owner_admits_and_its_keys_ignore_the_selected_document() {
    // The closed model owner `{kind, identity, node}` admits against an
    // authored selected Dimension declaration.
    let owner = model_owner("test/orders", "ix://test/orders/Length");
    let at_one = model_owned_package(owner.clone(), &["test/orders"]);
    let first = admitted_model(&at_one);

    // A selected document version change leaves the authored structural
    // declaration node and root identities unchanged.
    let other_document = model_document("2");
    let other_evidence = evidence_with_document(&at_one, "test/orders", &other_document);
    let CheckedPackageV2ReadResult::Admitted(second) = read(&at_one, &other_evidence) else {
        panic!("the same selected Dimension declaration must admit at version 2");
    };
    assert_eq!(
        first.graph().nodes[0].node_id,
        second.graph().nodes[0].node_id
    );
    assert_eq!(
        first.graph().nodes[1].node_id,
        second.graph().nodes[1].node_id
    );

    // A `version` member on a model owner violates its closed wire shape.
    let mut versioned = owner.clone();
    versioned["version"] = json!("1.0.0");
    let versioned = model_owned_package(versioned, &["test/orders"]);
    assert_eq!(
        refused(&versioned, &evidence_for(&versioned)),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            "/semantic_graph/nodes/0/owner"
        )
    );

    // An empty identity or node is not a valid owner.
    for (name, empty) in [
        ("identity", model_owner("", "ix://test/orders/Length")),
        ("node", model_owner("test/orders", "")),
    ] {
        let package = model_owned_package(empty, &["test/orders"]);
        assert_eq!(
            refused(&package, &evidence_for(&package)),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                "/semantic_graph/nodes/0/owner"
            ),
            "empty {name}"
        );
    }
}

/// AC-367 selected-declaration edit slice: a retained model-owned dimension
/// cannot silently bind to a different selected declaration under the same
/// domain-package identity and structural node id.
#[trace("TC-048", "FR-038-AC-367")]
#[test]
fn selected_model_declaration_edit_revalidates_the_retained_owner() {
    let owner = model_owner("test/orders", "ix://test/orders/Length");
    let package = model_owned_package(owner, &["test/orders"]);
    admitted_model(&package);
    let mut changed = model_document("1");
    changed["dimensions"][0]["identity"] = json!("ix://test/orders/Other");
    let evidence = evidence_with_document(&package, "test/orders", &changed);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("an edited selected declaration must not bind the old dimension owner");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
}

/// AC-367 source-meaning edit slice: the same source selection and declared
/// structural id do not exempt an edited source-owned body from wire checks.
#[trace("TC-048", "FR-038-AC-367")]
#[test]
fn edited_source_owned_body_revalidates_without_a_stale_identity_gate() {
    let mut package = v2_nominal();
    assert!(matches!(
        read(&package, &evidence_for(&package)),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nominal graph")
        .iter()
        .position(|node| node["owner"]["kind"] == "source")
        .expect("source-owned declaration");
    let original_id = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    let name = package["name"].clone();
    package["semantic_graph"]["nodes"][position]["body"] = json!({"term":"unknown-source-body"});
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence_for(&package))
    else {
        panic!("edited source body must be validated");
    };
    assert_eq!(package["name"], name);
    assert_eq!(
        package["semantic_graph"]["nodes"][position]["node_id"],
        original_id
    );
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}/body")),
        "{refusal:?}"
    );
}

#[trace("TC-048", "FR-038-AC-2", "FR-038-AC-5", "FR-038-AC-64")]
#[test]
fn tc_048_model_owners_join_model_roots_by_identity() {
    let owner = model_owner("test/orders", "ix://test/orders/Length");
    let base = model_owned_package(owner.clone(), &["test/orders"]);
    admitted_model(&base);

    // Owner-to-root join: by domain package identity, with a present node.
    let joins = [
        (
            "owner identity names no root",
            model_owned_package(
                model_owner("test/other", "ix://test/orders/Length"),
                &["test/orders"],
            ),
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingSelection),
            "/semantic_graph/nodes/0/node_id",
        ),
        (
            "no domain package selected",
            model_owned_package(owner.clone(), &[]),
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingSelection),
            "/semantic_graph/nodes/0/node_id",
        ),
        (
            "empty owner node",
            model_owned_package(model_owner("test/orders", ""), &["test/orders"]),
            CheckedPackageRefusalCode::MalformedWire,
            None,
            "/semantic_graph/nodes/0/owner",
        ),
    ];
    for (name, mutated, code, cause, path) in joins {
        let refusal = refused(&mutated, &model_evidence(&mutated));
        assert_eq!((refusal.code, refusal.cause), (code, cause), "{name}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|part| part.as_str()),
            Some(path),
            "{name}",
        );
        if code == CheckedPackageRefusalCode::MissingDeclaration {
            assert_eq!(
                refusal.locus,
                Some(
                    serde_json::from_value(
                        mutated["semantic_graph"]["nodes"][0]["node_id"].clone()
                    )
                    .expect("authored structural ID")
                ),
                "{name}",
            );
        }
    }

    // An owner carrying `authority`, `revision` or `export` is an unknown member.
    let retired = [
        (
            "compiled-model owner with export",
            model_owned_package(
                json!({"kind": "model", "authority": "agent-ix",
                       "identity": "test/orders", "export": "Length"}),
                &["test/orders"],
            ),
            "/semantic_graph/nodes/0/owner",
        ),
        (
            "domain package owner with export",
            model_owned_package(
                json!({"kind": "model", "identity": "test/orders",
                       "node": "ix://test/orders/Length", "export": "Length"}),
                &["test/orders"],
            ),
            "/semantic_graph/nodes/0/owner",
        ),
    ];
    let evidence = evidence_for(&base);
    for (name, mutated, path) in retired {
        assert_eq!(
            refused(&mutated, &evidence),
            refusal(CheckedPackageRefusalCode::MalformedWire, path),
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
    let owner = model_owner("test/orders", "ix://test/orders/Length");
    let base = model_owned_package(owner, &["test/orders"]);
    let root = base["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["semantic_form"] == "model_root")
        .expect("root");
    let at = format!("/semantic_graph/nodes/{root}/owner");
    let evidence = evidence_for(&base);
    for member in ["authority", "version"] {
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
    let mut retired_identity_member = base;
    retired_identity_member["identity_preimage"] = json!({});
    assert_eq!(
        refused(&retired_identity_member, &evidence),
        refusal(
            CheckedPackageRefusalCode::UnknownMember,
            "/identity_preimage"
        )
    );
}

#[trace("TC-048", "FR-038-AC-2")]
#[test]
fn tc_048_domain_package_selection_with_no_supplied_document_refuses_as_missing_import() {
    let owner = model_owner("test/orders", "ix://test/orders/Length");
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
    admitted_model(&base);
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
            let interface_id = checked_package::structural_id(
                "test/families",
                vec![
                    json!("model"),
                    json!("systems_interface"),
                    json!("ix://test/families/Interface"),
                ],
                0,
            );
            interface["node_id"] = interface_id.clone();
            interface["semantic_type"] = interface_id;
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
        checked_package::settle(&mut value);
        admitted(&value);
    }
    // This form needs a selected model declaration and owner, unlike the
    // anonymous node used for the other vocabulary controls.
    let (mut state_machine, evidence) = crate::checked_package_v2_refinement::machine_package();
    checked_package::index_fixture_node_ids(&mut state_machine);
    assert!(matches!(
        read(&state_machine, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let (mut persistence, evidence) =
        crate::checked_package_v2_refinement::selected_forms_package();
    checked_package::index_fixture_node_ids(&mut persistence);
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

/// A self-typed body-root literal is the graph-cycle carve-out. The same
/// self id nested in an aggregate, binding, or reference is a real cycle.
#[trace("TC-048", "FR-038-AC-18")]
#[test]
fn tc_048_self_typed_literal_root_is_exempt_but_nested_self_references_refuse() {
    let mut package = v2_all_families();
    let scalar = package["semantic_graph"]["nodes"][0].clone();
    let scalar_id = scalar["node_id"].clone();
    let scalar_map = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|row| row["node_id"] == scalar_id)
        .expect("scalar source")
        .clone();
    package["semantic_graph"]["nodes"] = json!([scalar.clone()]);
    package["source_map"] = json!([scalar_map.clone()]);
    admitted(&package);

    let mut value = scalar;
    value["node_id"] = json!({"package":"test/families","path":["self_value"],"ordinal":0});
    let self_id = value["node_id"].clone();
    value["node_tag"] = json!("value");
    value["semantic_form"] = json!("literal");
    value["semantic_type"] = self_id.clone();
    value["body"] = json!({"term":"literal","type":self_id,"value_kind":"boolean","value":true});
    let mut value_map = scalar_map;
    value_map["node_id"] = value["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(value);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(value_map);
    admitted(&package);

    let self_id = package["semantic_graph"]["nodes"][1]["node_id"].clone();
    let literal = json!({"term":"literal","type":self_id,"value_kind":"boolean","value":true});
    for (name, body) in [
        (
            "aggregate",
            json!({"term":"aggregate","members":[literal.clone()]}),
        ),
        (
            "binding",
            json!({"term":"aggregate","members":[{"term":"binding","name":"x","value":literal}]}),
        ),
        ("reference", json!({"term":"reference","target":self_id})),
    ] {
        let mut changed = package.clone();
        changed["semantic_graph"]["nodes"][1]["body"] = body;
        assert_eq!(
            refused(&changed, &evidence_for(&changed)),
            refusal(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/1"
            ),
            "{name}",
        );
    }
}

/// FR-038-AC-17: deleting `declaration`, `literal.type`,
/// `application.operation` or `application.result_type` from an otherwise
/// admitted structural graph refuses at the missing member's graph node.
#[trace("TC-048", "FR-038-AC-17")]
#[test]
fn deleting_a_declared_wire_member_refuses_at_its_graph_member() {
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
        checked_package::settle(&mut value);
        if *form == ExpressionForm::Case {
            // A case expression requires a case application body. The
            // fixture still carries a reference body, so the reader refuses
            // at that body with the expression's authored structural ID.
            let refusal = refused(&value, &evidence_for(&value));
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::InvalidSemanticGraph
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().map(ToString::to_string),
                Some(format!("/semantic_graph/nodes/{expression}/body")),
            );
            assert_eq!(
                refusal.locus,
                Some(
                    serde_json::from_value(
                        value["semantic_graph"]["nodes"][expression]["node_id"].clone(),
                    )
                    .expect("authored structural id")
                ),
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

#[trace("TC-048", "FR-038-AC-21")]
#[test]
fn tc_048_declared_name_must_equal_its_structural_path() {
    let mut value = v2_nominal();
    let declaration = 1;
    let retained_id = value["semantic_graph"]["nodes"][declaration]["node_id"].clone();
    value["semantic_graph"]["nodes"][declaration]["declaration"]["qualified_name"] =
        json!(["Example", "Renamed"]);
    let refusal = refused(&value, &evidence_for(&value));
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationNominalMismatch),
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{declaration}/node_id/path")),
    );
    assert_eq!(
        refusal.locus,
        Some(serde_json::from_value(retained_id).expect("authored structural id")),
    );
}

/// A source-declared dimension and unit sharing one qualified name. Their
/// producer-assigned ordinals keep the structural node identities distinct.
fn with_shared_name() -> (Value, [Value; 2]) {
    let mut value = v2_nominal();
    let unit = 2;
    let dimension = 3;
    let old_dimension = value["semantic_graph"]["nodes"][dimension]["node_id"].clone();
    let unit_id = value["semantic_graph"]["nodes"][unit]["node_id"].clone();
    let mut dimension_id = unit_id.clone();
    dimension_id["ordinal"] = json!(1);
    checked_package::reassign_node_id(&mut value, &old_dimension, &dimension_id);
    value["semantic_graph"]["nodes"][dimension]["declaration"]["qualified_name"] =
        value["semantic_graph"]["nodes"][unit]["declaration"]["qualified_name"].clone();
    checked_package::settle(&mut value);
    (value, [dimension_id, unit_id])
}

#[trace("TC-230", "FR-038-AC-301")]
#[test]
fn tc_230_unit_and_dimension_can_share_one_name() {
    let (value, _) = with_shared_name();
    let result = read(&value, &evidence_for(&value));
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
}

#[trace("TC-048", "FR-038-AC-21")]
#[test]
fn tc_048_a_nominal_name_mismatch_survives_a_shared_unit_dimension_name() {
    // The shared unit and dimension name is valid in separate namespaces.
    // A mismatched enum declaration still refuses at its own structural path.
    let (mut value, _) = with_shared_name();
    let declaration = 1;
    let retained_id = value["semantic_graph"]["nodes"][declaration]["node_id"].clone();
    value["semantic_graph"]["nodes"][declaration]["declaration"]["qualified_name"] =
        json!(["Example", "Renamed"]);
    let refusal = refused(&value, &evidence_for(&value));
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationNominalMismatch),
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{declaration}/node_id/path")),
    );
    assert_eq!(
        refusal.locus,
        Some(serde_json::from_value(retained_id).expect("authored structural id")),
    );
}

/// Gives one node a source declaration and its authored structural path.
fn declare(package: &mut Value, position: usize, name: &[&str], ordinal: u64) {
    let source = package["lock"]["sources"][0].clone();
    let old = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    let fresh = checked_package::structural_id(
        package["name"].as_str().expect("package name"),
        name.iter().map(|part| json!(part)).collect(),
        ordinal,
    );
    checked_package::reassign_node_id(package, &old, &fresh);
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    node["declaration"] = json!({"qualified_name": name});
    node["owner"] = json!({
        "kind": "source", "authority": source["authority"], "identity": source["identity"],
    });
    checked_package::settle(package);
}

#[trace("TC-048", "TC-230", "FR-038-AC-21", "FR-038-AC-301")]
#[test]
fn tc_048_a_declaration_refusal_precedes_an_operation_refusal() {
    // Control: the unknown operation alone reaches the operation stage.
    let mut operation_only = positive_operation_identities();
    operation_only["semantic_graph"]["nodes"][4]["body"]["operation"]["identity"] =
        json!("quire.op.unknown");
    checked_package::settle(&mut operation_only);
    let control = refused(&operation_only, &evidence_for(&operation_only));
    assert_eq!(
        control.cause,
        Some(CheckedPackageRefusalCause::UnknownOperation)
    );

    // The function gets the same name as the namespace declaration. Its
    // distinct ordinal makes the graph structurally valid, so declaration
    // ambiguity must win over the unknown operation.
    let mut both = operation_only;
    declare(&mut both, 3, &["Example", "Widget"], 1);
    let expected_id = both["semantic_graph"]["nodes"][3]["node_id"].clone();
    let refusal = refused(&both, &evidence_for(&both));
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::AmbiguousDeclaration
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::AmbiguousName)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/3/declaration/qualified_name".to_owned()),
    );
    assert_eq!(
        refusal.locus,
        Some(serde_json::from_value(expected_id).expect("authored structural id")),
    );
}

#[trace("TC-048", "TC-230", "FR-038-AC-21", "FR-038-AC-301")]
#[test]
fn tc_048_a_declaration_refusal_precedes_a_frame_refusal() {
    let mut frame_only = v2_all_families();
    let frame = frame_only["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "state" && node["semantic_form"] == "frame")
        .expect("frame node");
    frame_only["semantic_graph"]["nodes"][frame]["body"]["modifies"] = json!([{
        "kind": "relationship",
        "declaration": node_id("missing/relationship"),
    }]);
    checked_package::settle(&mut frame_only);
    let control = refused(&frame_only, &evidence_for(&frame_only));
    assert_eq!(control.code, CheckedPackageRefusalCode::MissingDeclaration);

    let mut both = frame_only;
    declare(&mut both, 0, &["Example", "Dup"], 0);
    declare(&mut both, 1, &["Example", "Dup"], 1);
    let expected_id = both["semantic_graph"]["nodes"][1]["node_id"].clone();
    let refusal = refused(&both, &evidence_for(&both));
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::AmbiguousDeclaration
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::AmbiguousName)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/1/declaration/qualified_name".to_owned()),
    );
    assert_eq!(
        refusal.locus,
        Some(serde_json::from_value(expected_id).expect("authored structural id")),
    );
}
