// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-053: project one exact admitted population binding and Type occurrence.

use super::v2::{BoundedDomainForm, CheckedNodeKind, CheckedTerm, LiteralKind};
use super::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageV2, CheckedSemanticId,
    CheckedSourceMapEntry, CheckedSourceRef, CheckedSourceRegion,
};
use std::collections::BTreeMap;

#[cfg(test)]
mod source_probe {
    use std::cell::Cell;

    thread_local! {
        static CANDIDATES: Cell<u64> = const { Cell::new(0) };
        static BUILDS: Cell<u64> = const { Cell::new(0) };
        static BUILD_ENTRIES: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn visited() {
        CANDIDATES.with(|count| count.set(count.get().saturating_add(1)));
    }

    pub(super) fn reset() {
        reset_candidates();
        BUILDS.with(|count| count.set(0));
        BUILD_ENTRIES.with(|count| count.set(0));
    }

    pub(super) fn reset_candidates() {
        CANDIDATES.with(|count| count.set(0));
    }

    pub(super) fn count() -> u64 {
        CANDIDATES.with(Cell::get)
    }

    pub(super) fn built() {
        BUILDS.with(|count| count.set(count.get() + 1));
    }

    pub(super) fn build_entry() {
        BUILD_ENTRIES.with(|count| count.set(count.get() + 1));
    }

    pub(super) fn builds() -> u64 {
        BUILDS.with(Cell::get)
    }

    pub(super) fn build_entries() -> u64 {
        BUILD_ENTRIES.with(Cell::get)
    }
}

pub(in crate::checked_package) fn source_index(
    entries: &[CheckedSourceMapEntry],
) -> BTreeMap<(CheckedNodeId, CheckedOccurrenceRole, u64), usize> {
    #[cfg(test)]
    source_probe::built();
    let mut positions = BTreeMap::new();
    for (position, entry) in entries.iter().enumerate() {
        #[cfg(test)]
        source_probe::build_entry();
        positions
            .entry((entry.node_id.clone(), entry.role.clone(), entry.ordinal))
            .or_insert(position);
    }
    positions
}

fn selected_source<'a>(
    entries: &'a [CheckedSourceMapEntry],
    positions: &BTreeMap<(CheckedNodeId, CheckedOccurrenceRole, u64), usize>,
    binding: &CheckedNodeId,
    ordinal: u64,
) -> Option<&'a CheckedSourceMapEntry> {
    positions
        .get(&(binding.clone(), CheckedOccurrenceRole::Type, ordinal))
        .and_then(|position| entries.get(*position))
        .inspect(|_| {
            #[cfg(test)]
            source_probe::visited();
        })
        .filter(|entry| {
            &entry.node_id == binding
                && entry.role == CheckedOccurrenceRole::Type
                && entry.ordinal == ordinal
        })
}

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
    let Some(node) = package.node(&selector.binding) else {
        return CheckedSelectedPopulationOutcome::StaleBinding(selector);
    };
    if selector.occurrence.role != CheckedOccurrenceRole::Type
        || !node.occurrences.contains(&selector.occurrence)
    {
        return CheckedSelectedPopulationOutcome::MissingOccurrence(selector);
    }
    let Some(source) = selected_source(
        package.source_map(),
        &package.source_index,
        &selector.binding,
        selector.occurrence.ordinal,
    ) else {
        return CheckedSelectedPopulationOutcome::MissingOccurrence(selector);
    };
    if package.node_kind(&selector.binding)
        != Some(CheckedNodeKind::BoundedDomain(
            BoundedDomainForm::ModelPopulation,
        ))
    {
        return CheckedSelectedPopulationOutcome::NotPopulation(selector);
    }
    let Some(maximum) = package.term(&selector.binding).and_then(population_maximum) else {
        return CheckedSelectedPopulationOutcome::NotPopulation(selector);
    };
    let Some(object_type) = population_object_type_typed(node, |id| package.term(id)) else {
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

#[cfg(test)]
mod source_index_tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn id(number: usize) -> CheckedNodeId {
        serde_json::from_value(json!({
            "domain": "quire.checked-semantic-node/v1",
            "digest": format!("{number:064x}"),
        }))
        .expect("node identity")
    }

    #[trace("TC-453", "FR-053-AC-2")]
    #[test]
    fn repeated_population_source_reads_stay_with_the_selected_node() {
        let mut measured = Vec::new();
        for entries in [4, 8, 16] {
            let selected = id(0);
            let sources: Vec<_> = (0..entries)
                .map(|number| CheckedSourceMapEntry {
                    node_id: selected.clone(),
                    role: CheckedOccurrenceRole::Type,
                    ordinal: number as u64,
                    regions: Vec::new(),
                })
                .collect();
            source_probe::reset();
            let positions = source_index(&sources);
            assert_eq!(source_probe::builds(), 1);
            assert_eq!(source_probe::build_entries(), entries as u64);
            for requests in [2, 4, 8] {
                source_probe::reset_candidates();
                crate::checked_package::shared::checked_node_comparisons::reset();
                for _ in 0..requests {
                    assert_eq!(
                        selected_source(&sources, &positions, &selected, (entries - 1) as u64),
                        sources.last()
                    );
                }
                let visits = source_probe::count();
                let comparisons = crate::checked_package::shared::checked_node_comparisons::count();
                assert_eq!(source_probe::builds(), 1);
                assert_eq!(source_probe::build_entries(), entries as u64);
                println!("population_sources K={requests} R={entries} candidates={visits} id_comparisons={comparisons}");
                if entries == 16 {
                    assert!(comparisons < requests * 16, "a full source scan must fail");
                }
                measured.push((entries, requests, visits));
            }
        }
        for (entries, requests, visits) in measured {
            assert_eq!(visits, requests, "entries={entries}, requests={requests}");
        }
    }
}

