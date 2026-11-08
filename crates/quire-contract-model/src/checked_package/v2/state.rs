//! The state step of the reader order (QSpec FR-341 and FR-342): after the
//! frame step and before the operation step, the reader checks, each in
//! ascending structural node-id order,
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

use super::frame::{join, FrameView, StepGraph};
use super::graph_storage::{first_repeat, GraphIndex};
use super::model_members::EmptyAggregateBody;
use super::model_members::{
    MemberKind, MemberType, ModelOwners, ModelRefusal, OperationDecl, Owner, Resolved,
};
use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::structural::{
    reference_type_target_ref, typed_anchor_body, typed_clause_body, typed_parameter_binding,
    typed_reference, typed_transition_body, AnchorBody, TransitionBody,
};
use super::term_view::{TermNode, TermView};
use super::{
    ApplicationOperator, CheckedNodeKind, CheckedSemanticNodeV2, CheckedTerm, CompositeTypeForm,
    ExecutionPoint, ExpressionForm, ModelForm, ScalarTypeForm, StateClauseKind, StateForm,
    ValueForm, WorkMeter,
};
use super::{
    CheckedFrameModification as ModifiesEntry, CheckedScopedDeletion as DeletesScopedEntry,
};
use crate::checked_package::common::{Step, Trail, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
};

fn node_path(position: usize) -> Path<'static> {
    Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position)
}

/// The state step. See the module documentation for the order.
pub(super) fn validate_state<Body: EmptyAggregateBody>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StepGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    validate_placement(&graph, meter)?;
    let anchor_count = graph
        .each(CheckedNodeKind::State(StateForm::OperationAnchor))
        .count();
    meter.at_node(0, "state");
    let mut anchors = meter.vec(anchor_count)?;
    for (position, node) in graph.each(CheckedNodeKind::State(StateForm::OperationAnchor)) {
        meter.at_node(position, "state");
        let body = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(typed_anchor_body)
            .ok_or_else(|| shape_refusal(node, position, meter))?;
        check_anchor(node, position, &body, terms, &graph, meter)?;
        anchors.push((position, node, body));
    }
    meter.at_node(0, "state");
    if let Some(repeat) = first_repeat(
        anchors
            .iter()
            .map(|(_, _, body)| (body.context, body.operation, body.execution_point)),
        meter,
    )? {
        let (position, node, _) = &anchors[repeat];
        return Err(fail(
            meter,
            *position,
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            node_path(*position),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            &node.node_id,
        ));
    }
    for (position, node) in graph.each(CheckedNodeKind::State(StateForm::StateClause)) {
        meter.at_node(position, "state");
        check_clause(node, position, terms, &graph, structural_keys, meter)?;
    }
    // A transition whose anchor and subject repeat an earlier one's refuses
    // after its own checks and before the next transition's: the repeat is
    // found over the keys of the transitions that read, then met in order.
    let transition_count = graph
        .each(CheckedNodeKind::State(StateForm::Transition))
        .count();
    meter.at_node(0, "state");
    let mut keys = meter.vec(transition_count)?;
    for (position, _) in graph.each(CheckedNodeKind::State(StateForm::Transition)) {
        let Some(body) = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(typed_transition_body)
        else {
            break;
        };
        keys.push((body.anchor, body.subject, position));
    }
    let repeated = first_repeat(
        keys.iter().map(|&(anchor, subject, _)| (anchor, subject)),
        meter,
    )?
    .map(|repeat| keys[repeat].2);
    for (position, node) in graph.each(CheckedNodeKind::State(StateForm::Transition)) {
        meter.at_node(position, "state");
        let body = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(typed_transition_body)
            .ok_or_else(|| shape_refusal(node, position, meter))?;
        check_transition(node, position, &body, terms, &graph, meter)?;
        if repeated == Some(position) {
            return Err(fail(
                meter,
                position,
                CheckedPackageRefusalCode::AmbiguousDeclaration,
                node_path(position),
                Some(CheckedPackageRefusalCause::AmbiguousName),
                &node.node_id,
            ));
        }
    }
    Ok(())
}

