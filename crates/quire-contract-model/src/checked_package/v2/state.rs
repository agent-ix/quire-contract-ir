//! The state step of the reader order (QSpec FR-341 and FR-342): after the
//! frame step and before the operation step, the reader checks, each in
//! ascending node-id digest order,
//!
//! 1. the placement of every `quire.op.state.clause` and
//!    `quire.op.state.transition` application: the body root of its matching
//!    state node and nowhere else, else
//!    `ill_typed`/`operator-ineligible` at the node that holds it;
//! 2. every `state`/`operation_anchor` node: its `context` and `frame`
//!    targets, the three `semantic_type`s, its operation name, and any scoped
//!    deletion parameters; then, once every anchor's own joins hold, that no
//!    two anchors share a (`context`, `operation`) pair;
//! 3. every `state`/`state_clause` node: its anchor, its parameters, then its
//!    signature;
//! 4. every `state`/`transition` node: anchor, subject, inputs, clauses and
//!    uniqueness,
//!
//! and reports the first defect.
//!
//! An operation name resolves through the selected domain package when the
//! anchor's context is a model declaration node, the scope FR-038's
//! "Model-owned members" gives the resolution, and the clause signature is
//! then checked against the resolved operation's parameters and result. A
//! context that carries its own `declaration` is declared by this package's
//! source and names no member in its body: its operation name and the
//! parameters past `self` are not resolved against any domain package.
//!
//! The body shapes themselves were checked by the graph-shape stage
//! (`structural`); a body that no longer parses here refuses as that stage
//! would.

use super::frame::{join, read_frame_body, DeletesScopedEntry, ModifiesEntry, StepGraph};
use super::model_members::{
    MemberKind, MemberType, ModelOwners, ModelRefusal, OperationDecl, Owner, Resolved,
};
use super::structural::{
    is_state_clause_application, is_state_transition_application, reference_type_target,
    typed_anchor_body, typed_clause_body, typed_parameter_binding, typed_reference,
    typed_transition_body, AnchorBody, TransitionBody,
};
use super::{
    ApplicationOperator, CheckedNodeKind, CheckedTerm, CompositeTypeForm, ExecutionPoint,
    ExpressionForm, ModelForm, RawSemanticNodeV2 as CheckedSemanticNodeV2, ScalarTypeForm,
    StateClauseKind, StateForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, Step, Trail, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use crate::checked_package::terms::{subterms, At, Cursor};
use quire_walk::{walk, Children, Walk};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::ControlFlow;

/// The state step. See the module documentation for the order.
pub(super) fn validate_state(
    nodes: &[CheckedSemanticNodeV2],
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
    validate_placement(&graph)?;
    let mut anchors = Vec::new();
    for (position, node) in graph.each(CheckedNodeKind::State(StateForm::OperationAnchor)) {
        let body = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(typed_anchor_body)
            .ok_or_else(|| shape_refusal(node, position))?;
        check_anchor(node, position, &body, &graph, meter)?;
        anchors.push((position, node, body));
    }
    let mut pairs = BTreeSet::new();
    for (position, node, body) in &anchors {
        if !pairs.insert((&body.context, body.operation, body.execution_point.as_ref())) {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::AmbiguousDeclaration,
                node_pointer(*position),
                Some(CheckedPackageRefusalCause::AmbiguousName),
                node.node_id.clone(),
            ));
        }
    }
    for (position, node) in graph.each(CheckedNodeKind::State(StateForm::StateClause)) {
        check_clause(node, position, terms, &graph, meter)?;
    }
    let mut pairs = BTreeSet::new();
    for (position, node) in graph.each(CheckedNodeKind::State(StateForm::Transition)) {
        let body = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(typed_transition_body)
            .ok_or_else(|| shape_refusal(node, position))?;
        check_transition(node, position, &body, terms, &graph, meter)?;
        if !pairs.insert((body.anchor, body.subject)) {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::AmbiguousDeclaration,
                node_pointer(position),
                Some(CheckedPackageRefusalCause::AmbiguousName),
                node.node_id.clone(),
            ));
        }
    }
    Ok(())
}

