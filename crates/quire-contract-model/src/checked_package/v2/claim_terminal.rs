//! FR-059 and FR-060: state-graph claims and model terminal declarations.
//!
//! These are separate body steps. Shape and form fit are checked before target
//! meaning, and all terminal nodes are checked before uniqueness by model.

use super::frame::StepGraph;
use super::graph_storage::GraphIndex;
use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::structural::{reference_type_target_ref, typed_parameter_binding};
use super::{
    CheckedNodeKind, CheckedNodeOwner, CheckedSemanticNodeV2, CheckedTerm, ClaimForm,
    CompositeTypeForm, LiteralKind, ModelForm, ScalarTypeForm, TemporalForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedTerminalDuplicate,
};

fn failure(
    meter: &mut WorkMeter,
    position: usize,
    code: Code,
    cause: Option<Cause>,
    path: Path<'_>,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    let mut refusal = Refusal::new(code, path).at_node(locus);
    refusal.cause = cause;
    stage_storage::refuse(
        meter,
        &Site::new(PackageReadStage::Graph, position, "claim", path),
        refusal,
    )
}

fn body_path(position: usize) -> Path<'static> {
    Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position)
        .key("body")
}

fn binding_path(position: usize, slot: usize) -> Path<'static> {
    body_path(position).key("members").index(slot)
}

fn target_path(position: usize, slot: usize) -> Path<'static> {
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

fn word<'a, Body>(term: &'a CheckedTerm, graph: &StepGraph<'_, '_, Body>) -> Option<&'a str> {
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

fn boolean_type<Body>(node: &CheckedSemanticNodeV2<Body>, graph: &StepGraph<'_, '_, Body>) -> bool {
    graph
        .node(&node.semantic_type)
        .is_some_and(|(_, kind)| kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
}

fn self_object_key<'g, Body: super::model_members::EmptyAggregateBody>(
    graph: &'g StepGraph<'_, '_, Body>,
    model: &CheckedNodeId,
    meter: &mut WorkMeter,
    position: usize,
) -> Result<Option<&'g CheckedNodeId>, ValidationFailure> {
    let Some((model_node, _)) = graph.node(model) else {
        return Ok(None);
    };
    let Ok(owner) = graph.owners.recover(model_node) else {
        return Ok(None);
    };
    // The one object type the population covers, and how often; a second
    // distinct object ends the search.
    let mut covered: Option<(&str, usize)> = None;
    let mut distinct = false;
    for members in owner.package.population_members.values() {
        stage_storage::work(
            meter,
            &Site::new(
                PackageReadStage::Graph,
                position,
                "claim",
                binding_path(position, 2),
            ),
            1,
        )?;
        for object in members {
            stage_storage::work(
                meter,
                &Site::new(
                    PackageReadStage::Graph,
                    position,
                    "claim",
                    binding_path(position, 2),
                ),
                1,
            )?;
            if owner.package.object_types.contains_key(object) {
                match &mut covered {
                    Some((held, count)) if *held == &**object => *count += 1,
                    Some(_) => distinct = true,
                    None => covered = Some((&**object, 1)),
                }
            }
        }
    }
    if distinct {
        return Ok(None);
    }
    let Some((object, count)) = covered else {
        return Ok(None);
    };
    if count != 1 {
        return Ok(None);
    }
    let mut candidates = graph.nodes.iter().zip(graph.kinds).filter(|(node, kind)| {
        **kind == CheckedNodeKind::Model(ModelForm::ObjectType)
            && matches!(&node.owner,
                    Some(CheckedNodeOwner::Model { identity, node: declaration })
                        if identity.as_ref() == owner.package.identity.as_ref()
                            && declaration.as_ref() == object)
    });
    let Some((candidate, _)) = candidates.next() else {
        return Ok(None);
    };
    if candidates.next().is_some() {
        return Ok(None);
    }
    Ok(Some(&candidate.node_id))
}

fn self_reads<'g, Body>(
    condition: &'g CheckedNodeId,
    graph: &'g StepGraph<'_, '_, Body>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
    position: usize,
) -> Result<Vec<&'g CheckedNodeId>, ValidationFailure> {
    // Wire IDs are flat, so ordering them never allocates; the tables are
    // charged sorted slices of borrowed IDs.
    let mut seen: Vec<&CheckedNodeId> = Vec::new();
    let mut pending: Vec<&CheckedNodeId> = Vec::new();
    meter.push(&mut pending, condition)?;
    let mut reads: Vec<&CheckedNodeId> = Vec::new();
    while let Some(key) = pending.pop() {
        if !meter.insert_sorted(&mut seen, key)? {
            continue;
        }
        stage_storage::work(
            meter,
            &Site::new(
                PackageReadStage::Graph,
                position,
                "claim",
                binding_path(position, 3),
            ),
            1,
        )?;
        let Some((node, kind)) = graph.node(key) else {
            continue;
        };
        if kind == CheckedNodeKind::Value(ValueForm::Parameter) {
            let declaration = graph
                .index
                .get(key)
                .and_then(|&at| terms.get(at))
                .and_then(Option::as_ref)
                .and_then(|body| {
                    typed_parameter_binding(body, &|id| graph.node(id).map(|(_, kind)| kind))
                });
            if declaration == Some(("self", "0")) {
                meter.insert_sorted(&mut reads, key)?;
            }
        } else {
            meter.room(&mut pending, node.dependencies.len())?;
            pending.extend(node.dependencies.iter());
        }
    }
    Ok(reads)
}

