// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-350: closed step-relation claim body and its V2 body step.

use super::frame::StepGraph;
use super::model_members::ModelOwners;
use super::structural::{typed_anchor_body, typed_clause_body, typed_parameter_binding};
use super::{
    CheckedNodeKind, CheckedNodeOwner, CheckedSemanticNodeV2, CheckedTerm, ClaimForm,
    CompositeTypeForm, CorrespondenceForm, LiteralKind, ModelForm, ScalarTypeForm, StateClauseKind,
    StateForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, JsonPointer,
};
use std::collections::{BTreeMap, BTreeSet};

/// One execution variable in an admitted step relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedStepExecution {
    /// Source variable name.
    pub variable: Box<str>,
    /// Selected model alias.
    pub alias: Box<str>,
    /// Operation anchor.
    pub operation: CheckedNodeId,
    /// This execution's binders in FR-341 postcondition order.
    pub parameters: Vec<CheckedNodeId>,
    /// Effective preconditions in node-id order.
    pub preconditions: Vec<CheckedNodeId>,
    /// Effective postconditions in node-id order.
    pub postconditions: Vec<CheckedNodeId>,
    /// Bound abstraction relation, when present.
    pub implementation: Option<CheckedNodeId>,
}

/// The closed, ordered body of a `claim`/`step_relation` node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedStepRelationBody {
    /// Distinct selected models in first-named order.
    pub subjects: Vec<(Box<str>, CheckedNodeId)>,
    /// Execution variables in source order.
    pub executions: Vec<CheckedStepExecution>,
    /// Each subject's invariant clauses in subject order.
    pub invariants: Vec<(Box<str>, Vec<CheckedNodeId>)>,
    /// Boolean relation expression.
    pub body: CheckedNodeId,
}

impl CheckedStepRelationBody {
    /// Whether every execution requests the code claim.
    pub fn requests_code_claim(&self) -> bool {
        !self.executions.is_empty()
            && self
                .executions
                .iter()
                .all(|entry| entry.implementation.is_some())
    }
}

impl CheckedSemanticNodeV2 {
    /// The typed relation body, only for an admitted step-relation claim.
    #[ix_trace_rs::string_edge]
    pub fn step_relation_body(&self) -> Option<CheckedStepRelationBody> {
        (self.node_tag.as_ref() == "claim" && self.semantic_form.as_ref() == "step_relation")
            .then(|| parse(&self.body, node_pointer(0).key("body")).ok())
            .flatten()
    }
}

fn members(term: &CheckedTerm) -> Option<&[CheckedTerm]> {
    match term {
        CheckedTerm::Aggregate { members } => Some(members),
        _ => None,
    }
}

fn binding<'a>(term: &'a CheckedTerm, name: &str) -> Option<&'a CheckedTerm> {
    match term {
        CheckedTerm::Binding { name: found, value } if found.as_ref() == name => Some(value),
        _ => None,
    }
}

fn named_binding(term: &CheckedTerm) -> Option<(&str, &CheckedTerm)> {
    match term {
        CheckedTerm::Binding { name, value } if !name.as_ref().is_empty() => Some((name, value)),
        _ => None,
    }
}

fn reference(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

fn references(term: &CheckedTerm, at: JsonPointer) -> Result<Vec<CheckedNodeId>, JsonPointer> {
    let entries = members(term).ok_or_else(|| at.clone())?;
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            reference(entry)
                .cloned()
                .ok_or_else(|| at.clone().key("members").index(i))
        })
        .collect()
}

fn ordered<'a, const N: usize>(
    term: &'a CheckedTerm,
    names: [&str; N],
    at: JsonPointer,
) -> Result<[&'a CheckedTerm; N], JsonPointer> {
    let entries = members(term).ok_or_else(|| at.clone())?;
    if entries.len() != N {
        return Err(at.key("members").index(entries.len().min(N)));
    }
    let mut result = Vec::with_capacity(N);
    for (i, (entry, name)) in entries.iter().zip(names).enumerate() {
        result.push(binding(entry, name).ok_or_else(|| at.clone().key("members").index(i))?);
    }
    result.try_into().map_err(|_| at)
}

