//! The temporal step of the reader order (QSpec FR-370 "Reader order"): after
//! the state step and before any operation refusal, the reader checks
//!
//! 1. the placement of every application of the three temporal classes that
//!    stand only at one node form's body root (`temporal`, `temporal_formula`
//!    and `temporal_fairness`; `case` is placed by the operation step), and of every `reference` to a
//!    `temporal`/`formula` or `temporal`/`fairness` node, over the whole graph in
//!    ascending structural node-id order;
//! 2. forbidden references in diagnostic details, in entry and detail order;
//! 3. then each `temporal`/`temporal_clause` node in ascending structural node-id
//!    order, each taken through its checks before the next clause is read:
//!    the profile identity (`unknown_profile`), the `over` reference, each
//!    fairness member's resolution, the profile fit of every interval and of the
//!    fairness argument, the interval bounds, then reachable aggregate atoms
//!    in ascending formula identity order,
//!
//! and reports the first defect. Operand family and count and `result_type`
//! stay the operation step's. The step reads only a well-formed member: a
//! member the operation step refuses as `operation-member-mismatch` is skipped
//! here.
//!
//! The module also owns the `temporal_interval` and `fairness` member shapes
//! ([`member_is_well_formed`]), the unbounded-integer comparison of integer
//! interval bounds ([`IntegerString`]) and the exact rational comparison of the
//! timed form's bounds ([`Rational`]), which the operation step and the term
//! walk read, and the timed form's lowest-terms graph check
//! ([`validate_timed_bounds_reduced`]), which runs before the frame step.

use super::frame::StepGraph;
use super::graph_storage::{GraphIndex, Stamps};
use super::identity::is_identifier;
use super::model_members::{MemberKind, ModelOwners, ModelRefusal, Resolved};
use super::natural::{compare_rationals, coprime};
use super::operation_catalog::OperationCatalog;
use super::operations::{resolve_family, OperationWire};
use super::structural::reference_target;
use super::{
    member_pointer, ApplicationOperator, CheckedDiagnosticV2, CheckedNodeKind,
    CheckedPackageLockV2, CheckedSelectionRole, CheckedTerm, CheckedTermView, ClockConfiguration,
    FairnessGranularity, FairnessKind, HybridForm, IntervalEnd, IntervalFit, LawRole, ModelForm,
    OperationMemberKind, RawSemanticNodeV2 as CheckedSemanticNodeV2, TemporalBinding, TemporalForm,
    TemporalProfile, TermNode, TermView, ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedNodeId, CheckedNodePathSegment, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, JsonPointer,
};
use quire_walk::{Children, Walk};
use serde::Deserialize;
use serde_json::Value;
use std::cmp::Ordering;
use std::ops::ControlFlow;

/// The three temporal operator classes whose applications stand only at the
/// body root of one node form, and that form (QSpec FR-370 "Placement and
/// profile fit"). A `case` application is placed by the operation step (merged
/// FR-440), not by this table.
const PLACED: [(ApplicationOperator, CheckedNodeKind); 3] = [
    (
        ApplicationOperator::Temporal,
        CheckedNodeKind::Temporal(TemporalForm::TemporalClause),
    ),
    (
        ApplicationOperator::TemporalFormula,
        CheckedNodeKind::Temporal(TemporalForm::Formula),
    ),
    (
        ApplicationOperator::TemporalFairness,
        CheckedNodeKind::Temporal(TemporalForm::Fairness),
    ),
];

/// Whether an application of this temporal operator class stands only at the
/// body root of its node form.
fn is_placed_class(operator: ApplicationOperator) -> bool {
    PLACED.iter().any(|(class, _)| *class == operator)
}

/// Whether a diagnostics entry's `details` term holds a `reference` to a node
/// `refused_target` names (a `temporal`/`formula`, `temporal`/`fairness` or
/// `expression`/`case` node), which is refused at the `details` term. A
/// reference to a union or union value node is an ordinary one (merged QSpec
/// FR-370 and FR-440: a `details` term may reference a `composite_type`/`union`
/// or `value`/`union_value` node). An application in a `details` term is
/// outside the body grammar and is refused ahead of this check (`flat_wire`).
pub(in crate::checked_package) fn references_refused_node(
    term: TermNode<'_>,
    refused_target: impl Fn(&CheckedNodeId) -> bool,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let mut pending: Vec<TermNode<'_>> = meter.vec(1)?;
    pending.push(term);
    while let Some(term) = pending.pop() {
        if let Some(TermView::Reference { target }) = term.view() {
            // The target ID is charged before it is allocated; the budget is
            // the caller's, or a free one for a meter that owns its limit.
            let id = match meter.budget_and_site() {
                Some((budget, site)) => target
                    .node_id(budget, site)
                    .map_err(|_| super::budgeted_read::denial(budget))?,
                None => target
                    .node_id(
                        &mut super::NativeJoinBudget::unlimited(),
                        super::read_charge::Site::new(super::PackageReadStage::Graph, 0, "details"),
                    )
                    .map_err(|_| {
                        super::budgeted_read::denial(&super::NativeJoinBudget::unlimited())
                    })?,
            };
            if id.valid().is_some_and(|id| refused_target(&id)) {
                return Ok(true);
            }
        }
        let subterms = term.subterms();
        meter.room(&mut pending, subterms.size_hint().0)?;
        for subterm in subterms {
            meter.push(&mut pending, subterm.value)?;
        }
    }
    Ok(false)
}

/// An integer string: the schema's `IntegerString`, `^(0|-?[1-9][0-9]*)$`.
/// Ordered as the integer it spells, of unbounded size: by sign, then digit
/// count, then digits, never as text and never through a fixed-width integer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::checked_package) struct IntegerString {
    negative: bool,
    zero: bool,
    /// The magnitude's digits, with no leading zero unless the magnitude is `0`.
    digits: Box<str>,
}

impl IntegerString {
    /// The integer `text` spells, `None` when it is not an `IntegerString`.
    #[ix_trace_rs::string_edge]
    pub(in crate::checked_package) fn parse(text: &str) -> Option<Self> {
        let (negative, digits) = match text.strip_prefix('-') {
            Some(digits) => (true, digits),
            None => (false, text),
        };
        let spelled = match digits.as_bytes() {
            [b'0'] => !negative,
            [first, rest @ ..] => {
                (b'1'..=b'9').contains(first) && rest.iter().all(u8::is_ascii_digit)
            }
            [] => false,
        };
        spelled.then(|| Self {
            negative,
            zero: digits == "0",
            digits: digits.into(),
        })
    }

    /// The magnitude's decimal digits.
    fn digits(&self) -> &str {
        &self.digits
    }

    /// Whether this is `0`.
    fn is_zero(&self) -> bool {
        self.zero
    }
}

/// Compare borrowed, canonical signed-decimal parts without numeric conversion.
/// Callers retain ownership and account for examining digits in their own meter.
#[ix_trace_rs::string_edge]
pub(super) fn compare_integer_parts(
    left_negative: bool,
    left_digits: &str,
    right_negative: bool,
    right_digits: &str,
) -> Ordering {
    match (left_negative, right_negative) {
        (false, true) => Ordering::Greater,
        (true, false) => Ordering::Less,
        _ => {
            let magnitude = left_digits
                .len()
                .cmp(&right_digits.len())
                .then_with(|| left_digits.cmp(right_digits));
            if left_negative {
                magnitude.reverse()
            } else {
                magnitude
            }
        }
    }
}

impl Ord for IntegerString {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_integer_parts(self.negative, &self.digits, other.negative, &other.digits)
    }
}

impl PartialOrd for IntegerString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The exact rational `{numerator, denominator}` of the schema's
/// `NonNegativeRational`: `numerator` matching `^(0|[1-9][0-9]*)$` and
/// `denominator` matching `^[1-9][0-9]*$`. Lowest terms is a later, graph check
/// ([`validate_timed_bounds_reduced`]), not part of the pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::checked_package) struct Rational {
    numerator: IntegerString,
    denominator: IntegerString,
}

impl Rational {
    /// The rational `value` spells, `None` when it is not the closed object of
    /// those two members or either member is outside its pattern.
    fn read(value: &Value) -> Option<Self> {
        let object = value.as_object()?;
        if object.len() != 2 {
            return None;
        }
        let numerator = IntegerString::parse(object.get("numerator")?.as_str()?)?;
        let denominator = IntegerString::parse(object.get("denominator")?.as_str()?)?;
        (!numerator.negative && !denominator.negative && !denominator.is_zero()).then_some(Self {
            numerator,
            denominator,
        })
    }

