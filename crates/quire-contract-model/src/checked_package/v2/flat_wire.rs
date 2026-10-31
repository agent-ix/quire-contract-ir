//! The flat wire: strict wire validation of every node `body` and every
//! diagnostic `details` term against the closed body grammar of merged QSpec
//! FR-322 "Body grammar" (FR-038 "The flat wire", FR-038-AC-114 through
//! FR-038-AC-116).
//!
//! The grammar has five strata, and no production names itself or a higher
//! one, so the JSON depth of every package is fixed whatever its node count:
//!
//! | Stratum | Productions |
//! | --- | --- |
//! | Leaf | `literal`, `reference`, `dependency_reference` |
//! | Group | an `aggregate` whose members are each a Leaf or a `binding` whose value is a Leaf |
//! | Tuple | an `aggregate` whose members are each a Leaf or a Group |
//! | Member | a Leaf, a Group, or a `binding` whose value is a Leaf, a Group or a Tuple |
//! | Body | a Leaf; an `application` whose `arguments` are each a Member; an `aggregate` whose members are each a Member; a frame body; an abstraction relation body; an extension form body |
//!
//! Each [`Place`] below is one of those positions, and [`Flat`] enters the
//! terms of a body in document pre-order, outermost first, on `quire-walk`'s
//! heap stack, refusing the first term the place it stands in does not admit.
//! A diagnostic's `details` are each a Member. The check runs over the decoded
//! document after the closed-schema decode and before any identity is
//! recomputed, so a body outside the grammar refuses at the term itself and
//! never at a stale identity (FR-038-AC-116).
//!
//! This module decides where an application may stand, and, after those
//! refusals, whether a `temporal_interval` bound is outside its form's pattern
//! (FR-038 "The timed interval form" stage 1); the term's own closed shape is
//! [`crate::checked_package::common::validate_term`]'s, later.

use super::temporal::{interval_bound_outside_pattern, timed_upper_is_null};
use super::{
    body_grammar, BodyGrammar, CheckedNodeId, CheckedNodeKind, CheckedNodeTag, CheckedTerm,
    CheckedTermView, HybridForm, TemporalForm, ValueForm,
};
use super::{ApplicationOperator, BodyTerm, RawCheckedPackageWireV2 as CheckedPackageWireV2};
use crate::checked_package::common::{
    application_operator, body_term, Step, Trail, ValidationFailure, WireTerm,
};
use crate::checked_package::shared::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
use crate::checked_package::terms::{subterms, subterms_of, At, Cursor};
use quire_walk::{walk, Children, Walk};
use serde::Deserialize;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::ControlFlow;

/// Where a term stands, which fixes what it may be.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Place {
    /// A node's `body`: a Leaf, an `application` or an `aggregate`.
    BodyRoot,
    /// An application's argument, a body aggregate's member, a `details` term:
    /// a Leaf, a Group or a `binding`.
    Member,
    /// A member of a Group: a Leaf or a `binding` whose value is a Leaf.
    GroupMember,
    /// A member of a Tuple: a Leaf or a Group.
    TupleMember,
    /// The value of a Group's `binding`: a Leaf.
    LeafOnly,
    /// The value of a Member's `binding`: a Leaf, a Group or a Tuple.
    BindingValue,
}

/// Classify the two published profile identities at the wire intake boundary.
#[ix_trace_rs::string_edge]
fn wrong_profile_member(operation: &Value) -> bool {
    let member_operator = operation["member"]["operator"].as_str();
    match operation["identity"].as_str() {
        Some("quire.op.state.transition") => {
            member_operator.is_some_and(|name| name != "transition")
        }
        Some("quire.op.protocol.control") => member_operator == Some("transition"),
        _ => false,
    }
}

/// Check the closed transition profile layout before semantic joins.
#[ix_trace_rs::string_edge]
fn transition_profile_wire_shape(operation: &Value) -> bool {
    operation["laws"]
        .as_array()
        .is_some_and(|laws| laws.len() == 1 && laws[0]["role"] == "protocol_profile")
        && operation["member"]["kind"] == "profile_operator"
        && operation["member"]["operator"] == "transition"
        && operation["mode"].is_null()
        && operation["leaves"].as_array().is_some_and(Vec::is_empty)
}

