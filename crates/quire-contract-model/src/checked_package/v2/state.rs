//! The state step of the reader order (QSpec FR-341 and FR-342): after the
//! frame step and before the operation step, the reader checks, each in
//! ascending node-id digest order,
//!
//! 1. the placement of every `quire.op.state.clause` application: the body
//!    root of a `state`/`state_clause` node and nowhere else, else
//!    `ill_typed`/`operator-ineligible` at the node that holds it;
//! 2. every `state`/`operation_anchor` node: its `context` and `frame`
//!    targets, the three `semantic_type`s, and its operation name; then, once
//!    every anchor's own joins hold, that no two anchors share a
//!    (`context`, `operation`) pair or a `frame` target;
//! 3. every `state`/`state_clause` node: its anchor, its parameters, then its
//!    signature,
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

use super::model_members::{MemberKind, ModelOwners, ModelRefusal, OperationDecl, Owner, Resolved};
use super::structural::{
    anchor_body, clause_body, is_state_clause_application, reference_type_target, AnchorBody,
};
use super::{
    BodyTerm, CheckedNodeKind, CheckedSemanticNodeV2, CompositeTypeForm, ModelForm,
    StateClauseKind, StateForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{body_term, node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use std::collections::{BTreeMap, BTreeSet};

/// The graph the state step reads.
struct StateGraph<'a, 'm> {
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    owners: &'a ModelOwners<'m>,
}

impl StateGraph<'_, '_> {
    fn node(&self, id: &CheckedNodeId) -> Option<(&CheckedSemanticNodeV2, CheckedNodeKind)> {
        let position = *self.index.get(id)?;
        Some((self.nodes.get(position)?, *self.kinds.get(position)?))
    }

    /// Every node of `kind`, as `(position, node)`, in ascending node-id
    /// digest order.
    fn each(
        &self,
        kind: CheckedNodeKind,
    ) -> impl Iterator<Item = (usize, &CheckedSemanticNodeV2)> + '_ {
        self.index.values().filter_map(move |&position| {
            (self.kinds.get(position) == Some(&kind))
                .then(|| self.nodes.get(position).map(|node| (position, node)))
                .flatten()
        })
    }
}

/// The state step. See the module documentation for the order.
pub(super) fn validate_state(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StateGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    validate_placement(&graph)?;
    let mut anchors = Vec::new();
    for (position, node) in graph.each(CheckedNodeKind::State(StateForm::OperationAnchor)) {
        let body = anchor_body(&node.body).ok_or_else(|| shape_refusal(node, position))?;
        check_anchor(node, position, &body, &graph, meter)?;
        anchors.push((position, node, body));
    }
    let mut pairs = BTreeSet::new();
    let mut frames = BTreeSet::new();
    for (position, node, body) in &anchors {
        if !pairs.insert((&body.context, body.operation)) || !frames.insert(&body.frame) {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::AmbiguousDeclaration,
                node_pointer(*position),
                Some(CheckedPackageRefusalCause::AmbiguousName),
                node.node_id.clone(),
            ));
        }
    }
    for (position, node) in graph.each(CheckedNodeKind::State(StateForm::StateClause)) {
        check_clause(node, position, &graph, meter)?;
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
fn validate_placement(graph: &StateGraph<'_, '_>) -> Result<(), ValidationFailure> {
    for &position in graph.index.values() {
        let (Some(node), Some(&kind)) = (graph.nodes.get(position), graph.kinds.get(position))
        else {
            continue;
        };
        let body_at = node_pointer(position).key("body");
        let root_admitted = kind == CheckedNodeKind::State(StateForm::StateClause);
        let mut pending = vec![(&node.body, body_at, true)];
        while let Some((term, at, is_root)) = pending.pop() {
            if is_state_clause_application(term) && !(is_root && root_admitted) {
                return Err(ineligible(node, at));
            }
            let children: &[(&str, bool)] = match body_term(term) {
                Some(BodyTerm::Application) => &[("arguments", true)],
                Some(BodyTerm::Aggregate) => &[("members", true)],
                Some(BodyTerm::Binding) => &[("value", false)],
                Some(
                    BodyTerm::Literal
                    | BodyTerm::Reference
                    | BodyTerm::DependencyReference
                    | BodyTerm::Frame,
                )
                | None => &[],
            };
            for &(member, is_array) in children {
                let Some(value) = term.get(member) else {
                    continue;
                };
                if is_array {
                    let items = value.as_array().map(Vec::as_slice).unwrap_or_default();
                    // Pushed in reverse so the walk visits terms in order.
                    for (item, child) in items.iter().enumerate().rev() {
                        pending.push((child, at.clone().key(member).index(item), false));
                    }
                } else {
                    pending.push((value, at.clone().key(member), false));
                }
            }
        }
    }
    Ok(())
}

/// Joins a reference `target` held by `holder` to a declared dependency of
/// the admitted kind: `missing-name` when it is not a declared dependency
/// or names no node, `malformed-declaration` for another kind.
fn join<'g>(
    holder: &CheckedSemanticNodeV2,
    target: &CheckedNodeId,
    admits: impl Fn(CheckedNodeKind) -> bool,
    graph: &'g StateGraph<'_, '_>,
) -> Result<(&'g CheckedSemanticNodeV2, CheckedNodeKind), ModelRefusal> {
    let (node, kind) = graph
        .node(target)
        .filter(|_| holder.dependencies.contains(target))
        .ok_or(ModelRefusal::missing_name())?;
    if admits(kind) {
        Ok((node, kind))
    } else {
        Err(ModelRefusal::malformed())
    }
}

