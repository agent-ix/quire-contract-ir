// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public-reader qualification of the owner's complete selected timed-slot package.

use super::*;
use std::collections::BTreeSet;

fn read_selected(wire: &Value, id: &str, fixtures: &Path) -> CheckedPackageV2ReadResult {
    read_package_with_document(
        wire,
        id,
        &proposal_dir_of(fixtures).join("domain-package-acme-timed-slots.json"),
    )
    .expect("published selected timed-slot package")
}

fn check_outcome(result: CheckedPackageV2ReadResult, case: &Value) {
    let id = case["id"].as_str().expect("case id");
    match (case["expected"].as_str(), result) {
        (Some("admitted"), CheckedPackageV2ReadResult::Admitted(_)) => {
            println!("{id}: admitted");
        }
        (Some("refused"), CheckedPackageV2ReadResult::Refused(refusal)) => {
            assert_eq!(
                Some(code_word(refusal.code)),
                case["expected_code"].as_str(),
                "{id}"
            );
            assert_eq!(
                refusal.cause.map(cause_word),
                case["expected_cause"].as_str(),
                "{id}"
            );
            let path = refusal.loci[0]
                .path
                .as_ref()
                .expect("refusal pointer")
                .to_string();
            if let Some(exact) = case["locus"].as_str() {
                assert_eq!(path, exact, "{id}");
            } else {
                let suffix = case["locus_suffix"]
                    .as_str()
                    .expect("published refusal suffix");
                assert!(path.ends_with(&format!("/{suffix}")), "{id}: {path}");
            }
            println!("{id}: {:?} {:?} at {path}", refusal.code, refusal.cause);
        }
        (expected, actual) => panic!("{id}: expected {expected:?}, got {actual:?}"),
    }
}

fn check_earlier_stages(wire: &Value, id: &str, fixtures: &Path) {
    let root = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| {
            node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model"
        })
        .unwrap();
    let mut malformed = wire.clone();
    malformed["semantic_graph"]["nodes"][root]["occurrences"] = json!("not-an-array");
    let CheckedPackageV2ReadResult::Refused(refusal) = read_selected(&malformed, id, fixtures)
    else {
        panic!("{id}: wire defect must precede the semantic defect");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::MalformedWire, None),
        "{id}"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{root}/occurrences"),
        "{id}"
    );

    let mut graph = wire.clone();
    let root_node = &mut graph["semantic_graph"]["nodes"][root];
    let first_place = root_node["body"]["members"][3]["value"]["members"][0]["target"].clone();
    let dependencies = root_node["dependencies"].as_array_mut().unwrap();
    let position = dependencies
        .iter()
        .position(|target| *target == first_place)
        .unwrap();
    dependencies.remove(position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_selected(&graph, id, fixtures) else {
        panic!("{id}: graph defect must precede the semantic defect");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidSemanticGraph, None),
        "{id}"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{root}/dependencies"),
        "{id}"
    );
}