fn claim_occurrences<Body>(
    node: &CheckedSemanticNodeV2<Body>,
    position: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for (ordinal, occurrence) in node.occurrences.iter().enumerate() {
        if occurrence.role != CheckedOccurrenceRole::Claim {
            return Err(failure(
                meter,
                position,
                Code::InvalidSemanticGraph,
                None,
                Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(position)
                    .key("occurrences")
                    .index(ordinal)
                    .key("role"),
                &node.node_id,
            ));
        }
    }
    Ok(())
}

fn graph_nodes<'a, 'm, Body>(
    nodes: &'a [CheckedSemanticNodeV2<Body>],
    kinds: &'a [CheckedNodeKind],
    index: &'a GraphIndex<'a>,
    owners: &'a super::model_members::ModelOwners<'m>,
) -> StepGraph<'a, 'm, Body> {
    StepGraph {
        nodes,
        kinds,
        index,
        owners,
    }
}

/// FR-475's state-graph body step, before the temporal step.
pub(super) fn validate_state_graph<Body>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    owners: &super::model_members::ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = graph_nodes(nodes, kinds, index, owners);
    for (position, node) in graph.each(CheckedNodeKind::Claim(ClaimForm::StateGraph)) {
        meter.at_node(position, "claim");
        let [kind, over, from, target] = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(|term| bindings(term, ["kind", "over", "from", "target"]))
            .ok_or_else(|| {
                failure(
                    meter,
                    position,
                    Code::InvalidSemanticGraph,
                    None,
                    body_path(position),
                    &node.node_id,
                )
            })?;
        let form = word(kind, &graph);
        if !matches!(form, Some("possible" | "always-possible" | "unique-path")) {
            return Err(failure(
                meter,
                position,
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                binding_path(position, 0),
                &node.node_id,
            ));
        }
        if !(empty(over) || reference(over).is_some()) {
            return Err(failure(
                meter,
                position,
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
                meter,
                position,
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            ));
        }
        let Some(target) = reference(target) else {
            return Err(failure(
                meter,
                position,
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
                    meter,
                    position,
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
                    meter,
                    position,
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    target_path(position, slot),
                    predicate,
                ));
            }
        }
        if !boolean_type(node, &graph) || node.owner.is_some() {
            return Err(failure(
                meter,
                position,
                Code::InvalidSemanticGraph,
                None,
                Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(position),
                &node.node_id,
            ));
        }
        claim_occurrences(node, position, meter)?;
    }
    Ok(())
}

