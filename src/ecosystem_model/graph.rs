//! Typed relation validation and deterministic graph projections.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::decision::{ModelCauseCode as Code, ModelDecision};
use super::manifest::{
    resource, u64_len, EcosystemLimits, ManifestEdge, ManifestEdgeKind, ManifestGap, ManifestNode,
};

/// One deterministic outgoing adjacency entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelAdjacency {
    /// Source-node identity.
    pub source: String,
    /// Sorted outgoing typed edges.
    pub edges: Vec<ModelAdjacentEdge>,
}

/// One typed target in an adjacency entry.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelAdjacentEdge {
    /// Closed relation kind.
    pub kind: ManifestEdgeKind,
    /// Target-node identity.
    pub target: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedGraph {
    pub adjacency: Vec<ModelAdjacency>,
    pub topological_order: Vec<String>,
}

pub(crate) fn validate(
    nodes: &[ManifestNode],
    edges: &[ManifestEdge],
    gaps: &[ManifestGap],
    limits: EcosystemLimits,
) -> Result<ValidatedGraph, ModelDecision> {
    let by_identity: BTreeMap<_, _> = nodes.iter().map(|node| (node.identity(), node)).collect();
    for gap in gaps {
        match by_identity.get(gap.requirement.as_str()) {
            None => {
                return Err(decision(
                    Code::DanglingEdge,
                    gap.requirement.as_str(),
                    "gap requirement is absent",
                ))
            }
            Some(ManifestNode::Requirement { .. }) => {}
            Some(_) => {
                return Err(decision(
                    Code::IllTypedEdge,
                    gap.requirement.as_str(),
                    "gap target is not a requirement",
                ))
            }
        }
    }
    for edge in edges {
        let source = by_identity.get(edge.source.as_str()).ok_or_else(|| {
            decision(
                Code::DanglingEdge,
                edge.source.as_str(),
                "edge source is absent",
            )
        })?;
        let target = by_identity.get(edge.target.as_str()).ok_or_else(|| {
            decision(
                Code::DanglingEdge,
                edge.target.as_str(),
                "edge target is absent",
            )
        })?;
        if edge.source == edge.target {
            return Err(decision(
                Code::SelfEdge,
                edge.source.as_str(),
                "self edge is forbidden",
            ));
        }
        if !relation_allowed(edge.kind, source, target) {
            return Err(decision(
                Code::IllTypedEdge,
                edge.source.as_str(),
                "edge endpoints do not admit this relation",
            ));
        }
    }
    validate_ownership(nodes, edges, &by_identity)?;
    let adjacency = adjacency(nodes, edges, limits)?;
    let topological_order = topological(nodes, edges, limits)?;
    Ok(ValidatedGraph {
        adjacency,
        topological_order,
    })
}

