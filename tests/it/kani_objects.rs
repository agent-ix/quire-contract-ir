use ix_trace_rs::trace;
use quire_contract_ir::kani::{
    lower_reaches, CapabilityDisposition, CapabilityEntry, DispatchIndex, FiniteInput,
    FiniteObject, FiniteReference, GraphRequest, KaniOutcomeKind, KaniProfile, ModuleDescriptor,
    PopulationCompleteness, ProfileSelection, ResourceBounds, SemanticFamily, PROFILE,
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
fn input() -> quire_contract_ir::kani::ValidatedFiniteInput {
    FiniteInput {
        model_id: "model".into(),
        source_id: "clause".into(),
        profile: selection(),
        completeness: PopulationCompleteness::Complete,
        bounds: ResourceBounds {
            max_objects: 3,
            max_references: 3,
            max_input_bytes: 20,
        },
        input_bytes: 1,
        objects: ["a", "b", "c"]
            .into_iter()
            .map(|identity| FiniteObject {
                identity: identity.into(),
                type_id: "Node".into(),
                snapshot_id: "s".into(),
            })
            .collect(),
        references: vec![
            FiniteReference {
                source_id: "a".into(),
                field_id: "parent".into(),
                target_id: "b".into(),
            },
            FiniteReference {
                source_id: "b".into(),
                field_id: "parent".into(),
                target_id: "a".into(),
            },
        ],
    }
    .validate()
    .expect("input")
}
fn profile() -> KaniProfile {
    KaniProfile::new(
        selection(),
        vec![CapabilityEntry {
            construct: "finite-reference-graph".into(),
            disposition: CapabilityDisposition::Supported {
                module: "kani-objects-graphs/1".into(),
            },
        }],
    )
    .expect("profile")
}
fn dispatch() -> DispatchIndex {
    DispatchIndex::new(vec![ModuleDescriptor {
        module_id: "kani-objects-graphs/1".into(),
        family: SemanticFamily::ObjectsReferencesGraphs,
        abi_revision: "kani-abi/1".into(),
        constructs: vec!["finite-reference-graph".into()],
    }])
    .expect("dispatch")
}

#[trace("TC-042", "FR-030-AC-1", "FR-031-AC-1")]
#[test]
fn tc_042_graph_lowering_preserves_identity_and_cycle_semantics() {
    let cycle = lower_reaches(
        &profile(),
        &dispatch(),
        &input(),
        GraphRequest {
            source_id: "clause".into(),
            start_id: "a".into(),
            target_id: "a".into(),
            field_id: "parent".into(),
            max_expansions: 3,
        },
    )
    .expect("cycle is finite");
    assert!(cycle.reachable, "a->b->a is positive-length reachability");
    assert_eq!(cycle.expanded, vec!["a", "b"]);
}

#[trace("TC-042", "FR-030-AC-2")]
#[test]
fn tc_042_graph_lowering_retains_exhaustion_as_non_boolean() {
    let exhausted = lower_reaches(
        &profile(),
        &dispatch(),
        &input(),
        GraphRequest {
            source_id: "clause".into(),
            start_id: "a".into(),
            target_id: "c".into(),
            field_id: "parent".into(),
            max_expansions: 1,
        },
    )
    .expect_err("graph budget exhausts");
    assert_eq!(exhausted.kind, KaniOutcomeKind::ResourceExhausted);
    assert_eq!(exhausted.boolean_claim(), None);
}