/// Checks every node body and every `details` term of `wire` against the flat
/// grammar, nodes first, in document order, then the diagnostics' `details`.
/// Closed node headers refuse before checking the single shipped graph, using
/// the same header decoder as the graph stage.
pub(super) fn check(
    wire: &CheckedPackageWireV2,
) -> Result<Vec<CheckedNodeKind>, ValidationFailure> {
    let mut kinds = Vec::with_capacity(wire.semantic_graph.nodes.len());
    for (position, node) in wire.semantic_graph.nodes.iter().enumerate() {
        let kind = super::decode_node_kind(node, |member| {
            crate::checked_package::common::node_pointer(position).key(member)
        })
        .map_err(|failure| {
            // These families' malformed forms precede even a bad schema
            // version. Decode the successful header only once; inspect the
            // typed vocabulary again solely to preserve that refusal order.
            let malformed_form = CheckedNodeTag::from_wire(&node.node_tag).is_some_and(|tag| {
                matches!(
                    tag,
                    CheckedNodeTag::Extension | CheckedNodeTag::Hybrid | CheckedNodeTag::TimedModel
                ) && CheckedNodeKind::decode(tag, &node.semantic_form).is_none()
            });
            if malformed_form {
                ValidationFailure::refused(
                    CheckedPackageRefusalCode::MalformedWire,
                    crate::checked_package::common::node_pointer(position).key("semantic_form"),
                )
            } else {
                failure
            }
        })?;
        if let CheckedNodeKind::Protocol(form) = kind {
            if form != super::ProtocolForm::ProtocolReference {
                let expected_role = if form == super::ProtocolForm::Queue {
                    crate::checked_package::shared::CheckedOccurrenceRole::Generated
                } else {
                    crate::checked_package::shared::CheckedOccurrenceRole::Declaration
                };
                // Empty occurrences are diagnosed by the source-map pass;
                // a present occurrence with the wrong role is a wire defect.
                if node
                    .occurrences
                    .iter()
                    .any(|occurrence| occurrence.role != expected_role)
                    || node.declaration.is_some() != (form != super::ProtocolForm::Queue)
                {
                    return Err(ValidationFailure::refused(
                        CheckedPackageRefusalCode::MalformedWire,
                        crate::checked_package::common::node_pointer(position),
                    ));
                }
            }
        }
        let steps = [
            Step::Key("semantic_graph"),
            Step::Key("nodes"),
            Step::Index(position),
            Step::Key("body"),
        ];
        let at = Trail::Base(&steps);
        let checked = match body_grammar(kind) {
            BodyGrammar::Extension => {
                if node.declaration.is_none() || node.owner.is_none() {
                    Err(ValidationFailure::refused(
                        CheckedPackageRefusalCode::MalformedWire,
                        crate::checked_package::common::node_pointer(position),
                    ))
                } else if node.occurrences.len() != 1
                    || node.occurrences[0].role
                        != crate::checked_package::shared::CheckedOccurrenceRole::Declaration
                    || node.occurrences[0].ordinal != 0
                {
                    Err(ValidationFailure::refused(
                        CheckedPackageRefusalCode::MalformedWire,
                        crate::checked_package::common::node_pointer(position).key("occurrences"),
                    ))
                } else {
                    super::extension::read_body(&node.body, at.pointer()).map(|_| ())
                }
            }
            // A frame body and an abstraction relation body are closed shapes
            // of their own, and no application stands in either; an
            // application at the root of an abstraction relation body is a
            // term of the closed grammar, placed by the abstraction step.
            BodyGrammar::Frame => scan_members(&node.body, &at).map(|_| ()),
            BodyGrammar::Term => {
                if kind == CheckedNodeKind::Claim(super::ClaimForm::StepRelation) {
                    scan_members(&node.body, &at).map(|_| ())
                } else if kind == CheckedNodeKind::State(super::StateForm::OperationAnchor) {
                    check_anchor_flat_wire(&node.body, &at)
                } else if kind == CheckedNodeKind::Protocol(super::ProtocolForm::ProtocolReference)
                    && body_term(&node.body) == Some(BodyTerm::Application)
                    && application_operator(&node.body)
                        == Some(ApplicationOperator::ProtocolControl)
                {
                    Err(ValidationFailure::refused(
                        CheckedPackageRefusalCode::MalformedWire,
                        at.pointer(),
                    ))
                } else {
                    check_term(&node.body, &at, Place::BodyRoot, false).and_then(|_| {
                        // QSpec FR-341-AC-19 fixes these two profile-member
                        // pairs in the published schema. Reject them during
                        // wire validation, before a stale application key or
                        // the operation step can decide the outcome.
                        let operation = &node.body["operation"];
                        if wrong_profile_member(operation) {
                            return Err(ValidationFailure::refused(
                                CheckedPackageRefusalCode::MalformedWire,
                                at.key("operation").key("member").key("operator").pointer(),
                            ));
                        }
                        let member = &operation["member"];
                        if member["kind"].as_str() == Some("relationship_end")
                            && member
                                .get("destination_type")
                                .and_then(CheckedNodeId::from_value)
                                .is_none()
                        {
                            return Err(ValidationFailure::refused(
                                CheckedPackageRefusalCode::MalformedWire,
                                at.key("operation")
                                    .key("member")
                                    .key("destination_type")
                                    .pointer(),
                            ));
                        }
                        if kind == CheckedNodeKind::State(super::StateForm::Transition) {
                            let body = CheckedTerm::decode_admitted(&node.body).ok();
                            let transition_shape = body
                                .as_ref()
                                .and_then(super::structural::typed_transition_body)
                                .is_some()
                                && transition_profile_wire_shape(operation);
                            if !transition_shape {
                                return Err(ValidationFailure::refused(
                                    CheckedPackageRefusalCode::MalformedWire,
                                    at.pointer(),
                                ));
                            }
                            if let Some(ordinal) = node.occurrences.iter().position(|occurrence| {
                                occurrence.role
                                    != crate::checked_package::shared::CheckedOccurrenceRole::Claim
                            }) {
                                return Err(ValidationFailure::refused(
                                    CheckedPackageRefusalCode::MalformedWire,
                                    crate::checked_package::common::node_pointer(position)
                                        .key("occurrences")
                                        .index(ordinal)
                                        .key("role"),
                                ));
                            }
                        }
                        let CheckedNodeKind::Protocol(form) = kind else {
                            return Ok(());
                        };
                        let Some(body) = CheckedTerm::decode_admitted(&node.body).ok() else {
                            return Ok(());
                        };
                        if let Some(defect) = super::protocol_body::wire_layout_defect(form, &body)
                        {
                            let path = match defect {
                                super::protocol_body::ProtocolLayoutWireDefect::Body => {
                                    at.pointer()
                                }
                                super::protocol_body::ProtocolLayoutWireDefect::NegativeQueueCapacity => {
                                    return Err(ValidationFailure::refused_at(
                                        CheckedPackageRefusalCode::InvalidPackage,
                                        at.pointer().key("arguments").index(1).key("value")
                                            .key("members").index(1).key("value"),
                                        Some(super::super::shared::CheckedPackageRefusalCause::InvalidValue),
                                        node.node_id.clone(),
                                    ));
                                }
                                super::protocol_body::ProtocolLayoutWireDefect::NegativeBound { argument, member, binding } => {
                                    let mut path = at.pointer().key("arguments").index(argument).key("value")
                                        .key("members").index(member);
                                    if binding { path = path.key("value"); }
                                    return Err(ValidationFailure::refused_at(
                                        CheckedPackageRefusalCode::InvalidPackage,
                                        path,
                                        Some(super::super::shared::CheckedPackageRefusalCause::InvalidValue),
                                        node.node_id.clone(),
                                    ));
                                }
                                super::protocol_body::ProtocolLayoutWireDefect::EnumValue {
                                    argument,
                                    member,
                                } => {
                                    let mut path =
                                        at.pointer().key("arguments").index(argument).key("value");
                                    if let Some((position, binding)) = member {
                                        path = path.key("members").index(position);
                                        if binding {
                                            path = path.key("value");
                                        }
                                    }
                                    path
                                }
                            };
                            Err(ValidationFailure::refused(
                                CheckedPackageRefusalCode::MalformedWire,
                                path,
                            ))
                        } else {
                            Ok(())
                        }
                    })
                }
            }
            BodyGrammar::TimedModel => {
                let CheckedNodeKind::TimedModel(form) = kind else {
                    unreachable!("timed grammar requires a timed kind")
                };
                super::timed_model::check_wire(node, form, position)
            }
            BodyGrammar::Hybrid => {
                let super::CheckedNodeKind::Hybrid(form) = kind else {
                    unreachable!("hybrid grammar is selected only for hybrid nodes")
                };
                super::hybrid::check_wire(node, form, position)
            }
            BodyGrammar::AbstractionRelation => {
                if body_term(&node.body) == Some(BodyTerm::Application) {
                    check_term(&node.body, &at, Place::BodyRoot, false)
                } else {
                    scan_members(&node.body, &at).map(|_| ())
                }
            }
            BodyGrammar::Refinement => super::refinement::check_wire(&node.body, &at),
        };
        checked.map_err(|failure| super::with_node_locus(failure, &node.node_id))?;
        kinds.push(kind);
    }
    for (entry_index, entry) in wire.diagnostics.entries.iter().enumerate() {
        for (detail_index, detail) in entry.details.iter().enumerate() {
            let steps = [
                Step::Key("diagnostics"),
                Step::Key("entries"),
                Step::Index(entry_index),
                Step::Key("details"),
                Step::Index(detail_index),
            ];
            check_term(detail, &Trail::Base(&steps), Place::Member, true)?;
        }
    }
    Ok(kinds)
}