fn parse(term: &CheckedTerm, at: JsonPointer) -> Result<CheckedStepRelationBody, JsonPointer> {
    let [subjects, executions, invariants, body] = ordered(
        term,
        ["subjects", "executions", "invariants", "body"],
        at.clone(),
    )?;
    let subject_at = at.clone().key("members").index(0).key("value");
    let execution_at = at.clone().key("members").index(1).key("value");
    let invariant_at = at.clone().key("members").index(2).key("value");
    let body_at = at.key("members").index(3).key("value");
    let mut selected = Vec::new();
    for (i, entry) in members(subjects)
        .ok_or_else(|| subject_at.clone())?
        .iter()
        .enumerate()
    {
        let path = subject_at.clone().key("members").index(i);
        let (alias, target) = named_binding(entry).ok_or_else(|| path.clone())?;
        selected.push((
            Box::from(alias),
            reference(target).cloned().ok_or(path.key("value"))?,
        ));
    }
    let mut variables = Vec::new();
    for (i, entry) in members(executions)
        .ok_or_else(|| execution_at.clone())?
        .iter()
        .enumerate()
    {
        let path = execution_at.clone().key("members").index(i);
        let (variable, value) = named_binding(entry).ok_or_else(|| path.clone())?;
        let [alias, operation, parameters, preconditions, postconditions, implementation] =
            ordered(
                value,
                [
                    "alias",
                    "operation",
                    "parameters",
                    "preconditions",
                    "postconditions",
                    "implementation",
                ],
                path.clone().key("value"),
            )?;
        let alias = match alias {
            CheckedTerm::Literal {
                value_kind: LiteralKind::Text,
                value,
                ..
            } => value.as_str(),
            _ => None,
        }
        .ok_or_else(|| {
            path.clone()
                .key("value")
                .key("members")
                .index(0)
                .key("value")
        })?;
        let operation = reference(operation).cloned().ok_or_else(|| {
            path.clone()
                .key("value")
                .key("members")
                .index(1)
                .key("value")
        })?;
        let parameters = references(
            parameters,
            path.clone()
                .key("value")
                .key("members")
                .index(2)
                .key("value"),
        )?;
        let preconditions = references(
            preconditions,
            path.clone()
                .key("value")
                .key("members")
                .index(3)
                .key("value"),
        )?;
        let postconditions = references(
            postconditions,
            path.clone()
                .key("value")
                .key("members")
                .index(4)
                .key("value"),
        )?;
        let implementation = if let Some(id) = reference(implementation) {
            Some(id.clone())
        } else if members(implementation).is_some_and(<[CheckedTerm]>::is_empty) {
            None
        } else {
            return Err(path.key("value").key("members").index(5).key("value"));
        };
        variables.push(CheckedStepExecution {
            variable: Box::from(variable),
            alias: Box::from(alias),
            operation,
            parameters,
            preconditions,
            postconditions,
            implementation,
        });
    }
    let mut invariant_sets = Vec::new();
    for (i, entry) in members(invariants)
        .ok_or_else(|| invariant_at.clone())?
        .iter()
        .enumerate()
    {
        let path = invariant_at.clone().key("members").index(i);
        let (alias, value) = named_binding(entry).ok_or_else(|| path.clone())?;
        invariant_sets.push((Box::from(alias), references(value, path.key("value"))?));
    }
    Ok(CheckedStepRelationBody {
        subjects: selected,
        executions: variables,
        invariants: invariant_sets,
        body: reference(body).cloned().ok_or(body_at)?,
    })
}

fn refuse(
    code: Code,
    cause: Option<Cause>,
    path: JsonPointer,
    node: &CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, path, cause, node.clone())
}

fn malformed(path: JsonPointer, node: &CheckedNodeId) -> ValidationFailure {
    refuse(
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        path,
        node,
    )
}

fn subject_model(node: &CheckedSemanticNodeV2<serde_json::Value>) -> Option<&str> {
    match &node.owner {
        Some(CheckedNodeOwner::ModelRoot { identity, .. }) => Some(identity),
        _ => None,
    }
}

