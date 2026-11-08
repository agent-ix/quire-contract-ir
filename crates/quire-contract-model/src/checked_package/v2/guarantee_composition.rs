// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-038: the `correspondence`/`implementation_guarantee` and
//! `correspondence`/`component_composition` node bodies of QSpec FR-490 and
//! FR-491, and the guarantee step of the reader order.
//!
//! Both bodies are flat (node keys, identifiers, Rust paths and fixed
//! spellings), so the per-node body loop validates them as the abstraction
//! relation body is validated ([`super::abstraction::scan_body`]) and this
//! step reads them. The step runs after the abstraction step, guarantee nodes
//! first, then composition nodes, each in ascending structural node-id order.
//! For one node the first failing check is its defect:
//!
//! 0. **Shape.** The body root is not an application and the body is the
//!    closed shape; else `invalid_semantic_graph` at the offending entry, or
//!    at the body.
//! 1. **Identity.** `semantic_type` is the node itself and `dependencies` are
//!    exactly the unique, ascending set of the body's targets; else
//!    `invalid_semantic_graph` at the node.
//! 2. **Order.** The canonical order of every array; else
//!    `invalid_semantic_graph` at the array.
//! 3. **Targets,** in body order, for a guarantee: `clause` is an
//!    infinite-trace temporal clause, `relation` an `abstraction_relation`
//!    node, a `context` an object type and an `assumption` a `predicate`.
//! 4. **Members,** a guarantee's operations, Rust spellings and uniqueness
//!    (QSpec FR-490 static checks 4 to 6), a composition's component names,
//!    the components its entries name and its premises (QSpec FR-491 static
//!    checks 2 to 5).
//!
//! A composition's targets resolve through admitted dependency packages. Its
//! clauses are checked against their component model subjects and FR-363's
//! safety/liveness classification; scheduling operations resolve against the
//! same admitted model declarations.
//!
//! Implements: FR-038 "Implementation guarantee and component composition bodies".

use super::abstraction::{
    ascending, closed, conflict, defect_failure, entries, failure, identifier, is_object_type,
    model_refusal, node_key, path, text, Defect, Parse, Site,
};
use super::dependency_references::SuppliedDependencies;
use super::frame::{join, StepGraph};
use super::guarantee_terms::{granularity_spelling, kind_spelling};
use super::model_members::{Budget, MemberKind, ModelOwners, ModelRefusal, Resolved};
use super::native_join_budget::PackageReadStage;
use super::rust_spelling::is_rust_path;
use super::stage_storage::{self, Path, Site as ChargeSite};
use super::state::resolve_operation;
use super::term_view::TermNode;
use super::{
    BodyTerm, CheckedComponentClause, CheckedCompositionComponent, CheckedCompositionPremise,
    CheckedEnvironmentEntry, CheckedEnvironmentSupplies, CheckedFairnessGranularity,
    CheckedFairnessKind, CheckedFairnessSupply, CheckedGuaranteeMeasure, CheckedGuaranteeProgress,
    CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2, CheckedSemanticNodeV2, CheckedTerm,
    CorrespondenceForm, FunctionForm, TemporalForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
};

/// The four composition rule spellings of QSpec FR-491.
const RULES: [&str; 4] = [
    "assume-guarantee",
    "protocol-refinement",
    "refinement-transfer",
    "interface",
];

/// The temporal profile an implementation guarantee's clause selects.
const INFINITE_TRACE: &str = "quire.temporal.infinite-trace/v1";

/// An implementation guarantee body, entries in wire order.
struct GuaranteeBody {
    clause: CheckedNodeId,
    relation: CheckedNodeId,
    initial: Vec<Box<str>>,
    progress: Vec<CheckedGuaranteeProgress>,
    environment: Vec<CheckedEnvironmentEntry>,
}

/// A component composition body, entries in wire order.
struct CompositionBody {
    components: Vec<CheckedCompositionComponent>,
    guarantees: Vec<CheckedComponentClause>,
    relies: Vec<CheckedCompositionPremise>,
    liveness_dependencies: Vec<CheckedCompositionPremise>,
    supplies: Vec<CheckedFairnessSupply>,
    environment: Vec<CheckedEnvironmentEntry>,
}

fn environment_targets<'a>(
    entries: &'a [CheckedEnvironmentEntry],
    targets: &mut Vec<&'a CheckedNodeId>,
) {
    targets.extend(entries.iter().map(|entry| match &entry.supplies {
        CheckedEnvironmentSupplies::Fairness(supply) => &supply.context,
        CheckedEnvironmentSupplies::Assumption(id) => id,
    }));
}
impl GuaranteeBody {
    fn targets(&self, meter: &mut WorkMeter) -> Result<Vec<&CheckedNodeId>, ValidationFailure> {
        let mut targets = meter.vec(2 + self.progress.len() + self.environment.len())?;
        targets.extend([&self.clause, &self.relation]);
        targets.extend(self.progress.iter().map(|entry| &entry.context));
        environment_targets(&self.environment, &mut targets);
        targets.sort_unstable();
        targets.dedup();
        Ok(targets)
    }
}
impl CompositionBody {
    fn targets(&self, meter: &mut WorkMeter) -> Result<Vec<&CheckedNodeId>, ValidationFailure> {
        let mut targets = meter.vec(
            self.components.len()
                + self.guarantees.len()
                + 2 * (self.relies.len() + self.liveness_dependencies.len())
                + self.supplies.len()
                + self.environment.len(),
        )?;
        targets.extend(self.components.iter().map(|entry| &entry.subject));
        targets.extend(self.guarantees.iter().map(|entry| &entry.clause));
        for premise in self.relies.iter().chain(&self.liveness_dependencies) {
            targets.extend([&premise.guarantee.clause, &premise.on.clause]);
        }
        targets.extend(self.supplies.iter().map(|entry| &entry.context));
        environment_targets(&self.environment, &mut targets);
        targets.sort_unstable();
        targets.dedup();
        Ok(targets)
    }
}

