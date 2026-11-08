// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec TC-233: bounded Text literal membership at the public V2 reader.

use super::{fixture_support, fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use quire_exact::TextProfile;
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[path = "../it/support/qspec_text_inputs.rs"]
mod inputs;
use inputs::{bounded_literal, text_bounds_package};

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &fixture_support::canonical(package),
        CheckedPackageReadLimits::bounded(),
        &fixture_support::evidence_for(package),
    )
}

fn assert_text_shape_refusal(package: &Value, position: usize, context: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!(
            "{context}: malformed Text bounds must refuse: {:?}",
            read(package)
        );
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{context}: {refusal:?}");
    assert_eq!(
        refusal.cause,
        Some(Cause::InvalidDerivedShape),
        "{context}: {refusal:?}"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}")),
        "{context}: {refusal:?}"
    );
    assert_eq!(
        refusal.locus,
        Some(
            serde_json::from_value(package["semantic_graph"]["nodes"][position]["node_id"].clone())
                .expect("Text bounds node identity")
        ),
        "{context}: structural node locus"
    );
}

#[trace("TC-048", "FR-038-AC-417", "FR-038-AC-418", "FR-038-AC-419")]
#[test]
fn qspec_text_bounds_shape_vectors_match_reader() {
    let vectors =
        read_json(&proposal_dir_of(&fixtures_dir()).join("text-bounds-shape-vectors.json"))
            .expect("QSpec Text bounds shape vectors");
    let cases = vectors["cases"].as_array().expect("QSpec cases");
    let actual_ids = cases
        .iter()
        .map(|case| case["id"].as_str().expect("case id").to_owned())
        .collect::<BTreeSet<_>>();
    let expected_ids = (1..=10)
        .map(|number| format!("TS-{number:02}"))
        .collect::<BTreeSet<_>>();
    assert_eq!(cases.len(), 10, "published TS case count");
    assert_eq!(actual_ids, expected_ids, "exact published TS id set");
    for case in cases {
        let id = case["id"].as_str().expect("case id");
        let members = case["members"].as_array().expect("shape members");
        let literal = match id {
            "TS-02" => "ab",
            "TS-03" => "e\u{301}", // Not NFC: shape refusal must win over membership.
            _ => "outside-bounds",
        };
        let (package, position) = text_bounds_package(members, literal);
        match (case["expected"].as_str(), read(&package)) {
            (Some("admitted"), CheckedPackageV2ReadResult::Admitted(_)) => {}
            (Some("refused"), _) => {
                assert_eq!(case["code"], "invalid_package", "{id}: vector code");
                assert_eq!(case["cause"], "invalid-derived-shape", "{id}: vector cause");
                assert_eq!(
                    case["locus"], "/semantic_graph/nodes/{n}",
                    "{id}: vector locus"
                );
                assert_text_shape_refusal(&package, position, id);
            }
            (expected, actual) => panic!("{id}: expected {expected:?}, got {actual:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-417", "FR-038-AC-418")]
#[test]
fn text_bounds_reordered_wrong_kind_and_negative_bound_refuse_before_membership() {
    let correct = vec![
        json!({"name": "min", "value_kind": "integer", "value": "0"}),
        json!({"name": "max", "value_kind": "integer", "value": "1"}),
        json!({"name": "text_profile", "value_kind": "text", "value": "nfc"}),
    ];
    let mut reordered = correct.clone();
    reordered.swap(0, 1);
    let mut wrong_kind = correct.clone();
    wrong_kind[0]["value_kind"] = json!("text");
    let mut negative = correct;
    negative[0]["value"] = json!("-1");
    for (name, members) in [
        ("reordered", reordered),
        ("wrong-kind", wrong_kind),
        ("negative", negative),
    ] {
        let (package, position) = text_bounds_package(&members, "outside-bounds");
        assert_text_shape_refusal(&package, position, name);
    }
}

#[trace("TC-048", "FR-038-AC-417", "FR-038-AC-418")]
#[test]
fn text_bounds_accept_all_six_profiles_and_refuse_noncanonical_minimum() {
    for profile in [
        "unicode-scalars",
        "nfc",
        "nfd",
        "nfkc",
        "nfkd",
        "binary-utf8",
    ] {
        let members = vec![
            json!({"name": "min", "value_kind": "integer", "value": "1"}),
            json!({"name": "max", "value_kind": "integer", "value": "1"}),
            json!({"name": "text_profile", "value_kind": "text", "value": profile}),
        ];
        let (package, _) = text_bounds_package(&members, "a");
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{profile}: complete equal bounds must admit: {:?}",
            read(&package)
        );
    }
    let noncanonical = vec![
        json!({"name": "min", "value_kind": "integer", "value": "01"}),
        json!({"name": "max", "value_kind": "integer", "value": "1"}),
        json!({"name": "text_profile", "value_kind": "text", "value": "nfc"}),
    ];
    let (package, position) = text_bounds_package(&noncanonical, "a");
    assert_text_shape_refusal(&package, position, "noncanonical minimum");
}

#[trace("TC-048", "FR-038-AC-417", "FR-038-AC-418")]
#[test]
fn canonical_text_bounds_above_u64_are_shapes_and_check_literal_membership() {
    let beyond_u64 = "18446744073709551616";
    let members = |min: &str, max: &str| {
        vec![
            json!({"name": "min", "value_kind": "integer", "value": min}),
            json!({"name": "max", "value_kind": "integer", "value": max}),
            json!({"name": "text_profile", "value_kind": "text", "value": "nfc"}),
        ]
    };
    let (package, _) = text_bounds_package(&members("0", beyond_u64), "a");
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "canonical maximum beyond u64 admits an in-range literal: {:?}",
        read(&package)
    );

    let (package, bounds_position) = text_bounds_package(&members(beyond_u64, beyond_u64), "a");
    let literal_position = bounds_position + 1;
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!(
            "large minimum must reach literal membership: {:?}",
            read(&package)
        );
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::TypeMismatch), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{literal_position}/body/value"
        )),
        "{refusal:?}"
    );
    assert_eq!(
        refusal.locus,
        Some(
            serde_json::from_value(
                package["semantic_graph"]["nodes"][literal_position]["node_id"].clone()
            )
            .expect("literal node identity")
        )
    );

    let (package, position) =
        text_bounds_package(&members("18446744073709551617", beyond_u64), "a");
    assert_text_shape_refusal(&package, position, "inverted large bounds");
}