/// FR-476's terminal body step, after abstraction and before later body steps.
pub(super) fn validate_terminal<Body: super::model_members::EmptyAggregateBody>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    owners: &super::model_members::ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = graph_nodes(nodes, kinds, index, owners);
    let mut by_model: Vec<(&CheckedNodeId, &CheckedSemanticNodeV2<Body>)> = Vec::new();
    let mut terminals = Vec::new();
    for (position, node) in graph.each(CheckedNodeKind::Temporal(TemporalForm::Terminal)) {
        meter.at_node(position, "temporal");
        let [model, mode, parameters, condition] = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(|term| bindings(term, ["model", "mode", "parameters", "condition"]))
            .ok_or_else(|| {
                failure(
                    meter,
                    position,
                    Code::InvalidSemanticGraph,
                    None,
                    body_path(position),
                    &node.node_id,
                )
            })?;
        let Some(model) = reference(model) else {
            return Err(failure(
                meter,
                position,
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 0),
                &node.node_id,
            ));
        };
        let mode = word(mode, &graph);
        if !matches!(mode, Some("when" | "any")) {
            return Err(failure(
                meter,
                position,
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                binding_path(position, 1),
                &node.node_id,
            ));
        }
        let parameters = aggregate(parameters).ok_or_else(|| {
            failure(
                meter,
                position,
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            )
        })?;
        if parameters.len() > 1 || (mode == Some("any") && !parameters.is_empty()) {
            return Err(failure(
                meter,
                position,
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 2),
                &node.node_id,
            ));
        }
        let parameter = parameters.first().and_then(reference);
        if !parameters.is_empty() && parameter.is_none() {
            return Err(failure(
                meter,
                position,
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
                meter,
                position,
                Code::InvalidSemanticGraph,
                None,
                binding_path(position, 3),
                &node.node_id,
            ));
        }
        let model_kind = graph.node(model).map(|(_, kind)| kind);
        if !matches!(model_kind, Some(CheckedNodeKind::Model(_))) {
            return Err(failure(
                meter,
                position,
                Code::InvalidModelBinding,
                Some(Cause::MalformedDeclaration),
                target_path(position, 0),
                model,
            ));
        }
        if let Some(parameter) = parameter {
            let candidate = graph.node(parameter).and_then(|(target_node, kind)| {
                (kind == CheckedNodeKind::Value(ValueForm::Parameter)).then_some(target_node)
            });
            let target_position = graph.index.get(parameter).copied();
            let name_level = target_position
                .and_then(|at| terms.get(at))
                .and_then(Option::as_ref)
                .and_then(|term| {
                    typed_parameter_binding(term, &|id| graph.node(id).map(|(_, kind)| kind))
                });
            let reference_target = candidate
                .and_then(|target_node| graph.node(&target_node.semantic_type))
                .filter(|(_, kind)| {
                    *kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Reference)
                })
                .and_then(|(type_node, _)| graph.index.get(&type_node.node_id))
                .and_then(|&at| terms.get(at))
                .and_then(Option::as_ref)
                .and_then(reference_type_target_ref);
            let valid = if name_level == Some(("self", "0")) && reference_target.is_some() {
                let expected = self_object_key(&graph, model, meter, position)?;
                reference_target
                    .as_ref()
                    .is_some_and(|target| expected == Some(*target))
            } else {
                false
            };
            if !valid {
                return Err(failure(
                    meter,
                    position,
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
                    meter,
                    position,
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    target_path(position, 3),
                    condition,
                ));
            };
            if !matches!(kind, CheckedNodeKind::Expression(_)) {
                return Err(failure(
                    meter,
                    position,
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    target_path(position, 3),
                    condition,
                ));
            }
            if !boolean_type(target_node, &graph) {
                return Err(failure(
                    meter,
                    position,
                    Code::IllTyped,
                    Some(Cause::NonBooleanRoot),
                    target_path(position, 3),
                    condition,
                ));
            }
            let reads = self_reads(condition, &graph, terms, meter, position)?;
            if reads.len() != usize::from(parameter.is_some())
                || parameter.is_some_and(|parameter| reads.binary_search(&parameter).is_err())
            {
                return Err(failure(
                    meter,
                    position,
                    Code::InvalidSemanticGraph,
                    None,
                    binding_path(position, 2),
                    &node.node_id,
                ));
            }
        }
        if !boolean_type(node, &graph) {
            return Err(failure(
                meter,
                position,
                Code::InvalidSemanticGraph,
                None,
                Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(position)
                    .key("semantic_type"),
                &node.node_id,
            ));
        }
        claim_occurrences(node, position, meter)?;
        meter.push(&mut terminals, (position, node, model))?;
    }
    for (position, node, model) in terminals {
        meter.at_node(position, "temporal");
        let first = match by_model.binary_search_by(|(held, _)| (*held).cmp(model)) {
            Ok(at) => Some(by_model[at].1),
            Err(at) => {
                meter.room(&mut by_model, 1)?;
                by_model.insert(at, (model, node));
                None
            }
        };
        if let Some(first) = first {
            meter.charge_id_memory(&first.node_id)?;
            meter.charge_id_memory(&node.node_id)?;
            let mut refusal = failure(
                meter,
                position,
                Code::AmbiguousDeclaration,
                Some(Cause::AmbiguousName),
                Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(position),
                &node.node_id,
            );
            if let ValidationFailure::Refused(ref mut value) = refusal {
                value.terminal_duplicate = Some(CheckedTerminalDuplicate::Nodes {
                    first: first.node_id.clone(),
                    second: node.node_id.clone(),
                });
            }
            return Err(refusal);
        }
        if node.occurrences.len() > 1 {
            meter.charge_id_memory(&node.node_id)?;
            let mut refusal = failure(
                meter,
                position,
                Code::AmbiguousDeclaration,
                Some(Cause::AmbiguousName),
                Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(position),
                &node.node_id,
            );
            if let ValidationFailure::Refused(ref mut value) = refusal {
                value.terminal_duplicate = Some(CheckedTerminalDuplicate::Occurrences {
                    node: node.node_id.clone(),
                    first: node.occurrences[0].clone(),
                    second: node.occurrences[1].clone(),
                });
            }
            return Err(refusal);
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
    fn typed_claim_shape_fault_prepays_exact_path_and_held_node() {
        let id = node_id('a');
        let node: CheckedSemanticNodeV2<CheckedJson> = serde_json::from_value(json!({"node_id":id,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"claim","semantic_form":"state_graph","semantic_type":id,"dependencies":[],"occurrences":[],"body":{"term":"aggregate","members":[]}})).unwrap();
        let nodes = [node];
        let kinds = [CheckedNodeKind::Claim(ClaimForm::StateGraph)];
        let terms = [Some(CheckedTerm::Aggregate {
            members: Vec::new(),
        })];
        let index = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (&n.node_id, i))
            .collect::<GraphIndex<'_>>();
        let owners = super::super::model_members::ModelOwners::default();
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                result = Some(validate_state_graph(
                    &nodes,
                    &kinds,
                    &terms,
                    &index,
                    &owners,
                    &mut WorkMeter::over(&mut budget),
                ));
            });
            (result.unwrap(), measured.bytes_total, budget)
        };
        let (result, allocated, budget) = run(u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("missing claim fields");
        };
        assert_eq!(refusal.code, Code::InvalidSemanticGraph);
        assert_eq!(refusal.locus.as_ref(), Some(&nodes[0].node_id));
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            "/semantic_graph/nodes/0/body"
        );
        let required = budget.read_storage_consumed();
        assert_eq!(allocated, required);
        assert!(matches!(
            run(required).0,
            Err(ValidationFailure::Refused(_))
        ));
        assert!(matches!(
            run(required - 1).0,
            Err(ValidationFailure::Budget(_))
        ));
        assert_eq!(run(0).1, 0);
    }
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn typed_terminal_duplicate_pays_tables_and_all_three_held_ids() {
        let ids = ['a', 'b', 'c', 'd', 'e'].map(node_id);
        let nodes: Vec<CheckedSemanticNodeV2<CheckedJson>> = ids.iter().map(|id| serde_json::from_value(json!({"node_id":id,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"temporal","semantic_form":"terminal","semantic_type":ids[2],"dependencies":[],"occurrences":[],"body":{"term":"aggregate","members":[]}})).unwrap()).collect();
        let kinds = [
            CheckedNodeKind::Temporal(TemporalForm::Terminal),
            CheckedNodeKind::Temporal(TemporalForm::Terminal),
            CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean),
            CheckedNodeKind::Model(ModelForm::ObjectType),
            CheckedNodeKind::ScalarType(ScalarTypeForm::Text),
        ];
        let terminal = || CheckedTerm::Aggregate {
            members: vec![
                CheckedTerm::Binding {
                    name: "model".into(),
                    value: Box::new(CheckedTerm::Reference {
                        target: ids[3].clone(),
                    }),
                },
                CheckedTerm::Binding {
                    name: "mode".into(),
                    value: Box::new(CheckedTerm::Literal {
                        type_id: ids[4].clone(),
                        value_kind: LiteralKind::Text,
                        value: serde_json::from_value(json!("any")).unwrap(),
                    }),
                },
                CheckedTerm::Binding {
                    name: "parameters".into(),
                    value: Box::new(CheckedTerm::Aggregate { members: vec![] }),
                },
                CheckedTerm::Binding {
                    name: "condition".into(),
                    value: Box::new(CheckedTerm::Aggregate { members: vec![] }),
                },
            ],
        };
        let terms = [Some(terminal()), Some(terminal()), None, None, None];
        let index = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (&n.node_id, i))
            .collect::<GraphIndex<'_>>();
        let owners = super::super::model_members::ModelOwners::default();
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                result = Some(validate_terminal(
                    &nodes,
                    &kinds,
                    &terms,
                    &index,
                    &owners,
                    &mut WorkMeter::over(&mut budget),
                ));
            });
            (result.unwrap(), measured.bytes_total, budget)
        };
        let (result, allocated, budget) = run(u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("second terminal must refuse");
        };
        assert_eq!(refusal.code, Code::AmbiguousDeclaration);
        assert_eq!(refusal.cause, Some(Cause::AmbiguousName));
        assert_eq!(refusal.locus.as_ref(), Some(&ids[1]));
        assert_eq!(
            refusal.terminal_duplicate,
            Some(CheckedTerminalDuplicate::Nodes {
                first: ids[0].clone(),
                second: ids[1].clone()
            })
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            "/semantic_graph/nodes/1"
        );
        let required = budget.read_storage_consumed();
        assert_eq!(allocated, required);
        assert!(matches!(
            run(required).0,
            Err(ValidationFailure::Refused(_))
        ));
        assert!(matches!(
            run(required - 1).0,
            Err(ValidationFailure::Budget(_))
        ));
        assert_eq!(run(0).1, 0);
    }
}