fn is_object_type(kind: CheckedNodeKind) -> bool {
    kind == CheckedNodeKind::Model(ModelForm::ObjectType)
}

/// A resolved operation and its owner, `None` for a context that is not a
/// model declaration node, or the refusal FR-342's operation check
/// determines.
type OperationResolution<'m> = Result<Option<(Owner<'m>, &'m OperationDecl)>, ModelRefusal>;

/// The operation a model-owned context declares under `name`: `Ok(None)`
/// when the context is not a model declaration node, else the owner and
/// the operation, or the refusal FR-342's operation check determines.
fn resolve_operation<'m>(
    context: &CheckedSemanticNodeV2,
    context_kind: CheckedNodeKind,
    name: &str,
    graph: &StateGraph<'_, 'm>,
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

/// FR-342's checks over one anchor: context, frame, types, operation.
fn check_anchor(
    anchor: &CheckedSemanticNodeV2,
    position: usize,
    body: &AnchorBody<'_>,
    graph: &StateGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let target_at = |member: usize| {
        node_pointer(position)
            .key("body")
            .key("members")
            .index(member)
    };
    let (context, context_kind) =
        join(anchor, &body.context, is_object_type, graph).map_err(|refusal| {
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
        Ok(_) => Ok(()),
        Err(refusal) => Err(model_refusal(refusal, target_at(1), &anchor.node_id)),
    }
}

/// FR-341's checks over one clause: anchor, parameters, signature.
fn check_clause(
    clause: &CheckedSemanticNodeV2,
    position: usize,
    graph: &StateGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body = clause_body(&clause.body).ok_or_else(|| shape_refusal(clause, position))?;
    let arguments_at = node_pointer(position).key("body").key("arguments");
    let invariant = body.clause == StateClauseKind::Invariant;
    let (anchor, _) = join(
        clause,
        &body.anchor,
        |kind| {
            kind == if invariant {
                CheckedNodeKind::Model(ModelForm::ObjectType)
            } else {
                CheckedNodeKind::State(StateForm::OperationAnchor)
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
    let anchor_body = if invariant {
        None
    } else {
        Some(anchor_body(&anchor.body).ok_or_else(signature_refusal)?)
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
        return if parameters.len() == 1 {
            Ok(())
        } else {
            Err(signature_refusal())
        };
    };
    let Some((context, context_kind)) = graph.node(context_id) else {
        return Err(signature_refusal());
    };
    let (owner, operation) =
        match resolve_operation(context, context_kind, anchor_body.operation, graph, meter)? {
            Ok(Some(resolved)) => resolved,
            // A context that is not a model declaration node: nothing past
            // `self` is resolved.
            Ok(None) => return Ok(()),
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
        let key = owner.package.slot_type(slot).map(|ty| ty.node_key());
        if key.as_deref() != Some(&*parameter.semantic_type.digest) {
            return Err(signature_refusal());
        }
    }
    Ok(())
}