/// The graph-shape refusal of a body that is not its form's closed shape.
fn shape_refusal(node: &CheckedSemanticNodeV2, position: usize) -> ValidationFailure {
    ValidationFailure::refused_at(
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        node_pointer(position).key("body"),
        None,
        node.node_id.clone(),
    )
}

/// `ill_typed`/`operator-ineligible` at `path`, located at `node`.
fn ineligible(node: &CheckedSemanticNodeV2, path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused_at(
        CheckedPackageRefusalCode::IllTyped,
        path,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        node.node_id.clone(),
    )
}

/// A model refusal at `path`, located at `locus`.
fn model_refusal(
    refusal: ModelRefusal,
    path: JsonPointer,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(refusal.code, path, Some(refusal.cause), locus.clone())
}

/// Every `quire.op.state.clause` application stands as the body root of a
/// `state`/`state_clause` node.
fn validate_placement(graph: &StepGraph<'_, '_>) -> Result<(), ValidationFailure> {
    for transition in [false, true] {
        for &position in graph.index.values() {
            let (Some(node), Some(&kind)) = (graph.nodes.get(position), graph.kinds.get(position))
            else {
                continue;
            };
            let steps = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(position),
                Step::Key("body"),
            ];
            let cursor = Cursor::at(&Trail::Base(&steps));
            let root = cursor.root(&node.body, true);
            let mut placement = StateApplicationPlacement {
                node,
                root_admitted: kind
                    == CheckedNodeKind::State(if transition {
                        StateForm::Transition
                    } else {
                        StateForm::StateClause
                    }),
                transition,
                cursor,
            };
            if let ControlFlow::Break(failure) = walk(&mut placement, root) {
                return Err(failure);
            }
        }
    }
    Ok(())
}

/// The walk of [`validate_placement`] over one node's body: each term with
/// whether it is the body root. A state clause application nested in a body is
/// refused `malformed_wire` ahead of this step (`flat_wire`); the check here is
/// the body root's.
struct StateApplicationPlacement<'a> {
    node: &'a CheckedSemanticNodeV2,
    root_admitted: bool,
    transition: bool,
    cursor: Cursor<'a>,
}

impl<'a> Walk for StateApplicationPlacement<'a> {
    type Node = At<'a, bool>;
    type Frame = ();
    type Stop = ValidationFailure;

    fn enter(
        &mut self,
        node: At<'a, bool>,
        children: &mut Children<'_, At<'a, bool>>,
    ) -> ControlFlow<ValidationFailure> {
        self.cursor.enter(&node);
        let application = if self.transition {
            is_state_transition_application(node.value)
        } else {
            is_state_clause_application(node.value)
        };
        if application && !(node.extra && self.root_admitted) {
            return ControlFlow::Break(ineligible(self.node, self.cursor.trail().pointer()));
        }
        children.extend(subterms(node.value).map(|subterm| self.cursor.subterm(subterm, false)));
        ControlFlow::Continue(())
    }

    fn exit(&mut self, (): ()) -> ControlFlow<ValidationFailure> {
        ControlFlow::Continue(())
    }
}

fn is_operation_context(kind: CheckedNodeKind) -> bool {
    matches!(
        kind,
        CheckedNodeKind::Model(
            ModelForm::ObjectType | ModelForm::StateMachine | ModelForm::Process
        )
    )
}

/// A resolved operation and its owner, `None` for a context that is not a
/// model declaration node, or the refusal FR-342's operation check
/// determines.
pub(super) type OperationResolution<'m> =
    Result<Option<(Owner<'m>, &'m OperationDecl)>, ModelRefusal>;