/// The fourth anchor binding is a bare point object, outside the term grammar.
/// The graph-shape stage owns its closed binding and value refusals.
fn check_anchor_flat_wire(body: &Value, at: &Trail<'_>) -> Result<(), ValidationFailure> {
    let Some(members) = body.get("members").and_then(Value::as_array) else {
        return check_term(body, at, Place::BodyRoot, false);
    };
    if body_term(body) != Some(BodyTerm::Aggregate) {
        return check_term(body, at, Place::BodyRoot, false);
    }
    if members.len() < 4
        && !members
            .iter()
            .any(|member| member.get("name").and_then(Value::as_str) == Some("execution_point"))
    {
        return check_term(body, at, Place::BodyRoot, false);
    }
    for (index, member) in members.iter().enumerate() {
        if member.get("name").and_then(Value::as_str) == Some("execution_point") {
            continue;
        }
        check_term(
            member,
            &at.key("members").index(index),
            Place::Member,
            false,
        )?;
    }
    Ok(())
}

/// A `temporal_interval` bound outside its form's pattern (FR-038 "The timed
/// interval form" stage 1; merged QSpec FR-370: "during strict wire validation,
/// before any step"), refused `invalid_package`/`invalid-value` at the bound,
/// first in member order and in node position order, after the nested-application
/// refusals above (the spec is silent on the order of the two wire checks, an IR
/// reading). One flat loop over the nodes: no recursion.
pub(super) fn check_interval_bounds(
    wire: &CheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
) -> Result<(), ValidationFailure> {
    let mut hybrid_formulas = None;
    for (position, (node, &kind)) in wire.semantic_graph.nodes.iter().zip(kinds).enumerate() {
        let body = WireTerm::decode(&node.body);
        if matches!(
            body_grammar(kind),
            BodyGrammar::Frame
                | BodyGrammar::Extension
                | BodyGrammar::TimedModel
                | BodyGrammar::Hybrid
        ) || body.kind() != Some(BodyTerm::Application)
        {
            continue;
        }
        let operation = match CheckedTerm::view_from_wire(body) {
            Some(CheckedTermView::Application { operation, .. }) => Some(Cow::Borrowed(operation)),
            _ => ApplicationProjection::deserialize(&node.body)
                .ok()
                .and_then(|projection| projection.operation)
                .map(Cow::Owned),
        };
        let Some(operation) = operation else {
            continue;
        };
        let outside_pattern = interval_bound_outside_pattern(&operation);
        let nonhybrid_open_upper = timed_upper_is_null(&operation)
            && !hybrid_formulas
                .get_or_insert_with(|| hybrid_formula_nodes(wire, kinds))
                .contains(&node.node_id);
        if let Some(bound) = outside_pattern.or(nonhybrid_open_upper.then_some("upper")) {
            let steps = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(position),
                Step::Key("body"),
            ];
            let failure = ValidationFailure::refused_because(
                CheckedPackageRefusalCode::InvalidPackage,
                Trail::Base(&steps)
                    .key("operation")
                    .key("member")
                    .key("interval")
                    .key(bound)
                    .pointer(),
                CheckedPackageRefusalCause::InvalidValue,
            );
            return Err(super::with_node_locus(failure, &node.node_id));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct ApplicationProjection {
    #[serde(default)]
    operation: Option<Value>,
    #[serde(default)]
    arguments: Option<Value>,
}

#[derive(Deserialize)]
struct ReferenceProjection {
    #[serde(default)]
    target: Option<CheckedNodeId>,
}

fn reference_target(term: &CheckedTerm) -> Option<CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target.clone()),
        _ => None,
    }
}

