// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Published single-body V2 wire packages, read from the owning QSpec checkout.

use super::{
    fixtures_dir, positive_document, proposal_dir_of, read_json, read_package,
    read_package_with_document, vector_selected_documents, POSITIVE_ALL,
};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    encode_checked_package_v2, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn fixture(name: &str) -> Value {
    read_json(&fixtures_dir().join(name)).unwrap_or_else(|why| panic!("{why}"))
}

fn admitted(package: &Value) -> Box<CheckedPackageV2> {
    match read_package(package).expect("published evidence can be prepared") {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("single-body package must admit, got {other:?}"),
    }
}

fn admitted_positive(
    name: &str,
    package: &Value,
    proposal: &std::path::Path,
    documents: &std::collections::BTreeMap<String, String>,
) -> Box<CheckedPackageV2> {
    let document = proposal.join(positive_document(name, documents));
    match read_package_with_document(package, name, &document)
        .expect("published evidence can be prepared")
    {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("{name}: single-body package must admit, got {other:?}"),
    }
}

#[trace("TC-048", "FR-038-AC-107")]
#[test]
fn published_single_body_packages_round_trip_through_the_writer() {
    let proposal = proposal_dir_of(&fixtures_dir());
    let documents = vector_selected_documents(&proposal).expect("published document selections");
    for name in POSITIVE_ALL {
        let package = fixture(name);
        let read = admitted_positive(name, &package, &proposal, &documents);
        assert_eq!(
            read.name(),
            package["name"].as_str().expect("published name")
        );
        assert_eq!(
            serde_json::to_value(read.graph()).expect("typed graph"),
            package["semantic_graph"],
            "{name}: complete graph and structural node ids"
        );
        let bytes = encode_checked_package_v2(
            read.name(),
            read.lock(),
            read.graph(),
            quire_contract_model::CheckedPackageSourceCorrespondenceRef {
                source_map: read.source_map(),
                refinement_step_rows: read.refinement_step_rows(),
            },
            quire_contract_model::CheckedPackageSidecarsRef {
                operation_effects: None,
                checked_term_data: read.checked_term_data(),
                temporal_bindings: read.temporal_bindings(),
            },
            read.capability_report(),
            read.diagnostics(),
            Limits::new(1 << 24),
        )
        .expect("published envelope encodes");
        let encoded: Value = serde_json::from_slice(&bytes).expect("encoded JSON");
        assert_eq!(
            encoded, package,
            "{name}: no second wire body or lost member"
        );
        assert_eq!(
            bytes,
            quire_canonical::to_vec(&package, Limits::new(1 << 24)).unwrap()
        );
        assert_eq!(
            admitted_positive(name, &encoded, &proposal, &documents).name(),
            read.name()
        );
    }
}

/// Trace: FR-038-AC-4
#[trace("TC-048", "FR-038-AC-4")]
#[test]
fn package_name_stays_fixed_while_admitted_body_changes_complete_value() {
    let base = fixture("positive-all-families.json");
    let original = admitted(&base);
    let mut changed = base.clone();
    let position = changed["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["body"]["term"] == "literal" && node["body"]["value_kind"] == "boolean"
        })
        .expect("published Boolean literal");
    changed["semantic_graph"]["nodes"][position]["body"]["value"] = json!(false);
    assert_eq!(changed["name"], base["name"]);
    assert_eq!(
        changed["semantic_graph"]["nodes"][position]["node_id"],
        base["semantic_graph"]["nodes"][position]["node_id"]
    );
    let edited = admitted(&changed);
    assert_eq!(edited.name(), original.name());
    assert_ne!(
        serde_json::to_value(edited.graph()).expect("edited graph"),
        serde_json::to_value(original.graph()).expect("original graph")
    );

    let mut reordered = base.clone();
    reordered["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .swap(0, 1);
    let changed_order = admitted(&reordered);
    assert_eq!(changed_order.name(), original.name());
    assert_ne!(
        serde_json::to_value(changed_order.graph()).expect("reordered graph"),
        serde_json::to_value(original.graph()).expect("original graph")
    );
}

/// Trace: FR-038-AC-310
#[trace("TC-048", "FR-038-AC-310")]
#[test]
fn retired_top_level_identity_members_refuse_at_the_exact_member() {
    let base = fixture("positive-all-families.json");
    assert_eq!(admitted(&base).name(), base["name"].as_str().expect("name"));
    for member in ["identity_preimage", "package_id", "identity_projection"] {
        let mut package = base.clone();
        package[member] = json!({});
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&package).expect("published evidence")
        else {
            panic!("retired top-level {member} must refuse")
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::UnknownMember,
            "{member}"
        );
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(format!("/{member}").as_str()),
            "{member}"
        );
        package.as_object_mut().expect("package").remove(member);
        assert_eq!(
            admitted(&package).name(),
            base["name"].as_str().expect("name")
        );
    }
}
