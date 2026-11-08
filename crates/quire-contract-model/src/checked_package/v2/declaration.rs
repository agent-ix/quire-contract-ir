//! FR-362: the implementation guarantee and component composition bodies of
//! QSpec FR-490 and FR-491, and the declaration step of the reader order.
//!
//! A `correspondence`/`implementation_guarantee` or
//! `correspondence`/`component_composition` node's `body` is a flat shape of
//! node keys, identifiers, Rust paths and strings, with no application,
//! aggregate, binding or literal term, so it is scanned like the abstraction
//! relation body and read here.
//!
//! [`validate_declarations`] runs after the abstraction step: every
//! implementation guarantee node in ascending structural node-id order, then
//! every component composition node. For one node the checks run in this
//! order and the first failing check is its defect:
//!
//! 0. **Shape.** The body is the closed shape of its form; else
//!    `invalid_semantic_graph` at the body.
//! 1. **Identity.** `semantic_type` is the node itself and `dependencies`
//!    are exactly the body's targets; else `invalid_semantic_graph` at the node.
//! 2. **Order.** The canonical order of every array; else
//!    `invalid_semantic_graph` at the array.
//! 3. **Targets,** in body order: the join to a declared dependency of the
//!    admitted kind (`missing_declaration`/`missing-name` when it is not a
//!    declared dependency, `invalid_model_binding`/`malformed-declaration`
//!    for another kind).
//! 4. **Members,** in body order: the operation each `progress` and
//!    scheduling entry names, then the Rust paths, then a repeated key
//!    (`invalid_model_binding`/`conflicting-binding`).
//!
//! A composition target that is no node of this graph is a component's own,
//! in another package, and is joined through the dependency packages (QSpec
//! FR-491), not through this graph, so it is checked only for well-formedness
//! and is not listed in `dependencies`, which name nodes of this graph.

use super::abstraction::{ascending, conflict, model_refusal, Site};
use super::frame::{join, StepGraph};
use super::identity::is_identifier;
use super::model_members::{ModelOwners, ModelRefusal};
use super::rust_spelling::is_rust_path;
use super::state::resolve_operation;
use super::temporal_binding::profile_law;
use super::{
    BodyTerm, CheckedEnvironmentSupply, CheckedGuaranteeEnvironment, CheckedNodeKind,
    CheckedProgressMeasure, CheckedScheduling, CheckedTerm, CorrespondenceForm, FunctionForm,
    ModelForm, RawSemanticNodeV2 as CheckedSemanticNodeV2, TemporalForm, TemporalProfile,
    WorkMeter,
};
use crate::checked_package::common::{body_term, exact_members, ValidationFailure};
use crate::checked_package::shared::{CheckedNodeId, JsonPointer};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The largest `bound` a progress entry names: the greatest integer a JSON
/// number carries exactly.
const MAX_BOUND: u64 = 9_007_199_254_740_991;

fn guarantee_members() -> &'static [&'static str] {
    &[
        "term",
        "clause",
        "relation",
        "initial",
        "progress",
        "environment",
    ]
}

fn composition_members() -> &'static [&'static str] {
    &[
        "term",
        "rule",
        "components",
        "guarantees",
        "relies",
        "liveness_dependencies",
        "supplies",
        "environment",
    ]
}

/// Reads one node's body as its closed shape; `None` is a shape defect.
fn read_body(node: &CheckedSemanticNodeV2, form: CorrespondenceForm) -> Option<CheckedTerm> {
    let (term, members) = match form {
        CorrespondenceForm::ImplementationGuarantee => {
            (BodyTerm::ImplementationGuarantee, guarantee_members())
        }
        _ => (BodyTerm::ComponentComposition, composition_members()),
    };
    let Value::Object(object) = &node.body else {
        return None;
    };
    if !exact_members(object, members) || body_term(&node.body) != Some(term) {
        return None;
    }
    CheckedTerm::decode_admitted(&node.body).ok()
}

/// The environment entries' shape: identifier names, a non-empty supplier, an
/// identifier operation.
fn environment_shape(entries: &[CheckedGuaranteeEnvironment]) -> bool {
    entries.iter().all(|entry| {
        is_identifier(&entry.name)
            && !entry.supplier.is_empty()
            && match &entry.supplies {
                CheckedEnvironmentSupply::Fairness(scheduling) => {
                    is_identifier(&scheduling.operation) && scheduling.context.is_well_formed()
                }
                CheckedEnvironmentSupply::Assumption(node) => node.is_well_formed(),
            }
    })
}

fn path_shape(path: &[Box<str>]) -> bool {
    !path.is_empty() && path.iter().all(|segment| !segment.is_empty())
}

