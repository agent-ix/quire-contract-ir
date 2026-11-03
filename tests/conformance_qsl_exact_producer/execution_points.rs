// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! An external consumer of fresh QSL clauses and IR's admitted point accessor.

use std::{env, fs, path::PathBuf};

use qsl_foundation::SourceIdentity;
use qsl_replay::{
    compile_package_with_execution_points, spine::DependencyInput, ReplayLimits, StageLimits,
};
use quire_contract_model::{
    CheckedExecutionPointError, CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    ExecutionPoint, ExecutionPointName,
};
use serde_json::Value;

// A CG-style consumer supplies no source binding or caller-selected point.
fn consume(
    package: &CheckedPackageV2,
    clause: &CheckedNodeId,
    claim: (&CheckedNodeId, &CheckedOccurrence),
) -> Result<ExecutionPoint, CheckedExecutionPointError> {
    package.clause_execution_point(clause, claim)
}

fn main() {
    let qsl = PathBuf::from(env::var_os("QUIRE_SPEC_LANGUAGE_DIR").expect("QSL checkout"));
    let document = fs::read(qsl.join("examples/switch-transition/switch.semantic-ir.json"))
        .expect("owner model document by reference");
    let source = String::from(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         model M = \"test/orders\";\n\
         invariant Startup using v on M::Switch at current { true }\n\
         invariant Tick using v on M::Switch at current { true }\n\
         pre BeforeAdjust using v on M::Switch::adjust { true }\n\
         post AfterAdjust using v on M::Switch::adjust { true }\n",
    );
    let points = [
        (
            "Startup",
            ExecutionPoint::Initialization {
                name: ExecutionPointName::new("startup").unwrap(),
            },
        ),
        (
            "Tick",
            ExecutionPoint::Handler {
                name: ExecutionPointName::new("tick").unwrap(),
            },
        ),
        (
            "BeforeAdjust",
            ExecutionPoint::Pre {
                operation: ExecutionPointName::new("adjust").unwrap(),
            },
        ),
        (
            "AfterAdjust",
            ExecutionPoint::Post {
                operation: ExecutionPointName::new("adjust").unwrap(),
            },
        ),
    ];
    let bindings = points
        .iter()
        .map(|(name, point)| ((*name).to_owned(), point.clone()))
        .collect();
    let compiled = compile_package_with_execution_points(
        SourceIdentity::new("agent-ix", "test:execution-points"),
        "execution-points".to_owned(),
        "execution-points.native",
        source.as_bytes(),
        [document.as_slice()],
        &DependencyInput::new(Vec::new()).expect("empty dependency acquisition"),
        &bindings,
        StageLimits::default(),
        ReplayLimits::default(),
    )
    .expect("genuine QSL four-clause checking and emission");
    let wire: Value = serde_json::from_slice(compiled.bytes()).expect("emitted envelope");
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document("test/orders", document.as_slice())
        .expect("selected owner document");
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("features")
    {
        evidence.support_feature(feature.as_str().expect("feature name"));
    }
    let package = match CheckedPackageV2::read(
        compiled.bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("fresh producer package must admit: {other:?}"),
    };
    // Select each authored clause independently through its retained source
    // region, before discarding the bytes. Never select by the returned point.
    let mut selections = Vec::new();
    for (name, expected) in points {
        let marker = format!(" {name} using ");
        let entries = wire["source_map"]
            .as_array()
            .expect("source map")
            .iter()
            .filter(|entry| {
                entry["role"] == "claim"
                    && entry["regions"]
                        .as_array()
                        .expect("regions")
                        .iter()
                        .any(|region| {
                            let start =
                                usize::try_from(region["start"].as_u64().expect("start")).unwrap();
                            let end =
                                usize::try_from(region["end"].as_u64().expect("end")).unwrap();
                            source
                                .get(start..end)
                                .expect("source region")
                                .contains(&marker)
                        })
            })
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 1, "one authentic claim for {name}");
        let entry = entries[0];
        let id: CheckedNodeId =
            serde_json::from_value(entry["node_id"].clone()).expect("selected clause");
        let claim = CheckedOccurrence {
            role: CheckedOccurrenceRole::Claim,
            ordinal: entry["ordinal"].as_u64().expect("claim ordinal"),
        };
        assert!(package
            .graph()
            .nodes
            .iter()
            .find(|node| node.node_id == id)
            .expect("retained node")
            .occurrences
            .contains(&claim));
        selections.push((id, claim, expected));
    }
    // The host validates these exact fresh bodies against QSpec by reference.
    println!("{}", serde_json::to_string(&wire).unwrap());
    drop(compiled);
    drop(wire);
    drop(bindings);
    drop(evidence);
    drop(document);
    drop(source);
    for (id, claim, expected) in selections {
        assert_eq!(consume(&package, &id, (&id, &claim)).unwrap(), expected);
        assert_eq!(consume(&package, &id, (&id, &claim)).unwrap(), expected);
    }
}