/// The graph-shape refusal of a body that is not its form's closed shape.
fn fail(
    meter: &mut WorkMeter,
    position: usize,
    code: CheckedPackageRefusalCode,
    path: Path<'_>,
    cause: Option<CheckedPackageRefusalCause>,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    let mut refusal = Refusal::new(code, path).at_node(locus);
    refusal.cause = cause;
    stage_storage::refuse(
        meter,
        &Site::new(PackageReadStage::Graph, position, "state", path),
        refusal,
    )
}
fn shape_refusal<Body>(
    node: &CheckedSemanticNodeV2<Body>,
    position: usize,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    fail(
        meter,
        position,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        node_path(position).key("body"),
        None,
        &node.node_id,
    )
}
fn ineligible<Body>(
    node: &CheckedSemanticNodeV2<Body>,
    path: Path<'_>,
    meter: &mut WorkMeter,
    position: usize,
) -> ValidationFailure {
    fail(
        meter,
        position,
        CheckedPackageRefusalCode::IllTyped,
        path,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &node.node_id,
    )
}
fn model_refusal(
    refusal: ModelRefusal,
    path: Path<'_>,
    locus: &CheckedNodeId,
    meter: &mut WorkMeter,
    position: usize,
) -> ValidationFailure {
    fail(
        meter,
        position,
        refusal.code,
        path,
        Some(refusal.cause),
        locus,
    )
}

/// Both complete placement passes precede every anchor check.
fn validate_placement<Body: EmptyAggregateBody>(
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for transition in [false, true] {
        for &position in graph.index.values() {
            let (Some(node), Some(&kind)) = (graph.nodes.get(position), graph.kinds.get(position))
            else {
                continue;
            };
            meter.at_node(position, super::graph_storage::node_form(&node.node_tag));
            let Some((root, _row)) = meter.body(position) else {
                return Err(shape_refusal(node, position, meter));
            };
            let root_admitted = kind
                == CheckedNodeKind::State(if transition {
                    StateForm::Transition
                } else {
                    StateForm::StateClause
                });
            let application = |term: TermNode<'_>| matches!(term.view(), Some(TermView::Application { operation, .. }) if operation.get("identity").and_then(TermNode::as_str).and_then(super::KnownOperation::from_wire) == Some(if transition { super::KnownOperation::StateTransition } else { super::KnownOperation::StateClause }));
            if application(root) && !root_admitted {
                return Err(ineligible(node, node_path(position), meter, position));
            }
            let mut path = meter.vec(4)?;
            path.extend([
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(position),
                Step::Key("body"),
            ]);
            let mut pending = Vec::new();
            meter.push(&mut pending, (root.subterms(), path.len()))?;
            while let Some((children, depth)) = pending.last_mut() {
                path.truncate(*depth);
                let Some(child) = children.next() else {
                    pending.pop();
                    continue;
                };
                meter.push(&mut path, Step::Key(child.key))?;
                if let Some(index) = child.index {
                    meter.push(&mut path, Step::Index(index))?;
                }
                if application(child.value) {
                    let trail = Trail::Base(&path);
                    let refusal = Refusal::new(CheckedPackageRefusalCode::IllTyped, trail)
                        .because(CheckedPackageRefusalCause::OperatorIneligible)
                        .at_node(&node.node_id);
                    return Err(stage_storage::refuse(
                        meter,
                        &Site::new(
                            PackageReadStage::Graph,
                            position,
                            "state",
                            node_path(position),
                        ),
                        refusal,
                    ));
                }
                meter.push(&mut pending, (child.value.subterms(), path.len()))?;
            }
        }
    }
    Ok(())
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
pub(super) fn resolve_operation<'m, Body: EmptyAggregateBody>(
    context: &CheckedSemanticNodeV2<Body>,
    context_kind: CheckedNodeKind,
    name: &str,
    graph: &StepGraph<'_, 'm, Body>,
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
fn check_anchor<Body: EmptyAggregateBody>(
    anchor: &CheckedSemanticNodeV2<Body>,
    position: usize,
    body: &AnchorBody<'_>,
    terms: &[Option<CheckedTerm>],
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let target_at = |member: usize| node_path(position).key("body").key("members").index(member);
    let (context, context_kind) =
        join(anchor, body.context, is_operation_context, graph).map_err(|refusal| {
            model_refusal(
                refusal,
                target_at(0).key("value").key("target"),
                body.context,
                meter,
                position,
            )
        })?;
    let (frame, _) = join(
        anchor,
        body.frame,
        |kind| kind == CheckedNodeKind::State(StateForm::Frame),
        graph,
    )
    .map_err(|refusal| {
        model_refusal(
            refusal,
            target_at(2).key("value").key("target"),
            body.frame,
            meter,
            position,
        )
    })?;
    if anchor.semantic_type != *body.context || frame.semantic_type != *body.context {
        return Err(model_refusal(
            ModelRefusal::malformed(),
            node_path(position),
            &anchor.node_id,
            meter,
            position,
        ));
    }
    match resolve_operation(context, context_kind, body.operation, graph, meter)? {
        Ok(None) => {}
        Ok(Some((owner, operation))) => {
            let frame_position = *graph
                .index
                .get(body.frame)
                .ok_or_else(|| shape_refusal(frame, position, meter))?;
            let frame_body = terms
                .get(frame_position)
                .and_then(Option::as_ref)
                .and_then(FrameView::of)
                .ok_or_else(|| shape_refusal(frame, frame_position, meter))?;
            let work = frame_body.work();
            stage_storage::work(
                meter,
                &Site::new(
                    PackageReadStage::Graph,
                    position,
                    "state",
                    node_path(frame_position).key("body"),
                ),
                work,
            )?;
            for entry in frame_body.deletes_scoped {
                let name = match entry {
                    DeletesScopedEntry::Receiver => continue,
                    DeletesScopedEntry::Parameter { name } => name,
                };
                let mut present = false;
                for slot in &operation.parameters {
                    stage_storage::work(
                        meter,
                        &Site::new(
                            PackageReadStage::Graph,
                            position,
                            "state",
                            node_path(frame_position).key("body"),
                        ),
                        1,
                    )?;
                    if slot.name.as_deref() == Some(name.as_ref())
                        && matches!(
                            owner.package.slot_type_charged(
                                slot,
                                false,
                                &mut super::model_members::Budget::new(
                                    meter,
                                    owner.package.root,
                                    owner.package.bytes
                                ),
                                "member_type"
                            )?,
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
                        node_path(frame_position),
                        &frame.node_id,
                        meter,
                        position,
                    ));
                }
            }
        }
        Err(refusal) => {
            return Err(model_refusal(
                refusal,
                target_at(1),
                &anchor.node_id,
                meter,
                position,
            ))
        }
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
                node_path(position),
                &anchor.node_id,
                meter,
                position,
            ));
        }
    }
    Ok(())
}

