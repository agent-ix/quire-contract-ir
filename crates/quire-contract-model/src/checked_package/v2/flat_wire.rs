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
//! | Body | a Leaf; an `application` whose `arguments` are each a Member; an `aggregate` whose members are each a Member; a frame body; an abstraction relation body |
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

use super::temporal::interval_bound_outside_pattern;
use super::{body_grammar, BodyGrammar, CheckedNodeKind, CheckedNodeTag};
use super::{ApplicationOperator, BodyTerm, CheckedPackageWireV2};
use crate::checked_package::common::{
    application_operator, body_term, Step, Trail, ValidationFailure,
};
use crate::checked_package::shared::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
use crate::checked_package::terms::{subterms, At, Cursor};
use quire_walk::{walk, Children, Walk};
use serde_json::Value;
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

/// Checks every node body and every `details` term of `wire` against the flat
/// grammar, nodes first, in document order, then the diagnostics' `details`.
/// A node whose family or form does not decode is left to the graph stage's
/// own refusal of it.
pub(super) fn check(wire: &CheckedPackageWireV2) -> Result<(), ValidationFailure> {
    for (position, node) in wire.semantic_graph.nodes.iter().enumerate() {
        let Some(kind) = CheckedNodeTag::from_wire(&node.node_tag)
            .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
        else {
            continue;
        };
        let steps = [
            Step::Key("semantic_graph"),
            Step::Key("nodes"),
            Step::Index(position),
            Step::Key("body"),
        ];
        let at = Trail::Base(&steps);
        let checked = match body_grammar(kind) {
            // A frame body and an abstraction relation body are closed shapes
            // of their own, and no application stands in either; an
            // application at the root of an abstraction relation body is a
            // term of the closed grammar, placed by the abstraction step.
            BodyGrammar::Frame => scan_members(&node.body, &at).map(|_| ()),
            BodyGrammar::Term => check_term(&node.body, &at, Place::BodyRoot, false),
            BodyGrammar::AbstractionRelation => {
                if body_term(&node.body) == Some(BodyTerm::Application) {
                    check_term(&node.body, &at, Place::BodyRoot, false)
                } else {
                    scan_members(&node.body, &at).map(|_| ())
                }
            }
        };
        checked.map_err(|failure| super::with_node_locus(failure, &node.node_id))?;
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
    check_interval_bounds(wire)
}

/// A `temporal_interval` bound outside its form's pattern (FR-038 "The timed
/// interval form" stage 1; merged QSpec FR-370: "during strict wire validation,
/// before any step"), refused `invalid_package`/`invalid-value` at the bound,
/// first in member order and in node position order, after the nested-application
/// refusals above (the spec is silent on the order of the two wire checks, an IR
/// reading). One flat loop over the nodes: no recursion.
fn check_interval_bounds(wire: &CheckedPackageWireV2) -> Result<(), ValidationFailure> {
    for (position, node) in wire.semantic_graph.nodes.iter().enumerate() {
        let Some(kind) = CheckedNodeTag::from_wire(&node.node_tag)
            .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
        else {
            continue;
        };
        if matches!(body_grammar(kind), BodyGrammar::Frame)
            || body_term(&node.body) != Some(BodyTerm::Application)
        {
            continue;
        }
        if let Some(bound) = node
            .body
            .get("operation")
            .and_then(interval_bound_outside_pattern)
        {
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
        let below = match body_term(value) {
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
        children.extend(subterms(value).map(|subterm| self.cursor.subterm(subterm, below)));
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
    let operator_ineligible = match application_operator(application) {
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
    use super::{check_term, scan_members, Place};
    use crate::checked_package::common::Trail;
    use crate::checked_package::shared::{CheckedPackageRefusalCode, JsonPointer};
    use serde_json::{json, Map, Value};

    /// `value` nested in `levels` arrays, each moved into the next: `json!`
    /// would copy the value below it, recursively.
    fn nested_in_arrays(value: Value, levels: usize) -> Value {
        (0..levels).fold(value, |inner, _| Value::Array(vec![inner]))
    }

    fn application() -> Value {
        json!({"term": "application", "operator": "call", "arguments": []})
    }

    /// The scan of a frame or abstraction relation body reaches an application
    /// at any depth without native recursion: one 200000 arrays deep is found,
    /// at a pointer of 200000 steps, on a thread whose stack is 256 KiB; the same
    /// body with none is charged one unit per value.
    ///
    /// Tracing: TC-048, FR-038-AC-117
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
        assert_eq!(refusal.path, JsonPointer::parse(&expected));
        // The frame object, its `creates` array levels and the scalar at the
        // bottom, and the `term` member: LEVELS + 3 values.
        assert_eq!(worked, Ok(u64::try_from(LEVELS + 3).expect("count")));
    }

    /// A term's stratum is the place it stands in: a binding stands at no body
    /// root, a Group's member is no aggregate, and a Tuple's member is no binding.
    ///
    /// Tracing: TC-048, FR-038-AC-114
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
                        .path
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
