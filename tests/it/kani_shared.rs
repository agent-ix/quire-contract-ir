use ix_trace_rs::trace;
use quire_contract_ir::kani::{
    CapabilityDisposition, CapabilityEntry, DispatchError, DispatchIndex, FiniteInput,
    FiniteObject, FiniteReference, GeneratorProvenance, KaniOutcome, KaniOutcomeKind, KaniProfile,
    KaniProviderResult, ModuleDescriptor, PopulationCompleteness, ProfileSelection, ResourceBounds,
    SemanticFamily, PROFILE,
};

fn selection() -> ProfileSelection {
    ProfileSelection {
        profile: PROFILE.into(),
        revision: "kani-bounded/1.0.0".into(),
        executable_digest: "sha256:kani".into(),
        options_digest: "sha256:options".into(),
        abi_revision: "kani-abi/1".into(),
    }
}

fn profile() -> KaniProfile {
    KaniProfile::new(
        selection(),
        vec![
            CapabilityEntry {
                construct: "checked-add".into(),
                disposition: CapabilityDisposition::Supported {
                    module: "kani-definedness-arithmetic/1".into(),
                },
            },
            CapabilityEntry {
                construct: "object-reference".into(),
                disposition: CapabilityDisposition::Refused {
                    code: "kani_object_module_unavailable".into(),
                },
            },
        ],
    )
    .expect("valid profile")
}

fn input(completeness: PopulationCompleteness) -> FiniteInput {
    FiniteInput {
        model_id: "model:demo@1".into(),
        source_id: "clause:demo".into(),
        profile: selection(),
        completeness,
        bounds: ResourceBounds {
            max_objects: 2,
            max_references: 1,
            max_input_bytes: 100,
        },
        input_bytes: 10,
        objects: vec![
            FiniteObject {
                identity: "object:a".into(),
                type_id: "Demo::Node".into(),
                snapshot_id: "snapshot:one".into(),
            },
            FiniteObject {
                identity: "object:b".into(),
                type_id: "Demo::Node".into(),
                snapshot_id: "snapshot:one".into(),
            },
        ],
        references: vec![FiniteReference {
            source_id: "object:a".into(),
            field_id: "parent".into(),
            target_id: "object:b".into(),
        }],
    }
}

#[trace("TC-042", "FR-029-AC-1", "FR-029-AC-2", "FR-029-AC-3")]
#[test]
fn tc_042_profile_requires_one_known_entry_per_encountered_construct() {
    let classified = profile()
        .classify(
            &["checked-add".into(), "object-reference".into()],
            "clause:demo",
        )
        .expect("complete matrix selection");
    assert_eq!(classified.len(), 2);

    let refusal = profile()
        .classify(&["unbounded-quantifier".into()], "clause:demo")
        .expect_err("unknown construct refuses before lowering");
    assert_eq!(refusal.kind, KaniOutcomeKind::Refused);
    assert_eq!(refusal.code, "kani_capability_missing");
    assert_eq!(refusal.boolean_claim(), None);
}

#[trace("TC-042", "FR-030-AC-1", "FR-030-AC-2")]
#[test]
fn tc_042_invalid_and_incomplete_populations_are_not_assumed_away() {
    let incomplete = input(PopulationCompleteness::Incomplete)
        .validate()
        .expect_err("incomplete input is a result, not an assumption");
    assert_eq!(incomplete.kind, KaniOutcomeKind::IncompleteInput);
    assert_eq!(incomplete.boolean_claim(), None);

    let mut duplicate = input(PopulationCompleteness::Complete);
    duplicate.objects[1].identity = "object:a".into();
    let invalid = duplicate
        .validate()
        .expect_err("duplicate identity is invalid input");
    assert_eq!(invalid.kind, KaniOutcomeKind::InvalidInput);
    assert_eq!(invalid.boolean_claim(), None);

    let mut duplicate_reference = input(PopulationCompleteness::Complete);
    duplicate_reference
        .references
        .push(duplicate_reference.references[0].clone());
    duplicate_reference.bounds.max_references = 2;
    let invalid_reference = duplicate_reference
        .validate()
        .expect_err("duplicate reference edge is invalid input");
    assert_eq!(invalid_reference.kind, KaniOutcomeKind::InvalidInput);

    let mut scalar = input(PopulationCompleteness::Complete);
    scalar.references.clear();
    scalar.bounds.max_references = 0;
    scalar
        .validate()
        .expect("scalar domains do not require a reference capacity");
}

#[trace("TC-042", "FR-030-AC-3")]
#[test]
fn tc_042_only_proof_and_counterexample_have_boolean_claims() {
    assert_eq!(
        KaniOutcome::proved("clause:demo", "profile").boolean_claim(),
        Some(true)
    );
    assert_eq!(
        KaniOutcome::counterexample("clause:demo", "profile").boolean_claim(),
        Some(false)
    );
    for kind in [
        KaniOutcomeKind::Refused,
        KaniOutcomeKind::InvalidInput,
        KaniOutcomeKind::IncompleteInput,
        KaniOutcomeKind::Unavailable,
        KaniOutcomeKind::TimedOut,
        KaniOutcomeKind::ResourceExhausted,
        KaniOutcomeKind::Cancelled,
        KaniOutcomeKind::Inconclusive,
    ] {
        assert_eq!(
            KaniOutcome::non_success(kind, "typed", "clause:demo", "profile").boolean_claim(),
            None
        );
    }
}

