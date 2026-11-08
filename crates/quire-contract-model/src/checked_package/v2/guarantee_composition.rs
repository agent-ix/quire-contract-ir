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
    ascending, closed, conflict, entries, identifier, is_object_type, model_refusal, node_key,
    path, text, Defect, Site,
};
use super::dependency_references::SuppliedDependencies;
use super::frame::{join, StepGraph};
use super::guarantee_terms::{granularity_spelling, kind_spelling};
use super::model_members::{Budget, MemberKind, ModelOwners, ModelRefusal, Resolved};
use super::rust_spelling::is_rust_path;
use super::state::resolve_operation;
use super::{
    BodyTerm, CheckedComponentClause, CheckedCompositionComponent, CheckedCompositionPremise,
    CheckedEnvironmentEntry, CheckedEnvironmentSupplies, CheckedFairnessGranularity,
    CheckedFairnessKind, CheckedFairnessSupply, CheckedGuaranteeMeasure, CheckedGuaranteeProgress,
    CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2, CheckedTerm, CorrespondenceForm,
    FunctionForm, RawSemanticNodeV2 as CheckedSemanticNodeV2, TemporalForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{body_term, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use serde_json::Value;
use std::collections::BTreeSet;

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
    targets: &mut BTreeSet<&'a CheckedNodeId>,
) {
    for entry in entries {
        targets.insert(match &entry.supplies {
            CheckedEnvironmentSupplies::Fairness(supply) => &supply.context,
            CheckedEnvironmentSupplies::Assumption(node) => node,
        });
    }
}

impl GuaranteeBody {
    fn targets(&self) -> Vec<CheckedNodeId> {
        let mut targets = BTreeSet::from([&self.clause, &self.relation]);
        targets.extend(self.progress.iter().map(|entry| &entry.context));
        environment_targets(&self.environment, &mut targets);
        targets.into_iter().cloned().collect()
    }
}

impl CompositionBody {
    fn targets(&self) -> Vec<CheckedNodeId> {
        let mut targets = BTreeSet::new();
        targets.extend(self.components.iter().map(|entry| &entry.subject));
        targets.extend(self.guarantees.iter().map(|entry| &entry.clause));
        for premise in self.relies.iter().chain(&self.liveness_dependencies) {
            targets.insert(&premise.guarantee.clause);
            targets.insert(&premise.on.clause);
        }
        targets.extend(self.supplies.iter().map(|entry| &entry.context));
        environment_targets(&self.environment, &mut targets);
        targets.into_iter().cloned().collect()
    }
}

fn kind_of(spelling: &str) -> Option<CheckedFairnessKind> {
    match spelling {
        "weak" => Some(CheckedFairnessKind::Weak),
        "strong" => Some(CheckedFairnessKind::Strong),
        _ => None,
    }
}

fn granularity_of(spelling: &str) -> Option<CheckedFairnessGranularity> {
    match spelling {
        "whole" => Some(CheckedFairnessGranularity::Whole),
        "each" => Some(CheckedFairnessGranularity::Each),
        _ => None,
    }
}

fn read_supply(value: &Value, at: &JsonPointer) -> Result<CheckedFairnessSupply, Defect> {
    let object = closed(value, &["kind", "granularity", "context", "operation"], at)?;
    let fail = || Defect::invalid(at);
    Ok(CheckedFairnessSupply {
        kind: text(object, "kind")
            .and_then(|kind| kind_of(&kind))
            .ok_or_else(fail)?,
        granularity: text(object, "granularity")
            .and_then(|granularity| granularity_of(&granularity))
            .ok_or_else(fail)?,
        context: node_key(object, "context", at)?,
        operation: identifier(object, "operation").ok_or_else(fail)?,
    })
}

fn read_environment(value: &Value, at: &JsonPointer) -> Result<CheckedEnvironmentEntry, Defect> {
    let object = closed(value, &["name", "supplier", "supplies"], at)?;
    let fail = || Defect::invalid(at);
    let supplies_at = at.clone().key("supplies");
    let supplies = match object.get("supplies") {
        Some(Value::Object(supplies)) if supplies.len() == 1 => supplies,
        _ => return Err(Defect::invalid(&supplies_at)),
    };
    let supplies = if let Some(fairness) = supplies.get("fairness") {
        CheckedEnvironmentSupplies::Fairness(read_supply(fairness, &supplies_at.key("fairness"))?)
    } else if supplies.contains_key("assumption") {
        CheckedEnvironmentSupplies::Assumption(node_key(supplies, "assumption", &supplies_at)?)
    } else {
        return Err(Defect::invalid(&supplies_at));
    };
    Ok(CheckedEnvironmentEntry {
        name: identifier(object, "name").ok_or_else(fail)?,
        supplier: object
            .get("supplier")
            .and_then(Value::as_str)
            .map(Box::from)
            .ok_or_else(fail)?,
        supplies,
    })
}

fn read_progress(value: &Value, at: &JsonPointer) -> Result<CheckedGuaranteeProgress, Defect> {
    let object = closed(value, &["context", "operation", "measure"], at)?;
    let fail = || Defect::invalid(at);
    let measure = match object.get("measure") {
        Some(Value::Object(measure)) if measure.len() == 1 => measure,
        _ => return Err(Defect::invalid(&at.clone().key("measure"))),
    };
    let measure = if measure.contains_key("decreases") {
        CheckedGuaranteeMeasure::Decreases(path(measure, "decreases").ok_or_else(fail)?)
    } else {
        CheckedGuaranteeMeasure::Bound(
            measure
                .get("bound")
                .and_then(Value::as_u64)
                .ok_or_else(fail)?,
        )
    };
    Ok(CheckedGuaranteeProgress {
        context: node_key(object, "context", at)?,
        operation: identifier(object, "operation").ok_or_else(fail)?,
        measure,
    })
}

fn read_clause(value: &Value, at: &JsonPointer) -> Result<CheckedComponentClause, Defect> {
    let object = closed(value, &["component", "clause"], at)?;
    Ok(CheckedComponentClause {
        component: identifier(object, "component").ok_or_else(|| Defect::invalid(at))?,
        clause: node_key(object, "clause", at)?,
    })
}

fn read_premise(value: &Value, at: &JsonPointer) -> Result<CheckedCompositionPremise, Defect> {
    let object = closed(value, &["guarantee", "on"], at)?;
    let member = |name: &str| {
        let value = object.get(name).ok_or_else(|| Defect::invalid(at))?;
        read_clause(value, &at.clone().key(name))
    };
    Ok(CheckedCompositionPremise {
        guarantee: member("guarantee")?,
        on: member("on")?,
    })
}

fn read_component(value: &Value, at: &JsonPointer) -> Result<CheckedCompositionComponent, Defect> {
    let object = closed(value, &["name", "package", "subject"], at)?;
    let fail = || Defect::invalid(at);
    Ok(CheckedCompositionComponent {
        name: identifier(object, "name").ok_or_else(fail)?,
        package: text(object, "package").ok_or_else(fail)?,
        subject: node_key(object, "subject", at)?,
    })
}

fn check_term(body: &Value, expected: BodyTerm, at: &JsonPointer) -> Result<(), Defect> {
    if body_term(body) == Some(expected) {
        Ok(())
    } else {
        Err(Defect::invalid(at))
    }
}

fn read_guarantee(body: &Value, at: &JsonPointer) -> Result<GuaranteeBody, Defect> {
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
    )?;
    check_term(body, BodyTerm::ImplementationGuarantee, at)?;
    Ok(GuaranteeBody {
        clause: node_key(object, "clause", at)?,
        relation: node_key(object, "relation", at)?,
        initial: path(object, "initial").ok_or_else(|| Defect::invalid(at))?,
        progress: entries(object, "progress", at, read_progress)?,
        environment: entries(object, "environment", at, read_environment)?,
    })
}