    /// The exact comparison of two rationals, charged to the work limit at `at`.
    fn compare(
        &self,
        other: &Self,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Ordering, ValidationFailure> {
        compare_rationals(
            (self.numerator.digits(), self.denominator.digits()),
            (other.numerator.digits(), other.denominator.digits()),
            meter,
            at,
        )
    }
}

/// A well-formed `temporal_interval` member's interval (QSpec FR-370).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::checked_package) enum Interval {
    /// `null`: an unbounded operator.
    Unbounded,
    /// `{lower, upper: null}`: a lower-bounded operator.
    LowerBounded,
    /// `{lower, upper}`: an interval operator.
    Closed {
        lower: IntegerString,
        upper: IntegerString,
    },
    /// `{lower, upper, lower_end, upper_end}`: the timed form.
    Timed {
        lower: Rational,
        upper: Option<Rational>,
        lower_end: IntervalEnd,
        upper_end: IntervalEnd,
    },
}

/// The form of a non-`null` `interval` object, told apart by its member set
/// alone (FR-038 "The timed interval form").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IntervalForm {
    /// Exactly `lower` and `upper`.
    Integer,
    /// Exactly `lower`, `upper`, `lower_end` and `upper_end`.
    Timed,
}

impl IntervalForm {
    /// The form `interval`'s member set names, `None` for any other member set.
    fn of(interval: &serde_json::Map<String, Value>) -> Option<Self> {
        let has = |names: &[&str]| {
            interval.len() == names.len() && names.iter().all(|name| interval.contains_key(*name))
        };
        if has(&["lower", "upper"]) {
            Some(Self::Integer)
        } else if has(&["lower", "upper", "lower_end", "upper_end"]) {
            Some(Self::Timed)
        } else {
            None
        }
    }
}

/// The interval of a `{kind: temporal_interval, interval}` member; `None` for
/// any other shape: a `null` member, another kind, a third member, an
/// `interval` that is neither `null` nor of one of the two forms' member sets,
/// a bound outside its form's pattern, or an end that is neither `closed` nor
/// `open`.
fn read_interval_member(member: &Value) -> Option<Interval> {
    let object = member.as_object()?;
    let kind = object.get("kind")?.as_str()?;
    if object.len() != 2
        || OperationMemberKind::from_wire(kind) != Some(OperationMemberKind::TemporalInterval)
    {
        return None;
    }
    let interval = match object.get("interval")? {
        Value::Null => return Some(Interval::Unbounded),
        Value::Object(interval) => interval,
        _ => return None,
    };
    match IntervalForm::of(interval)? {
        IntervalForm::Integer => {
            let lower = IntegerString::parse(interval.get("lower")?.as_str()?)?;
            match interval.get("upper")? {
                Value::Null => Some(Interval::LowerBounded),
                Value::String(upper) => Some(Interval::Closed {
                    lower,
                    upper: IntegerString::parse(upper)?,
                }),
                _ => None,
            }
        }
        IntervalForm::Timed => Some(Interval::Timed {
            lower: Rational::read(interval.get("lower")?)?,
            upper: match interval.get("upper")? {
                Value::Null => None,
                value => Some(Rational::read(value)?),
            },
            lower_end: IntervalEnd::from_wire(interval.get("lower_end")?.as_str()?)?,
            upper_end: IntervalEnd::from_wire(interval.get("upper_end")?.as_str()?)?,
        }),
    }
}

/// The `interval` object of a `temporal_interval` member of an application's
/// `operation`; `None` for any other member or a `null` interval.
fn interval_object(operation: &Value) -> Option<&serde_json::Map<String, Value>> {
    let member = operation.get("member")?;
    let kind = member.get("kind")?.as_str()?;
    if OperationMemberKind::from_wire(kind) != Some(OperationMemberKind::TemporalInterval) {
        return None;
    }
    member.get("interval")?.as_object()
}

/// The name of the first of `lower` and `upper`, in member order, of a
/// `temporal_interval` member of an application's `operation` that lies outside
/// its form's pattern: for the timed form (four members) the closed rational
/// object, so `null` in place of `upper`, a JSON integer or a string, a negative
/// or malformed `numerator` and a zero, negative or malformed `denominator`
/// all fail; for every other member set, the integer form's included, the
/// schema's non-negative integer-string pattern `^(0|[1-9][0-9]*)$`: a negative
/// bound, a malformed one (`"1.5"`, `"01"`, `"+1"`, `""`, `"3x"`) and a bound
/// that is no string. A timed `upper` of `null` is admitted only for a formula
/// reached by a hybrid claim; `flat_wire::check_interval_bounds` refuses it
/// `invalid-value` for every other formula before identity validation.
#[derive(Clone, Copy, Eq, PartialEq)]
enum IntervalBound {
    Lower,
    Upper,
}

impl IntervalBound {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Self {
        if text == "upper" {
            Self::Upper
        } else {
            Self::Lower
        }
    }
}

pub(super) fn interval_bound_outside_pattern(operation: &Value) -> Option<&'static str> {
    let interval = interval_object(operation)?;
    let timed = IntervalForm::of(interval) == Some(IntervalForm::Timed);
    ["lower", "upper"].into_iter().find(|bound| {
        let Some(value) = interval.get(*bound) else {
            // A missing bound is a defect of the member's shape.
            return false;
        };
        if timed {
            if IntervalBound::from_wire(bound) == IntervalBound::Upper && value.is_null() {
                return false;
            }
            return Rational::read(value).is_none();
        }
        if IntervalBound::from_wire(bound) == IntervalBound::Upper && value.is_null() {
            return false;
        }
        !value
            .as_str()
            .and_then(IntegerString::parse)
            .is_some_and(|bound| !bound.negative)
    })
}

/// Whether a timed interval spells the hybrid-only unbounded upper end.
pub(super) fn timed_upper_is_null(operation: &Value) -> bool {
    interval_object(operation).is_some_and(|interval| {
        IntervalForm::of(interval) == Some(IntervalForm::Timed)
            && interval.get("upper").is_some_and(Value::is_null)
    })
}

/// FR-038 "The timed interval form" stage 2, a graph check: a bound of a
/// timed-form interval that matches the rational pattern and is not in lowest
/// terms refuses `invalid_semantic_graph` at that bound, the first in ascending
/// structural `node_id` order and then member order. The GCD work is charged to the
/// work limit at the node's body.
pub(super) fn validate_timed_bounds_reduced(
    nodes: &[CheckedSemanticNodeV2],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for &position in index.values() {
        let Some(node) = nodes.get(position) else {
            continue;
        };
        let Some(CheckedTermView::Application { operation, .. }) = CheckedTerm::view(&node.body)
        else {
            continue;
        };
        let Some(interval) = interval_object(operation) else {
            continue;
        };
        if IntervalForm::of(interval) != Some(IntervalForm::Timed) {
            continue;
        }
        for bound in ["lower", "upper"] {
            let Some(rational) = interval.get(bound).and_then(Rational::read) else {
                continue;
            };
            let at_body = || node_pointer(position).key("body");
            if !coprime(
                rational.numerator.digits(),
                rational.denominator.digits(),
                meter,
                &at_body,
            )? {
                return Err(ValidationFailure::refused_at(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    at_body()
                        .key("operation")
                        .key("member")
                        .key("interval")
                        .key(bound),
                    None,
                    node.node_id.clone(),
                ));
            }
        }
    }
    Ok(())
}

/// A `{kind: fairness, fairness_kind, granularity, target}`
/// member (QSpec FR-370). The two vocabulary members are decoded for their
/// closed words and carried no further: nothing here evaluates fairness.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FairnessMember {
    kind: OperationMemberKind,
    #[serde(rename = "fairness_kind")]
    _fairness_kind: FairnessKind,
    #[serde(rename = "granularity")]
    _granularity: FairnessGranularity,
    target: super::CheckedFairnessTarget,
}

/// The fairness member `member` is, `None` for any other shape.
fn read_fairness_member(member: &Value) -> Option<FairnessMember> {
    let member = FairnessMember::deserialize(member).ok()?;
    let named = match &member.target {
        super::CheckedFairnessTarget::Operation { name, .. }
        | super::CheckedFairnessTarget::HybridTransition { name, .. } => is_identifier(name),
        super::CheckedFairnessTarget::ParallelBranch { branch, .. } => is_identifier(branch),
        _ => true,
    };
    (member.kind == OperationMemberKind::Fairness && named).then_some(member)
}

