// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Package-private reached-graph prerequisite for selected type closure.
//! These IDs are not declaration or Exact facts and cannot build an environment.

use super::*;
use crate::checked_package::v2::native_join_budget::{
    read_charge, read_node_storage, NativeReadDenied,
};
use crate::checked_package::v2::typed_terms::{
    SelectedTermEdge, SelectedTermTraversal, SelectedTermVisit,
};
use crate::checked_package::v2::{
    NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum EdgeRole {
    Dependency,
    SemanticType,
    TermReference,
    TermType,
    TermOperationMember,
    TermDependency,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SelectedGraphWalk {
    pub(super) nodes: Vec<CheckedNodeId>,
    pub(super) work: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SelectedGraphWalkErrorCode {
    UnknownNode,
    ForeignDependency,
    UnsupportedTerm,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SelectedGraphWalkError {
    pub(super) code: SelectedGraphWalkErrorCode,
    pub(super) at: CheckedNodeId,
    pub(super) work: u64,
    pub(super) attempted_work: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum RuntimeWalkError {
    Semantic(SelectedGraphWalkError),
    Native(NativeReadDenied),
}
impl From<NativeReadDenied> for RuntimeWalkError {
    fn from(error: NativeReadDenied) -> Self {
        Self::Native(error)
    }
}

struct Walk<'a, 'b> {
    package: &'a CheckedPackageV2,
    budget: &'b mut NativeJoinBudget,
    work: u64,
    pending: Vec<(CheckedNodeId, NativeJoinLocusSource)>,
    seen_nodes: BTreeSet<CheckedNodeId>,
    seen_edges: BTreeSet<(&'a CheckedNodeId, EdgeRole, &'a CheckedNodeId)>,
    nodes: Vec<CheckedNodeId>,
}

impl<'a> Walk<'a, '_> {
    fn error(&mut self, code: SelectedGraphWalkErrorCode, at: &CheckedNodeId) -> RuntimeWalkError {
        if let Err(error) = read_node_storage(self.budget, at) {
            return error.into();
        }
        RuntimeWalkError::Semantic(SelectedGraphWalkError {
            code,
            at: at.clone(),
            work: self.work,
            attempted_work: 0,
        })
    }

    fn charge(&mut self, _at: &CheckedNodeId) -> Result<(), RuntimeWalkError> {
        self.budget.set_locus_form("monitor_graph_inspection");
        read_charge(self.budget, NativeJoinChargeKind::Work, 1)?;
        self.work += 1;
        Ok(())
    }
    fn retain_node(&mut self, node: &CheckedNodeId) -> Result<(), RuntimeWalkError> {
        read_node_storage(self.budget, node)?;
        read_charge(self.budget, NativeJoinChargeKind::Storage, 1)?;
        Ok(())
    }

    fn seed(&mut self, id: &CheckedNodeId) -> Result<(), RuntimeWalkError> {
        self.charge(id)?;
        self.retain_node(id)?;
        self.pending.push((
            id.clone(),
            self.budget
                .active_locus()
                .expect("monitor seed context")
                .source,
        ));
        Ok(())
    }

    fn edge(
        &mut self,
        from: &'a CheckedNodeId,
        role: EdgeRole,
        to: &'a CheckedNodeId,
    ) -> Result<(), RuntimeWalkError> {
        // The enclosing admitted node/term inspection covers this borrowed
        // identity comparison. A repeated edge creates no owning key or charge.
        let edge = (from, role, to);
        if self.seen_edges.contains(&edge) {
            return Ok(());
        }
        self.charge(to)?;
        read_charge(self.budget, NativeJoinChargeKind::Storage, 1)?;
        self.seen_edges.insert(edge);
        self.retain_node(to)?;
        self.pending.push((
            to.clone(),
            self.budget
                .active_locus()
                .expect("monitor edge context")
                .source,
        ));
        Ok(())
    }

    fn visit(&mut self) -> Result<(), RuntimeWalkError> {
        while let Some((id, source)) = self.pending.pop() {
            self.budget.set_active_locus(Some(NativeJoinLocus {
                source,
                form: "monitor_graph_node",
            }));
            self.charge(&id)?;
            self.retain_node(&id)?;
            if !self.seen_nodes.insert(id.clone()) {
                continue;
            }
            self.charge(&id)?;
            let Some((position, node)) = self.package.node_with_position(&id) else {
                let code = if id.package.as_ref() == self.package.name() {
                    SelectedGraphWalkErrorCode::UnknownNode
                } else {
                    SelectedGraphWalkErrorCode::ForeignDependency
                };
                return Err(self.error(code, &id));
            };
            self.budget
                .set_locus_source(NativeJoinLocusSource::GraphNode(position));
            self.retain_node(&id)?;
            self.nodes.push(id.clone());
            let canonical = &node.node_id;
            for dependency in &node.dependencies {
                self.edge(canonical, EdgeRole::Dependency, dependency)?;
            }
            self.edge(canonical, EdgeRole::SemanticType, &node.semantic_type)?;
            let term = self
                .package
                .term(&id)
                .expect("admitted graph node has a retained typed term");
            term.visit_selected_edges_charged(canonical, |event| {
                let event = match event {
                    SelectedTermTraversal::Step => return Ok(()),
                    SelectedTermTraversal::StackStorage(amount) => {
                        self.budget.set_locus_form("monitor_term_stack");
                        return read_charge(self.budget, NativeJoinChargeKind::Storage, amount)
                            .map_err(Into::into);
                    }
                    SelectedTermTraversal::Visit(event) => event,
                };
                match event {
                    SelectedTermVisit::Term => self.charge(&id),
                    SelectedTermVisit::Edge(kind, target) => {
                        let role = match kind {
                            SelectedTermEdge::Reference => EdgeRole::TermReference,
                            SelectedTermEdge::Type => EdgeRole::TermType,
                            SelectedTermEdge::OperationMember => EdgeRole::TermOperationMember,
                            SelectedTermEdge::Dependency(package) => {
                                if package != self.package.name() {
                                    self.charge(target)?;
                                    return Err(self.error(
                                        SelectedGraphWalkErrorCode::ForeignDependency,
                                        target,
                                    ));
                                }
                                EdgeRole::TermDependency
                            }
                            SelectedTermEdge::Unsupported => {
                                self.charge(target)?;
                                return Err(
                                    self.error(SelectedGraphWalkErrorCode::UnsupportedTerm, target)
                                );
                            }
                        };
                        self.edge(canonical, role, target)
                    }
                }
            })?;
        }
        Ok(())
    }
}

impl CheckedMonitorPlanRef<'_> {
    /// Internal prerequisite only: reached graph IDs are not a type closure.
    pub(super) fn selected_graph_walk(
        &self,
        budget: &mut NativeJoinBudget,
    ) -> Result<SelectedGraphWalk, RuntimeWalkError> {
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::MonitorSeed(0),
            form: "monitor_graph_storage",
        }));
        read_charge(budget, NativeJoinChargeKind::Storage, 4)?;
        let before_work = budget.consumed().work;
        let mut walk = Walk {
            package: self.package,
            budget,
            work: 0,
            pending: Vec::new(),
            seen_nodes: BTreeSet::new(),
            seen_edges: BTreeSet::new(),
            nodes: Vec::new(),
        };
        let mut seed_index = 0;
        for seed in [
            self.state(),
            self.inputs(),
            self.scope(),
            self.results(),
            self.init().node_id(),
            self.on_event().node_id(),
            self.on_watermark().node_id(),
            self.on_gap().node_id(),
        ] {
            walk.budget.set_active_locus(Some(NativeJoinLocus {
                source: NativeJoinLocusSource::MonitorSeed(seed_index),
                form: "monitor_seed",
            }));
            walk.seed(seed)?;
            seed_index += 1;
        }
        for entry in [
            self.init(),
            self.on_event(),
            self.on_watermark(),
            self.on_gap(),
        ] {
            for ty in entry.parameter_types() {
                walk.budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::MonitorSeed(seed_index),
                    form: "monitor_parameter_seed",
                }));
                walk.seed(ty)?;
                seed_index += 1;
            }
            walk.budget.set_active_locus(Some(NativeJoinLocus {
                source: NativeJoinLocusSource::MonitorSeed(seed_index),
                form: "monitor_result_seed",
            }));
            walk.seed(entry.result_type())?;
            seed_index += 1;
        }
        walk.visit()?;
        Ok(SelectedGraphWalk {
            nodes: walk.nodes,
            work: walk.budget.consumed().work - before_work,
        })
    }
}