/// A malformed application still exposes its independent arguments to the
/// early interval exception check. Decode that one projection only when the
/// complete admitted term cannot be built; later shape checks own its refusal.
fn application_targets(body: &Value) -> Option<Vec<Option<CheckedNodeId>>> {
    if let Ok(CheckedTerm::Application { arguments, .. }) = CheckedTerm::decode_admitted(body) {
        return Some(arguments.iter().map(reference_target).collect());
    }
    let projection = ApplicationProjection::deserialize(body).ok()?;
    Some(
        projection
            .arguments?
            .as_array()?
            .iter()
            .map(|argument| ReferenceProjection::deserialize(argument).ok()?.target)
            .collect(),
    )
}

/// Formula nodes reached by a claim whose over parameter is typed by a
/// hybrid model. This classification uses the raw graph only to decide the
/// timed-null wire exception; authenticated identity and joins run later.
fn hybrid_formula_nodes(
    wire: &CheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
) -> BTreeSet<CheckedNodeId> {
    let nodes = &wire.semantic_graph.nodes;
    let positions = nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (&node.node_id, position))
        .collect::<BTreeMap<_, _>>();
    let mut pending = Vec::new();
    for (position, node) in nodes.iter().enumerate() {
        if kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::TemporalClause)) {
            continue;
        }
        let Some(arguments) = application_targets(&node.body) else {
            continue;
        };
        let Some(over) = arguments.first().cloned().flatten() else {
            continue;
        };
        let Some(&over_position) = positions.get(&over) else {
            continue;
        };
        if kinds.get(over_position) != Some(&CheckedNodeKind::Value(ValueForm::Parameter)) {
            continue;
        }
        let Some(over_node) = nodes.get(over_position) else {
            continue;
        };
        let Some(&model_position) = positions.get(&over_node.semantic_type) else {
            continue;
        };
        if kinds.get(model_position) == Some(&CheckedNodeKind::Hybrid(HybridForm::Model)) {
            pending.extend(arguments.get(5).cloned().flatten());
        }
    }
    let mut seen = BTreeSet::new();
    let mut formulas = BTreeSet::new();
    while let Some(key) = pending.pop() {
        if !seen.insert(key.clone()) {
            continue;
        }
        let Some(&position) = positions.get(&key) else {
            continue;
        };
        if kinds.get(position) == Some(&CheckedNodeKind::Temporal(TemporalForm::Formula)) {
            formulas.insert(key);
            let Some(node) = nodes.get(position) else {
                continue;
            };
            if let Some(arguments) = application_targets(&node.body) {
                pending.extend(arguments.into_iter().flatten());
            }
        }
    }
    formulas
}