/// Read the canonical fairness target during admission, before public views exist.
pub(super) fn fairness_target(member: &Value) -> Option<super::CheckedFairnessTarget> {
    Some(read_fairness_member(member)?.target)
}

/// Closed type-argument member; the declaration is validated as a node id.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TypeArgumentMember {
    kind: OperationMemberKind,
    #[serde(rename = "declaration")]
    _declaration: CheckedNodeId,
}

/// Whether `member` has the closed shape its catalogued member `kind`
/// requires; deeper member joins are checked elsewhere.
pub(in crate::checked_package) fn member_is_well_formed(
    kind: OperationMemberKind,
    member: &Value,
) -> bool {
    match kind {
        OperationMemberKind::TemporalInterval => read_interval_member(member).is_some(),
        OperationMemberKind::Fairness => read_fairness_member(member).is_some(),
        OperationMemberKind::TypeArgument => TypeArgumentMember::deserialize(member)
            .is_ok_and(|member| member.kind == OperationMemberKind::TypeArgument),
        OperationMemberKind::Field => {
            member.get("name").and_then(Value::as_str).is_some()
                && member
                    .get("declaration")
                    .is_some_and(|value| CheckedNodeId::from_value(value).is_some())
        }
        OperationMemberKind::Position
        | OperationMemberKind::Element
        | OperationMemberKind::RelationshipEnd
        | OperationMemberKind::Operation
        | OperationMemberKind::ProfileOperator
        | OperationMemberKind::StateClause => true,
    }
}

/// The clause context shared by the temporal checks.
pub(super) struct TemporalContext<'a> {
    pub(super) catalog: &'a OperationCatalog,
    pub(super) lock: &'a CheckedPackageLockV2,
    pub(super) diagnostics: &'a [CheckedDiagnosticV2],
    pub(super) bindings: Option<&'a [TemporalBinding]>,
}

/// The temporal step. See the module documentation for the order.
pub(super) fn validate_temporal(
    terms: &[Option<CheckedTerm>],
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    owners: &ModelOwners<'_>,
    context: TemporalContext<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StepGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    for &position in index.values() {
        if let Some(failure) = placement_defect(position, &graph, terms, meter)? {
            return Err(failure);
        }
    }
    validate_diagnostic_references(kinds, index, context.diagnostics, meter)?;
    meter.at_node(0, "temporal");
    let clock_units = aggregate_clock_units(context.bindings, meter)?;
    let mut covered = meter.filled(nodes.len(), false)?;
    let mut stamps = Stamps::new(nodes.len(), meter)?;
    for &position in index.values() {
        if graph.kinds.get(position)
            == Some(&CheckedNodeKind::Temporal(TemporalForm::TemporalClause))
        {
            check_clause(
                context.catalog,
                position,
                &graph,
                terms,
                context.lock,
                clock_units
                    .binary_search_by(|(declaration, _)| {
                        (*declaration).cmp(&graph.nodes[position].node_id)
                    })
                    .ok()
                    .and_then(|at| clock_units[at].1),
                meter,
                &mut covered,
                &mut stamps,
            )?;
        }
    }
    // A formula node no clause reaches is no clause's to fit, but its
    // interval is an interval all the same.
    for &position in index.values() {
        if graph.kinds.get(position) == Some(&CheckedNodeKind::Temporal(TemporalForm::Formula))
            && !covered.get(position).copied().unwrap_or(false)
        {
            if let Some(interval) = interval_of(context.catalog, position, &graph) {
                check_bounds(position, &interval, &graph, meter)?;
            }
        }
    }
    Ok(())
}

/// Only forbidden reference placement belongs to the temporal step; other
/// diagnostic validation remains at the diagnostics stage.
fn validate_diagnostic_references(
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    diagnostics: &[CheckedDiagnosticV2],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for (entry_index, entry) in diagnostics.iter().enumerate() {
        for (detail_index, detail) in entry.details.iter().enumerate() {
            meter.at_row("diagnostic_details", entry_index);
            if references_refused_node(
                TermNode::of(detail),
                |target| {
                    index
                        .get(target)
                        .and_then(|&position| kinds.get(position))
                        .is_some_and(|kind| {
                            matches!(
                                kind,
                                CheckedNodeKind::Temporal(
                                    TemporalForm::Formula | TemporalForm::Fairness
                                ) | CheckedNodeKind::Expression(super::ExpressionForm::Case)
                            )
                        })
                },
                meter,
            )? {
                return Err(ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::IllTyped,
                    member_pointer(&["diagnostics", "entries"])
                        .index(entry_index)
                        .key("details")
                        .index(detail_index),
                    CheckedPackageRefusalCause::OperatorIneligible,
                ));
            }
        }
    }
    Ok(())
}

/// `ill_typed`/`operator-ineligible` at the node.
fn ineligible(graph: &StepGraph<'_, '_>, position: usize) -> ValidationFailure {
    ValidationFailure::refused_at(
        CheckedPackageRefusalCode::IllTyped,
        node_pointer(position),
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        graph.nodes[position].node_id.clone(),
    )
}

/// What a `reference` at one place of a node's body may name.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Slot {
    /// Neither a formula nor a fairness node.
    Other,
    /// A `temporal`/`formula` node: a clause's formula argument or an operand
    /// of a `temporal_formula` application.
    Formula,
    /// The members of a clause's fairness argument.
    FairnessList,
    /// A `temporal`/`fairness` node: a member of a clause's fairness argument.
    Fairness,
}

/// The slot of argument `index` of the application at the root of a node of
/// this kind.
fn argument_slot(kind: CheckedNodeKind, index: usize) -> Slot {
    match (kind, index) {
        (CheckedNodeKind::Temporal(TemporalForm::TemporalClause), 4) => Slot::FairnessList,
        (CheckedNodeKind::Temporal(TemporalForm::TemporalClause), 5)
        | (CheckedNodeKind::Temporal(TemporalForm::Formula), _) => Slot::Formula,
        _ => Slot::Other,
    }
}

fn application_arguments(term: &Value) -> Option<&[Value]> {
    match CheckedTerm::view(term)? {
        CheckedTermView::Application { arguments, .. } => Some(arguments),
        _ => None,
    }
}

fn aggregate_members(term: &Value) -> Option<&[Value]> {
    match CheckedTerm::view(term)? {
        CheckedTermView::Aggregate { members } => Some(members),
        _ => None,
    }
}

/// The placement defect of one node: its body root against its form, and any
/// `reference` to a formula or fairness node from a place that may not name one.
/// An application nested in the body was refused ahead of every identity check.
fn placement_defect<'a>(
    position: usize,
    graph: &StepGraph<'a, '_>,
    terms: &'a [Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let Some(kind) = graph.kinds.get(position).copied() else {
        return Ok(None);
    };
    let root = terms.get(position).and_then(Option::as_ref);
    let root_class = match root {
        Some(CheckedTerm::Application { operator, .. }) => Some(*operator),
        _ => None,
    };
    let required = PLACED
        .iter()
        .find(|(_, form)| *form == kind)
        .map(|(class, _)| *class);
    match (required, root_class) {
        (Some(required), Some(root)) if root == required => {}
        (Some(_), _) => return Ok(Some(ineligible(graph, position))),
        (None, Some(root)) if is_placed_class(root) => {
            return Ok(Some(ineligible(graph, position)))
        }
        (None, _) => {}
    }
    // The closed plan step owns its formula references, including malformed
    // binding placement, so its binding-specific refusal keeps precedence.
    if kind == CheckedNodeKind::Temporal(TemporalForm::MonitorPlan) {
        return Ok(None);
    }
    let Some(root) = root else {
        return Ok(None);
    };
    meter.at_node(
        position,
        super::graph_storage::node_form(&graph.nodes[position].node_tag),
    );
    Ok(
        match meter.walk(
            &mut Placement {
                graph,
                position,
                kind,
            },
            (root, true, Slot::Other),
        )? {
            ControlFlow::Continue(()) => None,
            ControlFlow::Break(failure) => Some(failure),
        },
    )
}

/// The walk of [`placement_defect`] over a node's body: each term with whether
/// it is the body root and the slot it stands in.
struct Placement<'g, 'a, 'm> {
    graph: &'g StepGraph<'a, 'm>,
    position: usize,
    kind: CheckedNodeKind,
}

