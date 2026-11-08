// SPDX-License-Identifier: AGPL-3.0-or-later
//! OP vectors are read from QSpec; only the projection-to-document adapter lives here.

use super::{fixture_support as fixture, fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedCollectionKind, CheckedMemberType, CheckedNodeId, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("model-option-type-vectors.json"))
        .expect("the current QSpec option corpus")
}

#[path = "../it/support/qspec_option_inputs.rs"]
mod inputs;
use inputs::{apply, document, model_id, Input};

fn read(input: &Input) -> CheckedPackageV2ReadResult {
    let (package, evidence) = inputs::prepare(input);
    CheckedPackageV2::read(
        &fixture::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn projected_pointer(input: &Input, pointer: &str) -> String {
    if let Some(rest) = pointer.strip_prefix("/types/") {
        let (index, suffix) = rest
            .split_once('/')
            .map_or((rest, ""), |(index, suffix)| (index, suffix));
        let prefix = &input.origins[index.parse::<usize>().unwrap()];
        let suffix = if prefix.starts_with("/variant_types/") {
            suffix.replacen("variants", "cases", 1)
        } else {
            suffix.to_owned()
        };
        if suffix.is_empty() {
            prefix.clone()
        } else {
            format!("{prefix}/{suffix}")
        }
    } else {
        pointer.to_owned()
    }
}

fn expected(value: &Value, unit: &CheckedNodeId, identity: &str) -> CheckedMemberType {
    if let Some(inner) = value.get("option") {
        return CheckedMemberType::Option(Box::new(expected(inner, unit, identity)));
    }
    if let Some(inner) = value.get("sequence") {
        return CheckedMemberType::Collection {
            kind: CheckedCollectionKind::Sequence,
            element: Box::new(expected(inner, unit, identity)),
            bounds: Some((
                value["min"].as_str().unwrap().parse().unwrap(),
                value["max"].as_str().unwrap().parse().unwrap(),
            )),
        };
    }
    if let Some(name) = value["declaration"].as_str() {
        return CheckedMemberType::Record(
            serde_json::from_value(model_id(
                identity,
                &format!("ix://{identity}/{name}"),
                "record_value_type",
            ))
            .unwrap(),
        );
    }
    match value["type"].as_str().unwrap() {
        "Integer" => CheckedMemberType::Integer,
        "Quantity<ms>" => CheckedMemberType::Quantity(unit.clone()),
        other => panic!("new published expected type needs its independent decoder: {other}"),
    }
}

#[trace("TC-465", "FR-356-AC-1", "FR-356-AC-2", "FR-356-AC-5")]
#[test]
fn published_option_field_types_match_canonical_domains() {
    let vectors = vectors();
    let (input, field_queries) = inputs::field_query_inputs(&vectors);
    let admitted = match read(&input) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("base option projection must admit: {other:?}"),
    };
    let identity = input.document["package"]["identity"].as_str().unwrap();
    let holder: CheckedNodeId = serde_json::from_value(model_id(
        identity,
        &format!("ix://{identity}/Holder"),
        "object_type",
    ))
    .unwrap();
    let fields = admitted.model_object_fields(&holder).unwrap();
    let unit = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.declaration.as_ref().is_some_and(|declaration| {
                declaration
                    .qualified_name
                    .last()
                    .is_some_and(|name| name.as_ref() == "ms")
            })
        })
        .unwrap()
        .node_id
        .clone();
    let mut seen = BTreeSet::new();
    for case in field_queries {
        let name = case["name"].as_str().unwrap();
        assert!(seen.insert(name));
        if let Some(field) = case["field"].as_str() {
            let actual = fields.field(field).unwrap().member_type().unwrap();
            assert_eq!(
                actual,
                &expected(&case["expected"], &unit, identity),
                "{name}"
            );
            if let Some(other) = case["same_node_as"].as_str() {
                assert_eq!(
                    Some(actual),
                    fields.field(other).unwrap().member_type(),
                    "{name}: canonical option identity"
                );
            }
        } else {
            let census = admitted.model_operations(&holder, 100_000).unwrap();
            let operation = census
                .operations()
                .iter()
                .find(|op| Some(op.name()) == case["operation"].as_str())
                .unwrap();
            let parameters = case["expected"]["parameters"].as_array().unwrap();
            assert_eq!(operation.parameters().len(), parameters.len(), "{name}");
            for (actual, want) in operation.parameters().iter().zip(parameters) {
                assert_eq!(
                    actual.member_type(),
                    Some(&expected(want, &unit, identity)),
                    "{name}"
                );
            }
            assert_eq!(
                operation.result(),
                Some(Some(&expected(
                    &case["expected"]["result"],
                    &unit,
                    identity
                ))),
                "{name}"
            );
        }
        eprintln!("{name}: PASS");
    }
    for i in 1..=8 {
        assert!(seen.contains(format!("OP-{i:02}").as_str()));
    }
}

#[trace("TC-465", "FR-356-AC-1", "FR-356-AC-3", "FR-356-AC-4", "FR-356-AC-5")]
#[test]
fn published_option_intake_rows_match_exact_refusals() {
    let vectors = vectors();
    let mut seen = BTreeSet::new();
    let mut mismatches = BTreeMap::new();
    for case in vectors["intake_cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        assert!(seen.insert(name));
        let mut projection = vectors["domain_package"].clone();
        for patch in case["patch"].as_array().unwrap() {
            apply(&mut projection, patch);
        }
        let input = document(&projection);
        let result = read(&input);
        if case["expected"]["admitted"] == true {
            if !matches!(result, CheckedPackageV2ReadResult::Admitted(_)) {
                mismatches.insert(name, format!("expected admission, got {result:?}"));
            }
        } else if let CheckedPackageV2ReadResult::Refused(refusal) = result {
            let expected = &case["expected"]["refused"];
            let code = super::code_word(refusal.code);
            let cause = refusal.cause.map(super::cause_word);
            let at = refusal
                .document_pointer
                .as_ref()
                .map(|pointer| projected_pointer(&input, pointer.as_str()));
            if Some(code) != expected["code"].as_str()
                || cause != expected["cause"].as_str()
                || at.as_deref() != expected["at"].as_str()
            {
                mismatches.insert(
                    name,
                    format!("expected {expected}, got {code}/{cause:?} at {at:?} ({refusal:?})"),
                );
            }
        } else {
            mismatches.insert(name, format!("expected refusal, got {result:?}"));
        }
        eprintln!(
            "{name}: {}",
            if mismatches.contains_key(name) {
                "MISMATCH"
            } else {
                "PASS"
            }
        );
    }
    for i in 9..=41 {
        assert!(seen.contains(format!("OP-{i:02}").as_str()));
    }
    assert!(
        mismatches.is_empty(),
        "option cross-reader mismatches: {mismatches:#?}"
    );
}