fn read_composition(body: &Value, at: &JsonPointer) -> Result<(Box<str>, CompositionBody), Defect> {
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
    )?;
    check_term(body, BodyTerm::ComponentComposition, at)?;
    let rule = text(object, "rule")
        .filter(|rule| RULES.contains(&rule.as_ref()))
        .ok_or_else(|| Defect::invalid(at))?;
    Ok((
        rule,
        CompositionBody {
            components: entries(object, "components", at, read_component)?,
            guarantees: entries(object, "guarantees", at, read_clause)?,
            relies: entries(object, "relies", at, read_premise)?,
            liveness_dependencies: entries(object, "liveness_dependencies", at, read_premise)?,
            supplies: entries(object, "supplies", at, read_supply)?,
            environment: entries(object, "environment", at, read_environment)?,
        },
    ))
}

fn defect_failure(defect: Defect, node: &CheckedSemanticNodeV2) -> ValidationFailure {
    ValidationFailure::refused_at(defect.code, defect.path, None, node.node_id.clone())
}

fn refuse(
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
    at: JsonPointer,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, at, Some(cause), locus.clone())
}

fn malformed(at: JsonPointer, locus: &CheckedNodeId) -> ValidationFailure {
    model_refusal(ModelRefusal::malformed(), at, locus)
}