#[ix_trace_rs::string_edge]
fn kind_of(spelling: &str) -> Option<CheckedFairnessKind> {
    match spelling {
        "weak" => Some(CheckedFairnessKind::Weak),
        "strong" => Some(CheckedFairnessKind::Strong),
        _ => None,
    }
}

#[ix_trace_rs::string_edge]
fn granularity_of(spelling: &str) -> Option<CheckedFairnessGranularity> {
    match spelling {
        "whole" => Some(CheckedFairnessGranularity::Whole),
        "each" => Some(CheckedFairnessGranularity::Each),
        _ => None,
    }
}

fn read_supply(
    value: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<CheckedFairnessSupply, Defect> {
    let object = closed(
        value,
        &["kind", "granularity", "context", "operation"],
        at,
        ctx,
    )?;
    let fail = || ctx.invalid(at);
    Ok(CheckedFairnessSupply {
        kind: text(object, "kind", ctx)
            .and_then(|kind| kind_of(&kind))
            .ok_or_else(fail)?,
        granularity: text(object, "granularity", ctx)
            .and_then(|granularity| granularity_of(&granularity))
            .ok_or_else(fail)?,
        context: node_key(object, "context", at, ctx)?,
        operation: identifier(object, "operation", ctx).ok_or_else(fail)?,
    })
}

fn read_environment(
    value: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<CheckedEnvironmentEntry, Defect> {
    let object = closed(value, &["name", "supplier", "supplies"], at, ctx)?;
    let fail = || ctx.invalid(at);
    let supplies_at = at.key("supplies");
    let supplies = match object.get("supplies") {
        Some(supplies)
            if supplies.is_object()
                && supplies
                    .members()
                    .is_some_and(|members| members.count() == 1) =>
        {
            supplies
        }
        _ => return Err(ctx.invalid(&supplies_at)),
    };
    let supplies = if let Some(fairness) = supplies.get("fairness") {
        CheckedEnvironmentSupplies::Fairness(read_supply(
            fairness,
            &supplies_at.key("fairness"),
            ctx,
        )?)
    } else if supplies.get("assumption").is_some() {
        CheckedEnvironmentSupplies::Assumption(node_key(supplies, "assumption", &supplies_at, ctx)?)
    } else {
        return Err(ctx.invalid(&supplies_at));
    };
    Ok(CheckedEnvironmentEntry {
        name: identifier(object, "name", ctx).ok_or_else(fail)?,
        supplier: object
            .get("supplier")
            .and_then(TermNode::as_str)
            .and_then(|text| ctx.copy(text))
            .ok_or_else(fail)?,
        supplies,
    })
}

fn read_progress(
    value: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<CheckedGuaranteeProgress, Defect> {
    let object = closed(value, &["context", "operation", "measure"], at, ctx)?;
    let fail = || ctx.invalid(at);
    let measure = match object.get("measure") {
        Some(measure)
            if measure.is_object()
                && measure
                    .members()
                    .is_some_and(|members| members.count() == 1) =>
        {
            measure
        }
        _ => return Err(ctx.invalid(&at.key("measure"))),
    };
    let measure = if measure.get("decreases").is_some() {
        CheckedGuaranteeMeasure::Decreases(path(measure, "decreases", ctx).ok_or_else(fail)?)
    } else {
        CheckedGuaranteeMeasure::Bound(
            measure
                .get("bound")
                .and_then(TermNode::as_u64)
                .ok_or_else(fail)?,
        )
    };
    Ok(CheckedGuaranteeProgress {
        context: node_key(object, "context", at, ctx)?,
        operation: identifier(object, "operation", ctx).ok_or_else(fail)?,
        measure,
    })
}

fn read_clause(
    value: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<CheckedComponentClause, Defect> {
    let object = closed(value, &["component", "clause"], at, ctx)?;
    Ok(CheckedComponentClause {
        component: identifier(object, "component", ctx).ok_or_else(|| ctx.invalid(at))?,
        clause: node_key(object, "clause", at, ctx)?,
    })
}

fn read_premise(
    value: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<CheckedCompositionPremise, Defect> {
    let object = closed(value, &["guarantee", "on"], at, ctx)?;
    let member = |name: &'static str| {
        let value = object.get(name).ok_or_else(|| ctx.invalid(at))?;
        read_clause(value, &at.key(name), ctx)
    };
    Ok(CheckedCompositionPremise {
        guarantee: member("guarantee")?,
        on: member("on")?,
    })
}

fn read_component(
    value: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<CheckedCompositionComponent, Defect> {
    let object = closed(value, &["name", "package", "subject"], at, ctx)?;
    let fail = || ctx.invalid(at);
    Ok(CheckedCompositionComponent {
        name: identifier(object, "name", ctx).ok_or_else(fail)?,
        package: text(object, "package", ctx).ok_or_else(fail)?,
        subject: node_key(object, "subject", at, ctx)?,
    })
}

fn check_term(
    body: TermNode<'_>,
    expected: BodyTerm,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<(), Defect> {
    if body.kind() == Some(expected) {
        Ok(())
    } else {
        Err(ctx.invalid(at))
    }
}

fn read_guarantee(
    body: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<GuaranteeBody, Defect> {
    let object = closed(
        body,
        &[
            "term",
            "clause",
            "relation",
            "initial",
            "progress",
            "environment",
        ],
        at,
        ctx,
    )?;
    check_term(body, BodyTerm::ImplementationGuarantee, at, ctx)?;
    Ok(GuaranteeBody {
        clause: node_key(object, "clause", at, ctx)?,
        relation: node_key(object, "relation", at, ctx)?,
        initial: path(object, "initial", ctx).ok_or_else(|| ctx.invalid(at))?,
        progress: entries(object, "progress", at, read_progress, ctx)?,
        environment: entries(object, "environment", at, read_environment, ctx)?,
    })
}

fn read_composition(
    body: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<(Box<str>, CompositionBody), Defect> {
    let object = closed(
        body,
        &[
            "term",
            "rule",
            "components",
            "guarantees",
            "relies",
            "liveness_dependencies",
            "supplies",
            "environment",
        ],
        at,
        ctx,
    )?;
    check_term(body, BodyTerm::ComponentComposition, at, ctx)?;
    let rule = text(object, "rule", ctx)
        .filter(|rule| RULES.contains(&rule.as_ref()))
        .ok_or_else(|| ctx.invalid(at))?;
    Ok((
        rule,
        CompositionBody {
            components: entries(object, "components", at, read_component, ctx)?,
            guarantees: entries(object, "guarantees", at, read_clause, ctx)?,
            relies: entries(object, "relies", at, read_premise, ctx)?,
            liveness_dependencies: entries(object, "liveness_dependencies", at, read_premise, ctx)?,
            supplies: entries(object, "supplies", at, read_supply, ctx)?,
            environment: entries(object, "environment", at, read_environment, ctx)?,
        },
    ))
}

fn refuse(
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
    at: Path<'static>,
    locus: &CheckedNodeId,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    failure(meter, code, at, Some(cause), locus)
}
fn malformed(at: Path<'static>, locus: &CheckedNodeId, meter: &mut WorkMeter) -> ValidationFailure {
    model_refusal(ModelRefusal::malformed(), at, locus, meter)
}

/// The guarantee step. See the module documentation for the order.
///
/// Implements: FR-038 "Implementation guarantee and component composition bodies".
pub(super) fn validate<Body: super::model_members::EmptyAggregateBody>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &super::graph_storage::GraphIndex<'_>,
    owners: &ModelOwners<'_>,
    dependencies: &SuppliedDependencies<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StepGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    for (position, node) in graph.each(CheckedNodeKind::Correspondence(
        CorrespondenceForm::ImplementationGuarantee,
    )) {
        meter.at_node(position, "correspondence");
        let site = Site { node, position };
        check_guarantee(&site, &graph, terms, meter)?;
    }
    for (position, node) in graph.each(CheckedNodeKind::Correspondence(
        CorrespondenceForm::ComponentComposition,
    )) {
        meter.at_node(position, "correspondence");
        let site = Site { node, position };
        check_composition(&site, &graph, dependencies, meter)?;
    }
    Ok(())
}

/// Shape, identity and order are common to both forms: the shape read has
/// produced the body; this checks identity against its `targets`.
fn check_identity<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    targets: Vec<&CheckedNodeId>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    if node.semantic_type == node.node_id && node.dependencies.iter().eq(targets) {
        Ok(())
    } else {
        Err(site.invalid(site.node_at(), meter))
    }
}

fn root_application<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    meter: &mut WorkMeter,
) -> Option<ValidationFailure> {
    (meter
        .body(site.position)
        .is_some_and(|(body, _)| body.kind() == Some(BodyTerm::Application)))
    .then(|| {
        refuse(
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            site.node_at(),
            &site.node.node_id,
            meter,
        )
    })
}

fn out_of_order<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    array_name: &'static str,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    site.invalid(site.body_at().key(array_name), meter)
}

fn check_guarantee<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    graph: &StepGraph<'_, '_, Body>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if let Some(failure) = root_application(site, meter) {
        return Err(failure);
    }
    let (document, row) = meter
        .body(site.position)
        .ok_or_else(|| site.invalid(site.body_at(), meter))?;
    let result = {
        let ctx = Parse::new(meter, row, site.position);
        read_guarantee(document, &site.body_at(), &ctx).map_err(|defect| ctx.finish(defect))
    };
    let body = result.map_err(|defect| defect_failure(defect, site, meter))?;
    check_identity(site, body.targets(meter)?, meter)?;
    if !ascending(
        body.progress
            .iter()
            .map(|entry| (&entry.context, entry.operation.as_ref())),
    ) {
        return Err(out_of_order(site, "progress", meter));
    }
    if !ascending(body.environment.iter().map(|entry| entry.name.as_ref())) {
        return Err(out_of_order(site, "environment", meter));
    }
    check_guarantee_targets(site, &body, graph, terms, meter)?;
    check_guarantee_members(site, &body, graph, terms, meter)
}

/// Whether the clause node's operation selects the infinite-trace temporal
/// profile through its one `temporal_profile` law.
#[ix_trace_rs::string_edge]
fn selects_infinite_trace<Body>(
    clause: &CheckedSemanticNodeV2<Body>,
    graph: &StepGraph<'_, '_, Body>,
    terms: &[Option<CheckedTerm>],
) -> bool {
    let Some(CheckedTerm::Application { operation, .. }) = graph
        .index
        .get(&clause.node_id)
        .and_then(|&p| terms.get(p))
        .and_then(Option::as_ref)
    else {
        return false;
    };
    operation.record().laws.iter().any(|law| {
        law.role.as_ref() == "temporal_profile"
            && law.definition.identity.as_ref() == INFINITE_TRACE
    })
}

/// The identity of the domain package a node's `owner` declares it in.
fn model_identity<Body>(node: &CheckedSemanticNodeV2<Body>) -> Option<&str> {
    match node.owner.as_ref()? {
        CheckedNodeOwner::Model { identity, .. } => Some(identity),
        _ => None,
    }
}

/// The model subject of a clause: the domain package of the type of the
/// `value`/`parameter` its first argument ranges `over`. A clause over a
/// parameter whose type is no model declaration has no model to compare.
fn subject_model<'g, Body>(
    clause: &CheckedSemanticNodeV2<Body>,
    graph: &'g StepGraph<'_, '_, Body>,
    terms: &[Option<CheckedTerm>],
) -> Option<&'g str> {
    let CheckedTerm::Application { arguments, .. } =
        terms.get(*graph.index.get(&clause.node_id)?)?.as_ref()?
    else {
        return None;
    };
    let over = super::structural::typed_reference(arguments.first()?)?;
    let (parameter, kind) = graph.node(over)?;
    if kind != CheckedNodeKind::Value(ValueForm::Parameter) {
        return None;
    }
    let (ty, kind) = graph.node(&parameter.semantic_type)?;
    if kind == CheckedNodeKind::CompositeType(super::CompositeTypeForm::Reference) {
        let target = terms
            .get(*graph.index.get(&ty.node_id)?)?
            .as_ref()
            .and_then(super::structural::reference_type_target_ref)?;
        return model_identity(graph.node(target)?.0);
    }
    model_identity(ty)
}

fn check_guarantee_targets<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    body: &GuaranteeBody,
    graph: &StepGraph<'_, '_, Body>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    let body_at = site.body_at();
    let mut target = |at: Path<'static>,
                      id: &CheckedNodeId,
                      admits: fn(CheckedNodeKind) -> bool|
     -> Result<&CheckedSemanticNodeV2<Body>, ValidationFailure> {
        join(node, id, admits, graph)
            .map(|(target, _)| target)
            .map_err(|refusal| model_refusal(refusal, at, id, meter))
    };
    let clause = target(body_at.key("clause"), &body.clause, |kind| {
        kind == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
    })?;
    if !selects_infinite_trace(clause, graph, terms) {
        return Err(malformed(body_at.key("clause"), &body.clause, meter));
    }
    target(body_at.key("relation"), &body.relation, |kind| {
        kind == CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
    })?;
    for (index, entry) in body.progress.iter().enumerate() {
        let at = body_at.key("progress").index(index);
        target(at, &entry.context, is_object_type)?;
    }
    for (index, entry) in body.environment.iter().enumerate() {
        let at = body_at.key("environment").index(index);
        match &entry.supplies {
            CheckedEnvironmentSupplies::Fairness(supply) => {
                target(at, &supply.context, is_object_type)?;
            }
            CheckedEnvironmentSupplies::Assumption(predicate) => {
                target(at, predicate, |kind| {
                    kind == CheckedNodeKind::Function(FunctionForm::Predicate)
                })?;
            }
        }
    }
    Ok(())
}