/// Checks the term `root` at `at`, standing in `place`, and every term below
/// it. `details` is whether the term belongs to a diagnostic's `details`,
/// which refuses an application of the temporal formula and fairness classes as
/// it refuses a `case` one.
fn check_term(
    root: &Value,
    at: &Trail<'_>,
    place: Place,
    details: bool,
) -> Result<(), ValidationFailure> {
    let cursor = Cursor::at(at);
    let node = cursor.root(root, place);
    match walk(&mut Flat { cursor, details }, node) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(failure) => Err(failure),
    }
}

/// A row expression stands at the root of its own SemanticTerm grammar.
pub(super) fn check_refinement_term(root: &Value, at: &Trail<'_>) -> Result<(), ValidationFailure> {
    check_term(root, at, Place::BodyRoot, false)
}

/// Walks the values below the root of a frame or abstraction relation body,
/// each an object member or array element, in document pre-order, outermost
/// first, on `quire-walk`'s heap stack, and refuses the first application: a
/// `case` one `ill_typed`/`operator-ineligible` at its `operator`, any other
/// `malformed_wire` at the application. Charges one unit of work per value
/// entered, the work the abstraction relation step's body read is charged. A
/// pointer is built only for the application that refuses.
pub(super) fn scan_members(body: &Value, at: &Trail<'_>) -> Result<u64, ValidationFailure> {
    let cursor = Cursor::at(at);
    let root = cursor.root(body, true);
    let mut scan = Scan { cursor, work: 0 };
    match walk(&mut scan, root) {
        ControlFlow::Continue(()) => Ok(scan.work),
        ControlFlow::Break(failure) => Err(failure),
    }
}

/// The walk of [`scan_members`]: every JSON value below the body root, an
/// object's members and an array's elements alike.
struct Scan<'a> {
    cursor: Cursor<'a>,
    work: u64,
}

impl<'a> Walk for Scan<'a> {
    /// A value, and whether it is the body root.
    type Node = At<'a, bool>;
    type Frame = ();
    type Stop = ValidationFailure;

    fn enter(
        &mut self,
        node: At<'a, bool>,
        children: &mut Children<'_, At<'a, bool>>,
    ) -> ControlFlow<ValidationFailure> {
        self.cursor.enter(&node);
        self.work = self.work.saturating_add(1);
        let is_root = node.extra;
        if !is_root && body_term(node.value) == Some(BodyTerm::Application) {
            return ControlFlow::Break(nested_application_refusal(
                node.value,
                &self.cursor.trail(),
                false,
            ));
        }
        match node.value {
            Value::Object(members) => children.extend(
                members
                    .iter()
                    .map(|(key, member)| self.cursor.child(member, Some(key), None, false)),
            ),
            Value::Array(items) => children.extend(
                items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| self.cursor.child(item, None, Some(index), false)),
            ),
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
        ControlFlow::Continue(())
    }

    fn exit(&mut self, (): ()) -> ControlFlow<ValidationFailure> {
        ControlFlow::Continue(())
    }
}

/// The walk of [`check_term`].
struct Flat<'a> {
    cursor: Cursor<'a>,
    details: bool,
}

impl<'a> Walk for Flat<'a> {
    type Node = At<'a, Place>;
    type Frame = ();
    type Stop = ValidationFailure;

    fn enter(
        &mut self,
        node: At<'a, Place>,
        children: &mut Children<'_, At<'a, Place>>,
    ) -> ControlFlow<ValidationFailure> {
        self.cursor.enter(&node);
        let value = node.value;
        let place = node.extra;
        let wire = WireTerm::decode(value);
        let below = match wire.kind() {
            // An application stands only at a body root; its arguments are
            // each a Member.
            Some(BodyTerm::Application) if place == Place::BodyRoot => Place::Member,
            Some(BodyTerm::Application) => {
                return ControlFlow::Break(nested_application_refusal(
                    value,
                    &self.cursor.trail(),
                    self.details,
                ));
            }
            Some(BodyTerm::Aggregate) => match place {
                Place::BodyRoot => Place::Member,
                Place::Member | Place::TupleMember => Place::GroupMember,
                Place::BindingValue => aggregate_stratum(value),
                Place::GroupMember | Place::LeafOnly => {
                    return ControlFlow::Break(self.outside_grammar());
                }
            },
            Some(BodyTerm::Binding) => match place {
                Place::Member => Place::BindingValue,
                Place::GroupMember => Place::LeafOnly,
                Place::BodyRoot | Place::TupleMember | Place::LeafOnly | Place::BindingValue => {
                    return ControlFlow::Break(self.outside_grammar());
                }
            },
            // A Leaf has no subterm. A term of another shape is the closed
            // shape check's to refuse, later, and a frame or abstraction
            // relation term is no subterm.
            Some(
                BodyTerm::Literal
                | BodyTerm::Reference
                | BodyTerm::DependencyReference
                | BodyTerm::Frame
                | BodyTerm::AbstractionRelation,
            )
            | None => return ControlFlow::Continue(()),
        };
        children.extend(subterms_of(wire).map(|subterm| self.cursor.subterm(subterm, below)));
        ControlFlow::Continue(())
    }

