//! Ordered duplicate-preserving bounded collection/query lowering.

use super::{
    CapabilityDisposition, DispatchIndex, KaniOutcome, KaniOutcomeKind, KaniProfile,
    SemanticFamily, ValidatedFiniteInput,
};

/// Supported query form over one ordered finite sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryKind {
    ForAllNonNegative,
    ExistsEqual(i128),
}

/// Exact selected collection query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionQuery {
    pub source_id: String,
    pub values: Vec<i128>,
    pub max_items: usize,
    pub kind: QueryKind,
}

/// Exact ordered query lowering result, not a Kani verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionLowering {
    pub query: CollectionQuery,
    pub value: bool,
    pub examined: usize,
}

/// Lowers one supported query while retaining sequence order and duplicates.
pub fn lower_query(
    profile: &KaniProfile,
    dispatch: &DispatchIndex,
    _input: &ValidatedFiniteInput,
    query: CollectionQuery,
) -> Result<CollectionLowering, KaniOutcome> {
    let entries = profile.classify(&["bounded-collection-query".to_owned()], &query.source_id)?;
    match &entries[0].disposition {
        CapabilityDisposition::Supported { module } => {
            let descriptor = dispatch.resolve("bounded-collection-query").map_err(|_| {
                KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_dispatch_unowned",
                    &query.source_id,
                    profile.selection.revision.clone(),
                )
            })?;
            if descriptor.family != SemanticFamily::CollectionsQueries
                || descriptor.module_id != *module
            {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_dispatch_profile_mismatch",
                    &query.source_id,
                    profile.selection.revision.clone(),
                ));
            }
        }
        CapabilityDisposition::Refused { code } => {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::Refused,
                code.clone(),
                &query.source_id,
                profile.selection.revision.clone(),
            ))
        }
        CapabilityDisposition::Inconclusive { code } => {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::Inconclusive,
                code.clone(),
                &query.source_id,
                profile.selection.revision.clone(),
            ))
        }
    }
    if query.values.len() > query.max_items {
        return Err(KaniOutcome::non_success(
            KaniOutcomeKind::ResourceExhausted,
            "kani_collection_bound_exhausted",
            &query.source_id,
            profile.selection.revision.clone(),
        ));
    }
    let (value, examined) = match query.kind {
        QueryKind::ForAllNonNegative => match query.values.iter().position(|value| *value < 0) {
            Some(index) => (false, index + 1),
            None => (true, query.values.len()),
        },
        QueryKind::ExistsEqual(expected) => {
            match query.values.iter().position(|value| *value == expected) {
                Some(index) => (true, index + 1),
                None => (false, query.values.len()),
            }
        }
    };
    Ok(CollectionLowering {
        query,
        value,
        examined,
    })
}
