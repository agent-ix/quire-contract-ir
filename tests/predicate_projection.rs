#[path = "support/v2_handoff.rs"]
mod v2_handoff;

use ix_trace_rs::trace;
use quire_contract_ir::bridge::{BridgeDigest, BridgeErrorCode, BridgeLimits, ContractSelection};
use quire_contract_ir::predicate::{
    project, read_projection, ExpectedProjection, PredicateCause, PredicateCauseCode,
    PredicateCauseDimension, PredicateDecision, PredicateProjectionKind, TargetSelection,
};
use quire_spec_language::protocol_artifact::{checked_predicate, v2, wire as w};

fn checked_predicates() -> Vec<checked_predicate::ValidatedCheckedPredicate> {
    let loaded = v2_handoff::Handoff::load().expect("published v2 handoff");
    let declarations = loaded.declarations();
    let inventories = loaded.inventories(&declarations);
    let expected = loaded.expected(&inventories);
    let admitted = quire_protocol::admit_and_link_v2(loaded.offer(), &expected, loaded.limits())
        .into_result()
        .expect("strict v2 package");
    checked_leaves(&admitted)
        .into_iter()
        .map(|(declaration, leaf)| {
            let selection = checked_predicate::ClauseSelection::new(declaration, leaf);
            let document = checked_predicate::derive(
                &admitted,
                selection.clone(),
                checked_predicate::Limits::default(),
            )
            .into_result()
            .expect("checked-predicate derivation");
            checked_predicate::read(
                document.bytes(),
                &admitted,
                selection,
                checked_predicate::Limits::default(),
            )
            .into_result()
            .expect("checked-predicate strict reader")
        })
        .collect()
}

fn checked_leaves(package: &v2::AdmittedPackage) -> Vec<(u32, w::Handle)> {
    let mut leaves = Vec::new();
    for (index, declaration) in package.inherited().declarations.iter().enumerate() {
        let declaration_index = u32::try_from(index).expect("fixture declaration index");
        match &declaration.body {
            w::Body::Predicate { root, .. } | w::Body::State { root, .. } => {
                leaves.push((declaration_index, root.clone()));
            }
            w::Body::Temporal { .. } => {
                leaves.extend(declaration.temporal.iter().filter_map(|node| {
                    if let w::TemporalOperation::Holds { value } = &node.operation {
                        Some((declaration_index, value.clone()))
                    } else {
                        None
                    }
                }));
            }
            w::Body::Protocol {
                controls, finish, ..
            } => {
                leaves.extend(controls.iter().filter_map(|control| {
                    if let w::ControlOperation::Check { value, .. } = &control.operation {
                        Some((declaration_index, value.clone()))
                    } else {
                        None
                    }
                }));
                leaves.push((declaration_index, finish.constraint.clone()));
            }
        }
    }
    leaves
}