/// The guarantee step. See the module documentation for the order.
///
/// Implements: FR-038 "Implementation guarantee and component composition bodies".
pub(super) fn validate(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
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
        let site = Site { node, position };
        check_guarantee(&site, &graph, meter)?;
    }
    for (position, node) in graph.each(CheckedNodeKind::Correspondence(
        CorrespondenceForm::ComponentComposition,
    )) {
        let site = Site { node, position };
        check_composition(&site, &graph, dependencies, meter)?;
    }
    Ok(())
}

/// Shape, identity and order are common to both forms: the shape read has
/// produced the body; this checks identity against its `targets`.
fn check_identity(site: &Site<'_>, targets: Vec<CheckedNodeId>) -> Result<(), ValidationFailure> {
    let node = site.node;
    if node.semantic_type == node.node_id && node.dependencies == targets {
        Ok(())
    } else {
        Err(site.invalid(site.node_at()))
    }
}

fn root_application(site: &Site<'_>) -> Option<ValidationFailure> {
    (body_term(&site.node.body) == Some(BodyTerm::Application)).then(|| {
        refuse(
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            site.node_at(),
            &site.node.node_id,
        )
    })
}

fn out_of_order(site: &Site<'_>, array_name: &str) -> ValidationFailure {
    site.invalid(site.body_at().key(array_name))
}

fn check_guarantee(
    site: &Site<'_>,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    if let Some(failure) = root_application(site) {
        return Err(failure);
    }
    let body = read_guarantee(&node.body, &site.body_at()).map_err(|d| defect_failure(d, node))?;
    check_identity(site, body.targets())?;
    if !ascending(
        body.progress
            .iter()
            .map(|entry| (&entry.context, entry.operation.as_ref())),
    ) {
        return Err(out_of_order(site, "progress"));
    }
    if !ascending(body.environment.iter().map(|entry| entry.name.as_ref())) {
        return Err(out_of_order(site, "environment"));
    }
    check_guarantee_targets(site, &body, graph)?;
    check_guarantee_members(site, &body, graph, meter)
}

/// Whether the clause node's operation selects the infinite-trace temporal
/// profile through its one `temporal_profile` law.
fn selects_infinite_trace(clause: &CheckedSemanticNodeV2) -> bool {
    let laws = clause
        .body
        .get("operation")
        .and_then(|operation| operation.get("laws"))
        .and_then(Value::as_array);
    laws.is_some_and(|laws| {
        laws.iter().any(|law| {
            law.get("role").and_then(Value::as_str) == Some("temporal_profile")
                && law
                    .get("definition")
                    .and_then(|definition| definition.get("identity"))
                    .and_then(Value::as_str)
                    == Some(INFINITE_TRACE)
        })
    })
}

