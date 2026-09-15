//! Identity-preserving finite-reference and bounded-reachability lowering.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    CapabilityDisposition, DispatchIndex, KaniOutcome, KaniOutcomeKind, KaniProfile,
    SemanticFamily, ValidatedFiniteInput,
};

/// One positive-length reachability request over the validated finite universe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphRequest {
    /// Source clause identity.
    pub source_id: String,
    /// Start object identity.
    pub start_id: String,
    /// Target object identity.
    pub target_id: String,
    /// Exact reference field to follow.
    pub field_id: String,
    /// Maximum expanded identities for this query.
    pub max_expansions: usize,
}

/// Exact finite graph lowering result, not a Kani verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphLowering {
    /// Request whose semantics were evaluated over the finite input.
    pub request: GraphRequest,
    /// Whether the target is reachable by at least one matching edge.
    pub reachable: bool,
    /// Exact identities expanded, in deterministic discovery order.
    pub expanded: Vec<String>,
}

/// Lowers positive-length reachability over the exact validated finite reference graph.
pub fn lower_reaches(
    profile: &KaniProfile,
    dispatch: &DispatchIndex,
    input: &ValidatedFiniteInput,
    request: GraphRequest,
) -> Result<GraphLowering, KaniOutcome> {
    let entries = profile.classify(&["finite-reference-graph".to_owned()], &request.source_id)?;
    match &entries[0].disposition {
        CapabilityDisposition::Supported { module } => {
            let descriptor = dispatch.resolve("finite-reference-graph").map_err(|_| {
                KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_dispatch_unowned",
                    &request.source_id,
                    profile.selection.revision.clone(),
                )
            })?;
            if descriptor.family != SemanticFamily::ObjectsReferencesGraphs
                || descriptor.module_id != *module
            {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_dispatch_profile_mismatch",
                    &request.source_id,
                    profile.selection.revision.clone(),
                ));
            }
        }
        CapabilityDisposition::Refused { code } => {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::Refused,
                code.clone(),
                &request.source_id,
                profile.selection.revision.clone(),
            ))
        }
        CapabilityDisposition::Inconclusive { code } => {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::Inconclusive,
                code.clone(),
                &request.source_id,
                profile.selection.revision.clone(),
            ))
        }
    }
    if request.max_expansions == 0 {
        return Err(KaniOutcome::non_success(
            KaniOutcomeKind::InvalidInput,
            "kani_graph_bound_invalid",
            &request.source_id,
            profile.selection.revision.clone(),
        ));
    }
    let objects: BTreeSet<&str> = input
        .input()
        .objects
        .iter()
        .map(|object| object.identity.as_str())
        .collect();
    if !objects.contains(request.start_id.as_str()) || !objects.contains(request.target_id.as_str())
    {
        return Err(KaniOutcome::non_success(
            KaniOutcomeKind::InvalidInput,
            "kani_graph_identity_invalid",
            &request.source_id,
            profile.selection.revision.clone(),
        ));
    }
    let mut edges: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &input.input().references {
        if edge.field_id == request.field_id {
            edges
                .entry(edge.source_id.as_str())
                .or_default()
                .push(edge.target_id.as_str());
        }
    }
    for targets in edges.values_mut() {
        targets.sort_unstable();
    }
    let mut visited = BTreeSet::new();
    let mut frontier = vec![request.start_id.as_str()];
    let mut expanded = Vec::new();
    while let Some(current) = frontier.pop() {
        if !visited.insert(current) {
            continue;
        }
        if expanded.len() == request.max_expansions {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::ResourceExhausted,
                "kani_graph_expansion_exhausted",
                &request.source_id,
                profile.selection.revision.clone(),
            ));
        }
        expanded.push(current.to_owned());
        let Some(targets) = edges.get(current) else {
            continue;
        };
        for target in targets.iter().rev() {
            if *target == request.target_id {
                return Ok(GraphLowering {
                    request,
                    reachable: true,
                    expanded,
                });
            }
            frontier.push(target);
        }
    }
    Ok(GraphLowering {
        request,
        reachable: false,
        expanded,
    })
}