fn validate_ownership(
    nodes: &[ManifestNode],
    edges: &[ManifestEdge],
    by_identity: &BTreeMap<&str, &ManifestNode>,
) -> Result<(), ModelDecision> {
    for node in nodes {
        if matches!(node, ManifestNode::Repository { .. }) {
            continue;
        }
        let repository_owners: Vec<_> = edges
            .iter()
            .filter(|edge| edge.kind == ManifestEdgeKind::Owns && edge.target == node.identity())
            .filter_map(|edge| match by_identity.get(edge.source.as_str()) {
                Some(ManifestNode::Repository { identity, .. }) => Some(identity.as_str()),
                _ => None,
            })
            .collect();
        if repository_owners.is_empty() {
            return Err(decision(
                Code::OwnershipIncomplete,
                node.identity(),
                "node has no repository owner",
            ));
        }
        if repository_owners.len() != 1 {
            return Err(decision(
                Code::MultipleOwners,
                node.identity(),
                "node has multiple repository owners",
            ));
        }
        if let ManifestNode::Contract { selection, .. } = node {
            let component_owners: Vec<_> = edges
                .iter()
                .filter(|edge| {
                    edge.kind == ManifestEdgeKind::Owns && edge.target == node.identity()
                })
                .filter_map(|edge| match by_identity.get(edge.source.as_str()) {
                    Some(ManifestNode::Component { identity }) => Some(identity.as_str()),
                    _ => None,
                })
                .collect();
            if component_owners.is_empty() {
                return Err(decision(
                    Code::OwnershipIncomplete,
                    node.identity(),
                    "executable contract has no component owner",
                ));
            }
            if component_owners.len() != 1 {
                return Err(decision(
                    Code::MultipleOwners,
                    node.identity(),
                    "executable contract has multiple component owners",
                ));
            }
            let component_repository = repository_owner(component_owners[0], edges, by_identity)?;
            if component_repository != repository_owners[0]
                || selection.repository() != repository_owners[0]
            {
                return Err(decision(
                    Code::IllTypedEdge,
                    node.identity(),
                    "contract selection and ownership repositories differ",
                ));
            }
        }
    }
    for repository in nodes {
        let ManifestNode::Repository { identity, .. } = repository else {
            continue;
        };
        let owns_component = edges.iter().any(|edge| {
            edge.kind == ManifestEdgeKind::Owns
                && edge.source == *identity
                && matches!(
                    by_identity.get(edge.target.as_str()),
                    Some(ManifestNode::Component { .. })
                )
        });
        if !owns_component {
            return Err(decision(
                Code::OwnershipIncomplete,
                identity.as_str(),
                "repository has no declared component",
            ));
        }
    }
    Ok(())
}

fn repository_owner<'a>(
    identity: &str,
    edges: &'a [ManifestEdge],
    by_identity: &BTreeMap<&str, &ManifestNode>,
) -> Result<&'a str, ModelDecision> {
    let mut owners = edges
        .iter()
        .filter(|edge| edge.kind == ManifestEdgeKind::Owns && edge.target == identity)
        .filter(|edge| {
            matches!(
                by_identity.get(edge.source.as_str()),
                Some(ManifestNode::Repository { .. })
            )
        })
        .map(|edge| edge.source.as_str());
    let owner = owners.next().ok_or_else(|| {
        decision(
            Code::OwnershipIncomplete,
            identity,
            "component has no repository owner",
        )
    })?;
    if owners.next().is_some() {
        return Err(decision(
            Code::MultipleOwners,
            identity,
            "component has multiple repository owners",
        ));
    }
    Ok(owner)
}

fn relation_allowed(kind: ManifestEdgeKind, source: &ManifestNode, target: &ManifestNode) -> bool {
    match kind {
        ManifestEdgeKind::Owns => {
            matches!(source, ManifestNode::Repository { .. })
                && !matches!(target, ManifestNode::Repository { .. })
                || matches!(
                    (source, target),
                    (
                        ManifestNode::Component { .. },
                        ManifestNode::Contract { .. }
                    )
                )
        }
        ManifestEdgeKind::RuntimeDependency => {
            matches!(
                (source, target),
                (
                    ManifestNode::Component { .. },
                    ManifestNode::Component { .. }
                )
            )
        }
        ManifestEdgeKind::NormativeReference => {
            matches!(
                source,
                ManifestNode::Component { .. }
                    | ManifestNode::Object { .. }
                    | ManifestNode::Interface { .. }
                    | ManifestNode::Contract { .. }
                    | ManifestNode::Requirement { .. }
            ) && matches!(
                target,
                ManifestNode::Object { .. }
                    | ManifestNode::Interface { .. }
                    | ManifestNode::Contract { .. }
                    | ManifestNode::Requirement { .. }
            )
        }
        ManifestEdgeKind::Consumes => {
            matches!(source, ManifestNode::Component { .. })
                && matches!(
                    target,
                    ManifestNode::Object { .. }
                        | ManifestNode::Interface { .. }
                        | ManifestNode::Contract { .. }
                )
        }
        ManifestEdgeKind::Verifies => {
            matches!(
                source,
                ManifestNode::Test { .. } | ManifestNode::Review { .. }
            ) && matches!(
                target,
                ManifestNode::Requirement { .. }
                    | ManifestNode::Interface { .. }
                    | ManifestNode::Contract { .. }
            )
        }
    }
}