/// QSpec FR-490 static checks 4 to 6 on the read values, each over the whole
/// node before the next.
fn check_guarantee_members<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    body: &GuaranteeBody,
    graph: &StepGraph<'_, '_, Body>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    let body_at = site.body_at();
    // 4: each operation a progress or scheduling member names resolves.
    let mut operations = meter.vec(
        body.progress.len()
            + body
                .environment
                .iter()
                .filter(|entry| matches!(entry.supplies, CheckedEnvironmentSupplies::Fairness(_)))
                .count(),
    )?;
    for (index, entry) in body.progress.iter().enumerate() {
        let at = body_at.key("progress").index(index);
        operations.push((at, &entry.context, &entry.operation));
    }
    for (index, entry) in body.environment.iter().enumerate() {
        if let CheckedEnvironmentSupplies::Fairness(supply) = &entry.supplies {
            let at = body_at.key("environment").index(index);
            operations.push((at, &supply.context, &supply.operation));
        }
    }
    let subject = graph
        .node(&body.clause)
        .and_then(|(clause, _)| subject_model(clause, graph, terms));
    for (at, context, operation) in operations {
        let Some((context_node, kind)) = graph.node(context) else {
            continue;
        };
        // The operation is one of the clause's model subject: its context is
        // an object type of the model the clause ranges over.
        if subject.is_none() || model_identity(context_node) != subject {
            return Err(model_refusal(
                ModelRefusal::missing_name(),
                at,
                context,
                meter,
            ));
        }
        if let Err(refusal) = resolve_operation(context_node, kind, operation, graph, meter)? {
            return Err(model_refusal(refusal, at, context, meter));
        }
    }
    // 5: Rust spellings and `bound 0`.
    if !is_rust_path(&body.initial) {
        return Err(malformed(body_at.key("initial"), &node.node_id, meter));
    }
    for (index, entry) in body.progress.iter().enumerate() {
        let valid = match &entry.measure {
            CheckedGuaranteeMeasure::Decreases(measure) => is_rust_path(measure),
            CheckedGuaranteeMeasure::Bound(count) => *count > 0,
        };
        if !valid {
            let at = body_at.key("progress").index(index);
            return Err(malformed(at, &entry.context, meter));
        }
    }
    // 6: one progress member per operation, one environment member per name.
    if let Some(index) = super::graph_storage::first_repeat(
        body.progress
            .iter()
            .map(|entry| (&entry.context, entry.operation.as_ref())),
        meter,
    )? {
        let entry = &body.progress[index];
        return Err(conflict(
            body_at.key("progress").index(index),
            &entry.context,
            meter,
        ));
    }
    check_environment_names(site, &body.environment, meter)?;
    Ok(())
}