impl<'a> Walk for Placement<'_, 'a, '_> {
    type Node = (&'a CheckedTerm, bool, Slot);
    type Frame = ();
    type Stop = ValidationFailure;

    fn enter(
        &mut self,
        (term, is_root, slot): (&'a CheckedTerm, bool, Slot),
        children: &mut Children<'_, (&'a CheckedTerm, bool, Slot)>,
    ) -> ControlFlow<ValidationFailure> {
        match term {
            // Only the body root is an application: a nested one is refused
            // `malformed_wire` ahead of this step (`flat_wire`).
            CheckedTerm::Application { arguments, .. } => {
                children.extend(arguments.iter().enumerate().map(|(at, argument)| {
                    let slot = if is_root {
                        argument_slot(self.kind, at)
                    } else {
                        Slot::Other
                    };
                    (argument, false, slot)
                }));
            }
            CheckedTerm::Aggregate { members } => {
                let member = if slot == Slot::FairnessList {
                    Slot::Fairness
                } else {
                    Slot::Other
                };
                children.extend(
                    members
                        .iter()
                        .map(|member_term| (member_term, false, member)),
                );
            }
            CheckedTerm::Binding { value, .. } => {
                children.extend(std::iter::once((value.as_ref(), false, Slot::Other)));
            }
            CheckedTerm::Reference { target } => {
                let target = self.graph.node(target);
                let misplaced = match target {
                    Some((_, CheckedNodeKind::Temporal(TemporalForm::Formula))) => {
                        slot != Slot::Formula
                    }
                    Some((_, CheckedNodeKind::Temporal(TemporalForm::Fairness))) => {
                        slot != Slot::Fairness
                    }
                    _ => false,
                };
                if misplaced {
                    return ControlFlow::Break(ineligible(self.graph, self.position));
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }

    fn exit(&mut self, (): ()) -> ControlFlow<ValidationFailure> {
        ControlFlow::Continue(())
    }
}

/// The wire `operation` of the application at the root of `node`'s body.
fn operation_of(node: &CheckedSemanticNodeV2) -> Option<OperationWire> {
    match CheckedTerm::view(&node.body)? {
        CheckedTermView::Application { operation, .. } => {
            OperationWire::deserialize(operation).ok()
        }
        _ => None,
    }
}

/// The interval of the formula node at `position`: the member of a
/// catalogued interval operator, when it is well formed.
fn interval_of(
    catalog: &OperationCatalog,
    position: usize,
    graph: &StepGraph<'_, '_>,
) -> Option<Interval> {
    let operation = operation_of(graph.nodes.get(position)?)?;
    let entry = catalog.entry(&operation.identity)?;
    if entry.operator != ApplicationOperator::TemporalFormula
        || entry.member != Some(OperationMemberKind::TemporalInterval)
    {
        return None;
    }
    read_interval_member(operation.member.as_ref()?)
}

/// `invalid_package`/`invalid-value` at the application when a closed integer
/// interval has `lower > upper`, or a timed one has `lower > upper` or an open
/// end with `lower = upper` (merged QSpec FR-370). The timed bounds compare
/// exactly by cross-multiplication, charged to the work limit at the body.
fn check_bounds(
    position: usize,
    interval: &Interval,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let at_body = || node_pointer(position).key("body");
    let defective = match interval {
        Interval::Unbounded | Interval::LowerBounded => false,
        Interval::Closed { lower, upper } => lower > upper,
        Interval::Timed {
            lower,
            upper,
            lower_end,
            upper_end,
        } => match upper {
            Some(upper) => match lower.compare(upper, meter, &at_body)? {
                Ordering::Greater => true,
                Ordering::Equal => {
                    *lower_end == IntervalEnd::Open || *upper_end == IntervalEnd::Open
                }
                Ordering::Less => false,
            },
            None => *upper_end != IntervalEnd::Open,
        },
    };
    if defective {
        Err(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidPackage,
            at_body(),
            Some(CheckedPackageRefusalCause::InvalidValue),
            graph.nodes[position].node_id.clone(),
        ))
    } else {
        Ok(())
    }
}

/// Whether the package's own lock selects `definition` under a role other than
/// `temporal_profile`: in `profile_selections`, or as the edition.
fn selected_under_another_role(
    lock: &CheckedPackageLockV2,
    definition: &CheckedArtifactRef,
) -> bool {
    lock.edition.definition == *definition
        || lock.profile_selections.iter().any(|selection| {
            selection.role != CheckedSelectionRole::TemporalProfile
                && selection.definition == *definition
        })
}

/// One clause's checks, in order: profile identity, `over`, fairness
/// resolution, profile fit, interval bounds, aggregate arguments. A part of the clause the
/// operation step refuses (an unreadable `operation`, a missing law, a
/// missing argument) is skipped here.
#[ix_trace_rs::string_edge]
fn check_clause(
    catalog: &OperationCatalog,
    position: usize,
    graph: &StepGraph<'_, '_>,
    terms: &[Option<CheckedTerm>],
    lock: &CheckedPackageLockV2,
    clock_unit: Option<&str>,
    meter: &mut WorkMeter,
    covered: &mut [bool],
    stamps: &mut Stamps,
) -> Result<(), ValidationFailure> {
    let node = &graph.nodes[position];
    meter.at_node(position, super::graph_storage::node_form(&node.node_tag));
    let Some(operation) = operation_of(node) else {
        return Ok(());
    };
    if catalog
        .entry(&operation.identity)
        .map(|entry| entry.operator)
        != Some(ApplicationOperator::Temporal)
    {
        return Ok(());
    }
    let at_node = node_pointer(position);
    let at_body = at_node.clone().key("body");
    let arguments: &[Value] = application_arguments(&node.body).unwrap_or(&[]);
    let hybrid_claim = arguments
        .first()
        .and_then(reference_target)
        .is_some_and(|over| {
            graph.node(&over).is_some_and(|(parameter, kind)| {
                kind == CheckedNodeKind::Value(ValueForm::Parameter)
                    && graph
                        .node(&parameter.semantic_type)
                        .is_some_and(|(_, kind)| kind == CheckedNodeKind::Hybrid(HybridForm::Model))
            })
        });

    // The profile identity, first: it decides the interval fit. Only a clause
    // whose `laws` is exactly one `temporal_profile` law has a profile to
    // check; any other law list is the operation step's defect (FR-038-AC-108).
    let mut profile = None;
    let sole_profile_law = match operation.laws.as_slice() {
        [law] if law.role_class() == Some(LawRole::TemporalProfile) => Some(law),
        _ => None,
    };
    if let Some(law) = sole_profile_law {
        match TemporalProfile::from_wire(&law.definition.identity) {
            Some(known) => profile = Some(known),
            None => {
                let cause = if selected_under_another_role(lock, &law.definition) {
                    CheckedPackageRefusalCause::WrongSelectionRole
                } else {
                    CheckedPackageRefusalCause::UnsupportedSelection
                };
                return Err(ValidationFailure::refused_at(
                    CheckedPackageRefusalCode::UnknownProfile,
                    at_body
                        .clone()
                        .key("operation")
                        .key("laws")
                        .index(0)
                        .key("definition"),
                    Some(cause),
                    node.node_id.clone(),
                ));
            }
        }
    }

    // `over`: a declared `value`/`parameter` dependency, as a frame entry's
    // reference is (FR-040), refused with the path on the argument and, as a
    // frame entry's refusal is, the target's key as locus; a target that is no
    // node has none, so the clause's key stands.
    if let Some(over) = arguments.first().and_then(reference_target) {
        let refusal = match graph.node(&over) {
            None => Some((ModelRefusal::missing_name(), node.node_id.clone())),
            Some(_) if !node.dependencies.contains(&over) => {
                Some((ModelRefusal::missing_name(), over.clone()))
            }
            Some((_, kind)) if kind != CheckedNodeKind::Value(ValueForm::Parameter) => {
                Some((ModelRefusal::malformed(), over.clone()))
            }
            Some(_) => None,
        };
        if let Some((refusal, locus)) = refusal {
            return Err(ValidationFailure::refused_at(
                refusal.code,
                at_body.clone().key("arguments").index(0),
                Some(refusal.cause),
                locus,
            ));
        }
    }

    // Each fairness member resolves on its declaring node.
    let fairness: &[Value] = arguments.get(4).and_then(aggregate_members).unwrap_or(&[]);
    for reference in fairness {
        let Some((fair, member)) = fairness_member(reference, graph) else {
            continue;
        };
        let super::CheckedFairnessTarget::Operation { declaration, name } = &member.target else {
            if !matches!(
                member.target,
                super::CheckedFairnessTarget::HybridTransition { .. }
            ) {
                super::protocol_body::validate_temporal_fairness(
                    position,
                    fair,
                    &member.target,
                    graph,
                    terms,
                    lock,
                    meter,
                )?;
            }
            if !hybrid_claim
                && matches!(
                    member.target,
                    super::CheckedFairnessTarget::HybridTransition { .. }
                )
            {
                return Err(ValidationFailure::refused_at(
                    CheckedPackageRefusalCode::InvalidModelBinding,
                    node_pointer(fair)
                        .key("body")
                        .key("operation")
                        .key("member")
                        .key("target")
                        .key("declaration"),
                    Some(CheckedPackageRefusalCause::MalformedDeclaration),
                    graph.nodes[fair].node_id.clone(),
                ));
            }
            continue;
        };
        if let Err(failure) = resolve_fairness(declaration, name, graph, meter)? {
            return Err(ValidationFailure::refused_at(
                failure.refusal.code,
                node_pointer(fair)
                    .key("body")
                    .key("operation")
                    .key("member")
                    .key("target")
                    .key(failure.member),
                Some(failure.refusal.cause),
                if failure.locus_is_target {
                    declaration.clone()
                } else {
                    graph.nodes[fair].node_id.clone()
                },
            ));
        }
    }

    // Profile fit of every interval, then of the fairness argument, then the
    // bounds of every interval.
    let FormulaReach { intervals, reached } = formula_intervals(
        catalog,
        arguments.get(5),
        graph,
        meter,
        &at_node,
        covered,
        stamps,
    )?;
    if let Some(profile) = profile {
        let fit = profile.interval_fit();
        for (formula, interval) in &intervals {
            if !interval_fits(fit, interval)
                || matches!(interval, Interval::Timed { upper: None, .. }) && !hybrid_claim
            {
                return Err(ValidationFailure::refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    node_pointer(*formula).key("body"),
                    Some(CheckedPackageRefusalCause::OperationMemberMismatch),
                    graph.nodes[*formula].node_id.clone(),
                ));
            }
        }
        if !fairness.is_empty() && !profile.admits_fairness() {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                at_body,
                Some(CheckedPackageRefusalCause::OperationMemberMismatch),
                node.node_id.clone(),
            ));
        }
    }
    for (formula, interval) in &intervals {
        check_bounds(*formula, interval, graph, meter)?;
    }
    let mut aggregates = reached;
    aggregates.retain(|&formula| {
        operation_of(&graph.nodes[formula])
            .is_some_and(|operation| operation.identity.as_ref() == "quire.op.temporal.aggregate")
    });
    // Equal ids keep position order, as the stable sort did.
    aggregates.sort_unstable_by(|&left, &right| {
        graph.nodes[left]
            .node_id
            .cmp(&graph.nodes[right].node_id)
            .then(left.cmp(&right))
    });
    // The clock unit is matched by `node_id.path` alone: the unit's own node
    // lives in the package that owns it, whatever the clause's package
    // (QSpec FR-370 "Window aggregate atom").
    for formula in aggregates {
        check_aggregate(formula, profile, clock_unit, graph, meter)?;
    }
    Ok(())
}

