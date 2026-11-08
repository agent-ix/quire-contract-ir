// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-351: closed step-relation claim body and its V2 body step.

use super::frame::StepGraph;
use super::graph_storage::GraphIndex;
use super::model_members::ModelOwners;
use super::structural::{typed_anchor_body, typed_clause_body, typed_parameter_binding};
use super::{
    CheckedNodeKind, CheckedNodeOwner, CheckedSemanticNodeV2, CheckedTerm, ClaimForm,
    CompositeTypeForm, CorrespondenceForm, LiteralKind, ModelForm, RawSemanticNodeV2,
    ScalarTypeForm, StateClauseKind, StateForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, JsonPointer,
};

/// One execution variable in an admitted step relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedStepExecution {
    /// Source variable name.
    pub variable: Box<str>,
    /// Selected model alias.
    pub alias: Box<str>,
    /// Text scalar type of the alias literal.
    pub alias_type: CheckedNodeId,
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

/// Capability kind distinguishing the two obligations over one claim occurrence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CheckedStepRelationCapability {
    /// The relation over reachable model transitions.
    TemporalSatisfaction,
    /// The relation over bound implementation executions.
    OperationContract,
}

impl CheckedStepRelationCapability {
    /// QSpec capability spelling for the requested item.
    #[ix_trace_rs::string_edge]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TemporalSatisfaction => "temporal-satisfaction",
            Self::OperationContract => "operation-contract",
        }
    }
}

/// FR-402 obligation identity: a claim node and its capability kind.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CheckedStepRelationObligationIdentity {
    /// The one checked claim node that owns both possible items.
    pub claim: CheckedNodeId,
    /// The requested item's distinct capability kind.
    pub capability: CheckedStepRelationCapability,
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
    /// Requested obligation identities over this step relation's claim occurrence.
    pub fn step_relation_obligations(&self) -> Option<Vec<CheckedStepRelationObligationIdentity>> {
        let body = self.step_relation_body()?;
        let model = CheckedStepRelationObligationIdentity {
            claim: self.node_id.clone(),
            capability: CheckedStepRelationCapability::TemporalSatisfaction,
        };
        let mut requested = vec![model];
        if body.requests_code_claim() {
            requested.push(CheckedStepRelationObligationIdentity {
                claim: self.node_id.clone(),
                capability: CheckedStepRelationCapability::OperationContract,
            });
        }
        Some(requested)
    }

    /// Parse this node's step-relation body into its closed typed production.
    /// Reader admission additionally checks the referenced graph joins.
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

/// A list of `reference` terms, every member a reference.
fn reference_list(term: &CheckedTerm, at: JsonPointer) -> Result<&[CheckedTerm], JsonPointer> {
    let entries = members(term).ok_or_else(|| at.clone())?;
    match entries.iter().position(|entry| reference(entry).is_none()) {
        Some(i) => Err(at.key("members").index(i)),
        None => Ok(entries),
    }
}

/// The targets of a validated reference list, in order.
fn targets(list: &[CheckedTerm]) -> impl Iterator<Item = &CheckedNodeId> + '_ {
    list.iter().filter_map(reference)
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
    let mut ordered = [term; N];
    for (i, ((entry, name), slot)) in entries.iter().zip(names).zip(&mut ordered).enumerate() {
        *slot = binding(entry, name).ok_or_else(|| at.clone().key("members").index(i))?;
    }
    Ok(ordered)
}

/// One execution variable, borrowed from the body term.
struct ExecutionView<'a> {
    variable: &'a str,
    alias: &'a str,
    alias_type: &'a CheckedNodeId,
    operation: &'a CheckedNodeId,
    parameters: &'a [CheckedTerm],
    preconditions: &'a [CheckedTerm],
    postconditions: &'a [CheckedTerm],
    implementation: Option<&'a CheckedNodeId>,
}

/// The closed body of a step relation, borrowed from its term.
struct StepView<'a> {
    subjects: Vec<(&'a str, &'a CheckedNodeId)>,
    executions: Vec<ExecutionView<'a>>,
    invariants: Vec<(&'a str, &'a [CheckedTerm])>,
    body: &'a CheckedNodeId,
}