/// FR-341's checks over one clause: anchor, parameters, signature.
fn check_clause<Body: EmptyAggregateBody>(
    clause: &CheckedSemanticNodeV2<Body>,
    position: usize,
    terms: &[Option<CheckedTerm>],
    graph: &StepGraph<'_, '_, Body>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body = terms
        .get(position)
        .and_then(Option::as_ref)
        .and_then(typed_clause_body)
        .ok_or_else(|| shape_refusal(clause, position, meter))?;
    let arguments_at = node_path(position).key("body").key("arguments");
    let model_clause = matches!(
        body.clause,
        StateClauseKind::Invariant | StateClauseKind::Constraint
    );
    let (anchor, _) = join(
        clause,
        body.anchor,
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
            arguments_at.index(1).key("target"),
            body.anchor,
            meter,
            position,
        )
    })?;
    let mut parameters = meter.vec(body.parameter_count())?;
    for (index, target) in body.parameters().enumerate() {
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
                    .index(0)
                    .key("members")
                    .index(index)
                    .key("target"),
                target,
                meter,
                position,
            )
        })?;
        parameters.push(parameter);
    }
    let signature_refusal =
        |meter: &mut WorkMeter| ineligible(clause, arguments_at.index(0), meter, position);
    // The anchor's context: the anchor itself for an invariant, else the
    // operation anchor's `context`.
    let anchor_body = if model_clause {
        None
    } else {
        let anchor_position = *graph
            .index
            .get(&anchor.node_id)
            .ok_or_else(|| signature_refusal(meter))?;
        Some(
            terms
                .get(anchor_position)
                .and_then(Option::as_ref)
                .and_then(typed_anchor_body)
                .ok_or_else(|| signature_refusal(meter))?,
        )
    };
    let context_id = anchor_body
        .as_ref()
        .map_or(body.anchor, |anchor| anchor.context);
    let self_is_reference = parameters.first().is_some_and(|parameter| {
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
            .and_then(reference_type_target_ref)
            == Some(context_id)
    });
    if !self_is_reference {
        return Err(signature_refusal(meter));
    }
    let Some(anchor_body) = &anchor_body else {
        if parameters.len() != 1 {
            return Err(signature_refusal(meter));
        }
        return check_clause_point(clause, position, &body, None, meter);
    };
    let Some((context, context_kind)) = graph.node(context_id) else {
        return Err(signature_refusal(meter));
    };
    let (owner, operation) =
        match resolve_operation(context, context_kind, anchor_body.operation, graph, meter)? {
            Ok(Some(resolved)) => resolved,
            // A context that is not a model declaration node: nothing past
            // `self` is resolved.
            Ok(None) => {
                return check_clause_point(clause, position, &body, Some(anchor_body), meter)
            }
            Err(_) => return Err(signature_refusal(meter)),
        };
    let result = operation
        .result
        .as_ref()
        .filter(|_| body.clause == StateClauseKind::Postcondition);
    let expected = result.into_iter().chain(&operation.parameters);
    let bound = parameters.get(1..).unwrap_or_default();
    if bound.len() != expected.clone().count() {
        return Err(signature_refusal(meter));
    }
    for (parameter, slot) in bound.iter().zip(expected) {
        let Some(slot_type) = owner.package.slot_type_charged(
            slot,
            false,
            &mut super::model_members::Budget::new(meter, owner.package.root, owner.package.bytes),
            "member_type",
        )?
        else {
            return Err(signature_refusal(meter));
        };
        let expected = slot_type
            .node_id_in(
                &clause.node_id.package,
                &mut super::model_members::Budget::new(
                    meter,
                    owner.package.root,
                    owner.package.bytes,
                ),
            )?
            .ok_or_else(|| {
                fail(
                    meter,
                    position,
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    arguments_at.index(0),
                    None,
                    &clause.node_id,
                )
            })?;
        let actual = graph
            .index
            .get(&parameter.semantic_type)
            .and_then(|&position| structural_keys.get(position))
            .map(std::sync::Arc::as_ref);
        let same = match actual {
            Some(actual) => meter.compare_ids(actual, &expected)? == std::cmp::Ordering::Equal,
            None => false,
        };
        if !same {
            return Err(signature_refusal(meter));
        }
    }
    check_clause_point(clause, position, &body, Some(anchor_body), meter)
}