fn interval_fits(fit: IntervalFit, interval: &Interval) -> bool {
    match (fit, interval) {
        (
            IntervalFit::IntegerForms,
            Interval::Unbounded | Interval::LowerBounded | Interval::Closed { .. },
        )
        | (IntervalFit::NullOrTimed, Interval::Unbounded | Interval::Timed { .. })
        | (IntervalFit::ClosedOnly, Interval::Closed { .. }) => true,
        (IntervalFit::IntegerForms, Interval::Timed { .. })
        | (IntervalFit::NullOrTimed, Interval::LowerBounded | Interval::Closed { .. })
        | (
            IntervalFit::ClosedOnly,
            Interval::Unbounded | Interval::LowerBounded | Interval::Timed { .. },
        ) => false,
    }
}

/// Where each aggregate member is held: one slot per name of the closed set,
/// so the table needs no heap.
#[derive(Default)]
struct Held([Option<usize>; AGGREGATE_NAMES.len()]);

impl Held {
    fn slot(name: &str) -> Option<usize> {
        AGGREGATE_NAMES.iter().position(|known| *known == name)
    }

    fn contains_key(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    fn get(&self, name: &str) -> Option<&usize> {
        self.0.get(Self::slot(name)?)?.as_ref()
    }

    fn insert(&mut self, name: &str, at: usize) {
        if let Some(slot) = Self::slot(name).and_then(|slot| self.0.get_mut(slot)) {
            *slot = Some(at);
        }
    }
}

impl std::ops::Index<&str> for Held {
    type Output = usize;

    fn index(&self, name: &str) -> &usize {
        self.get(name).expect("a held aggregate member")
    }
}

const AGGREGATE_NAMES: [&str; 11] = [
    "kind",
    "q",
    "predicate",
    "value",
    "filter",
    "window_kind",
    "lower",
    "upper",
    "min_count",
    "comparison",
    "threshold",
];

fn aggregate_refusal(
    position: usize,
    at: JsonPointer,
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
    graph: &StepGraph<'_, '_>,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, at, Some(cause), graph.nodes[position].node_id.clone())
}

#[ix_trace_rs::string_edge]
fn aggregate_literal<'a>(term: &'a Value, kind: &str) -> Option<&'a str> {
    if term.get("term")?.as_str()? != "literal" || term.get("value_kind")?.as_str()? != kind {
        return None;
    }
    term.get("value")?.as_str()
}

fn magnitude(text: &str) -> (&str, &str) {
    text.split_once('/').unwrap_or((text, "1"))
}

#[derive(Clone, Eq, PartialEq)]
struct AggregateFamily {
    kind: AggregateFamilyKind,
    unit: Option<CheckedNodeId>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AggregateFamilyKind {
    Boolean,
    Integer,
    Rational,
    Quantity,
}

impl AggregateFamilyKind {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            "boolean" => Some(Self::Boolean),
            "integer" => Some(Self::Integer),
            "rational" => Some(Self::Rational),
            "quantity" => Some(Self::Quantity),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AggregateKind {
    Count,
    Sum,
    Min,
    Max,
    Fraction,
    Quantile,
}

impl AggregateKind {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            "count" => Some(Self::Count),
            "sum" => Some(Self::Sum),
            "min" => Some(Self::Min),
            "max" => Some(Self::Max),
            "fraction" => Some(Self::Fraction),
            "quantile" => Some(Self::Quantile),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AggregateWindow {
    Positions,
    Duration,
}

impl AggregateWindow {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            "positions" => Some(Self::Positions),
            "duration" => Some(Self::Duration),
            _ => None,
        }
    }
}

#[ix_trace_rs::string_edge]
fn aggregate_family(term: &Value, graph: &StepGraph<'_, '_>) -> Option<AggregateFamily> {
    let (family, type_id) = match term.get("term")?.as_str()? {
        "literal" => (
            AggregateFamilyKind::from_wire(term.get("value_kind")?.as_str()?)?,
            CheckedNodeId::from_value(term.get("type")?)?,
        ),
        "reference" => {
            let target = reference_target(term)?;
            let (node, _) = graph.node(&target)?;
            let type_id = node.semantic_type.clone();
            let family = AggregateFamilyKind::from_wire(resolve_family(
                &type_id,
                graph.nodes,
                graph.kinds,
                graph.index,
            )?)?;
            (family, type_id)
        }
        _ => return None,
    };
    Some(AggregateFamily {
        kind: family,
        unit: (family == AggregateFamilyKind::Quantity).then_some(type_id),
    })
}

#[ix_trace_rs::string_edge]
fn aggregate_presence(kind: AggregateKind, name: &str) -> (bool, bool) {
    match name {
        "q" => (
            kind == AggregateKind::Quantile,
            kind == AggregateKind::Quantile,
        ),
        "predicate" => (
            kind == AggregateKind::Fraction,
            kind == AggregateKind::Fraction,
        ),
        "value" => (
            matches!(
                kind,
                AggregateKind::Sum
                    | AggregateKind::Min
                    | AggregateKind::Max
                    | AggregateKind::Quantile
            ),
            matches!(
                kind,
                AggregateKind::Sum
                    | AggregateKind::Min
                    | AggregateKind::Max
                    | AggregateKind::Quantile
            ),
        ),
        "filter" => (true, false),
        _ => (true, true),
    }
}

/// One scan of the occurrence rows records a clock unit only if every row
/// for a clause names the same timestamp unit. A mixed clock has no single
/// duration encoding and its aggregate bounds must refuse.
fn aggregate_clock_units<'b>(
    bindings: Option<&'b [TemporalBinding]>,
    meter: &mut WorkMeter,
) -> Result<Vec<(&'b CheckedNodeId, Option<&'b str>)>, ValidationFailure> {
    let rows = bindings.unwrap_or(&[]);
    let mut keyed: Vec<(&CheckedNodeId, Option<&str>, usize)> = meter.vec(rows.len())?;
    for (at, row) in rows.iter().enumerate() {
        let unit = match &row.clock {
            ClockConfiguration::TimestampedEvent { timestamp_unit } => {
                Some(timestamp_unit.as_ref())
            }
            _ => None,
        };
        keyed.push((&row.declaration, unit, at));
    }
    keyed.sort_unstable_by(|left, right| left.0.cmp(right.0).then(left.2.cmp(&right.2)));
    let clauses = keyed
        .iter()
        .enumerate()
        .filter(|(at, row)| *at == 0 || keyed[*at - 1].0 != row.0)
        .count();
    // One entry per clause, in id order: the unit its rows share, else none.
    let mut units: Vec<(&CheckedNodeId, Option<&str>)> = meter.vec(clauses)?;
    for (declaration, unit, _) in keyed {
        match units.last_mut() {
            Some((last, shared)) if *last == declaration => {
                if *shared != unit {
                    *shared = None;
                }
            }
            _ => units.push((declaration, unit)),
        }
    }
    Ok(units)
}