fn check_composition<'e, Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    graph: &StepGraph<'_, '_, Body>,
    dependencies: &SuppliedDependencies<'e>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if let Some(failure) = root_application(site, meter) {
        return Err(failure);
    }
    let (document, row) = meter
        .body(site.position)
        .ok_or_else(|| site.invalid(site.body_at(), meter))?;
    let result = {
        let ctx = Parse::new(meter, row, site.position);
        read_composition(document, &site.body_at(), &ctx).map_err(|defect| ctx.finish(defect))
    };
    let (_, body) = result.map_err(|defect| defect_failure(defect, site, meter))?;
    check_identity(site, body.targets(meter)?, meter)?;
    check_composition_order(site, &body, meter)?;
    let places = Places {
        dependencies,
        body: &body,
    };
    check_composition_targets(site, &body, &places, graph, meter)?;
    check_composition_members(site, &body, meter)?;
    check_composition_clauses(site, &body, &places, meter)?;
    check_composition_operations(site, &body, &places, meter)?;
    check_environment_names(site, &body.environment, meter)
}

/// A node of a directly selected component package: its kind, the package
/// and its position there.
type Found<'e> = (CheckedNodeKind, &'e CheckedPackageV2, usize);

/// The lookup of composition targets. QSpec FR-491 resolves a target through
/// FR-322's dependency packages: a component's subject and clauses in its
/// package, a `context` in any component package.
struct Places<'a, 'e> {
    dependencies: &'a SuppliedDependencies<'e>,
    body: &'a CompositionBody,
}