/// Whether `node` is a `bounded_domain` / `model_population` node.
pub(super) fn is_model_population(node: &super::v2::CheckedSemanticNodeV2) -> bool {
    node.node_tag.as_ref() == "bounded_domain" && node.semantic_form.as_ref() == "model_population"
}

/// The `Type` occurrence of `node` with the least ordinal: the one occurrence
/// a reaches application's population link names.
pub(super) fn least_type_occurrence(
    node: &super::v2::CheckedSemanticNodeV2,
) -> Option<&CheckedOccurrence> {
    node.occurrences
        .iter()
        .filter(|occurrence| occurrence.role == CheckedOccurrenceRole::Type)
        .min_by_key(|occurrence| occurrence.ordinal)
}

/// The inclusive maximum of a population node's admitted `max` binding.
pub(super) fn population_maximum(body: &CheckedTerm) -> Option<u64> {
    let CheckedTerm::Aggregate { members } = body else {
        return None;
    };
    let mut maximum_members = members.iter().filter(
        |member| matches!(member, CheckedTerm::Binding { name, .. } if name.as_ref() == "max"),
    );
    let maximum_member = maximum_members.next()?;
    if maximum_members.next().is_some() {
        return None;
    }
    let CheckedTerm::Binding { value, .. } = maximum_member else {
        return None;
    };
    let CheckedTerm::Literal {
        value_kind: LiteralKind::Integer,
        value,
        ..
    } = value.as_ref()
    else {
        return None;
    };
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse::<u64>().ok())
}

/// The object type a population node's `semantic_type` chain names, resolving
/// each node of the chain through `node`.
pub(super) fn population_object_type<'a>(
    population: &super::v2::CheckedSemanticNodeV2,
    node: impl Fn(&CheckedNodeId) -> Option<&'a super::v2::CheckedSemanticNodeV2>,
) -> Option<CheckedNodeId> {
    let set_type = node(&population.semantic_type)?;
    let reference_target = set_type
        .body
        .get("members")?
        .as_array()?
        .first()?
        .get("target")?;
    let reference_id: CheckedNodeId = serde_json::from_value(reference_target.clone()).ok()?;
    let reference = node(&reference_id)?;
    let object_id = reference
        .body
        .get("members")?
        .as_array()?
        .first()?
        .get("target")?;
    serde_json::from_value(object_id.clone()).ok()
}

/// Resolve the selected population's object type from the retained admitted terms.
fn population_object_type_typed<'a>(
    population: &super::v2::CheckedSemanticNodeV2,
    term: impl Fn(&CheckedNodeId) -> Option<&'a CheckedTerm>,
) -> Option<CheckedNodeId> {
    let first_reference = |id: &CheckedNodeId| {
        let CheckedTerm::Aggregate { members } = term(id)? else {
            return None;
        };
        let CheckedTerm::Reference { target } = members.first()? else {
            return None;
        };
        Some(target)
    };
    first_reference(first_reference(&population.semantic_type)?).cloned()
}