/// Whether a bound's type path is the clock's timestamp unit
/// (`scalar_type/unit/<name>`), or both are absent.
fn is_clock_unit_path(path: Option<&[CheckedNodePathSegment]>, unit: Option<&str>) -> bool {
    use CheckedNodePathSegment::Name;
    match (path, unit) {
        (None, None) => true,
        (Some([Name(kind), Name(form), Name(name)]), Some(unit)) => {
            kind.as_ref() == "scalar_type" && form.as_ref() == "unit" && name.as_ref() == unit
        }
        _ => false,
    }
}

#[ix_trace_rs::string_edge]
fn check_aggregate(
    formula: usize,
    profile: Option<TemporalProfile>,
    clock_unit: Option<&str>,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    use CheckedPackageRefusalCause as Cause;
    use CheckedPackageRefusalCode as Code;

    let body = node_pointer(formula).key("body");
    let argument = body.clone().key("arguments").index(0);
    let fail = |code, cause, at| aggregate_refusal(formula, at, code, cause, graph);
    let window = match profile {
        Some(TemporalProfile::EventPositionFalseExtension) => AggregateWindow::Positions,
        Some(TemporalProfile::TimestampedEventFiniteWindow) => AggregateWindow::Duration,
        _ => return Err(fail(Code::InvalidPackage, Cause::InvalidValue, body)),
    };
    let Some(members) = application_arguments(&graph.nodes[formula].body)
        .and_then(|arguments| arguments.first())
        .and_then(aggregate_members)
    else {
        return Ok(());
    };

    let mut held = Held::default();
    let mut last_rank = None;
    for (at, candidate) in members.iter().enumerate() {
        let rank = match CheckedTerm::view(candidate) {
            Some(CheckedTermView::Binding { name, .. }) => {
                AGGREGATE_NAMES.iter().position(|known| *known == name)
            }
            _ => None,
        };
        let pointer = argument.clone().key("members").index(at);
        let Some(rank) = rank else {
            return Err(fail(Code::InvalidPackage, Cause::UnknownMember, pointer));
        };
        let name = AGGREGATE_NAMES[rank];
        if held.contains_key(name) {
            return Err(fail(Code::InvalidPackage, Cause::DuplicateMember, pointer));
        }
        if last_rank.is_some_and(|last| rank < last) {
            return Err(fail(Code::InvalidPackage, Cause::InvalidValue, pointer));
        }
        held.insert(name, at);
        last_rank = Some(rank);
    }
    let Some(&kind_at) = held.get("kind") else {
        return Err(fail(Code::InvalidPackage, Cause::MissingMember, argument));
    };
    let value_of = |name: &str| &members[held[name]]["value"];
    let value_pointer = |at: usize| argument.clone().key("members").index(at).key("value");
    let Some(kind_text) = aggregate_literal(value_of("kind"), "text") else {
        return Err(fail(
            Code::InvalidPackage,
            Cause::WrongValueKind,
            value_pointer(kind_at),
        ));
    };
    let Some(kind) = AggregateKind::from_wire(kind_text) else {
        return Err(fail(
            Code::InvalidPackage,
            Cause::InvalidValue,
            value_pointer(kind_at),
        ));
    };
    for (at, candidate) in members.iter().enumerate() {
        let name = candidate["name"].as_str().unwrap_or("");
        if !aggregate_presence(kind, name).0 {
            return Err(fail(
                Code::InvalidPackage,
                Cause::UnknownMember,
                argument.clone().key("members").index(at),
            ));
        }
    }
    for name in AGGREGATE_NAMES {
        if aggregate_presence(kind, name).1 && !held.contains_key(name) {
            return Err(fail(Code::InvalidPackage, Cause::MissingMember, argument));
        }
    }

    let value_family = held
        .get("value")
        .and_then(|_| aggregate_family(value_of("value"), graph));
    for name in AGGREGATE_NAMES {
        let Some(&at) = held.get(name) else { continue };
        let term = value_of(name);
        let pointer = value_pointer(at);
        match name {
            "q" => {
                let Some(text) = aggregate_literal(term, "rational") else {
                    return Err(fail(Code::InvalidPackage, Cause::WrongValueKind, pointer));
                };
                let (numerator, denominator) = magnitude(text);
                if numerator == "0"
                    || numerator.starts_with('-')
                    || compare_rationals((numerator, denominator), ("1", "1"), meter, &|| {
                        pointer.clone()
                    })? != Ordering::Less
                {
                    return Err(fail(Code::InvalidPackage, Cause::InvalidValue, pointer));
                }
            }
            "predicate" | "filter" => {
                if aggregate_family(term, graph)
                    .as_ref()
                    .map(|family| family.kind)
                    != Some(AggregateFamilyKind::Boolean)
                {
                    return Err(fail(Code::IllTyped, Cause::TypeMismatch, pointer));
                }
            }
            "value" => {
                if !aggregate_family(term, graph)
                    .as_ref()
                    .is_some_and(|family| {
                        matches!(
                            family.kind,
                            AggregateFamilyKind::Integer
                                | AggregateFamilyKind::Rational
                                | AggregateFamilyKind::Quantity
                        )
                    })
                {
                    return Err(fail(Code::IllTyped, Cause::TypeMismatch, pointer));
                }
            }
            "window_kind" | "comparison" => {
                let Some(text) = aggregate_literal(term, "text") else {
                    return Err(fail(Code::InvalidPackage, Cause::WrongValueKind, pointer));
                };
                let parsed_window = (name == "window_kind")
                    .then(|| AggregateWindow::from_wire(text))
                    .flatten();
                let valid = if name == "window_kind" {
                    parsed_window.is_some()
                } else {
                    matches!(text, "<" | "<=" | "=" | ">=" | ">")
                };
                if !valid {
                    return Err(fail(Code::InvalidPackage, Cause::InvalidValue, pointer));
                }
                if name == "window_kind" && parsed_window != Some(window) {
                    return Err(fail(Code::IllTyped, Cause::UnitMismatch, pointer));
                }
            }
            "lower" | "upper" => {
                let required = if window == AggregateWindow::Positions {
                    "integer"
                } else {
                    "quantity"
                };
                let Some(text) = aggregate_literal(term, required) else {
                    return Err(fail(Code::InvalidPackage, Cause::WrongValueKind, pointer));
                };
                let bound_type = CheckedNodeId::from_value(&term["type"]);
                if window == AggregateWindow::Duration
                    && !is_clock_unit_path(
                        bound_type.as_ref().map(|id| id.path.as_slice()),
                        clock_unit,
                    )
                {
                    return Err(fail(Code::IllTyped, Cause::UnitMismatch, pointer));
                }
                if text.starts_with('-') {
                    return Err(fail(Code::InvalidPackage, Cause::InvalidValue, pointer));
                }
                if name == "upper" {
                    let Some(lower) = aggregate_literal(value_of("lower"), required) else {
                        return Err(fail(
                            Code::InvalidPackage,
                            Cause::WrongValueKind,
                            value_pointer(held["lower"]),
                        ));
                    };
                    if compare_rationals(magnitude(lower), magnitude(text), meter, &|| {
                        argument.clone()
                    })? == Ordering::Greater
                    {
                        return Err(fail(Code::InvalidPackage, Cause::InvalidValue, argument));
                    }
                }
            }
            "min_count" => {
                let Some(text) = aggregate_literal(term, "integer") else {
                    return Err(fail(Code::InvalidPackage, Cause::WrongValueKind, pointer));
                };
                if text.starts_with('-')
                    || (text == "0" && !matches!(kind, AggregateKind::Count | AggregateKind::Sum))
                {
                    return Err(fail(Code::InvalidPackage, Cause::InvalidValue, pointer));
                }
            }
            "threshold" => {
                let expected = match kind {
                    AggregateKind::Count => Some(AggregateFamily {
                        kind: AggregateFamilyKind::Integer,
                        unit: None,
                    }),
                    AggregateKind::Fraction => Some(AggregateFamily {
                        kind: AggregateFamilyKind::Rational,
                        unit: None,
                    }),
                    _ => value_family.clone(),
                };
                let actual = aggregate_family(term, graph);
                if expected
                    .as_ref()
                    .is_some_and(|family| family.kind == AggregateFamilyKind::Quantity)
                    && actual
                        .as_ref()
                        .is_some_and(|family| family.kind == AggregateFamilyKind::Quantity)
                    && expected != actual
                {
                    return Err(fail(Code::IllTyped, Cause::UnitMismatch, pointer));
                }
                if expected != actual {
                    return Err(fail(Code::IllTyped, Cause::TypeMismatch, pointer));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// The well-formed fairness member of the `temporal`/`fairness` node a clause's
/// fairness argument entry references, with that node's position.
fn fairness_member(entry: &Value, graph: &StepGraph<'_, '_>) -> Option<(usize, FairnessMember)> {
    let target = reference_target(entry)?;
    let position = *graph.index.get(&target)?;
    if graph.kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::Fairness)) {
        return None;
    }
    let operation = operation_of(graph.nodes.get(position)?)?;
    let member = read_fairness_member(operation.member.as_ref()?)?;
    Some((position, member))
}

/// A fairness resolution refusal and where it is located (the "Path and locus"
/// table of FR-038): the path is on the fairness node's `member` (`declaration`
/// or `name`), the locus the `declaration` target's key or the fairness node's.
struct FairnessRefusal {
    refusal: ModelRefusal,
    /// The member of the fairness member the path ends in.
    member: &'static str,
    /// Whether the locus is the `declaration` target rather than the fairness node.
    locus_is_target: bool,
}

impl FairnessRefusal {
    fn at_declaration(refusal: ModelRefusal, locus_is_target: bool) -> Self {
        Self {
            refusal,
            member: "declaration",
            locus_is_target,
        }
    }

    fn at_name(refusal: ModelRefusal) -> Self {
        Self {
            refusal,
            member: "name",
            locus_is_target: true,
        }
    }
}

/// Resolves a fairness member's operation on its declaring node, as an
/// operation anchor's `operation` name resolves (FR-040), except that an
/// operation the node only inherits is admitted: a `declaration` naming no node
/// is `missing-name` at the `declaration`, one that is no `model`/`object_type`
/// declaration node is `malformed-declaration` there with the target as locus
/// (as FR-342 step 1 refuses an anchor's context), an owner not recovered
/// refuses with its own refusal at the `declaration`, and a name no effective
/// operation matches is `missing-name` and two matches `ambiguous-name`, at the
/// `name`.
fn resolve_fairness(
    declaration: &CheckedNodeId,
    name: &str,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<Result<(), FairnessRefusal>, ValidationFailure> {
    // A `declaration` that names no node is a target that is no
    // `model`/`object_type` node (merged FR-370 "Fairness resolution" step 1),
    // located at the key as named.
    let Some((declaring, kind)) = graph.node(declaration) else {
        return Ok(Err(FairnessRefusal::at_declaration(
            ModelRefusal::malformed(),
            true,
        )));
    };
    if kind != CheckedNodeKind::Model(ModelForm::ObjectType)
        || !graph
            .owners
            .is_model_declaration_node(declaring, kind.tag())
    {
        return Ok(Err(FairnessRefusal::at_declaration(
            ModelRefusal::malformed(),
            true,
        )));
    }
    if let Err(refusal) = graph.owners.recover(declaring) {
        return Ok(Err(FairnessRefusal::at_declaration(refusal, false)));
    }
    Ok(
        match graph
            .owners
            .resolve_member(declaring, MemberKind::Operation, name, meter)?
        {
            Ok((_, Resolved::Operation(_))) => Ok(()),
            Ok((_, Resolved::Field(_))) => {
                Err(FairnessRefusal::at_name(ModelRefusal::missing_name()))
            }
            Err(refusal) if refusal == ModelRefusal::ineligible() => {
                Err(FairnessRefusal::at_name(ModelRefusal::missing_name()))
            }
            Err(refusal) => Err(FairnessRefusal::at_name(refusal)),
        },
    )
}

/// The interval of every interval operator of the formula tree under `root`,
/// in pre-order, each with its formula node's position: the `temporal`/`formula`
/// nodes reachable from the clause's formula argument by operand references,
/// each visited once and charged one unit of work. A node that is no formula
/// node ends its branch.
struct FormulaReach {
    intervals: Vec<(usize, Interval)>,
    /// The positions reached, ascending.
    reached: Vec<usize>,
}

fn formula_intervals(
    catalog: &OperationCatalog,
    root: Option<&Value>,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
    at: &JsonPointer,
    covered: &mut [bool],
    stamps: &mut Stamps,
) -> Result<FormulaReach, ValidationFailure> {
    stamps.next();
    let mut pending: Vec<usize> = Vec::new();
    if let Some(start) = root
        .and_then(reference_target)
        .and_then(|target| graph.index.get(&target).copied())
    {
        meter.push(&mut pending, start)?;
    }
    let mut reached: Vec<usize> = Vec::new();
    let mut intervals = Vec::new();
    while let Some(position) = pending.pop() {
        if graph.kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::Formula))
            || !stamps.insert(position)
        {
            continue;
        }
        meter.charge(1, || at.clone())?;
        meter.push(&mut reached, position)?;
        if let Some(mark) = covered.get_mut(position) {
            *mark = true;
        }
        if let Some(interval) = interval_of(catalog, position, graph) {
            meter.push(&mut intervals, (position, interval))?;
        }
        let operands = || {
            application_arguments(&graph.nodes[position].body)
                .into_iter()
                .flatten()
                .filter_map(reference_target)
                .filter_map(|target| graph.index.get(&target).copied())
        };
        // The operands are pushed last-first so the first is visited first.
        meter.room(&mut pending, operands().count())?;
        let first = pending.len();
        pending.extend(operands());
        pending[first..].reverse();
    }
    reached.sort_unstable();
    Ok(FormulaReach { intervals, reached })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::shared::CheckedSelection;
    use crate::checked_package::v2::CheckedJson;
    use crate::checked_package::v2::CheckedNodeTag;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn id(fill: char) -> Value {
        json!({ "package": "test/temporal", "path": [fill.to_string()], "ordinal": 0 })
    }

    #[trace("TC-048", "FR-038-AC-22")]
    #[test]
    fn tc_048_diagnostic_reference_requires_a_closed_term() {
        let target: CheckedNodeId = serde_json::from_value(id('a')).expect("target");
        let refused = |value: &Value| {
            let json = CheckedJson::from_value(value).expect("document");
            references_refused_node(
                TermNode::of(&json),
                |node| node == &target,
                &mut WorkMeter::new(u64::MAX),
            )
            .ok()
        };
        let reference = json!({"term": "reference", "target": id('a')});
        assert_eq!(refused(&reference), Some(true));

        let extra = json!({"term": "reference", "target": id('a'), "members": []});
        assert_eq!(refused(&extra), Some(false));
        // Below an aggregate and a binding, and a different target.
        let nested = json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "n", "value": reference}
        ]});
        assert_eq!(refused(&nested), Some(true));
        assert_eq!(
            refused(&json!({"term": "reference", "target": id('b')})),
            Some(false)
        );
    }

    /// The diagnostic reference walk requests its stack and each target ID
    /// before making them: the bytes the allocator is asked for equal the
    /// bytes charged, the measured requirement admits and one byte below is
    /// denied (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn diagnostic_reference_walk_allocates_exactly_what_it_charges() {
        let reference = json!({"term": "reference", "target": id('a')});
        let value = json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "n", "value": reference.clone()},
            reference,
            {"term": "reference", "target": id('b')},
        ]});
        let json = CheckedJson::from_value(&value).expect("document");
        let run = |limit: u64| {
            let mut budget =
                crate::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut walked = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                walked = Some(references_refused_node(
                    TermNode::of(&json),
                    |_| false,
                    &mut meter,
                ));
            });
            (
                walked.expect("ran").ok(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (walked, allocated, charged) = run(u64::MAX);
        assert_eq!(walked, Some(false));
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert_eq!(run(charged).0, walked, "the measured limit admits");
        assert_eq!(run(charged - 1).0, None, "one byte short is denied");
    }

    /// FR-038-AC-97: an interval bound is an integer string of unbounded size,
    /// ordered as the integer it spells and never as text.
    #[trace("TC-048", "FR-038-AC-97")]
    #[test]
    fn tc_048_interval_bounds_are_integer_strings_ordered_as_unbounded_integers() {
        for admitted in ["0", "3", "10", "-1", "18446744073709551617"] {
            assert!(IntegerString::parse(admitted).is_some(), "{admitted}");
        }
        for refused in ["1.5", "01", "+1", "", "3x", "-0", "--1", "-", " 1", "1 "] {
            assert!(IntegerString::parse(refused).is_none(), "{refused:?}");
        }
        let int = |text: &str| IntegerString::parse(text).expect("an integer string");
        for (lower, upper) in [
            ("9", "10"),
            ("0", "3"),
            ("-1", "0"),
            ("-10", "-9"),
            ("-5", "-2"),
            ("18446744073709551616", "18446744073709551617"),
            ("99999999999999999999", "100000000000000000000"),
        ] {
            assert!(int(lower) < int(upper), "{lower} < {upper}");
            assert!(int(upper) > int(lower), "{upper} > {lower}");
        }
        assert!(int("3") == int("3"));
    }

    fn lock() -> CheckedPackageLockV2 {
        CheckedPackageLockV2 {
            sources: Vec::new(),
            edition: CheckedSelection {
                role: CheckedSelectionRole::Edition,
                definition: CheckedArtifactRef {
                    authority: Box::from("test"),
                    identity: Box::from("test"),
                },
            },
            profile_selections: Vec::new(),
            definition_selections: Vec::new(),
            required_features: Vec::new(),
            dependency_selections: Vec::new(),
            refinement_internals: None,
        }
    }

    /// The clause graph: node 0 the clause over node 1 (a parameter) and node 2
    /// (a formula), with `dependencies` as its wire names them.
    fn clause_graph(dependencies: &[char]) -> Vec<CheckedSemanticNodeV2> {
        let node = |fill: char, tag: &str, form: &str, dependencies: &[char], body: Value| {
            serde_json::from_value::<CheckedSemanticNodeV2>(json!({
                "node_id": id(fill),
                "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag, "semantic_form": form, "semantic_type": id('9'),
                "dependencies": dependencies.iter().map(|fill| id(*fill)).collect::<Vec<_>>(),
                "occurrences": [], "body": body,
            }))
            .expect("a well-formed test node")
        };
        let application = |operator: &str, identity: &str, laws: Value, arguments: Value| {
            json!({
                "term": "application", "operator": operator,
                "operation": {"identity": identity, "laws": laws, "mode": null,
                              "member": null, "leaves": []},
                "result_type": id('9'), "arguments": arguments,
            })
        };
        let empty = json!({"term": "aggregate", "members": []});
        let reference = |fill: char| json!({"term": "reference", "target": id(fill)});
        vec![
            node(
                '0',
                "temporal",
                "temporal_clause",
                dependencies,
                application(
                    "temporal",
                    "quire.op.temporal.clause",
                    json!([{"role": "temporal_profile", "definition": {
                        "authority": "agent-ix",
                        "identity": "quire.temporal.event-position.false-extension/v1"}}]),
                    json!([
                        reference('1'),
                        {"term": "literal", "type": id('3'), "value_kind": "text", "value": "c"},
                        empty, empty, empty,
                        reference('2'),
                    ]),
                ),
            ),
            node(
                '1',
                "value",
                "parameter",
                &[],
                json!({"term": "aggregate", "members": []}),
            ),
            node(
                '2',
                "temporal",
                "formula",
                &[],
                application(
                    "temporal_formula",
                    "quire.op.temporal.true",
                    json!([]),
                    json!([]),
                ),
            ),
        ]
    }

    fn step(nodes: &[CheckedSemanticNodeV2]) -> Result<(), ValidationFailure> {
        let package_lock = lock();
        let kinds: Vec<CheckedNodeKind> = nodes
            .iter()
            .map(|node| {
                let tag = CheckedNodeTag::from_wire(&node.node_tag).expect("test node tag");
                CheckedNodeKind::decode(tag, &node.semantic_form).expect("test node form")
            })
            .collect();
        let index: GraphIndex<'_> = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let terms: Vec<_> = nodes
            .iter()
            .map(|node| {
                Some(CheckedTerm::decode_admitted(&node.body).expect("closed temporal fixture"))
            })
            .collect();
        validate_temporal(
            &terms,
            nodes,
            &kinds,
            &index,
            &ModelOwners::default(),
            TemporalContext {
                catalog: crate::checked_package::v2::operation_catalog::operation_catalog(),
                lock: &package_lock,
                diagnostics: &[],
                bindings: None,
            },
            &mut WorkMeter::new(1_000),
        )
    }

    /// Placement borrows terms and asks for the walker's storage before growth.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn temporal_placement_allocates_only_charged_walker_storage() {
        let nodes = clause_graph(&['1', '2']);
        let kinds: Vec<_> = nodes
            .iter()
            .map(|node| {
                let tag = CheckedNodeTag::from_wire(&node.node_tag).expect("node tag");
                CheckedNodeKind::decode(tag, &node.semantic_form).expect("node form")
            })
            .collect();
        let terms: Vec<_> = nodes
            .iter()
            .map(|node| {
                Some(CheckedTerm::decode_admitted(&node.body).expect("closed temporal fixture"))
            })
            .collect();
        let index: GraphIndex<'_> = nodes
            .iter()
            .enumerate()
            .map(|(at, node)| (&node.node_id, at))
            .collect();
        let owners = ModelOwners::default();
        let graph = StepGraph {
            nodes: &nodes,
            kinds: &kinds,
            index: &index,
            owners: &owners,
        };
        let run = |limit| {
            let mut budget =
                crate::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let allocations = allocation_counter::measure(|| {
                result = Some(placement_defect(
                    0,
                    &graph,
                    &terms,
                    &mut WorkMeter::over(&mut budget),
                ));
            });
            (
                result.expect("placement ran"),
                allocations.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (result, allocated, charged) = run(u64::MAX);
        assert!(
            matches!(result, Ok(None)),
            "valid clause has no placement defect"
        );
        assert!(charged > 0);
        assert_eq!(allocated, charged, "only walker storage is requested");
        assert!(matches!(run(charged).0, Ok(None)));
        assert!(
            run(charged - 1).0.is_err(),
            "denied walker storage stops placement"
        );
    }

    /// FR-038-AC-103: an `over` that names no declared dependency of its clause
    /// refuses `missing_declaration`/`missing-name` at the clause node, and one
    /// that does admits. (A package read cannot reach this: the application
    /// dependency join makes every referenced node a dependency first.)
    #[trace("TC-048", "FR-038-AC-103")]
    #[test]
    fn tc_048_an_over_outside_the_clauses_dependencies_is_a_missing_name() {
        assert!(step(&clause_graph(&['1', '2'])).is_ok());
        let nodes = clause_graph(&['2']);
        let Err(ValidationFailure::Refused(refusal)) = step(&nodes) else {
            panic!("an over outside the dependencies refuses");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
        assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
        assert_eq!(
            refusal.loci[0].path.as_ref().map(JsonPointer::as_str),
            Some("/semantic_graph/nodes/0/body/arguments/0")
        );
        // The path is on the argument and the locus is the target (a node that
        // is no declared dependency).
        assert_eq!(refusal.locus.as_ref(), Some(&nodes[1].node_id));
    }
}