impl<'a, 'e> Places<'a, 'e> {
    fn package_of(&self, component: &str) -> Option<&'a str> {
        self.body
            .components
            .iter()
            .find(|entry| entry.name.as_ref() == component)
            .map(|entry| entry.package.as_ref())
    }

    fn find(&self, package: &str, id: &CheckedNodeId) -> Option<Found<'e>> {
        let (selected, position) = self.dependencies.selected_node(package, id)?;
        Some((*selected.node_kinds().get(position)?, selected, position))
    }

    fn context(&self, id: &CheckedNodeId) -> Option<Found<'e>> {
        self.body
            .components
            .iter()
            .find_map(|entry| self.find(&entry.package, id))
    }
}

/// QSpec FR-490's target checks over a composition's targets, in body order:
/// a target that names no node is `missing-name`, a node of another kind is
/// `malformed-declaration`. A component's subject and clauses are looked up
/// in its package, a `context` in any component package.
fn check_composition_targets<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    body: &CompositionBody,
    places: &Places<'_, '_>,
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let mut check = |at: Path<'static>,
                     found: Option<CheckedNodeKind>,
                     id: &CheckedNodeId,
                     admits: fn(CheckedNodeKind) -> bool| match found {
        None => Err(model_refusal(ModelRefusal::missing_name(), at, id, meter)),
        Some(kind) if admits(kind) => Ok(()),
        Some(_) => Err(model_refusal(ModelRefusal::malformed(), at, id, meter)),
    };
    let kind_of = |found: Option<Found<'_>>| found.map(|(kind, _, _)| kind);
    let is_clause = |kind| kind == CheckedNodeKind::Temporal(TemporalForm::TemporalClause);
    let is_predicate = |kind| kind == CheckedNodeKind::Function(FunctionForm::Predicate);
    for (index, entry) in body.components.iter().enumerate() {
        let at = body_at.key("components").index(index);
        // The component names an imported package.
        if places
            .dependencies
            .selected_package(&entry.package)
            .is_none()
        {
            let at = at.key("package");
            return Err(model_refusal(
                ModelRefusal::missing_name(),
                at,
                &entry.subject,
                meter,
            ));
        }
        let found = kind_of(places.find(&entry.package, &entry.subject));
        check(at, found, &entry.subject, is_object_type)?;
    }
    let mut clause = |at: Path<'static>, entry: &CheckedComponentClause| {
        // A clause of an undeclared component is the member check's refusal.
        match places.package_of(&entry.component) {
            Some(package) => {
                let found = kind_of(places.find(package, &entry.clause));
                check(at, found, &entry.clause, is_clause)
            }
            None => Ok(()),
        }
    };
    for (index, entry) in body.guarantees.iter().enumerate() {
        clause(body_at.key("guarantees").index(index), entry)?;
    }
    for (member, premises) in [
        ("relies", &body.relies),
        ("liveness_dependencies", &body.liveness_dependencies),
    ] {
        for (index, premise) in premises.iter().enumerate() {
            let at = body_at.key(member).index(index);
            clause(at, &premise.guarantee)?;
            clause(at, &premise.on)?;
        }
    }
    for (index, entry) in body.supplies.iter().enumerate() {
        let at = body_at.key("supplies").index(index);
        check(
            at,
            kind_of(places.context(&entry.context)),
            &entry.context,
            is_object_type,
        )?;
    }
    for (index, entry) in body.environment.iter().enumerate() {
        let at = body_at.key("environment").index(index);
        match &entry.supplies {
            CheckedEnvironmentSupplies::Fairness(supply) => {
                let found = kind_of(places.context(&supply.context));
                check(at, found, &supply.context, is_object_type)?;
            }
            CheckedEnvironmentSupplies::Assumption(predicate) => {
                let found = graph.node(predicate).map(|(_, kind)| kind);
                check(at, found, predicate, is_predicate)?;
            }
        }
    }
    Ok(())
}

/// Recover the model subject from an admitted temporal clause's typed `over`
/// parameter, without decoding its body again.
fn component_clause_model<'a>(
    package: &'a CheckedPackageV2,
    clause: &CheckedNodeId,
) -> Option<&'a str> {
    let CheckedTerm::Application { arguments, .. } = package.term(clause)? else {
        return None;
    };
    let CheckedTerm::Reference { target } = arguments.first()? else {
        return None;
    };
    let parameter = package.node(target)?;
    let mut subject = package.node(&parameter.semantic_type)?;
    if let CheckedTerm::Aggregate { members } = &subject.body {
        if let [CheckedTerm::Reference { target }] = members.as_slice() {
            subject = package.node(target)?;
        }
    }
    match subject.owner.as_ref()? {
        CheckedNodeOwner::Model { identity, .. } => Some(identity),
        _ => None,
    }
}

