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
//!    checks 2 and 3).
//!
//! A composition's targets name nodes of the component packages, which the
//! graph does not hold, so they are joined to `dependencies` by the identity
//! step and to no node kind. FR-491 static checks 4 and 5 read the component
//! packages' clause classification and subject operations and are not made
//! here.
//!
//! Implements: FR-038 "Implementation guarantee and component composition bodies".

use super::abstraction::{
    ascending, closed, conflict, entries, identifier, is_object_type, model_refusal, node_key,
    path, text, Defect, Site,
};
use super::frame::{join, StepGraph};
use super::model_members::{ModelOwners, ModelRefusal};
use super::rust_spelling::is_rust_path;
use super::state::resolve_operation;
use super::{
    BodyTerm, CheckedComponentClause, CheckedCompositionComponent, CheckedCompositionPremise,
    CheckedEnvironmentEntry, CheckedEnvironmentSupplies, CheckedFairnessGranularity,
    CheckedFairnessKind, CheckedFairnessSupply, CheckedGuaranteeMeasure, CheckedGuaranteeProgress,
    CheckedNodeKind, CorrespondenceForm, FunctionForm, RawSemanticNodeV2 as CheckedSemanticNodeV2,
    TemporalForm, WorkMeter,
};
use crate::checked_package::common::{body_term, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

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
        check_composition(&site)?;
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
    for (at, context, operation) in operations {
        let Some((context_node, kind)) = graph.node(context) else {
            continue;
        };
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

fn check_composition(site: &Site<'_>) -> Result<(), ValidationFailure> {
    let node = site.node;
    if let Some(failure) = root_application(site) {
        return Err(failure);
    }
    let (_, body) =
        read_composition(&node.body, &site.body_at()).map_err(|d| defect_failure(d, node))?;
    check_identity(site, body.targets())?;
    check_composition_order(site, &body)?;
    check_composition_members(site, &body)
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
            entry.kind as u8,
            entry.granularity as u8,
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