fn same_model(
    node: &CheckedSemanticNodeV2<serde_json::Value>,
    model: &str,
    graph: &StepGraph<'_, '_>,
) -> bool {
    graph
        .owners
        .owner_by_key(&node.node_id)
        .is_some_and(|owner| owner.package.identity.as_ref() == model)
}

/// Validate the body step after terminal and before protocol.
pub(super) fn validate_step_relations(
    nodes: &[CheckedSemanticNodeV2<serde_json::Value>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
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
    for (position, node) in graph.each(CheckedNodeKind::Claim(ClaimForm::StepRelation)) {
        let at = node_pointer(position).key("body");
        let term = terms
            .get(position)
            .and_then(Option::as_ref)
            .ok_or_else(|| refuse(Code::InvalidSemanticGraph, None, at.clone(), &node.node_id))?;
        let body = parse(term, at.clone())
            .map_err(|path| refuse(Code::InvalidSemanticGraph, None, path, &node.node_id))?;
        let subject_at = at
            .clone()
            .key("members")
            .index(0)
            .key("value")
            .key("members");
        let execution_at = at
            .clone()
            .key("members")
            .index(1)
            .key("value")
            .key("members");
        let invariant_at = at
            .clone()
            .key("members")
            .index(2)
            .key("value")
            .key("members");
        let body_at = at.key("members").index(3).key("value").key("target");
        let mut aliases = BTreeMap::new();
        for (i, (alias, model)) in body.subjects.iter().enumerate() {
            if aliases.insert(alias.as_ref(), model).is_some() {
                return Err(refuse(
                    Code::InvalidSemanticGraph,
                    None,
                    subject_at.clone().index(i),
                    &node.node_id,
                ));
            }
        }
        let mut named = BTreeSet::new();
        for (i, entry) in body.executions.iter().enumerate() {
            if !aliases.contains_key(entry.alias.as_ref()) {
                return Err(refuse(
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    execution_at
                        .clone()
                        .index(i)
                        .key("value")
                        .key("members")
                        .index(0),
                    &node.node_id,
                ));
            }
            named.insert(entry.alias.as_ref());
        }
        for (i, (alias, _)) in body.subjects.iter().enumerate() {
            if !named.contains(alias.as_ref()) {
                return Err(refuse(
                    Code::InvalidSemanticGraph,
                    None,
                    subject_at.clone().index(i),
                    &node.node_id,
                ));
            }
        }
        let mut subject_models = BTreeMap::new();
        for (i, (alias, target)) in body.subjects.iter().enumerate() {
            let Some((model_node, kind)) = graph.node(target) else {
                return Err(malformed(
                    subject_at.clone().index(i).key("value").key("target"),
                    target,
                ));
            };
            if kind != CheckedNodeKind::Model(ModelForm::ModelRoot) {
                return Err(malformed(
                    subject_at.clone().index(i).key("value").key("target"),
                    target,
                ));
            }
            let Some(model) = subject_model(model_node) else {
                return Err(malformed(
                    subject_at.clone().index(i).key("value").key("target"),
                    target,
                ));
            };
            subject_models.insert(alias.as_ref(), model);
        }
        let mut anchors = Vec::new();
        for (i, entry) in body.executions.iter().enumerate() {
            let Some((_, kind)) = graph.node(&entry.operation) else {
                return Err(malformed(
                    execution_at
                        .clone()
                        .index(i)
                        .key("value")
                        .key("members")
                        .index(1)
                        .key("value")
                        .key("target"),
                    &entry.operation,
                ));
            };
            let anchor_body = graph
                .index
                .get(&entry.operation)
                .and_then(|&pos| terms.get(pos))
                .and_then(Option::as_ref)
                .and_then(typed_anchor_body);
            let operation_at = execution_at
                .clone()
                .index(i)
                .key("value")
                .key("members")
                .index(1)
                .key("value")
                .key("target");
            let Some(anchor_body) = anchor_body else {
                return Err(malformed(operation_at, &entry.operation));
            };
            let model = subject_models[entry.alias.as_ref()];
            let context_ok = graph
                .node(&anchor_body.context)
                .is_some_and(|(context, _)| same_model(context, model, &graph));
            if kind != CheckedNodeKind::State(StateForm::OperationAnchor) || !context_ok {
                return Err(malformed(
                    execution_at
                        .clone()
                        .index(i)
                        .key("value")
                        .key("members")
                        .index(1)
                        .key("value")
                        .key("target"),
                    &entry.operation,
                ));
            }
            if let Some(implementation) = &entry.implementation {
                let valid = graph.node(implementation).is_some_and(|(relation, kind)| {
                    kind == CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
                        && matches!(&relation.body, serde_json::Value::Object(_))
                        && relation
                            .body
                            .get("frames")
                            .and_then(serde_json::Value::as_array)
                            .is_some_and(|frames| {
                                frames.iter().any(|frame| {
                                    frame.get("context")
                                        == serde_json::to_value(&anchor_body.context).ok().as_ref()
                                        && frame
                                            .get("operation")
                                            .and_then(serde_json::Value::as_str)
                                            == Some(anchor_body.operation)
                                })
                            })
                });
                if !valid {
                    return Err(malformed(
                        execution_at
                            .clone()
                            .index(i)
                            .key("value")
                            .key("members")
                            .index(5)
                            .key("value")
                            .key("target"),
                        implementation,
                    ));
                }
            }
            anchors.push(anchor_body);
        }
        let mut level = 0usize;
        for (i, (entry, anchor)) in body.executions.iter().zip(&anchors).enumerate() {
            let param_at = execution_at
                .clone()
                .index(i)
                .key("value")
                .key("members")
                .index(2)
                .key("value");
            let Some((context, kind)) = graph.node(&anchor.context) else {
                return Err(malformed(param_at, &entry.operation));
            };
            let expected =
                super::state::resolve_operation(context, kind, anchor.operation, &graph, meter)?;
            let Ok(Some((owner, operation))) = expected else {
                return Err(malformed(param_at, &entry.operation));
            };
            let expected_count =
                1 + usize::from(operation.result.is_some()) + operation.parameters.len();
            if entry.parameters.len() != expected_count {
                return Err(malformed(param_at, &entry.operation));
            }
            for (j, id) in entry.parameters.iter().enumerate() {
                let Some((parameter, kind)) = graph.node(id) else {
                    return Err(malformed(param_at.clone(), id));
                };
                let declared = graph
                    .index
                    .get(id)
                    .and_then(|&pos| terms.get(pos))
                    .and_then(Option::as_ref)
                    .and_then(|body| {
                        typed_parameter_binding(body, &|id| graph.node(id).map(|(_, kind)| kind))
                    });
                let expected_name = if j == 0 {
                    Some("self")
                } else if j == 1 && operation.result.is_some() {
                    Some("result")
                } else {
                    operation
                        .result
                        .iter()
                        .chain(&operation.parameters)
                        .nth(j - 1)
                        .and_then(|slot| slot.name.as_deref())
                };
                let level_text = level.to_string();
                let type_ok = if j == 0 {
                    graph
                        .node(&parameter.semantic_type)
                        .filter(|(_, kind)| {
                            *kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Reference)
                        })
                        .and_then(|(node, _)| super::structural::reference_type_target(node))
                        .as_ref()
                        == Some(&anchor.context)
                } else {
                    operation
                        .result
                        .iter()
                        .chain(&operation.parameters)
                        .nth(j - 1)
                        .and_then(|slot| owner.package.slot_type(slot))
                        .and_then(|kind| kind.node_key(owner.package.bytes).ok())
                        .as_deref()
                        == Some(parameter.semantic_type.digest.as_ref())
                };
                if kind != CheckedNodeKind::Value(ValueForm::Parameter)
                    || !type_ok
                    || declared != expected_name.map(|name| (name, level_text.as_str()))
                {
                    return Err(malformed(param_at.clone(), id));
                }
                level += 1;
            }
            for (slot, clause_kind, provided) in [
                (3, StateClauseKind::Precondition, &entry.preconditions),
                (4, StateClauseKind::Postcondition, &entry.postconditions),
            ] {
                let mut expected = Vec::new();
                for (clause_pos, clause) in
                    graph.each(CheckedNodeKind::State(StateForm::StateClause))
                {
                    let parsed = terms
                        .get(clause_pos)
                        .and_then(Option::as_ref)
                        .and_then(typed_clause_body);
                    if parsed.as_ref().is_some_and(|body| {
                        body.clause == clause_kind && body.anchor == entry.operation
                    }) {
                        expected.push(clause.node_id.clone());
                    }
                }
                if provided != &expected {
                    return Err(malformed(
                        execution_at
                            .clone()
                            .index(i)
                            .key("value")
                            .key("members")
                            .index(slot)
                            .key("value"),
                        &entry.operation,
                    ));
                }
            }
        }
        if body.invariants.len() != body.subjects.len() {
            return Err(malformed(invariant_at, &node.node_id));
        }
        for (i, ((alias, model_id), (found, provided))) in
            body.subjects.iter().zip(&body.invariants).enumerate()
        {
            let path = invariant_at.clone().index(i);
            if alias != found {
                return Err(malformed(path, model_id));
            }
            let model = subject_models[alias.as_ref()];
            let mut expected = Vec::new();
            for (clause_pos, clause) in graph.each(CheckedNodeKind::State(StateForm::StateClause)) {
                let parsed = terms
                    .get(clause_pos)
                    .and_then(Option::as_ref)
                    .and_then(typed_clause_body);
                if let Some(parsed) = parsed {
                    if parsed.clause == StateClauseKind::Invariant
                        && graph.node(&parsed.anchor).is_some_and(|(context, kind)| {
                            matches!(
                                kind,
                                CheckedNodeKind::Model(
                                    ModelForm::ObjectType
                                        | ModelForm::Process
                                        | ModelForm::StateMachine
                                )
                            ) && same_model(context, model, &graph)
                        })
                    {
                        expected.push(clause.node_id.clone());
                    }
                }
            }
            if provided != &expected {
                return Err(malformed(path.key("value"), model_id));
            }
        }
        let valid_body = graph.node(&body.body).is_some_and(|(expression, kind)| {
            matches!(kind, CheckedNodeKind::Expression(_))
                && graph
                    .node(&expression.semantic_type)
                    .is_some_and(|(_, kind)| {
                        kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean)
                    })
        });
        if !valid_body {
            return Err(refuse(
                Code::IllTyped,
                Some(Cause::NonBooleanRoot),
                body_at,
                &body.body,
            ));
        }
        let binders: BTreeSet<_> = body
            .executions
            .iter()
            .flat_map(|entry| entry.parameters.iter())
            .collect();
        let mut seen = BTreeSet::new();
        let mut pending = vec![&body.body];
        while let Some(target) = pending.pop() {
            if !seen.insert(target) {
                continue;
            }
            meter.charge(1, || body_at.clone())?;
            let Some((read, kind)) = graph.node(target) else {
                continue;
            };
            if kind == CheckedNodeKind::Value(ValueForm::Parameter) {
                if !binders.contains(target) {
                    return Err(malformed(body_at.clone(), target));
                }
            } else {
                pending.extend(read.dependencies.iter());
            }
        }
        if graph
            .node(&node.semantic_type)
            .is_none_or(|(_, kind)| kind != CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
            || node
                .owner
                .as_ref()
                .is_none_or(|owner| !matches!(owner, CheckedNodeOwner::Source { .. }))
            || node.declaration.is_none()
            || node.occurrences.len() != 2
            || node
                .occurrences
                .iter()
                .filter(|item| item.role == CheckedOccurrenceRole::Declaration)
                .count()
                != 1
            || node
                .occurrences
                .iter()
                .filter(|item| item.role == CheckedOccurrenceRole::Claim)
                .count()
                != 1
        {
            return Err(refuse(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(position),
                &node.node_id,
            ));
        }
    }
    Ok(())
}
