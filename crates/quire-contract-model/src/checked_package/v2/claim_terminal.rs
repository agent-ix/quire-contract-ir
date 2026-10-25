//! FR-059 and FR-060: state-graph claims and model terminal declarations.
//!
//! These are separate body steps. Shape and form fit are checked before target
//! meaning, and all terminal nodes are checked before uniqueness by model.

use super::frame::StepGraph;
use super::model_members::{declaration_key, DeclarationForm};
use super::structural::{reference_type_target, typed_parameter_binding};
use super::{
    CheckedNodeKind, CheckedTerm, ClaimForm, CompositeTypeForm, LiteralKind,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, ScalarTypeForm, TemporalForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, JsonPointer,
};
use std::collections::{BTreeMap, BTreeSet};

fn failure(
    code: Code,
    cause: Option<Cause>,
    path: JsonPointer,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, path, cause, locus.clone())
}

fn body_path(position: usize) -> JsonPointer {
    node_pointer(position).key("body")
}

fn binding_path(position: usize, slot: usize) -> JsonPointer {
    body_path(position).key("members").index(slot)
}

fn target_path(position: usize, slot: usize) -> JsonPointer {
    binding_path(position, slot).key("value").key("target")
}

fn aggregate(term: &CheckedTerm) -> Option<&[CheckedTerm]> {
    match term {
        CheckedTerm::Aggregate { members } => Some(members),
        _ => None,
    }
}

fn binding_value<'a>(member: &'a CheckedTerm, expected: &str) -> Option<&'a CheckedTerm> {
    match member {
        CheckedTerm::Binding { name, value } if name.as_ref() == expected => Some(value),
        _ => None,
    }
}

fn bindings<'a>(term: &'a CheckedTerm, names: [&str; 4]) -> Option<[&'a CheckedTerm; 4]> {
    let [first, second, third, fourth] = aggregate(term)? else {
        return None;
    };
    Some([
        binding_value(first, names[0])?,
        binding_value(second, names[1])?,
        binding_value(third, names[2])?,
        binding_value(fourth, names[3])?,
    ])
}

fn reference(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

fn empty(term: &CheckedTerm) -> bool {
    aggregate(term).is_some_and(<[CheckedTerm]>::is_empty)
}

fn word<'a>(term: &'a CheckedTerm, graph: &StepGraph<'_, '_>) -> Option<&'a str> {
    let CheckedTerm::Literal {
        type_id,
        value_kind: LiteralKind::Text,
        value,
    } = term
    else {
        return None;
    };
    graph
        .node(type_id)
        .filter(|(_, kind)| *kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Text))?;
    value.as_str()
}

fn boolean_type(node: &CheckedSemanticNodeV2, graph: &StepGraph<'_, '_>) -> bool {
    graph
        .node(&node.semantic_type)
        .is_some_and(|(_, kind)| kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
}

fn self_object_key(graph: &StepGraph<'_, '_>, model: &CheckedNodeId) -> Option<String> {
    let (model_node, _) = graph.node(model)?;
    let owner = graph.owners.recover(model_node).ok()?;
    let mut covered = BTreeMap::<&str, usize>::new();
    for members in owner.package.population_members.values() {
        for object in members {
            if owner.package.object_types.contains_key(object) {
                *covered.entry(object).or_default() += 1;
            }
        }
    }
    if covered.len() != 1 {
        return None;
    }
    let (object, count) = covered.into_iter().next()?;
    if count != 1 {
        return None;
    }
    declaration_key(
        &owner.package.identity,
        DeclarationForm::ObjectType,
        object,
        owner.package.bytes,
    )
    .ok()
}

fn self_reads(
    condition: &CheckedNodeId,
    graph: &StepGraph<'_, '_>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
    position: usize,
) -> Result<BTreeSet<CheckedNodeId>, ValidationFailure> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![condition.clone()];
    let mut reads = BTreeSet::new();
    while let Some(key) = pending.pop() {
        if !seen.insert(key.clone()) {
            continue;
        }
        meter.charge(1, || binding_path(position, 3))?;
        let Some((node, kind)) = graph.node(&key) else {
            continue;
        };
        if kind == CheckedNodeKind::Value(ValueForm::Parameter) {
            let declaration = graph
                .index
                .get(&key)
                .and_then(|&at| terms.get(at))
                .and_then(Option::as_ref)
                .and_then(|body| {
                    typed_parameter_binding(body, &|id| graph.node(id).map(|(_, kind)| kind))
                });
            if declaration == Some(("self", "0")) {
                reads.insert(key);
            }
        } else {
            pending.extend(node.dependencies.iter().cloned());
        }
    }
    Ok(reads)
}

