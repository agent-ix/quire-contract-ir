use ix_trace_rs::trace;
use quire_contract_ir::kani::{
    lower_query, CapabilityDisposition, CapabilityEntry, CollectionQuery, DispatchIndex,
    FiniteInput, FiniteObject, KaniOutcomeKind, KaniProfile, ModuleDescriptor,
    PopulationCompleteness, ProfileSelection, QueryKind, ResourceBounds, SemanticFamily, PROFILE,
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
            max_objects: 1,
            max_references: 0,
            max_input_bytes: 10,
        },
        input_bytes: 1,
        objects: vec![FiniteObject {
            identity: "self".into(),
            type_id: "Scalar".into(),
            snapshot_id: "s".into(),
        }],
        references: vec![],
    }
    .validate()
    .expect("input")
}
fn profile() -> KaniProfile {
    KaniProfile::new(
        selection(),
        vec![CapabilityEntry {
            construct: "bounded-collection-query".into(),
            disposition: CapabilityDisposition::Supported {
                module: "kani-collections-queries/1".into(),
            },
        }],
    )
    .expect("profile")
}
fn dispatch() -> DispatchIndex {
    DispatchIndex::new(vec![ModuleDescriptor {
        module_id: "kani-collections-queries/1".into(),
        family: SemanticFamily::CollectionsQueries,
        abi_revision: "kani-abi/1".into(),
        constructs: vec!["bounded-collection-query".into()],
    }])
    .expect("dispatch")
}
#[trace("TC-042", "FR-030-AC-1", "FR-031-AC-1")]
#[test]
fn tc_042_collection_queries_preserve_order_duplicates_and_short_circuit() {
    let result = lower_query(
        &profile(),
        &dispatch(),
        &input(),
        CollectionQuery {
            source_id: "clause".into(),
            values: vec![4, 4, -1, 4],
            max_items: 4,
            kind: QueryKind::ForAllNonNegative,
        },
    )
    .expect("query");
    assert!(!result.value);
    assert_eq!(result.examined, 3);
    let empty = lower_query(
        &profile(),
        &dispatch(),
        &input(),
        CollectionQuery {
            source_id: "clause".into(),
            values: vec![],
            max_items: 0,
            kind: QueryKind::ExistsEqual(4),
        },
    )
    .expect("empty query");
    assert!(!empty.value);
}
#[trace("TC-042", "FR-030-AC-2")]
#[test]
fn tc_042_collection_bound_exhaustion_is_non_boolean() {
    let failure = lower_query(
        &profile(),
        &dispatch(),
        &input(),
        CollectionQuery {
            source_id: "clause".into(),
            values: vec![1, 2],
            max_items: 1,
            kind: QueryKind::ExistsEqual(2),
        },
    )
    .expect_err("bounded collection");
    assert_eq!(failure.kind, KaniOutcomeKind::ResourceExhausted);
    assert_eq!(failure.boolean_claim(), None);
}