#[trace("TC-048", "FR-038-AC-409", "FR-038-AC-410", "FR-038-AC-411")]
#[test]
fn qspec_text_literal_vectors_match_reader() {
    let vectors = read_json(&proposal_dir_of(&fixtures_dir()).join("text-literal-vectors.json"))
        .expect("QSpec text literal vectors");
    let types = vectors["types"].as_object().expect("QSpec types");
    let cases = vectors["cases"].as_array().expect("QSpec cases");
    assert_eq!(cases.len(), 15, "published TX-01..TX-14 plus TX-13s");
    let actual_ids = cases
        .iter()
        .map(|case| case["id"].as_str().expect("case id").to_owned())
        .collect::<BTreeSet<_>>();
    let expected_ids = (1..=14)
        .map(|number| format!("TX-{number:02}"))
        .chain(std::iter::once("TX-13s".to_owned()))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_ids, expected_ids, "exact published TX id set");
    for case in cases {
        let id = case["id"].as_str().expect("case id");
        let ty = &types[case["type"].as_str().expect("type key")];
        let min = ty["min"].as_u64().expect("minimum");
        let max = ty["max"].as_u64().expect("maximum");
        let profile = TextProfile::from_code(ty["text_profile"].as_str().expect("profile"))
            .expect("published Text profile");
        let retained = case["value"].as_str().expect("retained Text payload");
        let length = profile.length(retained);
        let normalized = profile
            .normalization()
            .is_none_or(|form| retained.chars().eq(form.apply(retained)));
        assert_eq!(
            case["length"].as_u64(),
            Some(length),
            "{id}: retained length"
        );
        assert_eq!(
            case["normalized"].as_bool(),
            Some(normalized),
            "{id}: retained normalization"
        );
        let expected_outcome = if normalized && (min..=max).contains(&length) {
            "admitted"
        } else {
            "refused"
        };
        assert_eq!(case["expected"], expected_outcome, "{id}: vector oracle");
        let (package, position) =
            bounded_literal(min, max, profile.as_str(), case["value"].clone());
        match (case["expected"].as_str(), read(&package)) {
            (Some("admitted"), CheckedPackageV2ReadResult::Admitted(_)) => {}
            (Some("refused"), CheckedPackageV2ReadResult::Refused(refusal)) => {
                assert_eq!(refusal.code, Code::IllTyped, "{id}: {refusal:?}");
                assert_eq!(
                    refusal.cause,
                    Some(Cause::TypeMismatch),
                    "{id}: {refusal:?}"
                );
                assert_eq!(
                    refusal.loci[0].path.as_ref().map(ToString::to_string),
                    Some(format!("/semantic_graph/nodes/{position}/body/value")),
                    "{id}: {refusal:?}"
                );
                assert_eq!(
                    refusal.locus,
                    Some(
                        serde_json::from_value(
                            package["semantic_graph"]["nodes"][position]["node_id"].clone()
                        )
                        .expect("containing graph-node identity")
                    ),
                    "{id}: graph-node locus"
                );
            }
            (expected, actual) => panic!("{id}: expected {expected:?}, got {actual:?}"),
        }
    }
}

