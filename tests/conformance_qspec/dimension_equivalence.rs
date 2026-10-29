// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec TC-233's published nominal preimages stay at their exact identities.

use super::{fixture_support, fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeOwner, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    NominalIdentityPreimage,
};
use serde_json::{json, Value};

fn vectors() -> Value {
    let path = proposal_dir_of(&fixtures_dir()).join("node-identity-vectors.json");
    read_json(&path).expect("QSpec node identity vectors")
}

fn named_row<'a>(rows: &'a [Value], name: &str) -> &'a Value {
    rows.iter()
        .find(|row| row["name"] == name)
        .unwrap_or_else(|| panic!("published case {name}"))
}

/// Trace: FR-063-AC-3
#[trace("TC-233", "FR-063-AC-3")]
#[test]
fn tc_233_dimension_equivalence_vectors_have_the_published_keys_and_scales() {
    let vectors = vectors();
    let rows = vectors["vectors"].as_array().expect("vectors array");
    let cases = [
        ("dimension-equivalence-rpc-flow-time", "1", "1"),
        ("dimension-equivalence-rpc-clock-time", "1000", "1"),
        ("dimension-equivalence-flow-clock-time", "1000", "1"),
    ];
    for (name, numerator, denominator) in cases {
        let row = named_row(rows, name);
        let expected_key = row["sha256"].as_str().expect("published key");
        let preimage: NominalIdentityPreimage =
            serde_json::from_value(row["preimage"].clone()).expect("typed published preimage");
        let NominalIdentityPreimage::DimensionEquivalence(edge) = &preimage else {
            panic!("published equivalence variant: {name}")
        };
        assert_eq!(edge.root_scale.numerator.as_ref(), numerator);
        assert_eq!(edge.root_scale.denominator.as_ref(), denominator);
        assert_eq!(
            preimage.digest(u64::MAX).expect("canonical key"),
            expected_key
        );
    }
    for name in [
        "model-dimension-acme-rpc-time",
        "model-dimension-acme-flow-time",
        "model-dimension-acme-clock-time",
    ] {
        let row = named_row(rows, name);
        let expected_key = row["sha256"].as_str().expect("published key");
        let preimage: NominalIdentityPreimage =
            serde_json::from_value(row["preimage"].clone()).expect("typed dimension");
        assert_eq!(
            preimage.digest(u64::MAX).expect("canonical key"),
            expected_key
        );
    }
}

/// Trace: FR-063-AC-3
#[trace("TC-233", "FR-063-AC-3")]
#[test]
fn tc_233_published_model_owned_time_graph_admits_as_one_class() {
    let vectors = vectors();
    let rows = vectors["vectors"].as_array().expect("vectors array");
    let names = [
        "model-dimension-acme-rpc-time",
        "model-dimension-acme-flow-time",
        "model-dimension-acme-clock-time",
        "dimension-equivalence-rpc-flow-time",
        "dimension-equivalence-rpc-clock-time",
        "dimension-equivalence-flow-clock-time",
    ];
    let members = names.map(|name| {
        let row = named_row(rows, name);
        (
            row["preimage"].clone(),
            row["sha256"].as_str().expect("published key").to_owned(),
        )
    });
    let mut package = fixture_support::nominal_package(&members);
    let published_package = read_json(&fixtures_dir().join("positive-nominal-identities.json"))
        .expect("QSpec nominal package");
    let source = published_package["lock"]["sources"][0].clone();
    assert_eq!(source["authority"], members[3].0["owner"]["authority"]);
    assert_eq!(source["identity"], members[3].0["owner"]["identity"]);
    package["lock"]["sources"][0] = source.clone();
    for entry in package["source_map"].as_array_mut().expect("source map") {
        entry["regions"][0]["source"] = source.clone();
    }
    for position in 0..3 {
        let node = &mut package["semantic_graph"]["nodes"][position];
        node.as_object_mut().expect("node").remove("declaration");
        node["occurrences"][0]["role"] = json!("type");
        package["source_map"][position]["role"] = json!("type");
    }
    for (preimage, _) in &members[..3] {
        let identity = preimage["owner"]["identity"]
            .as_str()
            .expect("model identity");
        fixture_support::select_model(&mut package, identity);
    }
    fixture_support::refresh_identity(&mut package);

    let admitted = match CheckedPackageV2::read(
        &fixture_support::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &fixture_support::evidence_for(&package),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("published three-package Time graph must admit: {other:?}"),
    };
    assert_eq!(admitted.graph().nodes.len(), names.len() + 3);
    for (position, (preimage, published_key)) in members.iter().enumerate() {
        let node = &admitted.graph().nodes[position];
        assert_eq!(node.node_id.digest.as_ref(), published_key, "{position}");
        let expected: NominalIdentityPreimage =
            serde_json::from_value(preimage.clone()).expect("published preimage");
        assert_eq!(node.nominal_identity_preimage.as_ref(), Some(&expected));
    }
    assert_eq!(
        admitted.graph().nodes[3..names.len()]
            .iter()
            .map(|node| node.occurrences[0].ordinal)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let mut identities = members[..3]
        .iter()
        .map(|(preimage, _)| preimage["owner"]["identity"].as_str().expect("identity"))
        .collect::<Vec<_>>();
    identities.sort_unstable();
    identities.dedup();
    let roots = admitted.graph().nodes[names.len()..]
        .iter()
        .filter(|node| node.semantic_form.as_ref() == "model_root")
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), identities.len());
    for root in roots {
        let Some(CheckedNodeOwner::ModelRoot { identity, .. }) = root.owner.as_ref() else {
            panic!("selected model root owner");
        };
        assert!(identities.contains(&identity.as_ref()));
        assert_eq!(root.occurrences[0].ordinal, 0);
    }
}