fn claim_occurrences(
    node: &CheckedSemanticNodeV2,
    position: usize,
) -> Result<(), ValidationFailure> {
    for (ordinal, occurrence) in node.occurrences.iter().enumerate() {
        if occurrence.role != CheckedOccurrenceRole::Claim {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(position)
                    .key("occurrences")
                    .index(ordinal)
                    .key("role"),
                &node.node_id,
            ));
        }
    }
    Ok(())
}

fn graph_nodes<'a, 'm>(
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    owners: &'a super::model_members::ModelOwners<'m>,
) -> StepGraph<'a, 'm> {
    StepGraph {
        nodes,
        kinds,
        index,
        owners,
    }
}

/// FR-475's state-graph body step, before the temporal step.
pub(super) fn validate_state_graph(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    owners: &super::model_members::ModelOwners<'_>,
) -> Result<(), ValidationFailure> {
    let graph = graph_nodes(nodes, kinds, index, owners);
    for (position, node) in graph.each(CheckedNodeKind::Claim(ClaimForm::StateGraph)) {
        let [kind, over, from, target] = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(|term| bindings(term, ["kind", "over", "from", "target"]))
            .ok_or_else(|| {
                failure(
                    Code::InvalidSemanticGraph,
                    None,
                    body_path(position),
                    &node.node_id,
                )
            })?;
        let form = word(kind, &graph);
        if !matches!(form, Some("possible" | "always-possible" | "unique-path")) {
            return Err(failure(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                binding_path(position, 0),
                &node.node_id,
            ));
        }
        if !(empty(over) || reference(over).is_some()) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 1),
                &node.node_id,
            ));
        }
        if !(if form == Some("possible") {
            empty(from)
        } else {
            reference(from).is_some()
        }) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            ));
        }
        let Some(target) = reference(target) else {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 3),
                &node.node_id,
            ));
        };
        if let Some(over) = reference(over) {
            let valid = graph
                .node(over)
                .is_some_and(|(_, kind)| kind == CheckedNodeKind::Value(ValueForm::Parameter));
            if !valid {
                return Err(failure(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    target_path(position, 1),
                    over,
                ));
            }
        }
        for (slot, predicate) in [(2, reference(from)), (3, Some(target))] {
            let Some(predicate) = predicate else { continue };
            let valid = graph.node(predicate).is_some_and(|(target_node, kind)| {
                matches!(
                    kind,
                    CheckedNodeKind::Expression(_) | CheckedNodeKind::Value(ValueForm::Literal)
                ) && boolean_type(target_node, &graph)
            });
            if !valid {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    target_path(position, slot),
                    predicate,
                ));
            }
        }
        if !boolean_type(node, &graph) || node.owner.is_some() {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(position),
                &node.node_id,
            ));
        }
        claim_occurrences(node, position)?;
    }
    Ok(())
}