#[trace("TC-038", "FR-025-AC-1", "FR-025-AC-2", "FR-025-AC-8")]
#[test]
fn tc_038_real_owner_projection_is_bijective_deterministic_and_strict_readable() {
    let predicates = checked_predicates();
    assert!(
        predicates.len() >= 2,
        "fixture must expose multiple checked leaves"
    );
    let target = TargetSelection::current();
    let projected = project(&predicates, target.clone(), BridgeLimits::default())
        .expect("owner checked predicates must project");
    assert_eq!(
        projected.decision().kind(),
        PredicateProjectionKind::Admitted
    );
    assert!(projected.decision().causes().is_empty());
    assert_eq!(
        projected
            .decision()
            .correspondences()
            .iter()
            .map(|value| value.predicate_ref().to_string())
            .collect::<Vec<_>>(),
        [
            "04fd63a816df34198f089601fc4b2ae2e15af76eb48d39cdd73208987536eb0a",
            "0ff5a3025b98ae5e669dd2cd7301f65329ae6bdd1818bf619fa7be7b6bc0177e",
            "3875ee8e4166e24707b0fa0429d6ceef6f9ec65340e61b276dbd03041209e8df",
            "3d5b7ec1edc684e7cf49c88efae8ca10ee5119cad4a7dc57cb1c9cbdccddee2a",
            "633ae632bd9d8c63e568ad5bd663a93a45d9690bbf20849174dba078f23f6caf",
            "cb18c3857c438268b0d78a7b9c17d71466d87fc42ea8593d4daa4b40ca07f2aa",
            "ce606b129fb4bae82945528ee0d62eaea2339c8d6926373d16f9edc4a149ddcc",
            "d7037367a0b4ac974accbf39271e6f900897c163e364e9a710c1fc4d463a84f9",
            "d8bce5911d67a26b837c60f11ad5e61b2bbc38cd313a3854482e62b9c24ba2be",
            "fa98fda6b7b6e816be9901c2b8b13486f09cfca2923d80e163b8962a3aafe68c",
        ]
    );
    assert_eq!(
        projected
            .decision()
            .signal_catalog_ref()
            .expect("signal ref")
            .to_string(),
        "e9c7360b2b317b5c241fd5c0a0183f6cad444ef5dbfea0749af5910b75794ad4"
    );
    assert_eq!(
        projected
            .decision()
            .proposition_map_ref()
            .expect("map ref")
            .to_string(),
        "56cf8b003df542e7ba1c755076a880676914dc965c45b4355d08c0792758ccc9"
    );
    assert_eq!(
        projected
            .decision()
            .projection_ref()
            .expect("projection ref")
            .to_string(),
        "039297db0d590cba792aaf0f617d0e139a8721e8eb019c5e603861a736a4453d"
    );

    let catalog = tl_syntax::SignalCatalogDocument::from_json_bytes(
        projected.signal_catalog_bytes(),
        tl_syntax::SyntaxArtifactLimits::OWNER_MAXIMA,
    )
    .expect("real TL catalog reader");
    let map = tl_syntax::PropositionMapDocument::from_json_bytes(
        projected.proposition_map_bytes(),
        tl_syntax::SyntaxArtifactLimits::OWNER_MAXIMA,
    )
    .expect("real TL proposition-map reader");
    let validated_catalog = catalog.validate().expect("catalog semantic validation");
    assert_eq!(validated_catalog.signal_count(), predicates.len());
    assert_eq!(validated_catalog.bindings().len(), predicates.len());
    assert_eq!(map.propositions().len(), predicates.len());
    for (ordinal, correspondence) in projected.decision().correspondences().iter().enumerate() {
        let expected_id = u32::try_from(ordinal).expect("bounded fixture ordinal");
        assert_eq!(correspondence.proposition_id(), expected_id);
        assert_eq!(correspondence.signal_id(), expected_id);
        assert_eq!(
            correspondence.proposition_name(),
            correspondence.signal_name()
        );
        assert_eq!(
            correspondence.signal_name(),
            format!("quire-predicate/{}", correspondence.predicate_ref())
        );
    }

    let expected = ExpectedProjection::new(
        &predicates,
        &target,
        projected.signal_catalog_bytes(),
        projected.proposition_map_bytes(),
    );
    let read = read_projection(
        projected.decision().bytes(),
        expected,
        BridgeLimits::default(),
    )
    .expect("strict projection re-reader");
    assert_eq!(&read, projected.validated());

    let mut reversed = predicates;
    reversed.reverse();
    let reordered = project(&reversed, target, BridgeLimits::default())
        .expect("permuted owner inputs must project");
    assert_eq!(projected.decision().bytes(), reordered.decision().bytes());
    assert_eq!(
        projected.signal_catalog_bytes(),
        reordered.signal_catalog_bytes()
    );
    assert_eq!(
        projected.proposition_map_bytes(),
        reordered.proposition_map_bytes()
    );
}

