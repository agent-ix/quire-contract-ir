// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-048: QSpec's published FMA application cases at the strict reader boundary.

use super::{cause_word, code_word, fixture_support, fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{
    ApplicationOperator, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedTerm,
};
use std::collections::BTreeSet;

#[path = "../it/support/qspec_fma_inputs.rs"]
mod inputs;
use inputs::{literal_id, operand_type_id, package_for};

/// Trace: FR-038-AC-431, FR-038-AC-432; QSpec FR-322-AC-307.
#[trace("TC-048", "FR-038-AC-431", "FR-038-AC-432")]
#[test]
fn tc_048_fma_applications_follow_published_qspec_cases() {
    let vectors = read_json(&proposal_dir_of(&fixtures_dir()).join("fma-application-vectors.json"))
        .expect("published QSpec FMA application vectors");
    let cases = vectors["cases"].as_array().expect("published cases");
    let mut seen = BTreeSet::new();
    for case in cases {
        let id = case["id"].as_str().expect("case id");
        assert!(seen.insert(id), "duplicate {id}");
        let (package, position) = package_for(case);
        let result = CheckedPackageV2::read(
            &fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &fixture_support::evidence_for(&package),
        );
        match case["expected"].as_str().expect("expected result") {
            "admitted" => {
                let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
                    panic!("{id}: expected admission, got {result:?}");
                };
                let node = &admitted.resolved_graph().nodes[position];
                let CheckedTerm::Application {
                    operator,
                    operation,
                    result_type,
                    arguments,
                } = &node.body
                else {
                    panic!("{id}: admitted node is not an application");
                };
                assert_eq!(operation.identity(), case["identity"].as_str(), "{id}");
                assert_eq!(*operator, ApplicationOperator::Call, "{id}: generic call");
                let read = operation.read().expect("admitted FMA operation");
                assert_eq!(
                    read.mode.map(|mode| (mode.kind.as_wire(), mode.value)),
                    Some(("rounding", "nearest-even")),
                    "{id}: rounding mode"
                );
                assert_eq!(read.laws.len(), 1, "{id}: selected IEEE profile");
                assert_eq!(read.laws[0].role.as_wire(), "ieee_profile", "{id}");
                assert!(read.member.is_none(), "{id}: no member");
                assert!(read.leaves.is_empty(), "{id}: no leaves");
                let width = case["result_width"].as_str().expect("result width");
                assert_eq!(
                    result_type,
                    &fixture_support::structural_fixture_id(&operand_type_id(width)),
                    "{id}: result type"
                );
                assert_eq!(arguments.len(), 3, "{id}: a, b, c");
                for (index, argument) in arguments.iter().enumerate() {
                    let width = case["argument_widths"][index].as_str().expect("width");
                    let expected =
                        fixture_support::structural_fixture_id(&literal_id(width, index));
                    assert!(
                        matches!(argument, CheckedTerm::Reference { target } if target == &expected),
                        "{id}: argument {index} references its operand"
                    );
                }
            }
            "refused" => {
                let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                    panic!("{id}: expected refusal, got {result:?}");
                };
                assert_eq!(
                    code_word(refusal.code),
                    case["code"].as_str().unwrap(),
                    "{id}: code"
                );
                assert_eq!(
                    refusal.cause.map(cause_word),
                    case["cause"].as_str(),
                    "{id}: cause"
                );
            }
            other => panic!("{id}: unknown expected result {other}"),
        }
    }
    let expected = (1..=12)
        .map(|index| format!("FM-{index:02}"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        seen,
        expected.iter().map(String::as_str).collect(),
        "exact published FM set"
    );
}