/// FR-476's terminal body step, after abstraction and before later body steps.
pub(super) fn validate_terminal(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    owners: &super::model_members::ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = graph_nodes(nodes, kinds, index, owners);
    let mut by_model = BTreeMap::<&CheckedNodeId, usize>::new();
    let mut terminals = Vec::new();
    for (position, node) in graph.each(CheckedNodeKind::Temporal(TemporalForm::Terminal)) {
        let [model, mode, parameters, condition] = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(|term| bindings(term, ["model", "mode", "parameters", "condition"]))
            .ok_or_else(|| {
                failure(
                    Code::InvalidSemanticGraph,
                    None,
                    body_path(position),
                    &node.node_id,
                )
            })?;
        let Some(model) = reference(model) else {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 0),
                &node.node_id,
            ));
        };
        let mode = word(mode, &graph);
        if !matches!(mode, Some("when" | "any")) {
            return Err(failure(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                binding_path(position, 1),
                &node.node_id,
            ));
        }
        let parameters = aggregate(parameters).ok_or_else(|| {
            failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            )
        })?;
        if parameters.len() > 1 || (mode == Some("any") && !parameters.is_empty()) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            ));
        }
        let parameter = parameters.first().and_then(reference);
        if !parameters.is_empty() && parameter.is_none() {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            ));
        }
        if !(if mode == Some("when") {
            reference(condition).is_some()
        } else {
            empty(condition)
        }) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 3),
                &node.node_id,
            ));
        }
        let model_kind = graph.node(model).map(|(_, kind)| kind);
        if !matches!(model_kind, Some(CheckedNodeKind::Model(_))) {
            return Err(failure(
                Code::InvalidModelBinding,
                Some(Cause::MalformedDeclaration),
                target_path(position, 0),
                model,
            ));
        }
        if let Some(parameter) = parameter {
            let valid = graph.node(parameter).is_some_and(|(target_node, kind)| {
                if kind != CheckedNodeKind::Value(ValueForm::Parameter) {
                    return false;
                }
                let Some(&target_position) = graph.index.get(parameter) else {
                    return false;
                };
                let name_level = terms
                    .get(target_position)
                    .and_then(Option::as_ref)
                    .and_then(|term| {
                        typed_parameter_binding(term, &|id| graph.node(id).map(|(_, kind)| kind))
                    });
                let reference_target = graph
                    .node(&target_node.semantic_type)
                    .filter(|(_, kind)| {
                        *kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Reference)
                    })
                    .and_then(|(type_node, _)| reference_type_target(type_node));
                name_level == Some(("self", "0"))
                    && reference_target.as_ref().is_some_and(|target| {
                        self_object_key(&graph, model).as_deref() == Some(target.digest.as_ref())
                    })
            });
            if !valid {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    binding_path(position, 2)
                        .key("value")
                        .key("members")
                        .index(0)
                        .key("target"),
                    parameter,
                ));
            }
        }
        if let Some(condition) = reference(condition) {
            let Some((target_node, kind)) = graph.node(condition) else {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    target_path(position, 3),
                    condition,
                ));
            };
            if !matches!(kind, CheckedNodeKind::Expression(_)) {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    target_path(position, 3),
                    condition,
                ));
            }
            if !boolean_type(target_node, &graph) {
                return Err(failure(
                    Code::IllTyped,
                    Some(Cause::NonBooleanRoot),
                    target_path(position, 3),
                    condition,
                ));
            }
            let reads = self_reads(condition, &graph, terms, meter, position)?;
            if reads.len() != usize::from(parameter.is_some())
                || parameter.is_some_and(|parameter| !reads.contains(parameter))
            {
                return Err(failure(
                    Code::InvalidSemanticGraph,
                    None,
                    binding_path(position, 2),
                    &node.node_id,
                ));
            }
        }
        if !boolean_type(node, &graph) {
            return Err(failure(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(position).key("semantic_type"),
                &node.node_id,
            ));
        }
        claim_occurrences(node, position)?;
        terminals.push((position, node, model));
    }
    for (position, node, model) in terminals {
        if by_model.insert(model, position).is_some() {
            return Err(failure(
                Code::AmbiguousDeclaration,
                Some(Cause::AmbiguousName),
                node_pointer(position),
                &node.node_id,
            ));
        }
        if node.occurrences.len() > 1 {
            return Err(failure(
                Code::AmbiguousDeclaration,
                Some(Cause::AmbiguousName),
                node_pointer(position),
                &node.node_id,
            ));
        }
    }
    Ok(())
}