#[trace("TC-038", "FR-025-AC-6")]
#[test]
fn tc_038_projection_rejects_duplicates_bounds_and_wrong_contracts_without_artifacts() {
    let predicates = checked_predicates();
    let one = &predicates[..1];
    // Constructor-private owner views are not Clone, so duplicate exact bytes are
    // independently admitted through the real owner reader in the fixture path.
    let first_population = checked_predicates();
    let second_population = checked_predicates();
    let duplicate_owned = vec![
        first_population.into_iter().next().expect("fixture leaf"),
        second_population.into_iter().next().expect("fixture leaf"),
    ];
    let error = project(
        &duplicate_owned,
        TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect_err("duplicate predicate identity must fail");
    let semantic = error.semantic().expect("typed semantic duplicate refusal");
    assert_eq!(semantic.kind(), PredicateProjectionKind::Refused);
    assert_eq!(
        semantic.causes()[0].code(),
        PredicateCauseCode::PopulationInvalid
    );
    assert!(semantic.correspondences().is_empty());
    let limits = BridgeLimits {
        predicates: 0,
        ..BridgeLimits::default()
    };
    let error = project(one, TargetSelection::current(), limits)
        .expect_err("one-over predicate limit must fail");
    assert_eq!(
        error
            .semantic()
            .expect("bounded semantic decision")
            .causes()[0]
            .code(),
        PredicateCauseCode::PopulationInvalid
    );

    let current = TargetSelection::current();
    let wrong_native = ContractSelection::new(
        current.native().contract(),
        current.native().package_version(),
        current.native().repository(),
        "0000000000000000000000000000000000000000",
        current.native().schema_digest(),
    );
    let wrong = TargetSelection::new(
        wrong_native,
        current.signal_catalog().clone(),
        current.proposition_map().clone(),
    );
    let error = project(one, wrong, BridgeLimits::default())
        .expect_err("wrong exact owner revision must fail");
    assert_eq!(
        error.semantic().expect("typed contract refusal").causes()[0].code(),
        PredicateCauseCode::NativeProfileUnsupported
    );

    let unavailable = TargetSelection::new(
        ContractSelection::new(
            "",
            current.native().package_version(),
            current.native().repository(),
            "",
            current.native().schema_digest(),
        ),
        current.signal_catalog().clone(),
        current.proposition_map().clone(),
    );
    let error = project(one, unavailable, BridgeLimits::default())
        .expect_err("absent native selection must be unavailable");
    assert_eq!(
        error.semantic().expect("typed unavailable decision").kind(),
        PredicateProjectionKind::Unavailable
    );
    assert_eq!(
        error
            .semantic()
            .expect("typed unavailable decision")
            .causes()[0]
            .code(),
        PredicateCauseCode::NativeContractUnavailable
    );

    let conflict = TargetSelection::new(
        ContractSelection::new(
            current.native().contract(),
            current.native().package_version(),
            current.native().repository(),
            current.native().revision(),
            BridgeDigest::raw(b"conflicting schema bytes"),
        ),
        current.signal_catalog().clone(),
        current.proposition_map().clone(),
    );
    let error = project(one, conflict, BridgeLimits::default())
        .expect_err("same owner identity with unequal schema must conflict");
    assert_eq!(
        error.semantic().expect("typed conflict decision").kind(),
        PredicateProjectionKind::Conflict
    );
    assert_eq!(
        error.semantic().expect("typed conflict decision").causes()[0].code(),
        PredicateCauseCode::NativeContractConflict
    );
}

#[trace("TC-038", "FR-025-AC-6", "FR-025-AC-8")]
#[test]
fn tc_038_projection_reader_rejects_noncanonical_unknown_and_trailing_bytes() {
    let predicates = checked_predicates();
    let target = TargetSelection::current();
    let projected = project(&predicates[..1], target.clone(), BridgeLimits::default())
        .expect("one checked predicate");
    let expected = || {
        ExpectedProjection::new(
            &predicates[..1],
            &target,
            projected.signal_catalog_bytes(),
            projected.proposition_map_bytes(),
        )
    };

    let mut trailing = projected.decision().bytes().to_vec();
    trailing.extend_from_slice(b"\n");
    let error = read_projection(&trailing, expected(), BridgeLimits::default())
        .expect_err("trailing data must fail");
    assert_eq!(
        error.operation().expect("structural error").code(),
        BridgeErrorCode::InvalidNativePredicateProjection
    );

    let mut value: serde_json::Value =
        serde_json::from_slice(projected.decision().bytes()).expect("decision JSON");
    value
        .as_object_mut()
        .expect("decision object")
        .insert("unknown".to_owned(), serde_json::Value::Bool(true));
    let unknown = serde_json::to_vec(&value).expect("mutated JSON");
    let error = read_projection(&unknown, expected(), BridgeLimits::default())
        .expect_err("unknown field must fail");
    assert_eq!(
        error.operation().expect("structural error").code(),
        BridgeErrorCode::InvalidNativePredicateProjection
    );

    let pretty = serde_json::to_vec_pretty(&value).expect("noncanonical JSON");
    assert!(matches!(
        read_projection(&pretty, expected(), BridgeLimits::default()),
        Err(PredicateDecision::Operation(_))
    ));

    let duplicate = format!(
        "{{\"format\":\"{}\",{}",
        quire_contract_ir::predicate::PROJECTION_DECISION_PROFILE,
        std::str::from_utf8(&projected.decision().bytes()[1..]).expect("decision UTF-8")
    );
    assert!(matches!(
        read_projection(duplicate.as_bytes(), expected(), BridgeLimits::default()),
        Err(PredicateDecision::Operation(_))
    ));
}

#[trace("TC-038", "FR-025-AC-1", "FR-025-AC-2")]
#[test]
fn tc_038_owner_schema_and_selection_digests_are_exact() {
    let current = TargetSelection::current();
    assert_eq!(
        current.native().revision(),
        "f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07"
    );
    assert_eq!(
        current.signal_catalog().revision(),
        "842d82553f045eb69a7f38745756d968254fc25e"
    );
    assert_eq!(
        current.proposition_map().revision(),
        "842d82553f045eb69a7f38745756d968254fc25e"
    );
    assert_eq!(
        current.native().schema_digest(),
        BridgeDigest::parse(checked_predicate::SCHEMA_SHA256).expect("QSL schema digest")
    );
    assert_eq!(
        current.signal_catalog().schema_digest(),
        BridgeDigest::parse(tl_syntax::SIGNAL_CATALOG_V1_SCHEMA_SHA256)
            .expect("signal schema digest")
    );
    assert_eq!(
        current.proposition_map().schema_digest(),
        BridgeDigest::parse(tl_syntax::PROPOSITION_MAP_V1_SCHEMA_SHA256)
            .expect("map schema digest")
    );
}

#[trace("TC-038", "FR-025-AC-4", "STD-001")]
#[test]
fn tc_038_closed_cause_catalog_has_one_dimension_and_unique_label_per_code() {
    let mut labels = std::collections::BTreeSet::new();
    for code in PredicateCauseCode::ALL {
        assert!(labels.insert(code.as_str()), "duplicate cause label");
        let assignments: Vec<_> = PredicateCauseDimension::ALL
            .into_iter()
            .filter_map(|dimension| PredicateCause::new(dimension, code, "fixture", None))
            .collect();
        assert_eq!(assignments.len(), 1, "{code:?} must have one dimension");
        assert_eq!(assignments[0].code(), code);
    }
}

#[trace("TC-038", "FR-025-AC-6")]
#[test]
fn tc_038_projection_resource_failpoints_emit_no_partial_artifact() {
    let predicates = checked_predicates();
    let allocation_short = BridgeLimits {
        allocation_bytes: 511,
        ..BridgeLimits::default()
    };
    let error = project(
        &predicates[..1],
        TargetSelection::current(),
        allocation_short,
    )
    .expect_err("deterministic projection allocation failpoint");
    assert_eq!(
        error.operation().expect("resource operation").code(),
        BridgeErrorCode::PredicateProjectionResourceExhausted
    );

    // The owner input may be larger than the decision, so its exact byte limit
    // governs projection admission before output is considered.
    let owner_exact = BridgeLimits {
        document_bytes: predicates[0].document().bytes().len(),
        ..BridgeLimits::default()
    };
    assert!(project(&predicates[..1], TargetSelection::current(), owner_exact).is_ok());
    let owner_short = BridgeLimits {
        document_bytes: predicates[0].document().bytes().len() - 1,
        ..BridgeLimits::default()
    };
    assert_eq!(
        project(&predicates[..1], TargetSelection::current(), owner_short)
            .expect_err("one-short owner document limit")
            .operation()
            .expect("resource operation")
            .code(),
        BridgeErrorCode::PredicateProjectionResourceExhausted
    );
}

#[trace("TC-038", "FR-025-AC-1", "FR-025-AC-6")]
#[test]
fn tc_038_target_contract_slots_cannot_be_cross_wired() {
    let predicates = checked_predicates();
    let current = TargetSelection::current();
    let swapped = TargetSelection::new(
        current.native().clone(),
        current.proposition_map().clone(),
        current.signal_catalog().clone(),
    );
    let error = project(&predicates[..1], swapped, BridgeLimits::default())
        .expect_err("target owner slots are not interchangeable");
    let semantic = error.semantic().expect("typed target refusal");
    assert!(semantic
        .causes()
        .iter()
        .all(|cause| cause.code() == PredicateCauseCode::TargetProfileUnsupported));
}