/// Reads the closed body shape, charging the three tables it builds. The
/// refusal is the pointer of the first value outside the shape.
fn view<'a>(
    term: &'a CheckedTerm,
    at: JsonPointer,
    meter: &mut WorkMeter,
) -> Result<Result<StepView<'a>, JsonPointer>, ValidationFailure> {
    let ordered_members = ordered(
        term,
        ["subjects", "executions", "invariants", "body"],
        at.clone(),
    );
    let [subjects, executions, invariants, body] = match ordered_members {
        Ok(found) => found,
        Err(path) => return Ok(Err(path)),
    };
    let subject_at = at.clone().key("members").index(0).key("value");
    let execution_at = at.clone().key("members").index(1).key("value");
    let invariant_at = at.clone().key("members").index(2).key("value");
    let body_at = at.key("members").index(3).key("value");
    let Some(subject_entries) = members(subjects) else {
        return Ok(Err(subject_at));
    };
    let mut selected = meter.vec(subject_entries.len())?;
    for (i, entry) in subject_entries.iter().enumerate() {
        let path = subject_at.clone().key("members").index(i);
        let Some((alias, target)) = named_binding(entry) else {
            return Ok(Err(path));
        };
        let Some(target) = reference(target) else {
            return Ok(Err(path.key("value")));
        };
        selected.push((alias, target));
    }
    let Some(execution_entries) = members(executions) else {
        return Ok(Err(execution_at));
    };
    let mut variables = meter.vec(execution_entries.len())?;
    for (i, entry) in execution_entries.iter().enumerate() {
        let path = execution_at.clone().key("members").index(i);
        let Some((variable, value)) = named_binding(entry) else {
            return Ok(Err(path));
        };
        let ordered_members = ordered(
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
        );
        let [alias, operation, parameters, preconditions, postconditions, implementation] =
            match ordered_members {
                Ok(found) => found,
                Err(path) => return Ok(Err(path)),
            };
        let member_at = |member: usize| {
            path.clone()
                .key("value")
                .key("members")
                .index(member)
                .key("value")
        };
        let alias = match alias {
            CheckedTerm::Literal {
                value_kind: LiteralKind::Text,
                value,
                type_id,
            } => value.as_str().map(|name| (name, type_id)),
            _ => None,
        };
        let Some((alias, alias_type)) = alias else {
            return Ok(Err(member_at(0)));
        };
        let Some(operation) = reference(operation) else {
            return Ok(Err(member_at(1)));
        };
        let parameters = match reference_list(parameters, member_at(2)) {
            Ok(list) => list,
            Err(path) => return Ok(Err(path)),
        };
        let preconditions = match reference_list(preconditions, member_at(3)) {
            Ok(list) => list,
            Err(path) => return Ok(Err(path)),
        };
        let postconditions = match reference_list(postconditions, member_at(4)) {
            Ok(list) => list,
            Err(path) => return Ok(Err(path)),
        };
        let implementation = if let Some(id) = reference(implementation) {
            Some(id)
        } else if members(implementation).is_some_and(<[CheckedTerm]>::is_empty) {
            None
        } else {
            return Ok(Err(path.key("value").key("members").index(5).key("value")));
        };
        variables.push(ExecutionView {
            variable,
            alias,
            alias_type,
            operation,
            parameters,
            preconditions,
            postconditions,
            implementation,
        });
    }
    let Some(invariant_entries) = members(invariants) else {
        return Ok(Err(invariant_at));
    };
    let mut invariant_sets = meter.vec(invariant_entries.len())?;
    for (i, entry) in invariant_entries.iter().enumerate() {
        let path = invariant_at.clone().key("members").index(i);
        let Some((alias, value)) = named_binding(entry) else {
            return Ok(Err(path));
        };
        match reference_list(value, path.key("value")) {
            Ok(list) => invariant_sets.push((alias, list)),
            Err(path) => return Ok(Err(path)),
        }
    }
    let Some(body) = reference(body) else {
        return Ok(Err(body_at));
    };
    Ok(Ok(StepView {
        subjects: selected,
        executions: variables,
        invariants: invariant_sets,
        body,
    }))
}