/// Whether every value the closed shape bounds holds on `term`.
fn shape_holds(term: &CheckedTerm) -> bool {
    match term {
        CheckedTerm::ImplementationGuarantee {
            clause,
            relation,
            initial,
            progress,
            environment,
        } => {
            clause.is_well_formed()
                && relation.is_well_formed()
                && path_shape(initial)
                && progress.iter().all(|entry| {
                    entry.context.is_well_formed()
                        && is_identifier(&entry.operation)
                        && match &entry.measure {
                            CheckedProgressMeasure::Decreases(path) => path_shape(path),
                            CheckedProgressMeasure::Bound(count) => (1..=MAX_BOUND).contains(count),
                        }
                })
                && environment_shape(environment)
        }
        CheckedTerm::ComponentComposition {
            components,
            guarantees,
            relies,
            liveness_dependencies,
            supplies,
            environment,
            ..
        } => {
            let clause = |entry: &super::CheckedCompositionClause| {
                is_identifier(&entry.component) && entry.clause.is_well_formed()
            };
            components.len() >= 2
                && components.iter().all(|entry| {
                    is_identifier(&entry.name)
                        && !entry.package.is_empty()
                        && entry.subject.is_well_formed()
                })
                && guarantees.iter().all(clause)
                && relies
                    .iter()
                    .chain(liveness_dependencies)
                    .all(|entry| clause(&entry.guarantee) && clause(&entry.on))
                && supplies
                    .iter()
                    .all(|entry| is_identifier(&entry.operation) && entry.context.is_well_formed())
                && environment_shape(environment)
        }
        _ => false,
    }
}

/// Every target of a body, in body order.
fn targets(term: &CheckedTerm) -> Vec<&CheckedNodeId> {
    let mut found = Vec::new();
    fn environment<'a>(
        entries: &'a [CheckedGuaranteeEnvironment],
        found: &mut Vec<&'a CheckedNodeId>,
    ) {
        for entry in entries {
            match &entry.supplies {
                CheckedEnvironmentSupply::Fairness(scheduling) => found.push(&scheduling.context),
                CheckedEnvironmentSupply::Assumption(node) => found.push(node),
            }
        }
    }
    match term {
        CheckedTerm::ImplementationGuarantee {
            clause,
            relation,
            progress,
            environment: entries,
            ..
        } => {
            found.push(clause);
            found.push(relation);
            found.extend(progress.iter().map(|entry| &entry.context));
            environment(entries, &mut found);
        }
        CheckedTerm::ComponentComposition {
            components,
            guarantees,
            relies,
            liveness_dependencies,
            supplies,
            environment: entries,
            ..
        } => {
            found.extend(components.iter().map(|entry| &entry.subject));
            found.extend(guarantees.iter().map(|entry| &entry.clause));
            for premise in relies.iter().chain(liveness_dependencies) {
                found.push(&premise.guarantee.clause);
                found.push(&premise.on.clause);
            }
            found.extend(supplies.iter().map(|entry| &entry.context));
            environment(entries, &mut found);
        }
        _ => {}
    }
    found
}