#[cfg(test)]
mod tests {
    use super::{population_maximum, population_object_type_typed};
    use crate::checked_package::v2::{CheckedSemanticNodeV2, CheckedTerm, LiteralKind};
    use crate::checked_package::CheckedNodeId;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("FR-349-AC-2", "FR-053-AC-3")]
    #[test]
    fn admitted_population_object_type_uses_retained_reference_chain() {
        let id = |digit: char| CheckedNodeId {
            domain: "quire.checked-semantic-node/v1".into(),
            digest: digit.to_string().repeat(64).into(),
        };
        let set = id('1');
        let reference = id('2');
        let object = id('3');
        let population: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id": id('4'), "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "bounded_domain", "semantic_form": "model_population",
            "semantic_type": set, "dependencies": [], "occurrences": [],
            "body": {"term": "aggregate", "members": []}
        }))
        .expect("population node");
        let set_term = CheckedTerm::Aggregate {
            members: vec![CheckedTerm::Reference {
                target: reference.clone(),
            }],
        };
        let reference_term = CheckedTerm::Aggregate {
            members: vec![CheckedTerm::Reference {
                target: object.clone(),
            }],
        };
        assert_eq!(
            population_object_type_typed(&population, |node| {
                if node == &set {
                    Some(&set_term)
                } else if node == &reference {
                    Some(&reference_term)
                } else {
                    None
                }
            }),
            Some(object)
        );
    }

    #[trace("FR-349-AC-2", "FR-053-AC-3")]
    #[test]
    fn admitted_maximum_reads_typed_binding_and_literal() {
        let integer = CheckedNodeId {
            domain: "quire.checked-semantic-node/v1".into(),
            digest: "0".repeat(64).into(),
        };
        let maximum = CheckedTerm::Binding {
            name: "max".into(),
            value: Box::new(CheckedTerm::Literal {
                type_id: integer,
                value_kind: LiteralKind::Integer,
                value: json!("0"),
            }),
        };
        let aggregate = CheckedTerm::Aggregate {
            members: vec![maximum.clone()],
        };
        assert_eq!(population_maximum(&aggregate), Some(0));
        let duplicate = CheckedTerm::Aggregate {
            members: vec![maximum.clone(), maximum],
        };
        assert_eq!(population_maximum(&duplicate), None);
    }

    #[trace("FR-053-AC-3")]
    #[test]
    fn maximum_requires_the_selected_integer_literal_binding() {
        let id = CheckedNodeId {
            domain: "quire.checked-semantic-node/v1".into(),
            digest: "0".repeat(64).into(),
        };
        let literal = |kind, value| CheckedTerm::Literal {
            type_id: id.clone(),
            value_kind: kind,
            value,
        };
        let binding = |name: &str, value| CheckedTerm::Binding {
            name: name.into(),
            value: Box::new(value),
        };
        let aggregate = |members| CheckedTerm::Aggregate { members };
        assert_eq!(
            population_maximum(&aggregate(vec![binding(
                "max",
                literal(LiteralKind::Integer, json!("0"))
            )])),
            Some(0)
        );
        for body in [
            aggregate(vec![]),
            aggregate(vec![
                binding("max", literal(LiteralKind::Integer, json!("3"))),
                binding("max", literal(LiteralKind::Integer, json!("5"))),
            ]),
            aggregate(vec![binding(
                "min",
                literal(LiteralKind::Integer, json!("99")),
            )]),
            aggregate(vec![literal(LiteralKind::Integer, json!("99"))]),
            aggregate(vec![binding(
                "max",
                literal(LiteralKind::String, json!("99")),
            )]),
            aggregate(vec![binding(
                "max",
                CheckedTerm::Reference { target: id.clone() },
            )]),
            aggregate(vec![binding(
                "max",
                literal(LiteralKind::Integer, json!("-1")),
            )]),
        ] {
            assert_eq!(
                population_maximum(&body),
                None,
                "unexpected max from {body:?}"
            );
        }
    }
}