/// The operation a model-owned context declares under `name`: `Ok(None)`
/// when the context is not a model declaration node, else the owner and
/// the operation, or the refusal FR-342's operation check determines.
pub(super) fn resolve_operation<'m>(
    context: &CheckedSemanticNodeV2,
    context_kind: CheckedNodeKind,
    name: &str,
    graph: &StepGraph<'_, 'm>,
    meter: &mut WorkMeter,
) -> Result<OperationResolution<'m>, ValidationFailure> {
    if !graph
        .owners
        .is_model_declaration_node(context, context_kind.tag())
    {
        return Ok(Ok(None));
    }
    Ok(
        match graph
            .owners
            .resolve_member(context, MemberKind::Operation, name, meter)?
        {
            Ok((owner, Resolved::Operation(operation))) => {
                let declared = owner.object_type().is_some_and(|object| {
                    object
                        .operations
                        .iter()
                        .any(|own| std::ptr::eq(own, operation))
                });
                if declared {
                    Ok(Some((owner, operation)))
                } else {
                    Err(ModelRefusal::malformed())
                }
            }
            Ok((_, Resolved::Field(_))) => Err(ModelRefusal::missing_name()),
            Err(refusal) if refusal == ModelRefusal::ineligible() => {
                Err(ModelRefusal::missing_name())
            }
            Err(refusal) => Err(refusal),
        },
    )
}

/// FR-342's checks over one anchor: context, frame, types, operation, scoped deletes.
fn check_anchor(
    anchor: &CheckedSemanticNodeV2,
    position: usize,
    body: &AnchorBody<'_>,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let target_at = |member: usize| {
        node_pointer(position)
            .key("body")
            .key("members")
            .index(member)
    };
    let (context, context_kind) = join(anchor, &body.context, is_operation_context, graph)
        .map_err(|refusal| {
            model_refusal(
                refusal,
                target_at(0).key("value").key("target"),
                &body.context,
            )
        })?;
    let (frame, _) = join(
        anchor,
        &body.frame,
        |kind| kind == CheckedNodeKind::State(StateForm::Frame),
        graph,
    )
    .map_err(|refusal| {
        model_refusal(
            refusal,
            target_at(2).key("value").key("target"),
            &body.frame,
        )
    })?;
    if anchor.semantic_type != body.context || frame.semantic_type != body.context {
        return Err(model_refusal(
            ModelRefusal::malformed(),
            node_pointer(position),
            &anchor.node_id,
        ));
    }
    match resolve_operation(context, context_kind, body.operation, graph, meter)? {
        Ok(None) => {}
        Ok(Some((owner, operation))) => {
            let frame_position = *graph
                .index
                .get(&body.frame)
                .ok_or_else(|| shape_refusal(frame, position))?;
            let steps = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(frame_position),
                Step::Key("body"),
            ];
            let (frame_body, work) = read_frame_body(&frame.body, &Trail::Base(&steps))?;
            meter.charge(work, || node_pointer(frame_position).key("body"))?;
            for entry in &frame_body.deletes_scoped {
                let name = match entry {
                    DeletesScopedEntry::Receiver => continue,
                    DeletesScopedEntry::Parameter { name } => name,
                };
                let mut present = false;
                for slot in &operation.parameters {
                    meter.charge(1, || node_pointer(frame_position).key("body"))?;
                    if slot.name.as_deref() == Some(name.as_ref())
                        && matches!(
                            owner.package.slot_type(slot),
                            Some(MemberType::Reference(_))
                        )
                    {
                        present = true;
                        break;
                    }
                }
                if !present {
                    return Err(model_refusal(
                        ModelRefusal::missing_name(),
                        node_pointer(frame_position),
                        &frame.node_id,
                    ));
                }
            }
        }
        Err(refusal) => return Err(model_refusal(refusal, target_at(1), &anchor.node_id)),
    }
    if let Some(point) = &body.execution_point {
        let matches_operation = match point {
            ExecutionPoint::Pre { operation } | ExecutionPoint::Post { operation } => {
                operation.as_ref() == body.operation
            }
            ExecutionPoint::Initialization { .. } | ExecutionPoint::Handler { .. } => false,
        };
        if !matches_operation {
            return Err(model_refusal(
                ModelRefusal::malformed(),
                node_pointer(position),
                &anchor.node_id,
            ));
        }
    }
    Ok(())
}