/// FR-363's safety test in negation normal form. Polarity applies the duals
/// without constructing a second formula: negative always/release introduce
/// eventually/until, and implication reverses its left operand's polarity.
/// Each referenced formula and polarity is visited at most once.
#[ix_trace_rs::string_edge]
fn clause_is_liveness(
    package: &CheckedPackageV2,
    clause: &CheckedNodeId,
    meter: &mut WorkMeter,
    at: &Path<'static>,
) -> Result<bool, ValidationFailure> {
    let Some(CheckedTerm::Application {
        arguments,
        operation,
        ..
    }) = package.term(clause)
    else {
        return Ok(false);
    };
    let infinite = operation.read().ok().is_some_and(|read| {
        read.laws
            .iter()
            .any(|law| law.definition.identity.as_ref() == INFINITE_TRACE)
    });
    if !infinite {
        return Ok(false);
    }
    let Some(formula) = arguments.get(5) else {
        return Ok(false);
    };
    let mut pending = meter.vec(1)?;
    pending.push((formula, false));
    let mut seen = Vec::new();
    while let Some((term, negative)) = pending.pop() {
        stage_storage::work(
            meter,
            &ChargeSite::new(
                PackageReadStage::Graph,
                meter.site.map_or(0, |site| site.index),
                "correspondence",
                *at,
            ),
            1,
        )?;
        match term {
            CheckedTerm::Reference { target } => {
                if seen.contains(&(target, negative)) {
                    continue;
                }
                meter.push(&mut seen, (target, negative))?;
                if package.node_kind(target)
                    == Some(CheckedNodeKind::Temporal(TemporalForm::Formula))
                {
                    if let Some(term) = package.term(target) {
                        meter.push(&mut pending, (term, negative))?;
                    }
                }
            }
            CheckedTerm::Application {
                operation,
                arguments,
                ..
            } => {
                let Ok(read) = operation.read() else { continue };
                let unbounded = match read.member {
                    Some(super::CheckedOperationMember::TemporalInterval { interval }) => {
                        match interval {
                            None => true,
                            Some(super::CheckedOperationInterval::Integer { upper, .. }) => {
                                upper.is_none()
                            }
                            Some(super::CheckedOperationInterval::Timed { upper, .. }) => {
                                upper.is_none()
                            }
                        }
                    }
                    _ => false,
                };
                let operator = FormulaOperator::of(read.identity);
                if unbounded
                    && matches!(
                        (operator, negative),
                        (FormulaOperator::Eventually | FormulaOperator::Until, false)
                            | (FormulaOperator::Always | FormulaOperator::Release, true)
                    )
                {
                    return Ok(true);
                }
                for (index, argument) in arguments.iter().enumerate().rev() {
                    let negative = match operator {
                        FormulaOperator::Not => !negative,
                        FormulaOperator::Implies if index == 0 => !negative,
                        _ => negative,
                    };
                    meter.push(&mut pending, (argument, negative))?;
                }
            }
            _ => {}
        }
    }
    Ok(false)
}

#[derive(Clone, Copy)]
enum FormulaOperator {
    Eventually,
    Until,
    Always,
    Release,
    Not,
    Implies,
    Other,
}
impl FormulaOperator {
    #[ix_trace_rs::string_edge]
    fn of(identity: &str) -> Self {
        match identity {
            "quire.op.temporal.eventually" => Self::Eventually,
            "quire.op.temporal.until" => Self::Until,
            "quire.op.temporal.always" => Self::Always,
            "quire.op.temporal.release" => Self::Release,
            "quire.op.temporal.not" => Self::Not,
            "quire.op.temporal.implies" => Self::Implies,
            _ => Self::Other,
        }
    }
}

/// FR-491 static checks 2 and 4: clauses belong to their component's model
/// subject, and safety and liveness premises cannot discharge each other.
fn check_composition_clauses<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    body: &CompositionBody,
    places: &Places<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let check_subject =
        |entry: &CheckedComponentClause, at: &Path<'static>, meter: &mut WorkMeter| {
            let component = body.components.iter().find(|c| c.name == entry.component);
            let found = component.and_then(|c| {
                places
                    .find(&c.package, &entry.clause)
                    .map(|(_, p, _)| (c, p))
            });
            if let Some((component, package)) = found {
                let subject = package.node(&component.subject).and_then(|node| {
                    match node.owner.as_ref()? {
                        CheckedNodeOwner::Model { identity, .. } => Some(identity.as_ref()),
                        _ => None,
                    }
                });
                if subject.is_none() || component_clause_model(package, &entry.clause) != subject {
                    return Err(model_refusal(
                        ModelRefusal::missing_name(),
                        *at,
                        &entry.clause,
                        meter,
                    ));
                }
            }
            Ok(())
        };
    for (index, entry) in body.guarantees.iter().enumerate() {
        check_subject(entry, &site.body_at().key("guarantees").index(index), meter)?;
    }
    for (member, premises, liveness) in [
        ("relies", &body.relies, false),
        ("liveness_dependencies", &body.liveness_dependencies, true),
    ] {
        for (index, premise) in premises.iter().enumerate() {
            let at = site.body_at().key(member).index(index);
            for (side, entry) in [&premise.guarantee, &premise.on].into_iter().enumerate() {
                check_subject(entry, &at, meter)?;
                // A rely constrains only its peer H; a depends constrains G and H.
                if !liveness && side == 0 {
                    continue;
                }
                if let Some(package) = places
                    .package_of(&entry.component)
                    .and_then(|name| places.dependencies.selected_package(name))
                {
                    if clause_is_liveness(package, &entry.clause, meter, &at)? != liveness {
                        return Err(malformed(at, &entry.clause, meter));
                    }
                }
            }
        }
    }
    Ok(())
}