/// The unique, ascending set a node's `dependencies` must equal: the targets
/// that are nodes of this graph. A target in a component's package is no node
/// of this graph and is named by the body alone.
fn dependency_set(term: &CheckedTerm, graph: &StepGraph<'_, '_>) -> Vec<CheckedNodeId> {
    targets(term)
        .into_iter()
        .filter(|id| graph.index.contains_key(*id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .cloned()
        .collect()
}

/// A `context` target: a `model`/`object_type` node.
fn is_object_type(kind: CheckedNodeKind) -> bool {
    kind == CheckedNodeKind::Model(ModelForm::ObjectType)
}

fn is_predicate(kind: CheckedNodeKind) -> bool {
    kind == CheckedNodeKind::Function(FunctionForm::Predicate)
}

fn is_clause(kind: CheckedNodeKind) -> bool {
    kind == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
}

fn is_subject(kind: CheckedNodeKind) -> bool {
    matches!(kind, CheckedNodeKind::Model(_))
}

/// The declaration step. See the module documentation for the order.
///
/// Implements: FR-362.
pub(super) fn validate_declarations(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StepGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    for form in [
        CorrespondenceForm::ImplementationGuarantee,
        CorrespondenceForm::ComponentComposition,
    ] {
        for (position, node) in graph.each(CheckedNodeKind::Correspondence(form)) {
            let site = Site { node, position };
            check_node(&site, form, &graph, meter)?;
        }
    }
    Ok(())
}

fn check_node(
    site: &Site<'_>,
    form: CorrespondenceForm,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    let body_at = site.body_at();
    let Some(term) = read_body(node, form).filter(shape_holds) else {
        return Err(site.invalid(body_at));
    };
    meter.charge(targets(&term).len() as u64 + 1, || body_at.clone())?;
    if node.semantic_type != node.node_id || node.dependencies != dependency_set(&term, graph) {
        return Err(site.invalid(site.node_at()));
    }
    check_order(site, &term)?;
    check_targets(site, &term, graph)?;
    check_members(site, &term, graph, meter)
}

/// Step 2: every array in its canonical order. A non-strict order: equal keys
/// are in order, and the member check refuses them.
fn check_order(site: &Site<'_>, term: &CheckedTerm) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let out_of_order = |member: &str| Err(site.invalid(body_at.clone().key(member)));
    match term {
        CheckedTerm::ImplementationGuarantee {
            progress,
            environment,
            ..
        } => {
            if !ascending(
                progress
                    .iter()
                    .map(|entry| (&entry.context, entry.operation.as_ref())),
            ) {
                return out_of_order("progress");
            }
            if !ascending(environment.iter().map(|entry| entry.name.as_ref())) {
                return out_of_order("environment");
            }
        }
        CheckedTerm::ComponentComposition {
            components,
            guarantees,
            relies,
            liveness_dependencies,
            environment,
            ..
        } => {
            if !ascending(components.iter().map(|entry| entry.name.as_ref())) {
                return out_of_order("components");
            }
            if !ascending(
                guarantees
                    .iter()
                    .map(|entry| (entry.component.as_ref(), &entry.clause)),
            ) {
                return out_of_order("guarantees");
            }
            for (member, premises) in [
                ("relies", relies),
                ("liveness_dependencies", liveness_dependencies),
            ] {
                if !ascending(premises.iter().map(|entry| {
                    (
                        entry.guarantee.component.as_ref(),
                        &entry.guarantee.clause,
                        entry.on.component.as_ref(),
                        &entry.on.clause,
                    )
                })) {
                    return out_of_order(member);
                }
            }
            if !ascending(environment.iter().map(|entry| entry.name.as_ref())) {
                return out_of_order("environment");
            }
        }
        _ => {}
    }
    Ok(())
}

/// Step 3: every target joins a declared dependency of its admitted kind.
fn check_targets(
    site: &Site<'_>,
    term: &CheckedTerm,
    graph: &StepGraph<'_, '_>,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let join_to = |at: JsonPointer,
                   id: &CheckedNodeId,
                   admits: fn(CheckedNodeKind) -> bool|
     -> Result<Option<&CheckedSemanticNodeV2>, ValidationFailure> {
        join(site.node, id, admits, graph)
            .map(|(target, _)| Some(target))
            .map_err(|refusal| model_refusal(refusal, at, id))
    };
    let environment = |entries: &[CheckedGuaranteeEnvironment], at: &JsonPointer| {
        for (index, entry) in entries.iter().enumerate() {
            let at = at.clone().index(index).key("supplies");
            match &entry.supplies {
                CheckedEnvironmentSupply::Fairness(scheduling) => {
                    join_to(at, &scheduling.context, is_object_type)?;
                }
                CheckedEnvironmentSupply::Assumption(node) => {
                    join_to(at, node, is_predicate)?;
                }
            }
        }
        Ok(())
    };
    match term {
        CheckedTerm::ImplementationGuarantee {
            clause,
            relation,
            progress,
            environment: entries,
            ..
        } => {
            let target = join_to(body_at.clone().key("clause"), clause, is_clause)?;
            if target.is_none_or(|clause| !is_infinite_trace(clause)) {
                return Err(model_refusal(
                    ModelRefusal::malformed(),
                    body_at.clone().key("clause"),
                    clause,
                ));
            }
            join_to(body_at.clone().key("relation"), relation, |kind| {
                kind == CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
            })?;
            for (index, entry) in progress.iter().enumerate() {
                let at = body_at.clone().key("progress").index(index);
                join_to(at, &entry.context, is_object_type)?;
            }
            environment(entries, &body_at.key("environment"))
        }
        CheckedTerm::ComponentComposition {
            components,
            guarantees,
            relies,
            liveness_dependencies,
            supplies,
            environment: entries,
            ..
        } => {
            let local = |id: &CheckedNodeId| graph.index.contains_key(id);
            let component =
                |at: JsonPointer, id: &CheckedNodeId, admits: fn(CheckedNodeKind) -> bool| {
                    if local(id) {
                        join_to(at, id, admits).map(|_| ())
                    } else {
                        Ok(())
                    }
                };
            for (index, entry) in components.iter().enumerate() {
                let at = body_at.clone().key("components").index(index);
                component(at, &entry.subject, is_subject)?;
            }
            for (index, entry) in guarantees.iter().enumerate() {
                let at = body_at.clone().key("guarantees").index(index);
                component(at, &entry.clause, is_clause)?;
            }
            for (member, premises) in [
                ("relies", relies),
                ("liveness_dependencies", liveness_dependencies),
            ] {
                for (index, entry) in premises.iter().enumerate() {
                    let at = body_at.clone().key(member).index(index);
                    component(
                        at.clone().key("guarantee"),
                        &entry.guarantee.clause,
                        is_clause,
                    )?;
                    component(at.key("on"), &entry.on.clause, is_clause)?;
                }
            }
            for (index, entry) in supplies.iter().enumerate() {
                let at = body_at.clone().key("supplies").index(index);
                component(at, &entry.context, is_object_type)?;
            }
            environment(entries, &body_at.key("environment"))
        }
        _ => Ok(()),
    }
}

/// Whether a `temporal_clause` node selects the infinite-trace profile.
fn is_infinite_trace(clause: &CheckedSemanticNodeV2) -> bool {
    profile_law(clause).and_then(|law| TemporalProfile::from_wire(&law.identity))
        == Some(TemporalProfile::InfiniteTrace)
}

/// The operation `scheduling` names must be an operation of its context.
fn check_operation(
    context: &CheckedNodeId,
    operation: &str,
    at: JsonPointer,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some((node, kind)) = graph.node(context) else {
        return Ok(());
    };
    match resolve_operation(node, kind, operation, graph, meter)? {
        Err(refusal) => Err(model_refusal(refusal, at, context)),
        Ok(_) => Ok(()),
    }
}

/// Step 4: operations, Rust paths and repeated keys, entries in body order.
fn check_members(
    site: &Site<'_>,
    term: &CheckedTerm,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let malformed = |at: JsonPointer, locus: &CheckedNodeId| {
        model_refusal(ModelRefusal::malformed(), at, locus)
    };
    let scheduled = |scheduling: &CheckedScheduling, at: JsonPointer, meter: &mut WorkMeter| {
        check_operation(&scheduling.context, &scheduling.operation, at, graph, meter)
    };
    match term {
        CheckedTerm::ImplementationGuarantee {
            clause,
            initial,
            progress,
            environment,
            ..
        } => {
            if !is_rust_path(initial) {
                return Err(malformed(body_at.key("initial"), clause));
            }
            let mut seen = BTreeSet::new();
            for (index, entry) in progress.iter().enumerate() {
                let at = body_at.clone().key("progress").index(index);
                check_operation(&entry.context, &entry.operation, at.clone(), graph, meter)?;
                if let CheckedProgressMeasure::Decreases(path) = &entry.measure {
                    if !is_rust_path(path) {
                        return Err(malformed(at.clone(), &entry.context));
                    }
                }
                if !seen.insert((&entry.context, entry.operation.as_ref())) {
                    return Err(conflict(at, &entry.context));
                }
            }
            check_environment(environment, &body_at.key("environment"), &scheduled, meter)
        }
        CheckedTerm::ComponentComposition {
            components,
            supplies,
            environment,
            ..
        } => {
            let mut names = BTreeSet::new();
            let mut subjects = BTreeSet::new();
            for (index, entry) in components.iter().enumerate() {
                let at = body_at.clone().key("components").index(index);
                if !names.insert(entry.name.as_ref())
                    || !subjects.insert((entry.package.as_ref(), &entry.subject))
                {
                    return Err(conflict(at, &entry.subject));
                }
            }
            for (index, entry) in supplies.iter().enumerate() {
                let at = body_at.clone().key("supplies").index(index);
                scheduled(entry, at, meter)?;
            }
            check_environment(environment, &body_at.key("environment"), &scheduled, meter)
        }
        _ => Ok(()),
    }
}

/// The environment entries: the operation a scheduling premise names, then a
/// repeated name.
fn check_environment(
    entries: &[CheckedGuaranteeEnvironment],
    at: &JsonPointer,
    scheduled: &dyn Fn(
        &CheckedScheduling,
        JsonPointer,
        &mut WorkMeter,
    ) -> Result<(), ValidationFailure>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut seen = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let at = at.clone().index(index);
        if let CheckedEnvironmentSupply::Fairness(scheduling) = &entry.supplies {
            scheduled(scheduling, at.clone().key("supplies"), meter)?;
        }
        if !seen.insert(entry.name.as_ref()) {
            let locus = match &entry.supplies {
                CheckedEnvironmentSupply::Fairness(scheduling) => &scheduling.context,
                CheckedEnvironmentSupply::Assumption(node) => node,
            };
            return Err(conflict(at, locus));
        }
    }
    Ok(())
}
