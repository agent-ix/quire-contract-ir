// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-053: project one exact admitted population binding and Type occurrence.

use super::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageV2, CheckedSemanticId,
    CheckedSourceRef, CheckedSourceRegion,
};
use serde_json::Value;

/// The exact admitted binding and occurrence requested by a graph consumer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedSelectedPopulationSelector {
    /// The admitted package the binding belongs to.
    pub package_id: CheckedSemanticId,
    /// The exact `bounded_domain` / `model_population` binding identity.
    pub binding: CheckedNodeId,
    /// The exact source `Type` occurrence, including its ordinal.
    pub occurrence: CheckedOccurrence,
}

/// A source selection for the population query.
#[derive(Clone, Debug)]
pub enum CheckedSelectedPopulationSource<'a> {
    /// An admitted V2 package and its exact binding selector.
    Admitted {
        /// Admitted package containing the selected binding.
        package: &'a CheckedPackageV2,
        /// Exact binding and source occurrence selector.
        selector: CheckedSelectedPopulationSelector,
    },
    /// A known domain-package `PopulationRecord` identity, which carries no maximum.
    ///
    /// The upstream adapter supplies this only for a selected, admitted
    /// population record; this model boundary does not depend on QSL.
    Declaration {
        /// Selected model identity.
        selected_model: CheckedSemanticId,
        /// Population declaration identity.
        population: CheckedNodeId,
    },
    /// QSL reported an open population extent before V2 admission.
    UnknownClosureOpenExtent {
        /// Selected model identity.
        selected_model: CheckedSemanticId,
        /// Population declaration identity.
        population: CheckedNodeId,
        /// Upstream source identity.
        source: CheckedSourceRef,
        /// Authentic upstream source regions.
        provenance: Vec<CheckedSourceRegion>,
    },
    /// QSL could not resolve the selected object type before V2 admission.
    UnknownClosureUnresolvedObjectType {
        /// Selected model identity.
        selected_model: CheckedSemanticId,
        /// Population declaration identity.
        population: CheckedNodeId,
        /// Upstream source identity.
        source: CheckedSourceRef,
        /// Authentic upstream source regions.
        provenance: Vec<CheckedSourceRegion>,
    },
    /// QSL could not resolve subtype closure before V2 admission.
    UnknownClosureUnresolvedSubtypeClosure {
        /// Selected model identity.
        selected_model: CheckedSemanticId,
        /// Population declaration identity.
        population: CheckedNodeId,
        /// Upstream source identity.
        source: CheckedSourceRef,
        /// Authentic upstream source regions.
        provenance: Vec<CheckedSourceRegion>,
    },
    /// No source selection was available.
    Unavailable,
}

/// A finite selected population and its exact package/source provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedSelectedPopulation {
    /// Selected admitted package identity.
    pub package_id: CheckedSemanticId,
    /// Exact population binding node identity.
    pub binding: CheckedNodeId,
    /// Exact selected source occurrence.
    pub occurrence: CheckedOccurrence,
    /// Object type identity selected by this population binding.
    pub object_type: CheckedNodeId,
    /// Inclusive lower endpoint, always zero for a selected population.
    pub minimum: u64,
    /// Inclusive maximum read from the selected binding's `max` field.
    pub maximum: u64,
    /// Authentic regions mapped to this exact binding/occurrence pair.
    pub provenance: Vec<CheckedSourceRegion>,
}

/// Typed finite, non-finite, unavailable, and invalid selection outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedSelectedPopulationOutcome {
    /// Exact selected binding and occurrence have a finite inclusive extent.
    Finite(CheckedSelectedPopulation),
    /// A known declaration has no binding maximum.
    Unbounded {
        /// Selected model identity.
        selected_model: CheckedSemanticId,
        /// Population declaration identity.
        population: CheckedNodeId,
    },
    /// The source declaration has an open extent.
    UnknownClosureOpenExtent {
        /// Selected model identity.
        selected_model: CheckedSemanticId,
        /// Population declaration identity.
        population: CheckedNodeId,
        /// Upstream source identity.
        source: CheckedSourceRef,
        /// Authentic upstream source regions.
        provenance: Vec<CheckedSourceRegion>,
    },
    /// The selected object type could not be resolved upstream.
    UnknownClosureUnresolvedObjectType {
        /// Selected model identity.
        selected_model: CheckedSemanticId,
        /// Population declaration identity.
        population: CheckedNodeId,
        /// Upstream source identity.
        source: CheckedSourceRef,
        /// Authentic upstream source regions.
        provenance: Vec<CheckedSourceRegion>,
    },
    /// Required subtype closure could not be resolved upstream.
    UnknownClosureUnresolvedSubtypeClosure {
        /// Selected model identity.
        selected_model: CheckedSemanticId,
        /// Population declaration identity.
        population: CheckedNodeId,
        /// Upstream source identity.
        source: CheckedSourceRef,
        /// Authentic upstream source regions.
        provenance: Vec<CheckedSourceRegion>,
    },
    /// No selection was available.
    Unavailable,
    /// The selector names another admitted package.
    ForeignPackage(CheckedSelectedPopulationSelector),
    /// The selector's binding identity is absent from its admitted package.
    StaleBinding(CheckedSelectedPopulationSelector),
    /// The binding exists but its exact Type source occurrence is not mapped.
    MissingOccurrence(CheckedSelectedPopulationSelector),
    /// The selected node is not a `bounded_domain` / `model_population`.
    NotPopulation(CheckedSelectedPopulationSelector),
}