#[test]
#[trace("TC-048")]
fn every_published_slot_and_presence_case_uses_the_strict_reader() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("timed-slot-selector-vectors.json")).unwrap();
    let original =
        read_json(&proposal.join(vectors["selected_package"].as_str().unwrap())).unwrap();
    let cases = vectors["cases"].as_array().expect("slot cases");
    let mut seen = BTreeSet::new();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        if case["expected"] == "admitted" {
            let position = usize::try_from(case["package_node"].as_u64().unwrap()).unwrap();
            assert_eq!(
                original["semantic_graph"]["nodes"][position], case["node"],
                "{id}"
            );
            check_outcome(read_selected(&original, id, &fixtures), case);
        } else {
            let selected = cases
                .iter()
                .find(|row| row["expected"] == "admitted" && row["field"] == case["field"])
                .expect("the selected package contains this declared field's valid place");
            let position = usize::try_from(selected["package_node"].as_u64().unwrap()).unwrap();
            let mut wire = original.clone();
            let old = wire["semantic_graph"]["nodes"][position]["node_id"].clone();
            let fresh = case["node"]["node_id"].clone();
            wire["semantic_graph"]["nodes"][position] = case["node"].clone();
            fixture_support::reassign_node_id(&mut wire, &old, &fresh);
            check_outcome(read_selected(&wire, id, &fixtures), case);
            check_earlier_stages(&wire, id, &fixtures);
        }
    }
    let presence = vectors["presence"]["cases"].as_array().unwrap();
    let baseline = presence.iter().find(|row| row["id"] == "PR-03").unwrap();
    let baseline_position = usize::try_from(baseline["package_node"].as_u64().unwrap()).unwrap();
    for case in presence {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        if case["expected"] == "admitted" {
            let position = usize::try_from(case["package_node"].as_u64().unwrap()).unwrap();
            assert_eq!(
                original["semantic_graph"]["nodes"][position], case["node"],
                "{id}"
            );
            check_outcome(read_selected(&original, id, &fixtures), case);
        } else {
            let mut wire = original.clone();
            let old = wire["semantic_graph"]["nodes"][baseline_position]["node_id"].clone();
            let fresh = case["node"]["node_id"].clone();
            wire["semantic_graph"]["nodes"][baseline_position] = case["node"].clone();
            fixture_support::reassign_node_id(&mut wire, &old, &fresh);
            check_outcome(read_selected(&wire, id, &fixtures), case);
            check_earlier_stages(&wire, id, &fixtures);
        }
    }
    assert_eq!(
        seen,
        BTreeSet::from([
            "SL-01", "SL-02", "SL-03", "SL-04", "SL-05", "SL-06", "SL-07", "SR-01", "SR-02",
            "SR-03", "SR-04", "PR-01", "PR-03", "PR-04", "PR-05", "PR-06",
        ])
    );
}

#[test]
#[trace("TC-048")]
fn every_published_guard_case_uses_the_strict_reader() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("timed-slot-selector-vectors.json")).unwrap();
    let mut seen = BTreeSet::new();
    for case in vectors["guards"].as_array().expect("guard vectors") {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id));
        let mut wire = read_json(&proposal.join(case["fixture"].as_str().unwrap())).unwrap();
        for patch in case["patch"].as_array().into_iter().flatten() {
            super::timed_model::apply_published_timed_patch(&mut wire, patch);
        }
        if id == "GD-01" {
            let result = read_selected(&wire, id, &fixtures);
            let CheckedPackageV2ReadResult::Admitted(package) = result else {
                panic!("GD-01 must admit: {result:?}");
            };
            let root = package
                .graph()
                .nodes
                .iter()
                .find(|node| {
                    node.node_tag.as_ref() == "timed_model"
                        && node.semantic_form.as_ref() == "timed_model"
                })
                .expect("admitted timed root");
            let CheckedTerm::Aggregate { members } = &root.body else {
                panic!("typed root aggregate");
            };
            for name in [
                "clock_places",
                "resets",
                "guards",
                "invariants",
                "urgencies",
                "constraints",
            ] {
                assert!(members.iter().any(|member| matches!(member,
                    CheckedTerm::Binding { name: actual, value } if actual.as_ref() == name
                        && matches!(value.as_ref(), CheckedTerm::Aggregate { members } if members.is_empty())
                )), "{name}");
            }
            println!("{id}: admitted with all six empty timed aggregates");
        } else {
            assert!(matches!(id, "GD-02" | "GD-03"));
            let CheckedPackageV2ReadResult::Refused(refusal) = read_selected(&wire, id, &fixtures)
            else {
                panic!("{id} must refuse");
            };
            assert_eq!(
                Some(code_word(refusal.code)),
                case["expected_code"].as_str(),
                "{id}"
            );
            assert_eq!(
                refusal.cause.map(cause_word),
                case["expected_cause"].as_str(),
                "{id}"
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().unwrap().as_str(),
                case["locus"].as_str().unwrap(),
                "{id}"
            );
            println!(
                "{id}: {:?} {:?} at {}",
                refusal.code,
                refusal.cause,
                refusal.loci[0].path.as_ref().unwrap()
            );
        }
    }
    assert_eq!(seen, BTreeSet::from(["GD-01", "GD-02", "GD-03"]));
}