/// FR-341's checks over one clause: anchor, parameters, signature.
fn check_clause(
    clause: &CheckedSemanticNodeV2,
    position: usize,
    terms: &[Option<CheckedTerm>],
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body = terms
        .get(position)
        .and_then(Option::as_ref)
        .and_then(typed_clause_body)
        .ok_or_else(|| shape_refusal(clause, position))?;
    let arguments_at = node_pointer(position).key("body").key("arguments");
    let model_clause = matches!(
        body.clause,
        StateClauseKind::Invariant | StateClauseKind::Constraint
    );
    let (anchor, _) = join(
        clause,
        &body.anchor,
        |kind| {
            if model_clause {
                is_operation_context(kind)
            } else {
                kind == CheckedNodeKind::State(StateForm::OperationAnchor)
            }
        },
        graph,
    )
    .map_err(|refusal| {
        model_refusal(
            refusal,
            arguments_at.clone().index(1).key("target"),
            &body.anchor,
        )
    })?;
    let mut parameters = Vec::with_capacity(body.parameters.len());
    for (index, target) in body.parameters.iter().enumerate() {
        let (parameter, _) = join(
            clause,
            target,
            |kind| kind == CheckedNodeKind::Value(ValueForm::Parameter),
            graph,
        )
        .map_err(|refusal| {
            model_refusal(
                refusal,
                arguments_at
                    .clone()
                    .index(0)
                    .key("members")
                    .index(index)
                    .key("target"),
                target,
            )
        })?;
        parameters.push(parameter);
    }
    let signature_refusal = || ineligible(clause, arguments_at.clone().index(0));
    // The anchor's context: the anchor itself for an invariant, else the
    // operation anchor's `context`.
    let anchor_body = if model_clause {
        None
    } else {
        let anchor_position = *graph
            .index
            .get(&anchor.node_id)
            .ok_or_else(signature_refusal)?;
        Some(
            terms
                .get(anchor_position)
                .and_then(Option::as_ref)
                .and_then(typed_anchor_body)
                .ok_or_else(signature_refusal)?,
        )
    };
    let context_id = anchor_body
        .as_ref()
        .map_or(&body.anchor, |anchor| &anchor.context);
    let self_is_reference = parameters.first().is_some_and(|parameter| {
        graph
            .node(&parameter.semantic_type)
            .filter(|(_, kind)| {
                *kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Reference)
            })
            .and_then(|(node, _)| reference_type_target(node))
            .as_ref()
            == Some(context_id)
    });
    if !self_is_reference {
        return Err(signature_refusal());
    }
    let Some(anchor_body) = &anchor_body else {
        if parameters.len() != 1 {
            return Err(signature_refusal());
        }
        return check_clause_point(clause, position, &body, None);
    };
    let Some((context, context_kind)) = graph.node(context_id) else {
        return Err(signature_refusal());
    };
    let (owner, operation) =
        match resolve_operation(context, context_kind, anchor_body.operation, graph, meter)? {
            Ok(Some(resolved)) => resolved,
            // A context that is not a model declaration node: nothing past
            // `self` is resolved.
            Ok(None) => return check_clause_point(clause, position, &body, Some(anchor_body)),
            Err(_) => return Err(signature_refusal()),
        };
    let result = operation
        .result
        .as_ref()
        .filter(|_| body.clause == StateClauseKind::Postcondition);
    let expected = result.into_iter().chain(&operation.parameters);
    let bound = parameters.get(1..).unwrap_or_default();
    if bound.len() != expected.clone().count() {
        return Err(signature_refusal());
    }
    for (parameter, slot) in bound.iter().zip(expected) {
        let Some(slot_type) = owner.package.slot_type(slot) else {
            return Err(signature_refusal());
        };
        let expected = slot_type.node_id(&clause.node_id.package).ok_or_else(|| {
            ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                arguments_at.clone().index(0),
                None,
                clause.node_id.clone(),
            )
        })?;
        if expected != parameter.semantic_type {
            return Err(signature_refusal());
        }
    }
    check_clause_point(clause, position, &body, Some(anchor_body))
}