    fn exit(&mut self, (): ()) -> ControlFlow<ValidationFailure> {
        ControlFlow::Continue(())
    }
}

impl Flat<'_> {
    /// `malformed_wire` at the entered term: it stands where its stratum is
    /// not admitted.
    fn outside_grammar(&self) -> ValidationFailure {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::MalformedWire,
            self.cursor.trail().pointer(),
        )
    }
}

/// The place of the members of an `aggregate` that stands as the value of a
/// Member's `binding`, where it may be a Group or a Tuple. The first member
/// that is not a Leaf decides: a `binding` makes it a Group, an `aggregate`
/// makes it a Tuple. An aggregate of Leaves is both, and its members need no
/// further place; a first non-Leaf member of another kind is refused by its own
/// place whichever stratum the aggregate is read as.
fn aggregate_stratum(aggregate: &Value) -> Place {
    subterms(aggregate)
        .find_map(|subterm| match body_term(subterm.value) {
            Some(BodyTerm::Aggregate) => Some(Place::TupleMember),
            Some(BodyTerm::Binding) => Some(Place::GroupMember),
            Some(
                BodyTerm::Literal
                | BodyTerm::Reference
                | BodyTerm::DependencyReference
                | BodyTerm::Application
                | BodyTerm::Frame
                | BodyTerm::AbstractionRelation,
            )
            | None => None,
        })
        .unwrap_or(Place::GroupMember)
}

/// The refusal of an application that stands nested inside a node body or
/// another term, at `at`, where only a body root may hold one: a `case`
/// application refuses `ill_typed` with cause `operator-ineligible` at its
/// `operator` (merged FR-322 "Body grammar", FR-440 "Case placement"), and so
/// does an application of the `temporal_formula` or `temporal_fairness` class in
/// a diagnostic's `details` term (merged FR-370-AC-12); an application of any
/// other class refuses `malformed_wire` at the application itself.
pub(super) fn nested_application_refusal(
    application: &Value,
    at: &Trail<'_>,
    details: bool,
) -> ValidationFailure {
    let operator = match CheckedTerm::view(application) {
        Some(CheckedTermView::Application { operator, .. }) => Some(operator),
        _ => application_operator(application),
    };
    let operator_ineligible = match operator {
        Some(ApplicationOperator::Case) => true,
        Some(ApplicationOperator::TemporalFormula | ApplicationOperator::TemporalFairness) => {
            details
        }
        Some(
            ApplicationOperator::Call
            | ApplicationOperator::Unary
            | ApplicationOperator::Binary
            | ApplicationOperator::Conditional
            | ApplicationOperator::Let
            | ApplicationOperator::Quantify
            | ApplicationOperator::Collection
            | ApplicationOperator::Query
            | ApplicationOperator::Convert
            | ApplicationOperator::Pre
            | ApplicationOperator::Present
            | ApplicationOperator::Value
            | ApplicationOperator::Deref
            | ApplicationOperator::Reaches
            | ApplicationOperator::Temporal
            | ApplicationOperator::ProtocolControl
            | ApplicationOperator::StateTransition
            | ApplicationOperator::Claim
            | ApplicationOperator::StateClause,
        )
        | None => false,
    };
    if operator_ineligible {
        ValidationFailure::refused_because(
            CheckedPackageRefusalCode::IllTyped,
            at.key("operator").pointer(),
            CheckedPackageRefusalCause::OperatorIneligible,
        )
    } else {
        ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at.pointer())
    }
}

#[cfg(test)]
mod tests {
    use super::{check, check_term, scan_members, Place};
    use crate::checked_package::common::Trail;
    use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
    use crate::checked_package::v2::{
        CheckedNodeTag, ExtensionForm, HybridForm, RawCheckedPackageWireV2, TimedModelForm,
    };
    use ix_trace_rs::trace;
    use serde_json::{json, Map, Value};

    /// `value` nested in `levels` arrays, each moved into the next: `json!`
    /// would copy the value below it, recursively.
    fn nested_in_arrays(value: Value, levels: usize) -> Value {
        (0..levels).fold(value, |inner, _| Value::Array(vec![inner]))
    }

    fn application() -> Value {
        json!({"term": "application", "operator": "call", "arguments": []})
    }

    /// Private wire-shape inputs only: check does not authenticate identity
    /// or admit a package. These inputs deliberately combine header defects.
    fn header_node(tag: &str, form: &str, schema: &str, body: Value) -> Value {
        let id = json!({"package": "test/header", "path": [tag, form], "ordinal": 0});
        json!({
            "schema_version": schema, "node_id": id.clone(), "semantic_type": id,
            "node_tag": tag, "semantic_form": form, "dependencies": [],
            "occurrences": [], "body": body,
        })
    }

