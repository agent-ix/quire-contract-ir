//! Typed population selector of an admitted `quire.op.model.reaches`
//! application.

use super::relationship_end::ReachesLookup;
use super::{BoundedDomainForm, CheckedNodeId, CheckedNodeKind, CheckedPackageV2};
use crate::checked_package::selected_population::{
    least_type_occurrence, CheckedSelectedPopulationSelector,
};
use thiserror::Error;

/// Why a reaches application's population cannot be read.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CheckedReachesPopulationError {
    /// The requested node is absent.
    #[error("unknown checked node")]
    UnknownNode,
    /// The requested node's body root is not a `quire.op.model.reaches`
    /// application.
    #[error("checked node is not a reaches application")]
    NotReachesApplication,
    /// The application's `population` no longer resolves to a population node
    /// with a `Type` occurrence as admission resolved it.
    #[error("reaches application's population is unresolved")]
    UnresolvedPopulation,
}

impl CheckedPackageV2 {
    /// Reads the population an admitted `quire.op.model.reaches` application
    /// names, as the selector FR-053's `select_population` reads.
    ///
    /// `application` is the node whose body root is the application. An
    /// application without a `population` key names no population and returns
    /// `Ok(None)`; this accessor never infers one. Otherwise the selector holds
    /// the package's own identity, the node id the member holds and the `Type`
    /// occurrence of that node with the least ordinal. The accessor reads no
    /// maximum and charges no work. On failure this returns one typed error and
    /// no partial selector.
    pub fn reaches_population(
        &self,
        application: &CheckedNodeId,
    ) -> Result<Option<CheckedSelectedPopulationSelector>, CheckedReachesPopulationError> {
        use CheckedReachesPopulationError as Error;

        let operation = self
            .reaches_application(application)
            .map_err(|lookup| match lookup {
                ReachesLookup::Unknown => Error::UnknownNode,
                ReachesLookup::NotReaches => Error::NotReachesApplication,
            })?;
        let Some(named) = operation.pointer("/member/population") else {
            return Ok(None);
        };
        let binding: CheckedNodeId =
            serde_json::from_value(named.clone()).map_err(|_| Error::UnresolvedPopulation)?;
        let population = self
            .node(&binding)
            .filter(|_| {
                self.node_kind(&binding)
                    == Some(CheckedNodeKind::BoundedDomain(
                        BoundedDomainForm::ModelPopulation,
                    ))
            })
            .ok_or(Error::UnresolvedPopulation)?;
        let occurrence = least_type_occurrence(population)
            .cloned()
            .ok_or(Error::UnresolvedPopulation)?;
        Ok(Some(CheckedSelectedPopulationSelector {
            package_name: self.name().into(),
            binding,
            occurrence,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::relationship_end::tests::{admitted, raw_id};
    use crate::checked_package::v2::CheckedSemanticNodeV2;
    use ix_trace_rs::trace;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    type Mutation = Box<dyn Fn(&mut CheckedPackageV2, &BTreeMap<char, CheckedNodeId>)>;

    /// The admitted reaches package with a population node added to its graph
    /// after admission, so that `population` mutations can be applied to an
    /// application that names a real population.
    fn naming_a_population() -> (
        CheckedPackageV2,
        BTreeMap<char, CheckedNodeId>,
        CheckedNodeId,
    ) {
        let (mut package, ids) = admitted();
        let population: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id": raw_id('e'), "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "bounded_domain", "semantic_form": "model_population",
            "semantic_type": raw_id('e'), "dependencies": [],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "body": {"term": "aggregate", "members": []},
        }))
        .expect("a population node");
        let id = population.node_id.clone();
        package
            .node_index
            .insert(id.clone(), package.wire.semantic_graph.nodes.len());
        package.wire.semantic_graph.nodes.push(population);
        package
            .kinds
            .push(super::super::CheckedNodeKind::BoundedDomain(
                super::super::BoundedDomainForm::ModelPopulation,
            ));
        package
            .terms
            .push(super::super::CheckedTerm::Aggregate { members: vec![] });
        let position = package.node_index[&ids[&'7']];
        let super::super::CheckedTerm::Application { operation, .. } = &mut package.terms[position]
        else {
            panic!("admitted reaches application term");
        };
        operation.record_mut()["member"]["population"] = json!(id);
        (package, ids, id)
    }

    /// Post-admission mutations of the `population` key refuse
    /// `UnresolvedPopulation`; the unmutated link resolves.
    ///
    #[trace("TC-048", "FR-038-AC-271")]
    #[test]
    fn tc_048_post_admission_mutations_of_the_population_refuse_unresolved_population() {
        let (package, ids, population) = naming_a_population();
        let selector = package
            .reaches_population(&ids[&'7'])
            .expect("the link resolves")
            .expect("a selector");
        assert_eq!(selector.binding, population);

        let mutations: [(&str, Mutation); 5] = [
            (
                "not a node id",
                Box::new(|package, ids| set_population(package, ids, json!(7))),
            ),
            (
                "null",
                Box::new(|package, ids| set_population(package, ids, Value::Null)),
            ),
            (
                "absent from the graph",
                Box::new(|package, ids| set_population(package, ids, raw_id('f'))),
            ),
            (
                "not a population node",
                Box::new(|package, ids| set_population(package, ids, json!(ids[&'a']))),
            ),
            (
                "population node with its Type occurrences removed",
                Box::new(|package, _| {
                    let node = package.node_index[&raw_node_id('e')];
                    package.wire.semantic_graph.nodes[node].occurrences.clear();
                }),
            ),
        ];
        for (label, mutate) in mutations {
            let (mut package, ids, _) = naming_a_population();
            mutate(&mut package, &ids);
            assert_eq!(
                package.reaches_population(&ids[&'7']),
                Err(CheckedReachesPopulationError::UnresolvedPopulation),
                "{label}"
            );
        }
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn reaches_population_uses_retained_kind_after_raw_node_tag_changes() {
        let (mut package, ids, population) = naming_a_population();
        let position = package.node_index[&population];
        package.wire.semantic_graph.nodes[position].node_tag = "scalar_type".into();
        let selected = package
            .reaches_population(&ids[&'7'])
            .expect("retained population kind")
            .expect("selected population");
        assert_eq!(selected.binding, population);
    }

    fn raw_node_id(digit: char) -> CheckedNodeId {
        serde_json::from_value(raw_id(digit)).expect("node id")
    }

    fn set_population(
        package: &mut CheckedPackageV2,
        ids: &BTreeMap<char, CheckedNodeId>,
        value: Value,
    ) {
        let position = package.node_index[&ids[&'7']];
        let super::super::CheckedTerm::Application { operation, .. } = &mut package.terms[position]
        else {
            panic!("admitted reaches application term");
        };
        operation.record_mut()["member"]["population"] = value;
    }
}