/// Applicability and clause-anchor agreement follow signature validation.
fn check_clause_point(
    clause: &CheckedSemanticNodeV2,
    position: usize,
    body: &super::structural::ClauseBody,
    anchor: Option<&AnchorBody<'_>>,
) -> Result<(), ValidationFailure> {
    let valid = match (&body.execution_point, anchor) {
        (None, None) => true,
        (Some(ExecutionPoint::Initialization { .. } | ExecutionPoint::Handler { .. }), None) => {
            true
        }
        (Some(_), None) => false,
        (point, Some(anchor)) => {
            let applicable = match point {
                None => true,
                Some(ExecutionPoint::Pre { operation } | ExecutionPoint::Post { operation }) => {
                    operation.as_ref() == anchor.operation
                }
                Some(ExecutionPoint::Initialization { .. } | ExecutionPoint::Handler { .. }) => {
                    false
                }
            };
            applicable && point == &anchor.execution_point
        }
    };
    if valid {
        Ok(())
    } else {
        Err(model_refusal(
            ModelRefusal::malformed(),
            node_pointer(position),
            &clause.node_id,
        ))
    }
}

/// The target locus of one transition input.
fn input_at(position: usize, index: usize) -> JsonPointer {
    node_pointer(position)
        .key("body")
        .key("arguments")
        .index(2)
        .key("value")
        .key("members")
        .index(index)
        .key("target")
}

/// Recognize the receiver parameter's exact wire spelling at the state intake boundary.
#[ix_trace_rs::string_edge]
fn is_self_parameter(name: &str, level: &str) -> bool {
    name == "self" && level == "0"
}