    fn header_wire(nodes: Vec<Value>) -> RawCheckedPackageWireV2 {
        serde_json::from_value(json!({
            "contract_version": "quire.checked-package/v2",
            "name": "test/header",
            "lock": {
                "sources": [], "edition": {"role": "edition", "definition": {"authority": "test", "identity": "edition"}},
                "profile_selections": [], "definition_selections": [],
                "required_features": [], "dependency_selections": [],
            },
            "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": nodes},
            "source_map": [], "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"}, "entries": []},
        })).expect("private flat-wire input")
    }

    fn header_refusal(nodes: Vec<Value>, code: CheckedPackageRefusalCode, path: &str) {
        let failure = check(&header_wire(nodes)).expect_err("damaged wire refuses");
        let crate::checked_package::common::ValidationFailure::Refused(refusal) = failure else {
            panic!("a refusal, not an incomplete result");
        };
        assert_eq!(refusal.code, code);
        assert_eq!(refusal.loci[0].path.clone(), JsonPointer::parse(path));
        assert_eq!(refusal.cause, None);
    }

    #[trace("TC-048")]
    #[test]
    fn flat_header_special_form_refusal_precedes_schema_and_body_defects() {
        for tag in [
            CheckedNodeTag::Extension,
            CheckedNodeTag::Hybrid,
            CheckedNodeTag::TimedModel,
        ] {
            for schema in [super::super::GRAPH_V2, "unsupported-schema"] {
                header_refusal(
                    vec![header_node(
                        tag.as_wire(),
                        "unknown-form",
                        schema,
                        application(),
                    )],
                    CheckedPackageRefusalCode::MalformedWire,
                    "/semantic_graph/nodes/0/semantic_form",
                );
            }
        }
    }

    /// QSpec TC-946 V6: a carrier cannot become a protocol declaration.
    #[trace("TC-048")]
    #[test]
    fn protocol_reference_rejects_protocol_application_at_wire_boundary() {
        header_refusal(
            vec![header_node(
                "protocol",
                "protocol_reference",
                super::super::GRAPH_V2,
                json!({"term": "application", "operator": "protocol_control", "arguments": []}),
            )],
            CheckedPackageRefusalCode::MalformedWire,
            "/semantic_graph/nodes/0/body",
        );
    }

    /// QSpec FR-464 step 1 rejects a known operator's damaged fixed layout
    /// before semantic graph validation can report a later defect.
    #[trace("TC-048")]
    #[test]
    fn protocol_clause_rejects_short_layout_at_wire_boundary() {
        let mut node = header_node(
            "protocol",
            "protocol_clause",
            super::super::GRAPH_V2,
            json!({
                "term": "application", "operator": "protocol_control",
                "operation": {
                    "identity": "quire.op.protocol.control", "laws": [], "mode": null,
                    "member": {"kind": "profile_operator", "operator": "protocol"},
                    "leaves": []
                },
                "result_type": {"package": "test/header", "path": ["protocol", "protocol_clause"], "ordinal": 0},
                "arguments": []
            }),
        );
        node["declaration"] = json!({"qualified_name": ["Main"]});
        header_refusal(
            vec![node],
            CheckedPackageRefusalCode::MalformedWire,
            "/semantic_graph/nodes/0/body",
        );
    }

    #[trace("TC-048")]
    #[test]
    fn flat_header_known_special_forms_preserve_schema_priority() {
        let forms = ExtensionForm::ALL
            .iter()
            .map(|form| (CheckedNodeTag::Extension, form.as_wire()))
            .chain(
                HybridForm::ALL
                    .iter()
                    .map(|form| (CheckedNodeTag::Hybrid, form.as_wire())),
            )
            .chain(
                TimedModelForm::ALL
                    .iter()
                    .map(|form| (CheckedNodeTag::TimedModel, form.as_wire())),
            );
        for (tag, form) in forms {
            header_refusal(
                vec![header_node(
                    tag.as_wire(),
                    form,
                    "unsupported-schema",
                    application(),
                )],
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "/semantic_graph/nodes/0/schema_version",
            );
        }
    }

    #[trace("TC-048")]
    #[test]
    fn flat_header_ordinary_unknown_tag_and_form_keep_decoder_dispositions() {
        for (tag, form, schema, code, member) in [
            (
                "unknown-tag",
                "integer",
                "unsupported-schema",
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "schema_version",
            ),
            (
                "unknown-tag",
                "integer",
                super::super::GRAPH_V2,
                CheckedPackageRefusalCode::UnsupportedNodeTag,
                "node_tag",
            ),
            (
                "scalar_type",
                "unknown-form",
                "unsupported-schema",
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "schema_version",
            ),
            (
                "scalar_type",
                "unknown-form",
                super::super::GRAPH_V2,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                "semantic_form",
            ),
        ] {
            header_refusal(
                vec![header_node(tag, form, schema, application())],
                code,
                &format!("/semantic_graph/nodes/0/{member}"),
            );
        }
    }

    #[trace("TC-048")]
    #[test]
    fn flat_header_checks_keep_body_and_later_header_in_document_order() {
        let first = header_node(
            "scalar_type",
            "integer",
            super::super::GRAPH_V2,
            json!({"term": "aggregate", "members": [application()]}),
        );
        let second = header_node(
            "unknown-tag",
            "integer",
            super::super::GRAPH_V2,
            application(),
        );
        header_refusal(
            vec![first.clone(), second.clone()],
            CheckedPackageRefusalCode::MalformedWire,
            "/semantic_graph/nodes/0/body/members/0",
        );
        header_refusal(
            vec![second, first],
            CheckedPackageRefusalCode::UnsupportedNodeTag,
            "/semantic_graph/nodes/0/node_tag",
        );
    }

    /// The scan of a frame or abstraction relation body reaches an application
    /// at any depth without native recursion: one 200000 arrays deep is found,
    /// at a pointer of 200000 steps, on a thread whose stack is 256 KiB; the same
    /// body with none is charged one unit per value.
    #[trace("TC-048", "FR-038-AC-117")]
    #[test]
    fn tc_048_the_member_scan_reaches_a_deep_application_on_a_small_stack() {
        const LEVELS: usize = 200_000;
        let outcome = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let mut body = Map::new();
                body.insert("term".to_owned(), json!("frame"));
                body.insert(
                    "creates".to_owned(),
                    nested_in_arrays(application(), LEVELS),
                );
                let body = Value::Object(body);
                let found = scan_members(&body, &Trail::Base(&[]));
                let mut clean = Map::new();
                clean.insert("term".to_owned(), json!("frame"));
                clean.insert("creates".to_owned(), nested_in_arrays(json!(1), LEVELS));
                let clean = Value::Object(clean);
                let worked = scan_members(&clean, &Trail::Base(&[]));
                quire_canonical::drop_value(body);
                quire_canonical::drop_value(clean);
                (found, worked)
            })
            .expect("spawns")
            .join()
            .expect("the scan ran to completion");
        let (found, worked) = outcome;
        let crate::checked_package::common::ValidationFailure::Refused(refusal) =
            found.expect_err("the application is found")
        else {
            panic!("a refusal");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
        let expected = format!("/creates{}", "/0".repeat(LEVELS));
        assert_eq!(refusal.loci[0].path.clone(), JsonPointer::parse(&expected));
        // The frame object, its `creates` array levels and the scalar at the
        // bottom, and the `term` member: LEVELS + 3 values.
        assert_eq!(worked, Ok(u64::try_from(LEVELS + 3).expect("count")));
    }

    /// A term's stratum is the place it stands in: a binding stands at no body
    /// root, a Group's member is no aggregate, and a Tuple's member is no binding.
    #[trace("TC-048", "FR-038-AC-114")]
    #[test]
    fn tc_048_a_term_is_checked_against_the_place_it_stands_in() {
        let leaf = json!({"term": "reference", "target": {}});
        let binding = |value: Value| json!({"term": "binding", "name": "n", "value": value});
        let aggregate = |members: Vec<Value>| json!({"term": "aggregate", "members": members});
        let refused_at = |term: Value, place: Place| {
            check_term(&term, &Trail::Base(&[]), place, false)
                .err()
                .map(|failure| match failure {
                    crate::checked_package::common::ValidationFailure::Refused(refusal) => refusal
                        .loci[0]
                        .path
                        .clone()
                        .map_or_else(|| "<none>".to_owned(), |path| path.as_str().to_owned()),
                    crate::checked_package::common::ValidationFailure::Incomplete(_) => {
                        "incomplete".to_owned()
                    }
                })
        };
        for (term, place, expected) in [
            (leaf.clone(), Place::BodyRoot, None),
            (binding(leaf.clone()), Place::BodyRoot, Some("")),
            (binding(leaf.clone()), Place::Member, None),
            (
                binding(binding(leaf.clone())),
                Place::Member,
                Some("/value"),
            ),
            (
                aggregate(vec![aggregate(vec![leaf.clone()])]),
                Place::Member,
                Some("/members/0"),
            ),
            (
                aggregate(vec![aggregate(vec![leaf.clone()])]),
                Place::BodyRoot,
                None,
            ),
            (
                binding(aggregate(vec![binding(leaf.clone())])),
                Place::Member,
                None,
            ),
            (
                binding(aggregate(vec![
                    aggregate(vec![leaf.clone()]),
                    binding(leaf.clone()),
                ])),
                Place::Member,
                Some("/value/members/1"),
            ),
            (
                binding(aggregate(vec![
                    binding(leaf.clone()),
                    aggregate(vec![leaf.clone()]),
                ])),
                Place::Member,
                Some("/value/members/1"),
            ),
        ] {
            assert_eq!(
                refused_at(term.clone(), place),
                expected.map(str::to_owned),
                "{term}"
            );
        }
    }
}