#[cfg(all(test, feature = "qspec-conformance"))]
#[allow(unused_imports)] // The shared fixture module serves several test binaries.
#[path = "../../../../../../tests/it/support/checked_package.rs"]
mod fixture_support;

#[cfg(all(test, feature = "qspec-conformance"))]
fn fixtures_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("QUIRE_SPECIFICATION_DIR").expect("QSpec checkout"))
        .join("proposals/checked-package-v2/fixtures")
}

#[cfg(all(test, feature = "qspec-conformance"))]
fn proposal_dir_of(fixtures: &std::path::Path) -> std::path::PathBuf {
    fixtures.join("..")
}

#[cfg(all(test, feature = "qspec-conformance"))]
fn read_json(path: &std::path::Path) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

#[cfg(all(test, feature = "qspec-conformance"))]
#[path = "../../../../../../tests/it/support/monitor_plan.rs"]
mod monitor_plan_fixtures;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "qspec-conformance")]
    use crate::checked_package::v2::{CheckedPackageReadLimits, CheckedPackageV2ReadResult};
    use ix_trace_rs::trace;

    #[cfg(feature = "qspec-conformance")]
    fn admitted_seeded_package(extra: bool) -> CheckedPackageV2 {
        let (mut wire, _) =
            monitor_plan_fixtures::materialize(&monitor_plan_fixtures::recipes()["base"]);
        if extra {
            let integer = monitor_plan_fixtures::find(&wire, "integer");
            monitor_plan_fixtures::constant(&mut wire, &integer, "unrelated");
            monitor_plan_fixtures::finish(&mut wire);
        }
        let document =
            read_json(&proposal_dir_of(&fixtures_dir()).join("domain-package-acme-orders.json"))
                .expect("selected QSpec domain document");
        let mut evidence = fixture_support::evidence_with_document(&wire, "acme/orders", &document);
        evidence
            .insert_extension_definition(
                crate::CheckedArtifactRef {
                    authority: "example".into(),
                    identity: "retention".into(),
                },
                fixture_support::retention_definition(),
            )
            .expect("selected QSpec extension definition");
        let result = CheckedPackageV2::read(
            &fixture_support::canonical(&wire),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        let CheckedPackageV2ReadResult::Admitted(package) = result else {
            panic!("QSpec monitor recipe admits: {result:?}")
        };
        *package
    }

    #[cfg(feature = "qspec-conformance")]
    fn id_for(package: &CheckedPackageV2, tag: &str, form: &str) -> CheckedNodeId {
        package
            .resolved_graph()
            .nodes
            .iter()
            .find(|node| node.node_tag.as_ref() == tag && node.semantic_form.as_ref() == form)
            .expect("fixture node")
            .node_id
            .clone()
    }

    /// Trace: FR-357-AC-12.
    #[cfg(feature = "qspec-conformance")]
    #[test]
    #[ignore = "requires QUIRE_SPECIFICATION_DIR checkout; run in QSpec conformance lane"]
    fn selected_walk_is_bounded_deduplicated_and_clone_stable() {
        let package = admitted_seeded_package(false);
        let plan = package.monitor_plans().next().expect("internal seed plan");
        let complete = plan
            .selected_graph_walk(&mut NativeJoinBudget::new(u64::MAX, u64::MAX))
            .expect("selected walk");
        let application = plan.init().node_id().clone();
        let frame = id_for(&package, "state", "frame");
        assert!(complete.nodes.contains(&application));
        assert!(!complete.nodes.contains(&frame));
        assert!(complete.work > complete.nodes.len() as u64);
        assert_eq!(
            plan.selected_graph_walk(&mut NativeJoinBudget::new(complete.work, u64::MAX)),
            Ok(complete.clone())
        );
        let denied = plan
            .selected_graph_walk(&mut NativeJoinBudget::new(complete.work - 1, u64::MAX))
            .unwrap_err();
        assert!(matches!(denied, RuntimeWalkError::Native(_)));
        assert_eq!(
            package
                .clone()
                .monitor_plans()
                .next()
                .unwrap()
                .selected_graph_walk(&mut NativeJoinBudget::new(complete.work, u64::MAX)),
            Ok(complete)
        );
        let extra = admitted_seeded_package(true);
        assert_eq!(
            extra
                .monitor_plans()
                .next()
                .unwrap()
                .selected_graph_walk(&mut NativeJoinBudget::new(u64::MAX, u64::MAX)),
            package
                .monitor_plans()
                .next()
                .unwrap()
                .selected_graph_walk(&mut NativeJoinBudget::new(u64::MAX, u64::MAX))
        );
    }

    /// Trace: FR-357-AC-12. Corrupt a reached typed term after strict admission
    /// to exercise the walk's foreign-reference guard at its native locus.
    #[cfg(feature = "qspec-conformance")]
    #[test]
    #[ignore = "requires QUIRE_SPECIFICATION_DIR checkout; run in QSpec conformance lane"]
    fn selected_walk_refuses_reached_foreign_typed_dependency() {
        let mut package = admitted_seeded_package(false);
        let entry = package
            .monitor_plans()
            .next()
            .unwrap()
            .init()
            .node_id()
            .clone();
        let position = package
            .resolved_graph()
            .nodes
            .iter()
            .position(|node| node.node_id == entry)
            .unwrap();
        let mut foreign = entry;
        foreign.package = "other/package".into();
        std::sync::Arc::make_mut(&mut package.wire)
            .semantic_graph
            .nodes[position]
            .body = CheckedTerm::DependencyReference {
            package: "other/package".into(),
            node: foreign.clone(),
        };
        let error = package
            .monitor_plans()
            .next()
            .unwrap()
            .selected_graph_walk(&mut NativeJoinBudget::new(u64::MAX, u64::MAX))
            .unwrap_err();
        let RuntimeWalkError::Semantic(error) = error else {
            panic!("expected semantic refusal")
        };
        assert_eq!(error.code, SelectedGraphWalkErrorCode::ForeignDependency);
        assert_eq!(error.at, foreign);
        assert_eq!(error.attempted_work, 0);
    }

    /// Trace: FR-357-AC-12. Specialized frames remain a typed stop for this
    /// prerequisite even when a caller reaches one from a selected entry.
    #[cfg(feature = "qspec-conformance")]
    #[test]
    #[ignore = "requires QUIRE_SPECIFICATION_DIR checkout; run in QSpec conformance lane"]
    fn selected_walk_refuses_specialized_frame() {
        let mut package = admitted_seeded_package(false);
        let entry = package
            .monitor_plans()
            .next()
            .unwrap()
            .init()
            .node_id()
            .clone();
        let position = package
            .resolved_graph()
            .nodes
            .iter()
            .position(|node| node.node_id == entry)
            .unwrap();
        std::sync::Arc::make_mut(&mut package.wire)
            .semantic_graph
            .nodes[position]
            .body = CheckedTerm::Frame {
            modifies: Vec::new(),
            creates: Vec::new(),
            deletes: Vec::new(),
            deletes_scoped: Vec::new(),
        };
        let error = package
            .monitor_plans()
            .next()
            .unwrap()
            .selected_graph_walk(&mut NativeJoinBudget::new(u64::MAX, u64::MAX))
            .unwrap_err();
        let RuntimeWalkError::Semantic(error) = error else {
            panic!("expected semantic refusal")
        };
        assert_eq!(error.code, SelectedGraphWalkErrorCode::UnsupportedTerm);
        assert_eq!(error.at, entry);
    }

    /// Trace: FR-357-AC-12. A second identical reference visits one more
    /// term, while the already visited graph edge costs no additional work.
    #[cfg(feature = "qspec-conformance")]
    #[test]
    #[ignore = "requires QUIRE_SPECIFICATION_DIR checkout; run in QSpec conformance lane"]
    fn repeated_reference_edge_is_charged_once() {
        let mut single = admitted_seeded_package(false);
        let entry = single
            .monitor_plans()
            .next()
            .unwrap()
            .init()
            .node_id()
            .clone();
        let target = single.monitor_plans().next().unwrap().state().clone();
        let position = single
            .resolved_graph()
            .nodes
            .iter()
            .position(|node| node.node_id == entry)
            .unwrap();
        std::sync::Arc::make_mut(&mut single.wire)
            .semantic_graph
            .nodes[position]
            .body = CheckedTerm::aggregate(vec![CheckedTerm::reference(target.clone())]);
        let mut repeated = single.clone();
        std::sync::Arc::make_mut(&mut repeated.wire)
            .semantic_graph
            .nodes[position]
            .body = CheckedTerm::aggregate(vec![
            CheckedTerm::reference(target.clone()),
            CheckedTerm::reference(target),
        ]);
        let once = single
            .monitor_plans()
            .next()
            .unwrap()
            .selected_graph_walk(&mut NativeJoinBudget::new(u64::MAX, u64::MAX))
            .unwrap();
        let twice = repeated
            .monitor_plans()
            .next()
            .unwrap()
            .selected_graph_walk(&mut NativeJoinBudget::new(u64::MAX, u64::MAX))
            .unwrap();
        assert_eq!(twice.nodes, once.nodes);
        assert_eq!(twice.work, once.work + 1);
    }

    /// Trace: FR-357-AC-12. A wide ordinary body stops on its first
    /// uncharged child; the visitor does not need a worklist entry per child.
    #[trace("TC-470", "FR-357-AC-3")]
    #[test]
    fn wide_ordinary_term_stops_at_low_limit() {
        let id = CheckedNodeId {
            package: "test/package".into(),
            path: vec![crate::CheckedNodePathSegment::Name("wide".into())],
            ordinal: 0,
        };
        let term = CheckedTerm::aggregate(
            (0..100_000)
                .map(|_| CheckedTerm::reference(id.clone()))
                .collect(),
        );
        let mut work = 0;
        let result = term.visit_selected_edges(&id, |_| {
            if work == 2 {
                return Err(());
            }
            work += 1;
            Ok(())
        });
        assert_eq!(result, Err(()));
        assert_eq!(work, 2);
    }

    /// Trace: FR-357-AC-12. Flat nominal IDs also stop at the first refused
    /// edge without collecting the other IDs.
    #[test]
    fn wide_nominal_term_stops_at_first_edge() {
        let id = CheckedNodeId {
            package: "test/package".into(),
            path: vec![crate::CheckedNodePathSegment::Name("dimension".into())],
            ordinal: 0,
        };
        let body = crate::NominalBody::try_dimension(
            false,
            (0..10_000)
                .map(|ordinal| {
                    let mut dimension_node_id = id.clone();
                    dimension_node_id.ordinal = ordinal;
                    crate::DimensionTerm {
                        dimension_node_id,
                        exponent: "1".into(),
                    }
                })
                .collect(),
        )
        .unwrap();
        let term = CheckedTerm::NominalBody(Box::new(body));
        let mut edges = 0;
        let result = term.visit_selected_edges(&id, |event| match event {
            SelectedTermVisit::Term => Ok(()),
            SelectedTermVisit::Edge(SelectedTermEdge::Reference, _) => {
                edges += 1;
                Err(())
            }
            SelectedTermVisit::Edge(_, _) => panic!("unexpected nominal edge"),
        });
        assert_eq!(result, Err(()));
        assert_eq!(edges, 1);
    }

    /// Trace: FR-357-AC-12.
    #[trace("TC-470", "FR-357-AC-3")]
    #[test]
    fn ordinary_typed_term_walk_uses_an_iterative_stack() {
        let id = CheckedNodeId {
            package: "test/package".into(),
            path: vec![crate::CheckedNodePathSegment::Name("deep".into())],
            ordinal: 0,
        };
        let mut term = CheckedTerm::reference(id.clone());
        for _ in 0..4096 {
            term = CheckedTerm::aggregate(vec![term]);
        }
        let mut terms = 0;
        let mut edges = 0;
        term.visit_selected_edges(&id, |event| {
            match event {
                SelectedTermVisit::Term => terms += 1,
                SelectedTermVisit::Edge(SelectedTermEdge::Reference, target) => {
                    assert_eq!(target, &id);
                    edges += 1;
                }
                SelectedTermVisit::Edge(_, _) => panic!("unexpected typed edge"),
            }
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(terms, 4097);
        assert_eq!(edges, 1);
        // The visit is iterative; dismantle the artificial term iteratively too.
        while let CheckedTerm::Aggregate { mut members } = term {
            term = members.pop().unwrap();
        }
    }
}