#[trace("TC-048", "FR-038-AC-409", "FR-038-AC-410")]
#[test]
fn compatibility_forms_and_family_precedence_are_reader_controls() {
    for (profile, retained, expected) in [
        ("nfkc", "ﬀ", false),
        ("nfkc", "ff", true),
        ("nfkd", "ﬀ", false),
        ("nfkd", "ff", true),
    ] {
        let (package, _) = bounded_literal(1, 2, profile, json!(retained));
        assert_eq!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            expected,
            "{profile}: {retained:?}: {:?}",
            read(&package)
        );
    }
    let (mut package, position) = bounded_literal(1, 2, "nfc", json!("abc"));
    package["semantic_graph"]["nodes"][position]["body"]["value_kind"] = json!("integer");
    fixture_support::settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("integer kind over Text family must refuse");
    };
    assert_eq!(refusal.code, Code::IllTyped);
    assert_eq!(refusal.cause, Some(Cause::OperatorIneligible));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}/body/type"))
    );
}

#[trace("TC-048", "FR-038-AC-409", "FR-038-AC-410")]
#[test]
fn scalar_text_stays_unbounded_and_nested_bounds_report_the_literal_pointer() {
    let (mut scalar, position) = bounded_literal(1, 2, "nfc", json!("beyond"));
    let text = fixture_support::node_id(&fixture_support::family_key("a1a1"));
    scalar["semantic_graph"]["nodes"][position]["semantic_type"] = text.clone();
    scalar["semantic_graph"]["nodes"][position]["dependencies"] = json!([]);
    scalar["semantic_graph"]["nodes"][position]["body"]["type"] =
        fixture_support::node_id(&fixture_support::family_key("a1a1"));
    fixture_support::settle(&mut scalar);
    assert!(
        matches!(read(&scalar), CheckedPackageV2ReadResult::Admitted(_)),
        "plain scalar Text has no bound: {:?}",
        read(&scalar)
    );

    let (mut nested, literal_position) = bounded_literal(1, 2, "nfc", json!("ab"));
    let bounds_id = nested["semantic_graph"]["nodes"][literal_position - 1]["node_id"].clone();
    // A literal node cannot hold an aggregate, so nest inside an aggregate-bodied value node.
    let root_literal = nested["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "function" && node["body"]["term"] == "aggregate")
        .expect("aggregate function node");
    let mut nested_node = nested["semantic_graph"]["nodes"][root_literal].clone();
    nested_node["node_id"] =
        json!({"ordinal": 0, "package": "test/families", "path": ["Example", "Nested"]});
    nested_node["declaration"] = json!({"qualified_name": ["Example", "Nested"]});
    nested_node["body"] = json!({"term": "aggregate", "members": [{
        "term": "binding", "name": "value", "value": {
            "term": "literal", "type": bounds_id, "value_kind": "text", "value": "abc"
        }
    }]});
    let nested_position = nested["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    nested["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(nested_node);
    fixture_support::settle(&mut nested);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&nested) else {
        panic!("nested out-of-bounds Text must refuse: {:?}", read(&nested));
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::TypeMismatch), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{nested_position}/body/members/0/value/value"
        )),
        "{refusal:?}"
    );
    assert_eq!(
        refusal.locus,
        Some(
            serde_json::from_value(
                nested["semantic_graph"]["nodes"][nested_position]["node_id"].clone()
            )
            .expect("nested containing node")
        ),
        "{refusal:?}"
    );
}