/// The identity of the domain package a node's `owner` declares it in.
fn model_identity(node: &CheckedSemanticNodeV2) -> Option<&str> {
    match node.owner.as_ref()? {
        CheckedNodeOwner::Model { identity, .. } => Some(identity),
        _ => None,
    }
}

/// The model subject of a clause: the domain package of the type of the
/// `value`/`parameter` its first argument ranges `over`. A clause over a
/// parameter whose type is no model declaration has no model to compare.
fn subject_model<'g>(
    clause: &CheckedSemanticNodeV2,
    graph: &'g StepGraph<'_, '_>,
) -> Option<&'g str> {
    let over = clause.body.get("arguments")?.get(0)?.get("target")?;
    let over: CheckedNodeId = serde_json::from_value(over.clone()).ok()?;
    let (parameter, kind) = graph.node(&over)?;
    if kind != CheckedNodeKind::Value(ValueForm::Parameter) {
        return None;
    }
    let (ty, kind) = graph.node(&parameter.semantic_type)?;
    if kind == CheckedNodeKind::CompositeType(super::CompositeTypeForm::Reference) {
        let target = super::structural::reference_type_target(ty)?;
        return model_identity(graph.node(&target)?.0);
    }
    model_identity(ty)
}

fn check_guarantee_targets(
    site: &Site<'_>,
    body: &GuaranteeBody,
    graph: &StepGraph<'_, '_>,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    let body_at = site.body_at();
    let target = |at: JsonPointer,
                  id: &CheckedNodeId,
                  admits: fn(CheckedNodeKind) -> bool|
     -> Result<&CheckedSemanticNodeV2, ValidationFailure> {
        join(node, id, admits, graph)
            .map(|(target, _)| target)
            .map_err(|refusal| model_refusal(refusal, at, id))
    };
    let clause = target(body_at.clone().key("clause"), &body.clause, |kind| {
        kind == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
    })?;
    if !selects_infinite_trace(clause) {
        return Err(malformed(body_at.clone().key("clause"), &body.clause));
    }
    target(body_at.clone().key("relation"), &body.relation, |kind| {
        kind == CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
    })?;
    for (index, entry) in body.progress.iter().enumerate() {
        let at = body_at.clone().key("progress").index(index);
        target(at, &entry.context, is_object_type)?;
    }
    for (index, entry) in body.environment.iter().enumerate() {
        let at = body_at.clone().key("environment").index(index);
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
fn check_guarantee_members(
    site: &Site<'_>,
    body: &GuaranteeBody,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    let body_at = site.body_at();
    // 4: each operation a progress or scheduling member names resolves.
    let mut operations = Vec::new();
    for (index, entry) in body.progress.iter().enumerate() {
        let at = body_at.clone().key("progress").index(index);
        operations.push((at, &entry.context, &entry.operation));
    }
    for (index, entry) in body.environment.iter().enumerate() {
        if let CheckedEnvironmentSupplies::Fairness(supply) = &entry.supplies {
            let at = body_at.clone().key("environment").index(index);
            operations.push((at, &supply.context, &supply.operation));
        }
    }
    let subject = graph
        .node(&body.clause)
        .and_then(|(clause, _)| subject_model(clause, graph));
    for (at, context, operation) in operations {
        let Some((context_node, kind)) = graph.node(context) else {
            continue;
        };
        // The operation is one of the clause's model subject: its context is
        // an object type of the model the clause ranges over.
        if subject.is_none() || model_identity(context_node) != subject {
            return Err(model_refusal(ModelRefusal::missing_name(), at, context));
        }
        if let Err(refusal) = resolve_operation(context_node, kind, operation, graph, meter)? {
            return Err(model_refusal(refusal, at, context));
        }
    }
    // 5: Rust spellings and `bound 0`.
    if !is_rust_path(&body.initial) {
        return Err(malformed(body_at.clone().key("initial"), &node.node_id));
    }
    for (index, entry) in body.progress.iter().enumerate() {
        let valid = match &entry.measure {
            CheckedGuaranteeMeasure::Decreases(measure) => is_rust_path(measure),
            CheckedGuaranteeMeasure::Bound(count) => *count > 0,
        };
        if !valid {
            let at = body_at.clone().key("progress").index(index);
            return Err(malformed(at, &entry.context));
        }
    }
    // 6: one progress member per operation, one environment member per name.
    let mut progress = BTreeSet::new();
    for (index, entry) in body.progress.iter().enumerate() {
        if !progress.insert((&entry.context, entry.operation.as_ref())) {
            let at = body_at.clone().key("progress").index(index);
            return Err(conflict(at, &entry.context));
        }
    }
    let mut names = BTreeSet::new();
    for (index, entry) in body.environment.iter().enumerate() {
        if !names.insert(entry.name.as_ref()) {
            let at = body_at.clone().key("environment").index(index);
            return Err(conflict(at, &node.node_id));
        }
    }
    Ok(())
}

fn check_composition<'e>(
    site: &Site<'_>,
    graph: &StepGraph<'_, '_>,
    dependencies: &SuppliedDependencies<'e>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    if let Some(failure) = root_application(site) {
        return Err(failure);
    }
    let (_, body) =
        read_composition(&node.body, &site.body_at()).map_err(|d| defect_failure(d, node))?;
    check_identity(site, body.targets())?;
    check_composition_order(site, &body)?;
    let places = Places {
        dependencies,
        body: &body,
    };
    check_composition_targets(site, &body, &places, graph)?;
    check_composition_members(site, &body)?;
    check_composition_clauses(site, &body, &places, meter)?;
    check_composition_operations(site, &body, &places, meter)?;
    check_environment_names(site, &body.environment)
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
fn check_composition_targets(
    site: &Site<'_>,
    body: &CompositionBody,
    places: &Places<'_, '_>,
    graph: &StepGraph<'_, '_>,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let check = |at: JsonPointer,
                 found: Option<CheckedNodeKind>,
                 id: &CheckedNodeId,
                 admits: fn(CheckedNodeKind) -> bool| match found {
        None => Err(model_refusal(ModelRefusal::missing_name(), at, id)),
        Some(kind) if admits(kind) => Ok(()),
        Some(_) => Err(model_refusal(ModelRefusal::malformed(), at, id)),
    };
    let kind_of = |found: Option<Found<'_>>| found.map(|(kind, _, _)| kind);
    let is_clause = |kind| kind == CheckedNodeKind::Temporal(TemporalForm::TemporalClause);
    let is_predicate = |kind| kind == CheckedNodeKind::Function(FunctionForm::Predicate);
    for (index, entry) in body.components.iter().enumerate() {
        let at = body_at.clone().key("components").index(index);
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
            ));
        }
        let found = kind_of(places.find(&entry.package, &entry.subject));
        check(at, found, &entry.subject, is_object_type)?;
    }
    let clause = |at: JsonPointer, entry: &CheckedComponentClause| {
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
        clause(body_at.clone().key("guarantees").index(index), entry)?;
    }
    for (member, premises) in [
        ("relies", &body.relies),
        ("liveness_dependencies", &body.liveness_dependencies),
    ] {
        for (index, premise) in premises.iter().enumerate() {
            let at = body_at.clone().key(member).index(index);
            clause(at.clone(), &premise.guarantee)?;
            clause(at, &premise.on)?;
        }
    }
    for (index, entry) in body.supplies.iter().enumerate() {
        let at = body_at.clone().key("supplies").index(index);
        check(
            at,
            kind_of(places.context(&entry.context)),
            &entry.context,
            is_object_type,
        )?;
    }
    for (index, entry) in body.environment.iter().enumerate() {
        let at = body_at.clone().key("environment").index(index);
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
fn clause_is_liveness(
    package: &CheckedPackageV2,
    clause: &CheckedNodeId,
    meter: &mut WorkMeter,
    at: &JsonPointer,
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
    let mut pending = vec![(formula, false)];
    let mut seen = Vec::new();
    while let Some((term, negative)) = pending.pop() {
        meter.charge(1, || at.clone())?;
        match term {
            CheckedTerm::Reference { target } => {
                if seen.contains(&(target, negative)) {
                    continue;
                }
                seen.push((target, negative));
                if package.node_kind(target)
                    == Some(CheckedNodeKind::Temporal(TemporalForm::Formula))
                {
                    if let Some(term) = package.term(target) {
                        pending.push((term, negative));
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
                    pending.push((argument, negative));
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
fn check_composition_clauses(
    site: &Site<'_>,
    body: &CompositionBody,
    places: &Places<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let check_subject = |entry: &CheckedComponentClause, at: &JsonPointer| {
        let component = body.components.iter().find(|c| c.name == entry.component);
        let found = component.and_then(|c| {
            places
                .find(&c.package, &entry.clause)
                .map(|(_, p, _)| (c, p))
        });
        if let Some((component, package)) = found {
            let subject =
                package
                    .node(&component.subject)
                    .and_then(|node| match node.owner.as_ref()? {
                        CheckedNodeOwner::Model { identity, .. } => Some(identity.as_ref()),
                        _ => None,
                    });
            if subject.is_none() || component_clause_model(package, &entry.clause) != subject {
                return Err(model_refusal(
                    ModelRefusal::missing_name(),
                    at.clone(),
                    &entry.clause,
                ));
            }
        }
        Ok(())
    };
    for (index, entry) in body.guarantees.iter().enumerate() {
        check_subject(entry, &site.body_at().key("guarantees").index(index))?;
    }
    for (member, premises, liveness) in [
        ("relies", &body.relies, false),
        ("liveness_dependencies", &body.liveness_dependencies, true),
    ] {
        for (index, premise) in premises.iter().enumerate() {
            let at = site.body_at().key(member).index(index);
            for (side, entry) in [&premise.guarantee, &premise.on].into_iter().enumerate() {
                check_subject(entry, &at)?;
                // A rely constrains only its peer H; a depends constrains G and H.
                if !liveness && side == 0 {
                    continue;
                }
                if let Some(package) = places
                    .package_of(&entry.component)
                    .and_then(|name| places.dependencies.selected_package(name))
                {
                    if clause_is_liveness(package, &entry.clause, meter, &at)? != liveness {
                        return Err(malformed(at, &entry.clause));
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
fn check_composition_operations(
    site: &Site<'_>,
    body: &CompositionBody,
    places: &Places<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let mut wanted = Vec::new();
    for (index, entry) in body.supplies.iter().enumerate() {
        let at = body_at.clone().key("supplies").index(index);
        wanted.push((at, &entry.context, entry.operation.as_ref()));
    }
    for (index, entry) in body.environment.iter().enumerate() {
        if let CheckedEnvironmentSupplies::Fairness(supply) = &entry.supplies {
            let at = body_at.clone().key("environment").index(index);
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
                return Err(model_refusal(ModelRefusal::missing_name(), at, context));
            }
            let group =
                if let Some(index) = foreign.iter().position(|(p, _)| p.name() == package.name()) {
                    &mut foreign[index].1
                } else {
                    foreign.push((package, Vec::new()));
                    &mut foreign.last_mut().expect("just inserted package").1
                };
            group.push((at, context, position, operation));
        }
    }
    for (package, operations) in foreign {
        check_foreign_operations(package, &operations, meter)?;
    }
    Ok(())
}

type ForeignOperation<'a> = (JsonPointer, &'a CheckedNodeId, usize, &'a str);
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
            return Err(model_refusal(refusal, at.clone(), context));
        }
    }
    Ok(())
}

/// QSpec FR-490 static check 6 over a composition's `environment`: two
/// members of one name.
fn check_environment_names(
    site: &Site<'_>,
    environment: &[CheckedEnvironmentEntry],
) -> Result<(), ValidationFailure> {
    let mut names = BTreeSet::new();
    for (index, entry) in environment.iter().enumerate() {
        if !names.insert(entry.name.as_ref()) {
            let at = site.body_at().key("environment").index(index);
            return Err(conflict(at, &site.node.node_id));
        }
    }
    Ok(())
}

fn check_composition_order(
    site: &Site<'_>,
    body: &CompositionBody,
) -> Result<(), ValidationFailure> {
    let clause_key =
        |entry: &CheckedComponentClause| (entry.component.clone(), entry.clause.clone());
    let premise_key =
        |entry: &CheckedCompositionPremise| (clause_key(&entry.guarantee), clause_key(&entry.on));
    if !ascending(body.components.iter().map(|entry| entry.name.as_ref())) {
        return Err(out_of_order(site, "components"));
    }
    if !ascending(body.guarantees.iter().map(clause_key)) {
        return Err(out_of_order(site, "guarantees"));
    }
    if !ascending(body.relies.iter().map(premise_key)) {
        return Err(out_of_order(site, "relies"));
    }
    if !ascending(body.liveness_dependencies.iter().map(premise_key)) {
        return Err(out_of_order(site, "liveness_dependencies"));
    }
    if !ascending(body.supplies.iter().map(|entry| {
        (
            &entry.context,
            entry.operation.as_ref(),
            kind_spelling(entry.kind),
            granularity_spelling(entry.granularity),
        )
    })) {
        return Err(out_of_order(site, "supplies"));
    }
    if !ascending(body.environment.iter().map(|entry| entry.name.as_ref())) {
        return Err(out_of_order(site, "environment"));
    }
    Ok(())
}

/// QSpec FR-491 static checks 2 and 3 on the read values.
fn check_composition_members(
    site: &Site<'_>,
    body: &CompositionBody,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let components_at = body_at.clone().key("components");
    let mut names = BTreeSet::new();
    let mut subjects = BTreeSet::new();
    for (index, entry) in body.components.iter().enumerate() {
        let repeated = !names.insert(entry.name.as_ref())
            | !subjects.insert((entry.package.as_ref(), &entry.subject));
        if repeated {
            return Err(conflict(components_at.index(index), &entry.subject));
        }
    }
    let missing = |at: JsonPointer, entry: &CheckedComponentClause| {
        refuse(
            CheckedPackageRefusalCode::MissingDeclaration,
            CheckedPackageRefusalCause::MissingName,
            at,
            &entry.clause,
        )
    };
    for (index, entry) in body.guarantees.iter().enumerate() {
        if !names.contains(entry.component.as_ref()) {
            return Err(missing(
                body_at.clone().key("guarantees").index(index),
                entry,
            ));
        }
    }
    let declared: BTreeSet<(&str, &CheckedNodeId)> = body
        .guarantees
        .iter()
        .map(|entry| (entry.component.as_ref(), &entry.clause))
        .collect();
    for (member, premises) in [
        ("relies", &body.relies),
        ("liveness_dependencies", &body.liveness_dependencies),
    ] {
        for (index, premise) in premises.iter().enumerate() {
            let at = body_at.clone().key(member).index(index);
            for side in [&premise.guarantee, &premise.on] {
                if !names.contains(side.component.as_ref()) {
                    return Err(missing(at, side));
                }
            }
            let guarantee = (
                premise.guarantee.component.as_ref(),
                &premise.guarantee.clause,
            );
            if !declared.contains(&guarantee) || premise.guarantee.component == premise.on.component
            {
                return Err(malformed(at, &premise.guarantee.clause));
            }
        }
    }
    Ok(())
}