#[trace("TC-042", "FR-031-AC-1")]
#[test]
fn tc_042_dispatch_rejects_cross_family_or_duplicate_ownership() {
    let module = ModuleDescriptor {
        module_id: "kani-definedness-arithmetic/1".into(),
        family: SemanticFamily::DefinednessArithmetic,
        abi_revision: "kani-abi/1".into(),
        constructs: vec!["checked-add".into()],
    };
    let index = DispatchIndex::new(vec![module.clone()]).expect("one owner");
    assert_eq!(
        index.resolve("checked-add").expect("owner").family,
        SemanticFamily::DefinednessArithmetic
    );
    assert_eq!(
        index.resolve("object-reference"),
        Err(DispatchError::Unowned("object-reference".into()))
    );

    assert!(matches!(
        DispatchIndex::new(vec![module.clone(), module]),
        Err(DispatchError::Conflict(_))
    ));
}

#[trace("TC-042", "FR-031-AC-2")]
#[test]
fn tc_042_provenance_identity_changes_with_an_assumption() {
    let base = GeneratorProvenance {
        clause_id: "clause:demo".into(),
        input_id: "input:demo".into(),
        profile_revision: "kani-bounded/1.0.0".into(),
        executable_digest: "sha256:kani".into(),
        options_digest: "sha256:options".into(),
        assumptions: vec!["finite-population-valid".into()],
        proof_dependencies: vec!["module:kani-definedness-arithmetic/1".into()],
    };
    let first = base
        .identify("kani-harness/1", b"artifact")
        .expect("complete provenance");
    let mut changed = base;
    changed.assumptions.push("checked-add-range".into());
    let second = changed
        .identify("kani-harness/1", b"artifact")
        .expect("complete provenance");
    assert_ne!(first.as_str(), second.as_str());
}

// Deliberately untraced: no acceptance criterion covers vacuous-proof
// classification yet (FR-030-AC-1..3 say nothing about it), so this test
// binds itself to no criterion rather than claim one it does not establish.
#[test]
fn a_proved_run_with_zero_success_checks_settles_inconclusive_as_vacuous() {
    let vacuous = KaniOutcome::proved_from_checks(0, "clause:demo", "profile");
    assert_eq!(vacuous.kind, KaniOutcomeKind::Inconclusive);
    assert_eq!(vacuous.code, "kani_vacuous_proof");
    assert_eq!(vacuous.boolean_claim(), None);

    let genuine = KaniOutcome::proved_from_checks(1, "clause:demo", "profile");
    assert_eq!(genuine, KaniOutcome::proved("clause:demo", "profile"));
    assert_eq!(genuine.boolean_claim(), Some(true));
}

/// The O-16 proof column, one row per kind. The exhaustive match (no
/// wildcard) makes a new `KaniOutcomeKind` a compile error here until its row
/// is written, and `every_kind` lists each variant the match names.
fn o16_row(kind: &KaniOutcomeKind) -> (KaniProviderResult, &'static str) {
    match kind {
        KaniOutcomeKind::Proved => (KaniProviderResult::Proved, "proved"),
        KaniOutcomeKind::Counterexample => (KaniProviderResult::Refuted, "refuted"),
        KaniOutcomeKind::Refused
        | KaniOutcomeKind::InvalidInput
        | KaniOutcomeKind::IncompleteInput => (KaniProviderResult::Declined, "declined"),
        KaniOutcomeKind::Unavailable => (KaniProviderResult::Unsupported, "unsupported"),
        KaniOutcomeKind::TimedOut
        | KaniOutcomeKind::ResourceExhausted
        | KaniOutcomeKind::Cancelled => (KaniProviderResult::Incomplete, "incomplete"),
        KaniOutcomeKind::Inconclusive => (KaniProviderResult::Inconclusive, "inconclusive"),
    }
}

fn every_kind() -> [KaniOutcomeKind; 10] {
    [
        KaniOutcomeKind::Proved,
        KaniOutcomeKind::Counterexample,
        KaniOutcomeKind::Refused,
        KaniOutcomeKind::InvalidInput,
        KaniOutcomeKind::IncompleteInput,
        KaniOutcomeKind::Unavailable,
        KaniOutcomeKind::TimedOut,
        KaniOutcomeKind::ResourceExhausted,
        KaniOutcomeKind::Cancelled,
        KaniOutcomeKind::Inconclusive,
    ]
}

/// Tracing: TC-223, FR-031-AC-5
#[trace("TC-223", "FR-031-AC-5")]
#[test]
fn tc_223_every_kani_outcome_kind_maps_to_its_one_fr331_result() {
    for kind in every_kind() {
        let (result, wire) = o16_row(&kind);
        assert_eq!(kind.provider_result(), result, "{kind:?}");
        assert_eq!(
            serde_json::to_value(result).expect("result"),
            serde_json::json!(wire)
        );
    }
    // The record keeps each outcome's cause, so kinds sharing a result stay
    // apart.
    let declined = [
        KaniOutcomeKind::Refused,
        KaniOutcomeKind::InvalidInput,
        KaniOutcomeKind::IncompleteInput,
    ]
    .map(|kind| {
        let code = format!("{kind:?}");
        KaniOutcome::non_success(kind, code, "source", "context").provider_record()
    });
    assert!(declined
        .iter()
        .all(|record| record.result == KaniProviderResult::Declined));
    let causes = declined
        .iter()
        .map(|record| record.cause.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(causes.len(), 3);
    // A zero-check proof built through `proved_from_checks` is already
    // `Inconclusive`, so it records `inconclusive` with its vacuity cause.
    let vacuous = KaniOutcome::proved_from_checks(0, "source", "context").provider_record();
    assert_eq!(vacuous.result, KaniProviderResult::Inconclusive);
    assert_eq!(vacuous.cause, "kani_vacuous_proof");
}