/// FR-341's ordered transition joins, after every anchor and clause has passed.
fn check_transition(
    transition: &CheckedSemanticNodeV2,
    position: usize,
    body: &TransitionBody,
    terms: &[Option<CheckedTerm>],
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let target_at = |index: usize| {
        node_pointer(position)
            .key("body")
            .key("arguments")
            .index(index)
            .key("value")
            .key("target")
    };
    let (anchor, _) = join(
        transition,
        &body.anchor,
        |kind| kind == CheckedNodeKind::State(StateForm::OperationAnchor),
        graph,
    )
    .map_err(|refusal| model_refusal(refusal, target_at(0), &body.anchor))?;
    let anchor_position = *graph
        .index
        .get(&body.anchor)
        .ok_or_else(|| shape_refusal(anchor, position))?;
    let anchor_body = terms
        .get(anchor_position)
        .and_then(Option::as_ref)
        .and_then(typed_anchor_body)
        .ok_or_else(|| shape_refusal(anchor, anchor_position))?;
    if anchor_body.execution_point.is_some() {
        return Err(model_refusal(
            ModelRefusal::malformed(),
            target_at(0),
            &body.anchor,
        ));
    }

    let (subject, _) = join(
        transition,
        &body.subject,
        |kind| kind == CheckedNodeKind::Expression(ExpressionForm::Query),
        graph,
    )
    .map_err(|refusal| model_refusal(refusal, target_at(1), &body.subject))?;
    let subject_position = *graph
        .index
        .get(&body.subject)
        .ok_or_else(|| shape_refusal(subject, position))?;
    let malformed_subject =
        || model_refusal(ModelRefusal::malformed(), target_at(1), &body.subject);
    let Some(CheckedTerm::Application {
        operator: ApplicationOperator::Query,
        operation: project,
        result_type: subject_result,
        arguments,
    }) = terms.get(subject_position).and_then(Option::as_ref)
    else {
        return Err(malformed_subject());
    };
    if project.get("identity").and_then(serde_json::Value::as_str)
        != Some("quire.op.record.project")
        || project
            .pointer("/member/kind")
            .and_then(serde_json::Value::as_str)
            != Some("field")
    {
        return Err(malformed_subject());
    }
    let Some(field_name) = project
        .pointer("/member/name")
        .and_then(serde_json::Value::as_str)
    else {
        return Err(malformed_subject());
    };
    let Some(declaring) = project
        .pointer("/member/declaration")
        .and_then(CheckedNodeId::from_value)
    else {
        return Err(malformed_subject());
    };
    let [receiver] = arguments.as_slice() else {
        return Err(malformed_subject());
    };
    let Some(deref_id) = typed_reference(receiver) else {
        return Err(malformed_subject());
    };
    let Some((deref, CheckedNodeKind::Expression(ExpressionForm::Deref))) = graph.node(deref_id)
    else {
        return Err(malformed_subject());
    };
    let Some(deref_position) = graph.index.get(deref_id).copied() else {
        return Err(malformed_subject());
    };
    let Some(CheckedTerm::Application {
        operator: ApplicationOperator::Deref,
        operation: deref_operation,
        arguments: deref_arguments,
        ..
    }) = terms.get(deref_position).and_then(Option::as_ref)
    else {
        return Err(malformed_subject());
    };
    if deref_operation
        .get("identity")
        .and_then(serde_json::Value::as_str)
        != Some("quire.op.model.deref")
        || deref.semantic_type != anchor_body.context
    {
        return Err(malformed_subject());
    }
    let [receiver] = deref_arguments.as_slice() else {
        return Err(malformed_subject());
    };
    let Some(self_id) = typed_reference(receiver) else {
        return Err(malformed_subject());
    };
    let Some((self_node, CheckedNodeKind::Value(ValueForm::Parameter))) = graph.node(self_id)
    else {
        return Err(malformed_subject());
    };
    let self_position = *graph.index.get(self_id).ok_or_else(malformed_subject)?;
    let is_self = terms
        .get(self_position)
        .and_then(Option::as_ref)
        .and_then(|term| typed_parameter_binding(term, &|id| graph.node(id).map(|(_, kind)| kind)))
        .is_some_and(|(name, level)| is_self_parameter(name, level));
    if !is_self
        || graph
            .node(&self_node.semantic_type)
            .filter(|(_, kind)| {
                *kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Reference)
            })
            .and_then(|(node, _)| reference_type_target(node))
            .as_ref()
            != Some(&anchor_body.context)
    {
        return Err(malformed_subject());
    }
    let Some((context, context_kind)) = graph.node(&anchor_body.context) else {
        return Err(malformed_subject());
    };
    let (owner, operation) =
        match resolve_operation(context, context_kind, anchor_body.operation, graph, meter)? {
            Ok(Some(value)) => value,
            _ => return Err(malformed_subject()),
        };
    let field_type =
        match graph
            .owners
            .resolve_member(context, MemberKind::Field, field_name, meter)?
        {
            Ok((field_owner, Resolved::Field(field))) => {
                let declared = graph
                    .node(&declaring)
                    .and_then(|(node, _)| graph.owners.recover(node).ok())
                    .is_some_and(|declaring_owner| {
                        declaring_owner.package.identity == field_owner.package.identity
                            && field.identity.as_ref()
                                == format!("{}/{}", declaring_owner.node, field_name)
                    });
                if !declared {
                    return Err(malformed_subject());
                }
                field_owner.package.field_type(field)
            }
            _ => return Err(malformed_subject()),
        };
    if field_type != Some(MemberType::Boolean) {
        return Err(ineligible(transition, node_pointer(position)));
    }
    let frame_position = *graph
        .index
        .get(&anchor_body.frame)
        .ok_or_else(malformed_subject)?;
    let frame = graph
        .nodes
        .get(frame_position)
        .ok_or_else(malformed_subject)?;
    let steps = [
        Step::Key("semantic_graph"),
        Step::Key("nodes"),
        Step::Index(frame_position),
        Step::Key("body"),
    ];
    let (frame_body, work) = read_frame_body(&frame.body, &Trail::Base(&steps))?;
    meter.charge(work, || node_pointer(frame_position).key("body"))?;
    if !frame_body.modifies.iter().any(|entry| matches!(entry, ModifiesEntry::Field { declaration, name, .. } if declaration == &declaring && name.as_ref() == field_name)) {
        return Err(malformed_subject());
    }
    if subject_result != &subject.semantic_type
        || graph.node(&subject.semantic_type).map(|(_, kind)| kind)
            != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
    {
        return Err(ineligible(transition, node_pointer(position)));
    }

    let mut actual_inputs = Vec::with_capacity(body.inputs.len());
    for (slot, target) in body.inputs.iter().enumerate() {
        let (parameter, _) = join(
            transition,
            target,
            |kind| kind == CheckedNodeKind::Value(ValueForm::Parameter),
            graph,
        )
        .map_err(|refusal| model_refusal(refusal, input_at(position, slot), target))?;
        actual_inputs.push(parameter);
    }
    for (index, parameter) in actual_inputs.iter().enumerate() {
        let parameter_position = *graph.index.get(&parameter.node_id).ok_or_else(|| {
            model_refusal(
                ModelRefusal::malformed(),
                input_at(position, index),
                &parameter.node_id,
            )
        })?;
        let name = terms
            .get(parameter_position)
            .and_then(Option::as_ref)
            .and_then(|term| {
                typed_parameter_binding(term, &|id| graph.node(id).map(|(_, kind)| kind))
            })
            .map(|(name, _)| name);
        if !name.is_some_and(|name| {
            operation
                .parameters
                .iter()
                .any(|slot| slot.name.as_deref() == Some(name))
        }) {
            return Err(model_refusal(
                ModelRefusal::malformed(),
                input_at(position, index),
                &parameter.node_id,
            ));
        }
    }
    if actual_inputs.len() != operation.parameters.len() {
        return Err(ineligible(transition, node_pointer(position)));
    }
    for (parameter, slot) in actual_inputs.iter().zip(&operation.parameters) {
        let Some(parameter_position) = graph.index.get(&parameter.node_id).copied() else {
            return Err(ineligible(transition, node_pointer(position)));
        };
        let name = terms
            .get(parameter_position)
            .and_then(Option::as_ref)
            .and_then(|term| {
                typed_parameter_binding(term, &|id| graph.node(id).map(|(_, kind)| kind))
            })
            .map(|(name, _)| name);
        if name != slot.name.as_deref()
            || owner.package.slot_type(slot) != Some(MemberType::Boolean)
            || graph.node(&parameter.semantic_type).map(|(_, kind)| kind)
                != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
        {
            return Err(ineligible(transition, node_pointer(position)));
        }
    }
    for (slot, target, expected) in [
        (3, body.pre.as_ref(), StateClauseKind::Precondition),
        (4, Some(&body.post), StateClauseKind::Postcondition),
    ] {
        let Some(target) = target else {
            continue;
        };
        let target_path = || {
            if slot == 3 {
                node_pointer(position)
                    .key("body")
                    .key("arguments")
                    .index(3)
                    .key("value")
                    .key("members")
                    .index(0)
                    .key("target")
            } else {
                target_at(4)
            }
        };
        let (clause, _) = join(
            transition,
            target,
            |kind| kind == CheckedNodeKind::State(StateForm::StateClause),
            graph,
        )
        .map_err(|refusal| model_refusal(refusal, target_path(), target))?;
        let clause_position = *graph
            .index
            .get(target)
            .ok_or_else(|| shape_refusal(clause, position))?;
        let clause_body = terms
            .get(clause_position)
            .and_then(Option::as_ref)
            .and_then(typed_clause_body)
            .ok_or_else(|| model_refusal(ModelRefusal::malformed(), target_path(), target))?;
        let matched_anchor = graph
            .node(&clause_body.anchor)
            .and_then(|_| graph.index.get(&clause_body.anchor).copied())
            .and_then(|clause_anchor_position| {
                terms
                    .get(clause_anchor_position)
                    .and_then(Option::as_ref)
                    .and_then(typed_anchor_body)
            });
        if clause_body.clause != expected
            || !matched_anchor.is_some_and(|other| {
                other.context == anchor_body.context && other.operation == anchor_body.operation
            })
        {
            return Err(model_refusal(
                ModelRefusal::malformed(),
                target_path(),
                target,
            ));
        }
    }
    Ok(())
}