/// QSpec FR-491 static check 5: each `supplies` and scheduling `environment`
/// operation is an operation of a component subject's model. Operations of
/// one component package are resolved together over one rebuilt model owner
/// index of that package.
fn check_composition_operations<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    body: &CompositionBody,
    places: &Places<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let mut wanted = meter.vec(
        body.supplies.len()
            + body
                .environment
                .iter()
                .filter(|entry| matches!(entry.supplies, CheckedEnvironmentSupplies::Fairness(_)))
                .count(),
    )?;
    for (index, entry) in body.supplies.iter().enumerate() {
        let at = body_at.key("supplies").index(index);
        wanted.push((at, &entry.context, entry.operation.as_ref()));
    }
    for (index, entry) in body.environment.iter().enumerate() {
        if let CheckedEnvironmentSupplies::Fairness(supply) = &entry.supplies {
            let at = body_at.key("environment").index(index);
            wanted.push((at, &supply.context, supply.operation.as_ref()));
        }
    }
    let mut foreign: Vec<ForeignOperations<'_>> = Vec::new();
    for (at, context, operation) in wanted {
        if let Some((_, package, position)) = places.context(context) {
            let context_model =
                package
                    .node(context)
                    .and_then(|node| match node.owner.as_ref()? {
                        CheckedNodeOwner::Model { identity, .. } => Some(identity.as_ref()),
                        _ => None,
                    });
            let belongs = context_model.is_some_and(|identity| {
                body.components.iter().any(|component| {
                    places
                        .dependencies
                        .selected_package(&component.package)
                        .and_then(|package| package.node(&component.subject))
                        .and_then(|node| match node.owner.as_ref()? {
                            CheckedNodeOwner::Model { identity, .. } => Some(identity.as_ref()),
                            _ => None,
                        })
                        == Some(identity)
                })
            });
            if !belongs {
                return Err(model_refusal(
                    ModelRefusal::missing_name(),
                    at,
                    context,
                    meter,
                ));
            }
            let group =
                if let Some(index) = foreign.iter().position(|(p, _)| p.name() == package.name()) {
                    &mut foreign[index].1
                } else {
                    meter.push(&mut foreign, (package, Vec::new()))?;
                    &mut foreign.last_mut().expect("just inserted package").1
                };
            meter.push(group, (at, context, position, operation))?;
        }
    }
    for (package, operations) in foreign {
        check_foreign_operations(package, &operations, meter)?;
    }
    Ok(())
}

type ForeignOperation<'a> = (Path<'static>, &'a CheckedNodeId, usize, &'a str);
type ForeignOperations<'a> = (&'a CheckedPackageV2, Vec<ForeignOperation<'a>>);

fn check_foreign_operations(
    package: &CheckedPackageV2,
    operations: &[ForeignOperation<'_>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = package.resolved_graph();
    let nodes = &graph.nodes;
    let mut budget = Budget::node(meter, 0);
    let owners = ModelOwners::build(
        package.models.selected_models(),
        nodes,
        &mut budget,
        |budget, root| {
            budget.set_index(root);
            budget.charge(1)
        },
    )?;
    for (at, context, position, operation) in operations {
        let Some(node) = nodes.get(*position) else {
            continue;
        };
        let resolution = owners.resolve_member(node, MemberKind::Operation, operation, meter)?;
        let refusal = match resolution {
            Ok((owner, Resolved::Operation(operation))) => {
                if owner.object_type().is_some_and(|object| {
                    object
                        .operations
                        .iter()
                        .any(|own| std::ptr::eq(own, operation))
                }) {
                    None
                } else {
                    Some(ModelRefusal::malformed())
                }
            }
            Ok((_, Resolved::Field(_))) => Some(ModelRefusal::missing_name()),
            Err(refusal) if refusal == ModelRefusal::ineligible() => {
                Some(ModelRefusal::missing_name())
            }
            Err(refusal) => Some(refusal),
        };
        if let Some(refusal) = refusal {
            return Err(model_refusal(refusal, *at, context, meter));
        }
    }
    Ok(())
}

/// QSpec FR-490 static check 6 over a composition's `environment`: two
/// members of one name.
fn check_environment_names<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    environment: &[CheckedEnvironmentEntry],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if let Some(index) = super::graph_storage::first_repeat(
        environment.iter().map(|entry| entry.name.as_ref()),
        meter,
    )? {
        return Err(conflict(
            site.body_at().key("environment").index(index),
            &site.node.node_id,
            meter,
        ));
    }
    Ok(())
}

fn check_composition_order<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    body: &CompositionBody,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    fn clause_key(entry: &CheckedComponentClause) -> (&str, &CheckedNodeId) {
        (&entry.component, &entry.clause)
    }
    fn premise_key(
        entry: &CheckedCompositionPremise,
    ) -> ((&str, &CheckedNodeId), (&str, &CheckedNodeId)) {
        (clause_key(&entry.guarantee), clause_key(&entry.on))
    }
    if !ascending(body.components.iter().map(|entry| entry.name.as_ref())) {
        return Err(out_of_order(site, "components", meter));
    }
    if !ascending(body.guarantees.iter().map(clause_key)) {
        return Err(out_of_order(site, "guarantees", meter));
    }
    if !ascending(body.relies.iter().map(premise_key)) {
        return Err(out_of_order(site, "relies", meter));
    }
    if !ascending(body.liveness_dependencies.iter().map(premise_key)) {
        return Err(out_of_order(site, "liveness_dependencies", meter));
    }
    if !ascending(body.supplies.iter().map(|entry| {
        (
            &entry.context,
            entry.operation.as_ref(),
            kind_spelling(entry.kind),
            granularity_spelling(entry.granularity),
        )
    })) {
        return Err(out_of_order(site, "supplies", meter));
    }
    if !ascending(body.environment.iter().map(|entry| entry.name.as_ref())) {
        return Err(out_of_order(site, "environment", meter));
    }
    Ok(())
}

