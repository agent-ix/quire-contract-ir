// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-019-AC-23: default-feature external consumer of the borrowed monitor API.

use quire_contract_model::{
    CheckedMonitorBufferRef, CheckedMonitorEntryRef, CheckedMonitorPlanRef, CheckedNodeId,
    CheckedPackageV2, CheckedRational, CheckedTerm,
};

fn entry<'a>(entry: CheckedMonitorEntryRef<'a>) {
    let _: &'a CheckedNodeId = entry.node_id();
    let _: &'a CheckedNodeId = entry.structural_key();
    let _: &'a CheckedNodeId = entry.result_type();
    let _: &'a CheckedTerm = entry.body();
    fn parameter_types<'a>(_: impl ExactSizeIterator<Item = &'a CheckedNodeId>) {}
    parameter_types(entry.parameter_types());
}
fn buffer<'a>(buffer: CheckedMonitorBufferRef<'a>) {
    let _: &'a CheckedNodeId = buffer.subformula();
    let _: &'a str = buffer.capacity();
}
fn plan<'a>(plan: CheckedMonitorPlanRef<'a>) {
    let _: &'a CheckedNodeId = plan.node_id();
    let _: &'a CheckedNodeId = plan.structural_key();
    let _: &'a CheckedNodeId = plan.clause();
    let _: &'a CheckedNodeId = plan.state();
    let _: &'a CheckedNodeId = plan.inputs();
    let _: &'a CheckedNodeId = plan.scope();
    let _: &'a CheckedNodeId = plan.results();
    let _: &'a CheckedNodeId = plan.tick_unit();
    let _: &'a CheckedTerm = plan.body();
    let _: u8 = plan.counter_width_bits();
    let _: u64 = plan.max_reading_gap();
    let _: &'a CheckedRational = plan.max_event_rate();
    let _: &'a CheckedRational = plan.uncertainty();
    let _: &'a str = plan.counter();
    let _: &'a str = plan.activation_capacity();
    entry(plan.init());
    entry(plan.on_event());
    entry(plan.on_watermark());
    entry(plan.on_gap());
    fn buffers<'a>(_: impl ExactSizeIterator<Item = CheckedMonitorBufferRef<'a>>) {}
    buffers(plan.buffers());
}
fn package(package: &CheckedPackageV2) {
    fn plans<'a>(_: impl ExactSizeIterator<Item = CheckedMonitorPlanRef<'a>>) {}
    plans(package.monitor_plans());
}

#[path = "../../../tests/it/support/checked_package.rs"]
mod fixture_support;
#[path = "../../../tests/it/support/monitor_plan.rs"]
mod monitor_plan_fixtures;

use quire_contract_model::{
    CheckedArtifactRef, CheckedExtensionDefinition, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2ReadResult,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    let root = std::env::var_os("QUIRE_SPECIFICATION_DIR")
        .expect("monitor consumer requires selected QUIRE_SPECIFICATION_DIR");
    let path = PathBuf::from(root).join("proposals/checked-package-v2/fixtures");
    assert!(path.is_dir(), "selected QSpec fixtures directory");
    path
}
fn proposal_dir_of(fixtures: &Path) -> PathBuf {
    fixtures.join("..")
}
fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn evidence(wire: &Value) -> CheckedPackageEvidence {
    let mut evidence = CheckedPackageEvidence::new();
    for identity in fixture_support::selected_model_identities(wire) {
        let file = format!("domain-package-{}.json", identity.replace('/', "-"));
        evidence
            .insert_domain_package_document(
                identity.as_str(),
                std::fs::read(proposal_dir_of(&fixtures_dir()).join(file)).unwrap(),
            )
            .unwrap();
    }
    for feature in wire["lock"]["required_features"].as_array().unwrap() {
        evidence.support_feature(feature.as_str().unwrap());
    }
    let definitions =
        read_json(&proposal_dir_of(&fixtures_dir()).join("extension-definitions.json")).unwrap();
    for definition in definitions["definitions"].as_array().unwrap() {
        evidence
            .insert_extension_definition(
                serde_json::from_value::<CheckedArtifactRef>(definition["reference"].clone())
                    .unwrap(),
                serde_json::from_value::<CheckedExtensionDefinition>(
                    definition["definition"].clone(),
                )
                .unwrap(),
            )
            .unwrap();
    }
    evidence
}