/// Select one source-backed finite population binding.
pub fn select_population(
    source: CheckedSelectedPopulationSource<'_>,
) -> CheckedSelectedPopulationOutcome {
    match source {
        CheckedSelectedPopulationSource::Unavailable => {
            CheckedSelectedPopulationOutcome::Unavailable
        }
        CheckedSelectedPopulationSource::Declaration {
            selected_model,
            population,
        } => CheckedSelectedPopulationOutcome::Unbounded {
            selected_model,
            population,
        },
        CheckedSelectedPopulationSource::UnknownClosureOpenExtent {
            selected_model,
            population,
            source,
            provenance,
        } => CheckedSelectedPopulationOutcome::UnknownClosureOpenExtent {
            selected_model,
            population,
            source,
            provenance,
        },
        CheckedSelectedPopulationSource::UnknownClosureUnresolvedObjectType {
            selected_model,
            population,
            source,
            provenance,
        } => CheckedSelectedPopulationOutcome::UnknownClosureUnresolvedObjectType {
            selected_model,
            population,
            source,
            provenance,
        },
        CheckedSelectedPopulationSource::UnknownClosureUnresolvedSubtypeClosure {
            selected_model,
            population,
            source,
            provenance,
        } => CheckedSelectedPopulationOutcome::UnknownClosureUnresolvedSubtypeClosure {
            selected_model,
            population,
            source,
            provenance,
        },
        CheckedSelectedPopulationSource::Admitted { package, selector } => {
            select_admitted(package, selector)
        }
    }
}

fn select_admitted(
    package: &CheckedPackageV2,
    selector: CheckedSelectedPopulationSelector,
) -> CheckedSelectedPopulationOutcome {
    if package.package_id() != &selector.package_id {
        return CheckedSelectedPopulationOutcome::ForeignPackage(selector);
    }
    let Some(node) = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id == selector.binding)
    else {
        return CheckedSelectedPopulationOutcome::StaleBinding(selector);
    };
    if selector.occurrence.role != CheckedOccurrenceRole::Type
        || !node.occurrences.contains(&selector.occurrence)
    {
        return CheckedSelectedPopulationOutcome::MissingOccurrence(selector);
    }
    let Some(source) = package.source_map().iter().find(|entry| {
        entry.node_id == selector.binding
            && entry.role == CheckedOccurrenceRole::Type
            && entry.ordinal == selector.occurrence.ordinal
    }) else {
        return CheckedSelectedPopulationOutcome::MissingOccurrence(selector);
    };
    if node.node_tag.as_ref() != "bounded_domain"
        || node.semantic_form.as_ref() != "model_population"
    {
        return CheckedSelectedPopulationOutcome::NotPopulation(selector);
    }
    let Some(maximum) = population_maximum(&node.body) else {
        return CheckedSelectedPopulationOutcome::NotPopulation(selector);
    };
    let Some(object_type) = population_object_type(node, package.graph().nodes.as_slice()) else {
        return CheckedSelectedPopulationOutcome::NotPopulation(selector);
    };
    CheckedSelectedPopulationOutcome::Finite(CheckedSelectedPopulation {
        package_id: package.package_id().clone(),
        binding: node.node_id.clone(),
        occurrence: selector.occurrence,
        object_type,
        minimum: 0,
        maximum,
        provenance: source.regions.clone(),
    })
}

fn population_maximum(body: &Value) -> Option<u64> {
    let maximum_member = body
        .get("members")?
        .as_array()?
        .iter()
        .find(|member| member.get("name").and_then(Value::as_str) == Some("max"))?;
    if maximum_member.get("term")?.as_str()? != "binding" {
        return None;
    }
    let maximum = maximum_member.get("value")?;
    if maximum.get("term")?.as_str()? != "literal"
        || maximum.get("value_kind")?.as_str()? != "integer"
    {
        return None;
    }
    maximum.get("value").and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_str()?.parse::<u64>().ok())
    })
}

fn population_object_type(
    node: &super::v2::CheckedSemanticNodeV2,
    nodes: &[super::v2::CheckedSemanticNodeV2],
) -> Option<CheckedNodeId> {
    let set_type = nodes
        .iter()
        .find(|candidate| candidate.node_id == node.semantic_type)?;
    let reference_target = set_type
        .body
        .get("members")?
        .as_array()?
        .first()?
        .get("target")?;
    let reference_id: CheckedNodeId = serde_json::from_value(reference_target.clone()).ok()?;
    let reference = nodes
        .iter()
        .find(|candidate| candidate.node_id == reference_id)?;
    let object_id = reference
        .body
        .get("members")?
        .as_array()?
        .first()?
        .get("target")?;
    serde_json::from_value(object_id.clone()).ok()
}