/// QSpec FR-491 static checks 2 and 3 on the read values.
fn check_composition_members<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    body: &CompositionBody,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let components_at = body_at.key("components");
    let name_repeat = super::graph_storage::first_repeat(
        body.components.iter().map(|entry| entry.name.as_ref()),
        meter,
    )?;
    let subject_repeat = super::graph_storage::first_repeat(
        body.components
            .iter()
            .map(|entry| (entry.package.as_ref(), &entry.subject)),
        meter,
    )?;
    if let Some(index) = name_repeat.into_iter().chain(subject_repeat).min() {
        return Err(conflict(
            components_at.index(index),
            &body.components[index].subject,
            meter,
        ));
    }
    let mut names = meter.vec(body.components.len())?;
    names.extend(body.components.iter().map(|entry| entry.name.as_ref()));
    names.sort_unstable();
    let missing = |at: Path<'static>, entry: &CheckedComponentClause, meter: &mut WorkMeter| {
        refuse(
            CheckedPackageRefusalCode::MissingDeclaration,
            CheckedPackageRefusalCause::MissingName,
            at,
            &entry.clause,
            meter,
        )
    };
    for (index, entry) in body.guarantees.iter().enumerate() {
        if names.binary_search(&entry.component.as_ref()).is_err() {
            return Err(missing(
                body_at.key("guarantees").index(index),
                entry,
                meter,
            ));
        }
    }
    let mut declared = meter.vec(body.guarantees.len())?;
    declared.extend(
        body.guarantees
            .iter()
            .map(|entry| (entry.component.as_ref(), &entry.clause)),
    );
    declared.sort_unstable();
    declared.dedup();
    for (member, premises) in [
        ("relies", &body.relies),
        ("liveness_dependencies", &body.liveness_dependencies),
    ] {
        for (index, premise) in premises.iter().enumerate() {
            let at = body_at.key(member).index(index);
            for side in [&premise.guarantee, &premise.on] {
                if names.binary_search(&side.component.as_ref()).is_err() {
                    return Err(missing(at, side, meter));
                }
            }
            let guarantee = (
                premise.guarantee.component.as_ref(),
                &premise.guarantee.clause,
            );
            if declared.binary_search(&guarantee).is_err()
                || premise.guarantee.component == premise.on.component
            {
                return Err(malformed(at, &premise.guarantee.clause, meter));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod document_storage_tests {
    use super::super::{test_support::node_id, CheckedJson, NativeJoinBudget};
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn flat_correspondence_shapes_keep_nested_first_fault_and_pay_exact_storage() {
        for guarantee in [false, true] {
            let body = if guarantee {
                json!({"term":"implementation_guarantee","clause":node_id('b'),"relation":node_id('c'),"initial":["initial"],"progress":[{"context":node_id('d'),"operation":"act","measure":{"bound":"wrong"}}],"environment":[]})
            } else {
                json!({"term":"component_composition","rule":"interface","components":[],"guarantees":[],"relies":[],"liveness_dependencies":[],"supplies":[],"environment":[]})
            };
            let nodes:[CheckedSemanticNodeV2<CheckedJson>;1]=[serde_json::from_value(json!({"node_id":node_id('a'),"schema_version":"quire.checked-semantic-graph/v2","node_tag":"correspondence","semantic_form":if guarantee {"implementation_guarantee"} else {"component_composition"},"semantic_type":node_id('a'),"dependencies":[],"occurrences":[],"body":body})).unwrap()];
            let kinds = [CheckedNodeKind::Correspondence(if guarantee {
                CorrespondenceForm::ImplementationGuarantee
            } else {
                CorrespondenceForm::ComponentComposition
            })];
            let index = nodes
                .iter()
                .enumerate()
                .map(|(p, n)| (&n.node_id, p))
                .collect::<super::super::graph_storage::GraphIndex<'_>>();
            let handles = [nodes[0].body.clone()];
            let rows = [vec![]];
            let owners = ModelOwners::default();
            let dependencies = SuppliedDependencies::default();
            let run = |limit| {
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let measured = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(&mut budget);
                    meter.attach_bodies(&handles, &rows);
                    result = Some(validate(
                        &nodes,
                        &kinds,
                        &[None],
                        &index,
                        &owners,
                        &dependencies,
                        &mut meter,
                    ));
                });
                (result.unwrap(), measured.bytes_total, budget)
            };
            let (result, allocated, budget) = run(u64::MAX);
            if guarantee {
                let Err(ValidationFailure::Refused(refusal)) = result else {
                    panic!("malformed nested measure");
                };
                assert_eq!(
                    refusal.code,
                    CheckedPackageRefusalCode::InvalidSemanticGraph
                );
                assert_eq!(
                    refusal.loci[0].path.as_ref().unwrap().as_str(),
                    "/semantic_graph/nodes/0/body/progress/0"
                );
            } else {
                result.unwrap();
            }
            let required = budget.read_storage_consumed();
            assert_eq!(allocated, required);
            assert_eq!(run(required).0.is_ok(), !guarantee);
            assert!(matches!(
                run(required - 1).0,
                Err(ValidationFailure::Budget(_))
            ));
            assert_eq!(run(0).1, 0);
        }
    }
}