/// The owned typed production of a step relation body. It is off the read
/// path and holds no budget.
fn parse(term: &CheckedTerm, at: JsonPointer) -> Result<CheckedStepRelationBody, JsonPointer> {
    let mut unmetered = WorkMeter::new(u64::MAX);
    let Ok(parsed) = view(term, at, &mut unmetered) else {
        return Err(JsonPointer::root());
    };
    let view = parsed?;
    let owned = |list: &[CheckedTerm]| targets(list).cloned().collect::<Vec<_>>();
    Ok(CheckedStepRelationBody {
        subjects: view
            .subjects
            .iter()
            .map(|&(alias, target)| (Box::from(alias), target.clone()))
            .collect(),
        executions: view
            .executions
            .iter()
            .map(|entry| CheckedStepExecution {
                variable: Box::from(entry.variable),
                alias: Box::from(entry.alias),
                alias_type: entry.alias_type.clone(),
                operation: entry.operation.clone(),
                parameters: owned(entry.parameters),
                preconditions: owned(entry.preconditions),
                postconditions: owned(entry.postconditions),
                implementation: entry.implementation.cloned(),
            })
            .collect(),
        invariants: view
            .invariants
            .iter()
            .map(|&(alias, list)| (Box::from(alias), owned(list)))
            .collect(),
        body: view.body.clone(),
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

fn subject_model(node: &RawSemanticNodeV2) -> Option<&str> {
    match &node.owner {
        Some(CheckedNodeOwner::ModelRoot { identity, .. }) => Some(identity),
        _ => None,
    }
}

fn same_model(node: &RawSemanticNodeV2, model: &str, graph: &StepGraph<'_, '_>) -> bool {
    graph
        .owners
        .owner_by_key(&node.node_id)
        .is_some_and(|owner| owner.package.identity.as_ref() == model)
}

/// Whether `text` is the canonical spelling of `value`.
fn spells(text: &str, value: usize) -> bool {
    let mut digits = [0_u8; 20];
    let mut at = digits.len();
    let mut rest = value;
    loop {
        at -= 1;
        digits[at] = b'0' + u8::try_from(rest % 10).unwrap_or(0);
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    text.as_bytes() == &digits[at..]
}

/// The index of the first key equal to an earlier one, over keys paired with
/// their positions and sorted by key then position.
fn first_repeated<K: Eq>(sorted: &[(K, usize)]) -> Option<usize> {
    sorted
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| pair[1].1)
        .min()
}

/// Whether the state clauses of the graph that `wants` selects are exactly
/// the `provided` references, in node-id order. Every clause is one work
/// unit, charged at `at` before it is read.
fn clauses_are(
    graph: &StepGraph<'_, '_>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
    at: &dyn Fn() -> JsonPointer,
    provided: &[CheckedTerm],
    wants: impl Fn(&super::structural::ClauseBody<'_>) -> bool,
) -> Result<bool, ValidationFailure> {
    let mut expected = targets(provided);
    let mut agrees = true;
    for (clause_pos, clause) in graph.each(CheckedNodeKind::State(StateForm::StateClause)) {
        meter.charge(1, at)?;
        let parsed = terms
            .get(clause_pos)
            .and_then(Option::as_ref)
            .and_then(typed_clause_body);
        if parsed.as_ref().is_some_and(&wants) && expected.next() != Some(&clause.node_id) {
            agrees = false;
        }
    }
    Ok(agrees && expected.next().is_none())
}

/// Validate the body step after terminal and before protocol.
pub(super) fn validate_step_relations(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
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
        meter.at_node(position, "claim");
        let at = node_pointer(position).key("body");
        let term = terms
            .get(position)
            .and_then(Option::as_ref)
            .ok_or_else(|| refuse(Code::InvalidSemanticGraph, None, at.clone(), &node.node_id))?;
        let body = view(term, at.clone(), meter)?
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
        // The subjects' aliases, sorted for lookup; the first repeat refuses.
        let mut aliases = meter.vec::<(&str, usize)>(body.subjects.len())?;
        aliases.extend(
            body.subjects
                .iter()
                .enumerate()
                .map(|(i, &(alias, _))| (alias, i)),
        );
        aliases.sort_unstable();
        if let Some(i) = first_repeated(&aliases) {
            return Err(refuse(
                Code::InvalidSemanticGraph,
                None,
                subject_at.clone().index(i),
                &node.node_id,
            ));
        }
        let mut variable_keys = meter.vec::<(&str, usize)>(body.executions.len())?;
        variable_keys.extend(
            body.executions
                .iter()
                .enumerate()
                .map(|(i, entry)| (entry.variable, i)),
        );
        variable_keys.sort_unstable();
        let repeated_variable = first_repeated(&variable_keys);
        drop(variable_keys);
        let mut named = meter.vec::<&str>(body.executions.len())?;
        for (i, entry) in body.executions.iter().enumerate() {
            if repeated_variable == Some(i) {
                return Err(refuse(
                    Code::InvalidSemanticGraph,
                    None,
                    execution_at.clone().index(i),
                    &node.node_id,
                ));
            }
            if graph
                .node(entry.alias_type)
                .is_none_or(|(_, kind)| kind != CheckedNodeKind::ScalarType(ScalarTypeForm::Text))
            {
                return Err(refuse(
                    Code::InvalidSemanticGraph,
                    None,
                    execution_at
                        .clone()
                        .index(i)
                        .key("value")
                        .key("members")
                        .index(0),
                    &node.node_id,
                ));
            }
            if aliases
                .binary_search_by(|(alias, _)| (*alias).cmp(entry.alias))
                .is_err()
            {
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
            named.push(entry.alias);
        }
        named.sort_unstable();
        for (i, &(alias, _)) in body.subjects.iter().enumerate() {
            if named.binary_search(&alias).is_err() {
                return Err(refuse(
                    Code::InvalidSemanticGraph,
                    None,
                    subject_at.clone().index(i),
                    &node.node_id,
                ));
            }
        }
        drop(named);
        // The first occurrence of each execution alias, in source order.
        let mut occurrences = meter.vec::<(&str, usize)>(body.executions.len())?;
        occurrences.extend(
            body.executions
                .iter()
                .enumerate()
                .map(|(i, entry)| (entry.alias, i)),
        );
        occurrences.sort_unstable();
        let mut heads = meter.vec::<usize>(occurrences.len())?;
        heads.extend(
            occurrences
                .iter()
                .enumerate()
                .filter(|&(at, (alias, _))| {
                    at == 0
                        || occurrences
                            .get(at - 1)
                            .is_none_or(|(before, _)| before != alias)
                })
                .map(|(_, &(_, first))| first),
        );
        heads.sort_unstable();
        for (i, (&(alias, _), &first)) in body.subjects.iter().zip(&heads).enumerate() {
            if body.executions.get(first).map(|entry| entry.alias) != Some(alias) {
                return Err(refuse(
                    Code::InvalidSemanticGraph,
                    None,
                    subject_at.clone().index(i),
                    &node.node_id,
                ));
            }
        }
        drop(heads);
        drop(occurrences);
        let mut subject_models = meter.vec::<(&str, &str)>(body.subjects.len())?;
        for (i, &(alias, target)) in body.subjects.iter().enumerate() {
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
            subject_models.push((alias, model));
        }
        subject_models.sort_unstable_by(|left, right| left.0.cmp(right.0));
        let model_of = |alias: &str| {
            subject_models
                .binary_search_by(|(found, _)| (*found).cmp(alias))
                .ok()
                .map(|at| subject_models[at].1)
        };
        let mut anchors = meter.vec(body.executions.len())?;
        for (i, entry) in body.executions.iter().enumerate() {
            let Some((_, kind)) = graph.node(entry.operation) else {
                return Err(malformed(
                    execution_at
                        .clone()
                        .index(i)
                        .key("value")
                        .key("members")
                        .index(1)
                        .key("value")
                        .key("target"),
                    entry.operation,
                ));
            };
            let anchor_body = graph
                .index
                .get(entry.operation)
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
                return Err(malformed(operation_at, entry.operation));
            };
            let Some(model) = model_of(entry.alias) else {
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
            };
            let context_ok = graph
                .node(anchor_body.context)
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
                    entry.operation,
                ));
            }
            if let Some(implementation) = entry.implementation {
                let valid = graph.node(implementation).is_some_and(|(_, kind)| {
                    kind == CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
                        && graph
                            .index
                            .get(implementation)
                            .and_then(|&pos| terms.get(pos))
                            .and_then(Option::as_ref)
                            .is_some_and(|term| match term {
                                CheckedTerm::AbstractionRelation { frames, .. } => {
                                    frames.iter().any(|frame| {
                                        frame.context == *anchor_body.context
                                            && frame.operation.as_ref() == anchor_body.operation
                                    })
                                }
                                _ => false,
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
            let Some((context, kind)) = graph.node(anchor.context) else {
                return Err(malformed(param_at, entry.operation));
            };
            let expected =
                super::state::resolve_operation(context, kind, anchor.operation, &graph, meter)?;
            let Ok(Some((owner, operation))) = expected else {
                return Err(malformed(param_at, entry.operation));
            };
            let expected_count =
                1 + usize::from(operation.result.is_some()) + operation.parameters.len();
            if entry.parameters.len() != expected_count {
                return Err(malformed(param_at, entry.operation));
            }
            for (j, id) in targets(entry.parameters).enumerate() {
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
                let type_ok = if j == 0 {
                    graph
                        .index
                        .get(&parameter.semantic_type)
                        .filter(|&&at| {
                            graph.kinds.get(at)
                                == Some(&CheckedNodeKind::CompositeType(
                                    CompositeTypeForm::Reference,
                                ))
                        })
                        .and_then(|&at| terms.get(at))
                        .and_then(Option::as_ref)
                        .and_then(super::structural::reference_type_target_ref)
                        == Some(anchor.context)
                } else {
                    operation
                        .result
                        .iter()
                        .chain(&operation.parameters)
                        .nth(j - 1)
                        .and_then(|slot| owner.package.slot_type(slot))
                        .and_then(|kind| kind.node_id(&parameter.node_id.package))
                        .as_ref()
                        == Some(&parameter.semantic_type)
                };
                let declared_ok = declared.map(|(name, text)| (name, spells(text, level)))
                    == expected_name.map(|name| (name, true));
                if kind != CheckedNodeKind::Value(ValueForm::Parameter) || !type_ok || !declared_ok
                {
                    return Err(malformed(param_at.clone(), id));
                }
                level += 1;
            }
            for (slot, clause_kind, provided) in [
                (3, StateClauseKind::Precondition, entry.preconditions),
                (4, StateClauseKind::Postcondition, entry.postconditions),
            ] {
                let matches = clauses_are(
                    &graph,
                    terms,
                    meter,
                    &|| execution_at.clone().index(i),
                    provided,
                    |parsed| parsed.clause == clause_kind && *parsed.anchor == *entry.operation,
                )?;
                if !matches {
                    return Err(malformed(
                        execution_at
                            .clone()
                            .index(i)
                            .key("value")
                            .key("members")
                            .index(slot)
                            .key("value"),
                        entry.operation,
                    ));
                }
            }
        }
        if body.invariants.len() != body.subjects.len() {
            return Err(malformed(invariant_at, &node.node_id));
        }
        for (i, (&(alias, model_id), &(found, provided))) in
            body.subjects.iter().zip(&body.invariants).enumerate()
        {
            let path = invariant_at.clone().index(i);
            if alias != found {
                return Err(malformed(path, model_id));
            }
            let Some(model) = model_of(alias) else {
                return Err(malformed(path, model_id));
            };
            let matches =
                clauses_are(&graph, terms, meter, &|| path.clone(), provided, |parsed| {
                    parsed.clause == StateClauseKind::Invariant
                        && graph.node(parsed.anchor).is_some_and(|(context, kind)| {
                            matches!(
                                kind,
                                CheckedNodeKind::Model(
                                    ModelForm::ObjectType
                                        | ModelForm::Process
                                        | ModelForm::StateMachine
                                )
                            ) && same_model(context, model, &graph)
                        })
                })?;
            if !matches {
                return Err(malformed(path.key("value"), model_id));
            }
        }
        let valid_body = graph.node(body.body).is_some_and(|(expression, kind)| {
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
                body.body,
            ));
        }
        let binder_count = body
            .executions
            .iter()
            .map(|entry| entry.parameters.len())
            .sum();
        let mut binders = meter.vec::<&CheckedNodeId>(binder_count)?;
        binders.extend(
            body.executions
                .iter()
                .flat_map(|entry| targets(entry.parameters)),
        );
        binders.sort_unstable();
        // The nodes met, by position; a target that names no node is met once
        // per distinct id, in a small sorted table.
        let mut seen = meter.filled(nodes.len(), false)?;
        let mut unresolved: Vec<&CheckedNodeId> = Vec::new();
        let mut pending = meter.vec::<&CheckedNodeId>(1)?;
        pending.push(body.body);
        while let Some(target) = pending.pop() {
            match graph.index.get(target) {
                Some(&met) => {
                    if seen.get(met).copied().unwrap_or(true) {
                        continue;
                    }
                    if let Some(flag) = seen.get_mut(met) {
                        *flag = true;
                    }
                }
                None => match unresolved.binary_search(&target) {
                    Ok(_) => continue,
                    Err(at) => {
                        meter.room(&mut unresolved, 1)?;
                        unresolved.insert(at, target);
                    }
                },
            }
            meter.charge(1, || body_at.clone())?;
            let Some((read, kind)) = graph.node(target) else {
                continue;
            };
            if kind == CheckedNodeKind::Value(ValueForm::Parameter) {
                if binders.binary_search(&target).is_err() {
                    return Err(malformed(body_at.clone(), target));
                }
            } else {
                meter.room(&mut pending, read.dependencies.len())?;
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
