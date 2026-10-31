// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! External test consumer for the feature-gated operation fixture surface.

#![cfg(feature = "test-support")]

use quire_contract_model::{
    CheckedApplicationOperation, CheckedApplicationOperationFixture,
    CheckedApplicationOperationFixtureError as FixtureError, CheckedArtifactRef, CheckedNodeId,
    CheckedNodePathSegment, OperationModeKind,
};

fn definition(identity: &str) -> CheckedArtifactRef {
    CheckedArtifactRef {
        authority: "quire".into(),
        identity: identity.into(),
    }
}

fn node_key() -> CheckedNodeId {
    CheckedNodeId {
        package: "fixture-package".into(),
        path: vec![CheckedNodePathSegment::Name("declaration".into())],
        ordinal: 0,
    }
}

fn check_fixture(fixture: CheckedApplicationOperationFixture) {
    let expected = fixture.clone();
    let operation = CheckedApplicationOperation::from_test_fixture(fixture).unwrap();
    assert!(operation.is_test_fixture());
    assert_eq!(operation.identity(), expected.identity.as_deref());
    assert_eq!(
        operation
            .mode()
            .expect("fixture views")
            .map(|mode| (mode.kind, mode.value)),
        expected
            .mode
            .as_ref()
            .map(|(kind, value)| (*kind, value.as_ref()))
    );
    assert_eq!(
        operation.law_definitions().expect("fixture views"),
        expected.law_definitions
    );
    assert_eq!(
        operation.member_declaration().expect("fixture views"),
        expected.member_declaration.as_ref()
    );
}

/// Trace: FR-019-AC-14, TC-058.
#[test]
fn typed_controls_preserve_all_four_scalar_decision_inputs() {
    check_fixture(CheckedApplicationOperationFixture::default());
    for identity in [
        None,
        Some("quire.op.rational.eq"),
        Some("quire.op.quantity.add"),
        Some("quire.op.quantity.convert"),
    ] {
        for value in ["exact", "nearest-even", "toward-zero"] {
            check_fixture(CheckedApplicationOperationFixture {
                identity: identity.map(Into::into),
                mode: Some((OperationModeKind::Rounding, value.into())),
                member_declaration: Some(node_key()),
                ..Default::default()
            });
        }
    }
    for identities in [
        vec![],
        vec!["quire.value.other/v1"],
        vec![
            "quire.value.integer-division.floor/v1",
            "quire.value.integer-division.euclidean/v1",
        ],
        vec!["quire.value.integer-division.truncating/v1"],
        vec!["quire.value.integer-division.floor/v1"],
        vec!["quire.value.integer-division.euclidean/v1"],
    ] {
        check_fixture(CheckedApplicationOperationFixture {
            identity: Some("quire.op.integer.div".into()),
            law_definitions: identities.into_iter().map(definition).collect(),
            ..Default::default()
        });
    }
    for mode in [
        None,
        Some((OperationModeKind::TextProfile, "nearest-even".into())),
        Some((OperationModeKind::Rounding, "toward-zero".into())),
        Some((OperationModeKind::Rounding, "nearest-even".into())),
    ] {
        check_fixture(CheckedApplicationOperationFixture {
            identity: Some("quire.op.decimal.add".into()),
            mode,
            ..Default::default()
        });
    }
}

/// Trace: FR-019-AC-15, TC-058.
#[test]
fn malformed_present_fields_are_typed_refusals() {
    let refuse = |fixture, expected| {
        assert_eq!(
            CheckedApplicationOperation::from_test_fixture(fixture),
            Err(expected)
        );
    };
    refuse(
        CheckedApplicationOperationFixture {
            identity: Some("".into()),
            ..Default::default()
        },
        FixtureError::EmptyIdentity,
    );
    refuse(
        CheckedApplicationOperationFixture {
            mode: Some((OperationModeKind::Rounding, "".into())),
            ..Default::default()
        },
        FixtureError::EmptyModeValue,
    );
    for (definition, expected) in [
        (
            CheckedArtifactRef {
                authority: "".into(),
                identity: "a".into(),
            },
            FixtureError::EmptyLawAuthority,
        ),
        (
            CheckedArtifactRef {
                authority: "a".into(),
                identity: "".into(),
            },
            FixtureError::EmptyLawIdentity,
        ),
    ] {
        refuse(
            CheckedApplicationOperationFixture {
                law_definitions: vec![definition],
                ..Default::default()
            },
            expected,
        );
    }
    let mut empty_package = node_key();
    empty_package.package = "".into();
    let mut empty_path = node_key();
    empty_path.path.clear();
    let mut oversized_ordinal = node_key();
    oversized_ordinal.ordinal = 9_007_199_254_740_992;
    for key in [empty_package, empty_path, oversized_ordinal] {
        refuse(
            CheckedApplicationOperationFixture {
                member_declaration: Some(key),
                ..Default::default()
            },
            FixtureError::InvalidMemberNodeId,
        );
    }
}

/// Trace: FR-019-AC-15, TC-058.
#[test]
fn fixture_provenance_is_private_and_does_not_survive_wire_roundtrip() {
    let fixture = CheckedApplicationOperation::from_test_fixture(Default::default()).unwrap();
    assert!(fixture.is_test_fixture());
    let bytes = serde_json::to_vec(&fixture).unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire.get("test_fixture").is_none());
    let deserialized: CheckedApplicationOperation = serde_json::from_slice(&bytes).unwrap();
    assert!(!deserialized.is_test_fixture());
}
