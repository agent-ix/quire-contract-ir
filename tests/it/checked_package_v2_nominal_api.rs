// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! External construction and reader admission of closed V2 nominal bodies.

use crate::support::checked_package::{canonical, evidence_for, v2_nominal};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedRational, CheckedTerm, DimensionTerm, NominalBody,
};
use serde_json::{json, Value};

fn node_id(value: &Value) -> CheckedNodeId {
    serde_json::from_value(value.clone()).expect("fixture structural node id")
}

fn rational(value: &Value) -> CheckedRational {
    serde_json::from_value(value.clone()).expect("fixture rational")
}

fn constructor(body: &Value) -> NominalBody {
    match body["term"].as_str().expect("fixture nominal term") {
        "enum" => NominalBody::try_enum(
            body["ordered"].as_bool().expect("ordered flag"),
            body["members"]
                .as_array()
                .expect("enum members")
                .iter()
                .map(|member| member.as_str().expect("case name").into())
                .collect(),
        ),
        "dimension" => NominalBody::try_dimension(
            body["terms"]
                .as_array()
                .expect("dimension terms")
                .iter()
                .map(|term| {
                    serde_json::from_value::<DimensionTerm>(term.clone()).expect("dimension term")
                })
                .collect(),
        ),
        "unit" => NominalBody::try_unit(
            node_id(&body["dimension_node_id"]),
            body["target_unit_node_id"]
                .as_object()
                .map(|_| node_id(&body["target_unit_node_id"])),
            rational(&body["scale"]),
            rational(&body["offset"]),
        ),
        "dimension_equivalence" => NominalBody::try_dimension_equivalence(
            [
                node_id(&body["dimension_node_ids"][0]),
                node_id(&body["dimension_node_ids"][1]),
            ],
            rational(&body["root_scale"]),
        ),
        other => panic!("unexpected nominal body {other}"),
    }
    .expect("fixture body shape")
}

#[trace("TC-048", "TC-058", "FR-038-AC-374")]
#[test]
fn public_constructors_round_trip_all_four_nominal_forms_through_reader() {
    let wire = v2_nominal();
    let nodes = wire["semantic_graph"]["nodes"].as_array().expect("nodes");
    let mut seen = std::collections::BTreeSet::new();
    let constructed = nodes
        .iter()
        .enumerate()
        .filter_map(|(position, node)| {
            let body = &node["body"];
            let term = body["term"].as_str()?;
            if !["enum", "dimension", "unit", "dimension_equivalence"].contains(&term) {
                return None;
            }
            seen.insert(term);
            let typed = CheckedTerm::NominalBody(Box::new(constructor(body)));
            assert_eq!(serde_json::to_value(&typed).expect("closed body"), *body);
            Some((position, typed))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        seen,
        ["dimension", "dimension_equivalence", "enum", "unit"].into()
    );
    let admitted = match CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("nominal fixture must admit: {other:?}"),
    };
    for (position, expected) in constructed {
        assert_eq!(&admitted.graph().nodes[position].body, &expected);
    }
}

fn assert_malformed(
    result: Result<NominalBody, quire_contract_model::CheckedPackageRefusal>,
    at: &str,
) {
    let refusal = result.expect_err("malformed nominal body must refuse");
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(at)
    );
}

#[trace("TC-048", "TC-058", "FR-038-AC-374")]
#[test]
fn public_constructors_refuse_malformed_members_at_body_relative_pointers() {
    assert_malformed(NominalBody::try_enum(false, vec![]), "/members");
    assert_malformed(
        NominalBody::try_enum(false, vec!["Case".into(), "Case".into()]),
        "/members/1",
    );
    assert_malformed(
        NominalBody::try_enum(false, vec!["bad-case".into()]),
        "/members/0",
    );

    let id = node_id(&json!({"package":"units","path":["Time"],"ordinal":0}));
    assert_malformed(
        NominalBody::try_dimension(vec![DimensionTerm {
            dimension_node_id: id.clone(),
            exponent: "01".into(),
        }]),
        "/terms/0/exponent",
    );
    let invalid_id = CheckedNodeId {
        package: "".into(),
        path: id.path.clone(),
        ordinal: 0,
    };
    assert_malformed(
        NominalBody::try_dimension(vec![DimensionTerm {
            dimension_node_id: invalid_id,
            exponent: "1".into(),
        }]),
        "/terms/0/dimension_node_id",
    );
    let one = CheckedRational {
        numerator: "1".into(),
        denominator: "1".into(),
    };
    let zero = CheckedRational {
        numerator: "0".into(),
        denominator: "1".into(),
    };
    assert_malformed(
        NominalBody::try_unit(
            id.clone(),
            None,
            CheckedRational {
                numerator: "1.5".into(),
                denominator: "1".into(),
            },
            zero.clone(),
        ),
        "/scale/numerator",
    );
    assert_malformed(
        NominalBody::try_unit(
            id.clone(),
            None,
            CheckedRational {
                numerator: "1".into(),
                denominator: "0".into(),
            },
            zero,
        ),
        "/scale/denominator",
    );
    for numerator in ["0", "-1"] {
        assert_malformed(
            NominalBody::try_dimension_equivalence(
                [id.clone(), id.clone()],
                CheckedRational {
                    numerator: numerator.into(),
                    denominator: "1".into(),
                },
            ),
            "/root_scale/numerator",
        );
    }
    assert!(NominalBody::try_dimension_equivalence([id.clone(), id], one).is_ok());
}