fn adjacency(
    nodes: &[ManifestNode],
    edges: &[ManifestEdge],
    limits: EcosystemLimits,
) -> Result<Vec<ModelAdjacency>, ModelDecision> {
    let retained = u64_len(nodes.len())?.saturating_add(u64_len(edges.len())?);
    if retained > limits.allocation_bytes || retained > limits.visited_work {
        return Err(resource(
            "graph.adjacency",
            "adjacency reservation ceiling exceeded",
        ));
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(nodes.len())
        .map_err(|_| resource("graph.adjacency", "adjacency reservation failed"))?;
    for node in nodes {
        let mut outgoing: Vec<_> = edges
            .iter()
            .filter(|edge| edge.source == node.identity())
            .map(|edge| ModelAdjacentEdge {
                kind: edge.kind,
                target: edge.target.clone(),
            })
            .collect();
        outgoing.sort();
        result.push(ModelAdjacency {
            source: node.identity().to_owned(),
            edges: outgoing,
        });
    }
    Ok(result)
}

fn topological(
    nodes: &[ManifestNode],
    edges: &[ManifestEdge],
    limits: EcosystemLimits,
) -> Result<Vec<String>, ModelDecision> {
    let mut outgoing: BTreeMap<&str, BTreeSet<&str>> = nodes
        .iter()
        .map(|node| (node.identity(), BTreeSet::new()))
        .collect();
    let mut incoming: BTreeMap<&str, usize> =
        nodes.iter().map(|node| (node.identity(), 0)).collect();
    for edge in edges {
        let ordered = match edge.kind {
            ManifestEdgeKind::Owns => Some((edge.source.as_str(), edge.target.as_str())),
            ManifestEdgeKind::RuntimeDependency => {
                Some((edge.target.as_str(), edge.source.as_str()))
            }
            ManifestEdgeKind::NormativeReference
            | ManifestEdgeKind::Consumes
            | ManifestEdgeKind::Verifies => None,
        };
        if let Some((before, after)) = ordered {
            let inserted = outgoing
                .get_mut(before)
                .is_some_and(|targets| targets.insert(after));
            if inserted {
                let count = incoming.get_mut(after).ok_or_else(|| {
                    decision(Code::GraphMismatch, after, "topology target disappeared")
                })?;
                *count = count.saturating_add(1);
            }
        }
    }
    let mut ready: BTreeSet<_> = incoming
        .iter()
        .filter_map(|(node, count)| (*count == 0).then_some(*node))
        .collect();
    let mut order = Vec::new();
    order
        .try_reserve_exact(nodes.len())
        .map_err(|_| resource("graph.topological-order", "topology reservation failed"))?;
    let mut work = 0_u64;
    while let Some(node) = ready.pop_first() {
        order.push(node.to_owned());
        if let Some(targets) = outgoing.get(node) {
            for target in targets {
                work = work.saturating_add(1);
                if work > limits.visited_work {
                    return Err(resource(
                        "graph.topological-order",
                        "topology work ceiling exceeded",
                    ));
                }
                let count = incoming.get_mut(target).ok_or_else(|| {
                    decision(Code::GraphMismatch, *target, "topology target disappeared")
                })?;
                *count = count.saturating_sub(1);
                if *count == 0 {
                    ready.insert(target);
                }
            }
        }
    }
    if order.len() != nodes.len() {
        return Err(decision(
            Code::DependencyCycle,
            "graph.topological-order",
            "ownership/runtime dependency graph contains a cycle",
        ));
    }
    Ok(order)
}

fn decision(code: Code, path: impl Into<Box<str>>, detail: impl Into<Box<str>>) -> ModelDecision {
    ModelDecision::new(code, path, detail)
}