#[ix_trace_rs::trace(
    "TC-048",
    "FR-019-AC-23",
    "FR-038-AC-426",
    "FR-038-AC-433",
    "FR-038-AC-434"
)]
#[test]
fn default_feature_consumer_reads_an_actual_admitted_monitor_plan() {
    let vectors = monitor_plan_fixtures::recipes();
    let (mut wire, root) = monitor_plan_fixtures::materialize(&vectors["base"]);
    let gap = monitor_plan_fixtures::position(
        &wire,
        &wire["semantic_graph"]["nodes"][root]["body"]["members"][15]["value"]["target"],
    );
    monitor_plan_fixtures::append_measure(&mut wire, gap);
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        &fixture_support::canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence(&wire),
    ) else {
        panic!("complete selected monitor fixture admits")
    };
    assert_eq!(package.monitor_plans().len(), 1);
    let view = package.monitor_plans().next().unwrap();
    // Every helper above takes the exact specified borrowed signatures and
    // executes against this selected admitted package in a default build.
    plan(view);
    entry(view.on_gap());
    buffer(view.buffers().next().unwrap());
    assert_eq!(view.counter_width_bits(), 32);
    assert_eq!(view.max_reading_gap(), 1000);
    assert_eq!(view.max_event_rate().numerator.as_ref(), "10");
    assert_eq!(view.max_event_rate().denominator.as_ref(), "1");
    assert_eq!(view.uncertainty().numerator.as_ref(), "1");
    assert_eq!(view.uncertainty().denominator.as_ref(), "2");
    assert_eq!(view.counter(), "ticks");
    assert_eq!(view.activation_capacity(), "8");
    assert_eq!(
        view.buffers().map(|b| b.capacity()).collect::<Vec<_>>(),
        ["4", "7"]
    );
    assert_eq!(view.on_gap().parameter_types().len(), 2);
    let CheckedTerm::Aggregate { members } = view.on_gap().body() else {
        panic!("ordinary optional-measure function")
    };
    let CheckedTerm::Binding { name, .. } = &members[2] else {
        panic!("measure binding")
    };
    assert_eq!(name.as_ref(), "measure");
    assert_eq!(view.init().result_type(), view.state());
    let CheckedTerm::Aggregate { members } = package.term(view.state()).unwrap() else {
        panic!("admitted State record body")
    };
    let CheckedTerm::Binding { name, value } = members.last().unwrap() else {
        panic!("final State field")
    };
    assert_eq!(name.as_ref(), "event_position");
    let CheckedTerm::Reference { target } = value.as_ref() else {
        panic!("event_position is an ordinary type reference")
    };
    let integer = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "integer")
        .unwrap();
    assert_eq!(target, &integer.node_id);

    assert!(std::ptr::eq(
        view.structural_key(),
        package.structural_key(view.node_id()).unwrap()
    ));
    for e in [
        view.init(),
        view.on_event(),
        view.on_watermark(),
        view.on_gap(),
    ] {
        assert!(std::ptr::eq(
            e.structural_key(),
            package.structural_key(e.node_id()).unwrap()
        ));
    }
    assert_eq!(
        serde_json::to_value(view.body()).unwrap()["members"]
            .as_array()
            .unwrap()
            .len(),
        17
    );
    let bytes = quire_contract_model::encode_checked_package_v2(
        package.name(),
        package.lock(),
        package.graph(),
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: package.source_map(),
            refinement_step_rows: package.refinement_step_rows(),
        },
        quire_contract_model::CheckedPackageSidecarsRef {
            temporal_clause_names: None,
            operation_effects: None,
            checked_term_data: package.checked_term_data(),
            temporal_bindings: package.temporal_bindings(),
            schedulability_claims: package.schedulability_claim_rows(),
        },
        package.capability_report(),
        package.diagnostics(),
        quire_canonical::Limits::new(1 << 20),
    )
    .unwrap();
    assert_eq!(bytes, fixture_support::canonical(&wire));
    let CheckedPackageV2ReadResult::Admitted(roundtrip) = CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence(&wire),
    ) else {
        panic!("encoded admitted monitor retains admission")
    };
    let other = roundtrip.monitor_plans().next().unwrap();
    assert_eq!(view.structural_key(), other.structural_key());
    assert_eq!(view.body(), other.body());
    assert_eq!(package.term(view.state()), roundtrip.term(other.state()));
    assert_eq!(view.max_event_rate(), other.max_event_rate());
    assert_eq!(view.uncertainty(), other.uncertainty());
    assert_eq!(view.activation_capacity(), other.activation_capacity());
    for (a, b) in [
        (view.init(), other.init()),
        (view.on_event(), other.on_event()),
        (view.on_watermark(), other.on_watermark()),
        (view.on_gap(), other.on_gap()),
    ] {
        assert_eq!(a.structural_key(), b.structural_key());
        assert_eq!(a.body(), b.body());
        assert_eq!(
            a.parameter_types().collect::<Vec<_>>(),
            b.parameter_types().collect::<Vec<_>>()
        );
    }
    self::package(&package);
}