/// Applicability and clause-anchor agreement follow signature validation.
fn check_clause_point<Body: EmptyAggregateBody>(
    clause: &CheckedSemanticNodeV2<Body>,
    position: usize,
    body: &super::structural::ClauseBody,
    anchor: Option<&AnchorBody<'_>>,
    meter: &mut WorkMeter,
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
            node_path(position),
            &clause.node_id,
            meter,
            position,
        ))
    }
}

/// The target locus of one transition input.
fn input_at(position: usize, index: usize) -> Path<'static> {
    node_path(position)
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
fn check_transition<Body: EmptyAggregateBody>(
    transition: &CheckedSemanticNodeV2<Body>,
    position: usize,
    body: &TransitionBody,
    terms: &[Option<CheckedTerm>],
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let target_at = |index: usize| {
        node_path(position)
            .key("body")
            .key("arguments")
            .index(index)
            .key("value")
            .key("target")
    };
    let (anchor, _) = join(
        transition,
        body.anchor,
        |kind| kind == CheckedNodeKind::State(StateForm::OperationAnchor),
        graph,
    )
    .map_err(|refusal| model_refusal(refusal, target_at(0), body.anchor, meter, position))?;
    let anchor_position = *graph
        .index
        .get(body.anchor)
        .ok_or_else(|| shape_refusal(anchor, position, meter))?;
    let anchor_body = terms
        .get(anchor_position)
        .and_then(Option::as_ref)
        .and_then(typed_anchor_body)
        .ok_or_else(|| shape_refusal(anchor, anchor_position, meter))?;
    if anchor_body.execution_point.is_some() {
        return Err(model_refusal(
            ModelRefusal::malformed(),
            target_at(0),
            body.anchor,
            meter,
            position,
        ));
    }

    let (subject, _) = join(
        transition,
        body.subject,
        |kind| kind == CheckedNodeKind::Expression(ExpressionForm::Query),
        graph,
    )
    .map_err(|refusal| model_refusal(refusal, target_at(1), body.subject, meter, position))?;
    let subject_position = *graph
        .index
        .get(body.subject)
        .ok_or_else(|| shape_refusal(subject, position, meter))?;
    let malformed_subject = |meter: &mut WorkMeter| {
        model_refusal(
            ModelRefusal::malformed(),
            target_at(1),
            body.subject,
            meter,
            position,
        )
    };
    let Some(CheckedTerm::Application {
        operator: ApplicationOperator::Query,
        operation: project,
        result_type: subject_result,
        arguments,
    }) = terms.get(subject_position).and_then(Option::as_ref)
    else {
        return Err(malformed_subject(meter));
    };
    if project.identity() != Some("quire.op.record.project")
        || project.member_text("kind") != Some("field")
    {
        return Err(malformed_subject(meter));
    }
    let Some(field_name) = project.member_text("name") else {
        return Err(malformed_subject(meter));
    };
    let Some(declaring) = project.member_key("declaration") else {
        return Err(malformed_subject(meter));
    };
    let [receiver] = arguments.as_slice() else {
        return Err(malformed_subject(meter));
    };
    let Some(deref_id) = typed_reference(receiver) else {
        return Err(malformed_subject(meter));
    };
    let Some((deref, CheckedNodeKind::Expression(ExpressionForm::Deref))) = graph.node(deref_id)
    else {
        return Err(malformed_subject(meter));
    };
    let Some(deref_position) = graph.index.get(deref_id).copied() else {
        return Err(malformed_subject(meter));
    };
    let Some(CheckedTerm::Application {
        operator: ApplicationOperator::Deref,
        operation: deref_operation,
        arguments: deref_arguments,
        ..
    }) = terms.get(deref_position).and_then(Option::as_ref)
    else {
        return Err(malformed_subject(meter));
    };
    if deref_operation.identity() != Some("quire.op.model.deref")
        || deref.semantic_type != *anchor_body.context
    {
        return Err(malformed_subject(meter));
    }
    let [receiver] = deref_arguments.as_slice() else {
        return Err(malformed_subject(meter));
    };
    let Some(self_id) = typed_reference(receiver) else {
        return Err(malformed_subject(meter));
    };
    let Some((self_node, CheckedNodeKind::Value(ValueForm::Parameter))) = graph.node(self_id)
    else {
        return Err(malformed_subject(meter));
    };
    let self_position = *graph
        .index
        .get(self_id)
        .ok_or_else(|| malformed_subject(meter))?;
    let is_self = terms
        .get(self_position)
        .and_then(Option::as_ref)
        .and_then(|term| typed_parameter_binding(term, &|id| graph.node(id).map(|(_, kind)| kind)))
        .is_some_and(|(name, level)| is_self_parameter(name, level));
    if !is_self
        || graph
            .index
            .get(&self_node.semantic_type)
            .filter(|&&at| {
                graph.kinds.get(at)
                    == Some(&CheckedNodeKind::CompositeType(
                        CompositeTypeForm::Reference,
                    ))
            })
            .and_then(|&at| terms.get(at))
            .and_then(Option::as_ref)
            .and_then(reference_type_target_ref)
            != Some(anchor_body.context)
    {
        return Err(malformed_subject(meter));
    }
    let Some((context, context_kind)) = graph.node(anchor_body.context) else {
        return Err(malformed_subject(meter));
    };
    let (owner, operation) =
        match resolve_operation(context, context_kind, anchor_body.operation, graph, meter)? {
            Ok(Some(value)) => value,
            _ => return Err(malformed_subject(meter)),
        };
    let field_type =
        match graph
            .owners
            .resolve_member(context, MemberKind::Field, field_name, meter)?
        {
            Ok((field_owner, Resolved::Field(field))) => {
                let declared = graph
                    .node(declaring)
                    .and_then(|(node, _)| graph.owners.recover(node).ok())
                    .is_some_and(|declaring_owner| {
                        declaring_owner.package.identity == field_owner.package.identity
                            && field
                                .identity
                                .strip_prefix(declaring_owner.node)
                                .and_then(|rest| rest.strip_prefix('/'))
                                == Some(field_name)
                    });
                if !declared {
                    return Err(malformed_subject(meter));
                }
                field_owner.package.field_type_charged(
                    field,
                    &mut super::model_members::Budget::new(
                        meter,
                        field_owner.package.root,
                        field_owner.package.bytes,
                    ),
                    "member_type",
                )?
            }
            _ => return Err(malformed_subject(meter)),
        };
    if field_type != Some(MemberType::Boolean) {
        return Err(ineligible(transition, node_path(position), meter, position));
    }
    let frame_position = *graph
        .index
        .get(anchor_body.frame)
        .ok_or_else(|| malformed_subject(meter))?;
    let frame = graph
        .nodes
        .get(frame_position)
        .ok_or_else(|| malformed_subject(meter))?;
    let frame_body = terms
        .get(frame_position)
        .and_then(Option::as_ref)
        .and_then(FrameView::of)
        .ok_or_else(|| shape_refusal(frame, frame_position, meter))?;
    let work = frame_body.work();
    stage_storage::work(
        meter,
        &Site::new(
            PackageReadStage::Graph,
            position,
            "state",
            node_path(frame_position).key("body"),
        ),
        work,
    )?;
    if !frame_body.modifies.iter().any(|entry| matches!(entry, ModifiesEntry::Field { declaration, name, .. } if declaration == declaring && name.as_ref() == field_name)) {
        return Err(malformed_subject(meter));
    }
    if subject_result != &subject.semantic_type
        || graph.node(&subject.semantic_type).map(|(_, kind)| kind)
            != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
    {
        return Err(ineligible(transition, node_path(position), meter, position));
    }

    let mut actual_inputs = meter.vec(body.input_count())?;
    for (slot, target) in body.inputs().enumerate() {
        let (parameter, _) = join(
            transition,
            target,
            |kind| kind == CheckedNodeKind::Value(ValueForm::Parameter),
            graph,
        )
        .map_err(|refusal| {
            model_refusal(refusal, input_at(position, slot), target, meter, position)
        })?;
        actual_inputs.push(parameter);
    }
    for (index, parameter) in actual_inputs.iter().enumerate() {
        let parameter_position = *graph.index.get(&parameter.node_id).ok_or_else(|| {
            model_refusal(
                ModelRefusal::malformed(),
                input_at(position, index),
                &parameter.node_id,
                meter,
                position,
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
                meter,
                position,
            ));
        }
    }
    if actual_inputs.len() != operation.parameters.len() {
        return Err(ineligible(transition, node_path(position), meter, position));
    }
    for (parameter, slot) in actual_inputs.iter().zip(&operation.parameters) {
        let Some(parameter_position) = graph.index.get(&parameter.node_id).copied() else {
            return Err(ineligible(transition, node_path(position), meter, position));
        };
        let name = terms
            .get(parameter_position)
            .and_then(Option::as_ref)
            .and_then(|term| {
                typed_parameter_binding(term, &|id| graph.node(id).map(|(_, kind)| kind))
            })
            .map(|(name, _)| name);
        if name != slot.name.as_deref()
            || owner.package.slot_type_charged(
                slot,
                false,
                &mut super::model_members::Budget::new(
                    meter,
                    owner.package.root,
                    owner.package.bytes,
                ),
                "member_type",
            )? != Some(MemberType::Boolean)
            || graph.node(&parameter.semantic_type).map(|(_, kind)| kind)
                != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
        {
            return Err(ineligible(transition, node_path(position), meter, position));
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
                node_path(position)
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
        .map_err(|refusal| model_refusal(refusal, target_path(), target, meter, position))?;
        let clause_position = *graph
            .index
            .get(target)
            .ok_or_else(|| shape_refusal(clause, position, meter))?;
        let clause_body = terms
            .get(clause_position)
            .and_then(Option::as_ref)
            .and_then(typed_clause_body)
            .ok_or_else(|| {
                model_refusal(
                    ModelRefusal::malformed(),
                    target_path(),
                    target,
                    meter,
                    position,
                )
            })?;
        let matched_anchor = graph
            .node(clause_body.anchor)
            .and_then(|_| graph.index.get(clause_body.anchor).copied())
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
                meter,
                position,
            ));
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
    use std::sync::Arc;

    fn application(transition: bool) -> serde_json::Value {
        json!({"term":"application", "operator":if transition {"state_transition"} else {"state_clause"}, "operation":{"identity":if transition {"quire.op.state.transition"} else {"quire.op.state.clause"}}, "result_type":node_id('c'), "arguments":[]})
    }
    fn metadata(id: char, body: serde_json::Value) -> CheckedSemanticNodeV2<CheckedJson> {
        serde_json::from_value(json!({"node_id":node_id(id),"schema_version":"quire.checked-semantic-graph/v2","node_tag":"expression","semantic_form":"query","semantic_type":node_id('c'),"dependencies":[],"occurrences":[],"body":body})).unwrap()
    }

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn state_document_placement_keeps_two_pass_order_and_pays_deep_first_fault() {
        let mut deep = application(false);
        for _ in 0..40 {
            deep = json!({"term":"aggregate", "members":[deep]});
        }
        // The lower-ID transition fault must wait for the complete clause pass.
        let nodes = [metadata('a', application(true)), metadata('b', deep)];
        let kinds = [CheckedNodeKind::Expression(ExpressionForm::Query); 2];
        let terms = [None, None];
        let index = nodes
            .iter()
            .enumerate()
            .map(|(p, n)| (&n.node_id, p))
            .collect::<GraphIndex<'_>>();
        let keys = nodes
            .iter()
            .map(|n| Arc::new(n.node_id.clone()))
            .collect::<Vec<_>>();
        let handles = nodes.iter().map(|n| n.body.clone()).collect::<Vec<_>>();
        let rows = vec![vec![], vec![]];
        let owners = ModelOwners::default();
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.attach_bodies(&handles, &rows);
                result = Some(validate_state(
                    &nodes, &kinds, &terms, &index, &keys, &owners, &mut meter,
                ));
            });
            (result.unwrap(), measured.bytes_total, budget)
        };
        let (result, allocated, budget) = run(u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("nested clause placement must refuse");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        );
        assert_eq!(refusal.locus.as_ref(), Some(&nodes[1].node_id));
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            format!("/semantic_graph/nodes/1/body{}", "/members/0".repeat(40))
        );
        assert_eq!(
            budget.consumed().work,
            0,
            "placement traversal does not invent semantic work units"
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
    fn state_document_walk_admits_plain_bodies_and_refuses_root_at_node() {
        for root_fault in [false, true] {
            let nodes = [metadata(
                'a',
                if root_fault {
                    application(false)
                } else {
                    json!({"term":"aggregate","members":[]})
                },
            )];
            let kinds = [CheckedNodeKind::Expression(ExpressionForm::Query)];
            let index = nodes
                .iter()
                .enumerate()
                .map(|(p, n)| (&n.node_id, p))
                .collect::<GraphIndex<'_>>();
            let keys = [Arc::new(nodes[0].node_id.clone())];
            let handles = [nodes[0].body.clone()];
            let rows = [vec![]];
            let owners = ModelOwners::default();
            let run = |limit| {
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let measured = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(&mut budget);
                    meter.attach_bodies(&handles, &rows);
                    result = Some(validate_state(
                        &nodes,
                        &kinds,
                        &[None],
                        &index,
                        &keys,
                        &owners,
                        &mut meter,
                    ));
                });
                (result.unwrap(), measured.bytes_total, budget)
            };
            let (result, allocated, budget) = run(u64::MAX);
            if root_fault {
                let Err(ValidationFailure::Refused(refusal)) = result else {
                    panic!("wrong root kind");
                };
                assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
                assert_eq!(
                    refusal.loci[0].path.as_ref().unwrap().as_str(),
                    "/semantic_graph/nodes/0"
                );
            } else {
                result.unwrap();
            }
            let required = budget.read_storage_consumed();
            assert_eq!(allocated, required);
            assert_eq!(run(required).0.is_ok(), !root_fault);
            assert!(matches!(
                run(required - 1).0,
                Err(ValidationFailure::Budget(_))
            ));
            assert_eq!(run(0).1, 0);
        }
    }
}
