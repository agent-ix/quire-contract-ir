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
//! ([`member_is_well_formed_node`]), the unbounded-integer comparison of integer
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
use super::operations::resolve_family;
// These validators consume the strict decoder's authenticated typed table.
use super::structural::typed_admitted_reference as typed_reference;
use super::{
    member_pointer, ApplicationOperator, CheckedDiagnosticV2, CheckedNodeKind,
    CheckedPackageLockV2, CheckedSelectionRole, CheckedTerm, ClockConfiguration,
    FairnessGranularity, FairnessKind, HybridForm, IntervalEnd, IntervalFit, LawRole, LiteralKind,
    ModelForm, OperationMemberKind, RawSemanticNodeV2 as CheckedSemanticNodeV2, TemporalBinding,
    TemporalForm, TemporalProfile, TermNode, TermView, ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedNodeId, CheckedNodePathSegment, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, JsonPointer,
};
use quire_walk::{Children, Walk};
#[cfg(test)]
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
    fn parts(text: &str) -> Option<(bool, &str)> {
        let (negative, digits) = text
            .strip_prefix('-')
            .map_or((false, text), |digits| (true, digits));
        let spelled = match digits.as_bytes() {
            [b'0'] => !negative,
            [first, rest @ ..] => {
                (b'1'..=b'9').contains(first) && rest.iter().all(u8::is_ascii_digit)
            }
            [] => false,
        };
        spelled.then_some((negative, digits))
    }

    /// The integer `text` spells, outside a budgeted read.
    #[ix_trace_rs::string_edge]
    #[cfg(test)]
    pub(in crate::checked_package) fn parse(text: &str) -> Option<Self> {
        let (negative, digits) = Self::parts(text)?;
        Some(Self {
            negative,
            zero: digits == "0",
            digits: digits.into(),
        })
    }

    #[ix_trace_rs::string_edge]
    fn parse_charged(text: &str, meter: &mut WorkMeter) -> Result<Option<Self>, ValidationFailure> {
        let Some((negative, digits)) = Self::parts(text) else {
            return Ok(None);
        };
        Ok(Some(Self {
            negative,
            zero: digits == "0",
            digits: meter.boxed_str(digits)?,
        }))
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
    fn read_node(
        value: TermNode<'_>,
        meter: &mut WorkMeter,
    ) -> Result<Option<Self>, ValidationFailure> {
        if !value.exact_members(&["numerator", "denominator"]) {
            return Ok(None);
        }
        let (Some(numerator), Some(denominator)) = (
            value.get("numerator").and_then(TermNode::as_str),
            value.get("denominator").and_then(TermNode::as_str),
        ) else {
            return Ok(None);
        };
        let (Some(numerator), Some(denominator)) = (
            IntegerString::parse_charged(numerator, meter)?,
            IntegerString::parse_charged(denominator, meter)?,
        ) else {
            return Ok(None);
        };
        Ok(
            (!numerator.negative && !denominator.negative && !denominator.is_zero()).then_some(
                Self {
                    numerator,
                    denominator,
                },
            ),
        )
    }

    /// The exact comparison of two rationals, charged to the work limit at `at`.
    fn compare(
        &self,
        other: &Self,
        meter: &mut WorkMeter,
        at: super::stage_storage::Site<'_>,
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
    fn of_node(interval: TermNode<'_>) -> Option<Self> {
        if interval.exact_members(&["lower", "upper"]) {
            Some(Self::Integer)
        } else if interval.exact_members(&["lower", "upper", "lower_end", "upper_end"]) {
            Some(Self::Timed)
        } else {
            None
        }
    }
}

fn interval_object_node(operation: TermNode<'_>) -> Option<TermNode<'_>> {
    let member = operation.get("member")?;
    if OperationMemberKind::from_wire(member.get("kind")?.as_str()?)
        != Some(OperationMemberKind::TemporalInterval)
    {
        return None;
    }
    let interval = member.get("interval")?;
    interval.is_object().then_some(interval)
}

/// Document-backed counterpart used by the strict flat-wire pass.
#[ix_trace_rs::string_edge]
pub(super) fn interval_bound_outside_pattern_node(operation: TermNode<'_>) -> Option<&'static str> {
    let member = operation.get("member")?;
    if OperationMemberKind::from_wire(member.get("kind")?.as_str()?)
        != Some(OperationMemberKind::TemporalInterval)
    {
        return None;
    }
    let interval = member.get("interval")?;
    if !interval.is_object() {
        return None;
    }
    let timed = interval.exact_members(&["lower", "upper", "lower_end", "upper_end"]);
    let integer = interval.exact_members(&["lower", "upper"]);
    ["lower", "upper"].into_iter().find(|bound| {
        let Some(value) = interval.get(bound) else {
            return false;
        };
        if timed {
            if *bound == "upper" && value.is_null() {
                return false;
            }
            let Some(rational) = value.is_object().then_some(value) else {
                return true;
            };
            if !rational.exact_members(&["numerator", "denominator"]) {
                return true;
            }
            let (Some(numerator), Some(denominator)) = (
                rational.get("numerator").and_then(TermNode::as_str),
                rational.get("denominator").and_then(TermNode::as_str),
            ) else {
                return true;
            };
            let Some((negative_n, _)) = IntegerString::parts(numerator) else {
                return true;
            };
            let Some((negative_d, digits_d)) = IntegerString::parts(denominator) else {
                return true;
            };
            negative_n || negative_d || digits_d == "0"
        } else if integer {
            if *bound == "upper" && value.is_null() {
                return false;
            }
            value
                .as_str()
                .and_then(IntegerString::parts)
                .is_none_or(|(negative, _)| negative)
        } else {
            false
        }
    })
}

/// Whether the retained `temporal_interval` member has a timed null upper end.
pub(super) fn timed_upper_is_null_node(operation: TermNode<'_>) -> bool {
    let Some(member) = operation.get("member") else {
        return false;
    };
    if OperationMemberKind::from_wire(
        member
            .get("kind")
            .and_then(TermNode::as_str)
            .unwrap_or_default(),
    ) != Some(OperationMemberKind::TemporalInterval)
    {
        return false;
    }
    let Some(interval) = member.get("interval") else {
        return false;
    };
    interval.exact_members(&["lower", "upper", "lower_end", "upper_end"])
        && interval.get("upper").is_some_and(TermNode::is_null)
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
        let Some((body, _row)) = meter.body(position) else {
            continue;
        };
        let Some(super::TermView::Application { operation, .. }) = body.view() else {
            continue;
        };
        let Some(interval) = interval_object_node(operation) else {
            continue;
        };
        if IntervalForm::of_node(interval) != Some(IntervalForm::Timed) {
            continue;
        }
        for bound in ["lower", "upper"] {
            let Some(bound_node) = interval.get(bound) else {
                continue;
            };
            let Some(rational) = Rational::read_node(bound_node, meter)? else {
                continue;
            };
            if !coprime(
                rational.numerator.digits(),
                rational.denominator.digits(),
                meter,
                super::stage_storage::Site::new(
                    super::PackageReadStage::Graph,
                    position,
                    "temporal",
                    super::stage_storage::Path::root()
                        .key("semantic_graph")
                        .key("nodes")
                        .index(position)
                        .key("body"),
                ),
            )? {
                let at = super::stage_storage::Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(position)
                    .key("body")
                    .key("operation")
                    .key("member")
                    .key("interval")
                    .key(bound);
                return Err(super::stage_storage::refuse(
                    meter,
                    &super::stage_storage::Site::new(
                        super::PackageReadStage::Graph,
                        position,
                        "temporal",
                        at,
                    ),
                    super::stage_storage::Refusal::new(
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        at,
                    )
                    .at_node(&node.node_id),
                ));
            }
        }
    }
    Ok(())
}

/// A `{kind: fairness, fairness_kind, granularity, target}`
/// member (QSpec FR-370). The two vocabulary members are decoded for their
/// closed words and carried no further: nothing here evaluates fairness.
struct FairnessMember {
    _fairness_kind: FairnessKind,
    _granularity: FairnessGranularity,
    target: super::CheckedFairnessTarget,
}

/// Member grammar over the retained document, before typed admission views exist.
pub(super) fn member_is_well_formed_node(
    kind: OperationMemberKind,
    member: TermNode<'_>,
    ids: &super::typed_terms::CheckedOperationMemberIds,
) -> bool {
    if member
        .get("kind")
        .and_then(TermNode::as_str)
        .and_then(OperationMemberKind::from_wire)
        != Some(kind)
    {
        return false;
    }
    #[ix_trace_rs::string_edge]
    fn integer(value: Option<TermNode<'_>>) -> bool {
        value.and_then(TermNode::as_str).is_some_and(|text| {
            let digits = text.strip_prefix('-').unwrap_or(text);
            match digits.as_bytes() {
                [b'0'] => text == "0",
                [first, rest @ ..] => {
                    (b'1'..=b'9').contains(first) && rest.iter().all(u8::is_ascii_digit)
                }
                [] => false,
            }
        })
    }
    #[ix_trace_rs::string_edge]
    fn rational(value: TermNode<'_>) -> bool {
        value.exact_members(&["numerator", "denominator"])
            && integer(value.get("numerator"))
            && integer(value.get("denominator"))
            && value
                .get("numerator")
                .and_then(TermNode::as_str)
                .is_some_and(|text| !text.starts_with('-'))
            && value
                .get("denominator")
                .and_then(TermNode::as_str)
                .is_some_and(|text| text != "0" && !text.starts_with('-'))
    }
    match kind {
        OperationMemberKind::TemporalInterval => {
            if !member.exact_members(&["kind", "interval"]) {
                return false;
            }
            let Some(interval) = member.get("interval") else {
                return false;
            };
            if interval.is_null() {
                return true;
            }
            let upper = interval.get("upper");
            if interval.exact_members(&["lower", "upper"]) {
                return integer(interval.get("lower"))
                    && upper.is_some_and(|value| value.is_null() || integer(Some(value)));
            }
            interval.exact_members(&["lower", "upper", "lower_end", "upper_end"])
                && interval.get("lower").is_some_and(rational)
                && upper.is_some_and(|value| value.is_null() || rational(value))
                && ["lower_end", "upper_end"].iter().all(|field| {
                    interval
                        .get(field)
                        .and_then(TermNode::as_str)
                        .and_then(IntervalEnd::from_wire)
                        .is_some()
                })
        }
        OperationMemberKind::Fairness => {
            if !member.exact_members(&["kind", "fairness_kind", "granularity", "target"])
                || member
                    .get("fairness_kind")
                    .and_then(TermNode::as_str)
                    .and_then(FairnessKind::from_wire)
                    .is_none()
                || member
                    .get("granularity")
                    .and_then(TermNode::as_str)
                    .and_then(FairnessGranularity::from_wire)
                    .is_none()
            {
                return false;
            }
            let Some(target) = member.get("target") else {
                return false;
            };
            match target.get("kind").and_then(TermNode::as_str) {
                Some("operation" | "hybrid_transition") => {
                    target.exact_members(&["kind", "declaration", "name"])
                        && ids.target_declaration.is_some()
                        && target
                            .get("name")
                            .and_then(TermNode::as_str)
                            .is_some_and(is_identifier)
                }
                Some("protocol_root" | "attempt_node" | "compensation_template") => {
                    target.exact_members(&["kind", "node"]) && ids.target_node.is_some()
                }
                Some("parallel_branch") => {
                    target.exact_members(&["kind", "parallel", "branch"])
                        && ids.target_parallel.is_some()
                        && target
                            .get("branch")
                            .and_then(TermNode::as_str)
                            .is_some_and(is_identifier)
                }
                _ => false,
            }
        }
        OperationMemberKind::TypeArgument => {
            member.exact_members(&["kind", "declaration"]) && ids.declaration.is_some()
        }
        OperationMemberKind::Field => {
            member.get("name").and_then(TermNode::as_str).is_some() && ids.declaration.is_some()
        }
        _ => true,
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
                &context,
                position,
                &graph,
                terms,
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
            if let Some(interval) = interval_of(context.catalog, position, terms, meter)? {
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
    /// The `quantifiers` binding of a `claim`/`hyperproperty` body.
    HyperQuantifiers,
    /// The Tuple a `quantifiers` binding holds.
    QuantifierTuple,
    /// One quantifier Group of that Tuple.
    QuantifierGroup,
    /// A direct member of a quantifier Group: where a fairness reference
    /// stands (QSpec FR-370-AC-21).
    GroupMember,
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
            return Ok(Some(ineligible(graph, position)));
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
                let hyper_root =
                    is_root && self.kind == CheckedNodeKind::Claim(super::ClaimForm::Hyperproperty);
                children.extend(members.iter().enumerate().map(|(at, member_term)| {
                    let member = match slot {
                        Slot::FairnessList => Slot::Fairness,
                        Slot::QuantifierTuple => Slot::QuantifierGroup,
                        Slot::QuantifierGroup => Slot::GroupMember,
                        _ if hyper_root && at == 3 => Slot::HyperQuantifiers,
                        _ => Slot::Other,
                    };
                    (member_term, false, member)
                }));
            }
            CheckedTerm::Binding { name, value } => {
                let value_slot = if slot == Slot::HyperQuantifiers
                    && name.kind()
                        == super::typed_terms::CheckedBindingNameKind::Other("quantifiers")
                {
                    Slot::QuantifierTuple
                } else {
                    Slot::Other
                };
                children.extend(std::iter::once((value.as_ref(), false, value_slot)));
            }
            CheckedTerm::Reference { target } => {
                let target = self.graph.node(target);
                let misplaced = match target {
                    Some((_, CheckedNodeKind::Temporal(TemporalForm::Formula))) => {
                        slot != Slot::Formula
                    }
                    Some((_, CheckedNodeKind::Temporal(TemporalForm::Fairness))) => {
                        !matches!(slot, Slot::Fairness | Slot::GroupMember)
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

/// The retained typed operation of an application body.
fn operation_of(
    term: Option<&CheckedTerm>,
) -> Option<&super::typed_terms::CheckedApplicationOperationRecord> {
    let CheckedTerm::Application { operation, .. } = term? else {
        return None;
    };
    Some(operation.record())
}

fn read_interval_member_node(
    member: TermNode<'_>,
    meter: &mut WorkMeter,
) -> Result<Option<Interval>, ValidationFailure> {
    macro_rules! need {
        ($value:expr) => {
            match $value {
                Some(value) => value,
                None => return Ok(None),
            }
        };
    }
    fn integer(
        value: TermNode<'_>,
        meter: &mut WorkMeter,
    ) -> Result<Option<IntegerString>, ValidationFailure> {
        let Some(text) = value.as_str() else {
            return Ok(None);
        };
        IntegerString::parse_charged(text, meter)
    }
    fn rational(
        value: TermNode<'_>,
        meter: &mut WorkMeter,
    ) -> Result<Option<Rational>, ValidationFailure> {
        let numerator = need!(integer(need!(value.get("numerator")), meter)?);
        let denominator = need!(integer(need!(value.get("denominator")), meter)?);
        Ok(Some(Rational {
            numerator,
            denominator,
        }))
    }
    if !member_is_well_formed_node(
        OperationMemberKind::TemporalInterval,
        member,
        &Default::default(),
    ) {
        return Ok(None);
    }
    let interval = need!(member.get("interval"));
    if interval.is_null() {
        return Ok(Some(Interval::Unbounded));
    }
    let lower = need!(interval.get("lower"));
    let upper = need!(interval.get("upper"));
    if interval.exact_members(&["lower", "upper"]) {
        let lower = need!(integer(lower, meter)?);
        return Ok(Some(if upper.is_null() {
            Interval::LowerBounded
        } else {
            Interval::Closed {
                lower,
                upper: need!(integer(upper, meter)?),
            }
        }));
    }
    Ok(Some(Interval::Timed {
        lower: need!(rational(lower, meter)?),
        upper: if upper.is_null() {
            None
        } else {
            Some(need!(rational(upper, meter)?))
        },
        lower_end: need!(interval
            .get("lower_end")
            .and_then(TermNode::as_str)
            .and_then(IntervalEnd::from_wire)),
        upper_end: need!(interval
            .get("upper_end")
            .and_then(TermNode::as_str)
            .and_then(IntervalEnd::from_wire)),
    }))
}

/// The interval of a catalogued interval operator, read from its retained member.
fn interval_of(
    catalog: &OperationCatalog,
    position: usize,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<Option<Interval>, ValidationFailure> {
    let Some(operation) = operation_of(terms.get(position).and_then(Option::as_ref)) else {
        return Ok(None);
    };
    let Some(entry) = operation
        .identity
        .as_deref()
        .and_then(|identity| catalog.entry(identity))
    else {
        return Ok(None);
    };
    if entry.operator != ApplicationOperator::TemporalFormula
        || entry.member != Some(OperationMemberKind::TemporalInterval)
    {
        return Ok(None);
    }
    let Some(member) = operation.member.as_ref().map(TermNode::of) else {
        return Ok(None);
    };
    read_interval_member_node(member, meter)
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
    let at_body = super::stage_storage::Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position)
        .key("body");
    let defective = match interval {
        Interval::Unbounded | Interval::LowerBounded => false,
        Interval::Closed { lower, upper } => lower > upper,
        Interval::Timed {
            lower,
            upper,
            lower_end,
            upper_end,
        } => match upper {
            Some(upper) => match lower.compare(
                upper,
                meter,
                super::stage_storage::Site::new(
                    super::PackageReadStage::Graph,
                    position,
                    "temporal",
                    super::stage_storage::Path::root()
                        .key("semantic_graph")
                        .key("nodes")
                        .index(position)
                        .key("body"),
                ),
            )? {
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
        Err(super::stage_storage::refuse(
            meter,
            &super::stage_storage::Site::new(
                super::PackageReadStage::Graph,
                position,
                "temporal",
                at_body,
            ),
            super::stage_storage::Refusal::new(CheckedPackageRefusalCode::InvalidPackage, at_body)
                .because(CheckedPackageRefusalCause::InvalidValue)
                .at_node(&graph.nodes[position].node_id),
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

/// An owned temporal fault, paid at its original holder before materialization.
fn temporal_fault(
    meter: &mut WorkMeter,
    position: usize,
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
    at: super::stage_storage::Path<'_>,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    super::stage_storage::refuse(
        meter,
        &super::stage_storage::Site::new(super::PackageReadStage::Graph, position, "temporal", at),
        super::stage_storage::Refusal::new(code, at)
            .because(cause)
            .at_node(locus),
    )
}

/// One clause's checks, in order: profile identity, `over`, fairness
/// resolution, profile fit, interval bounds, aggregate arguments. A part of the clause the
/// operation step refuses (an unreadable `operation`, a missing law, a
/// missing argument) is skipped here.
#[ix_trace_rs::string_edge]
fn check_clause(
    context: &TemporalContext<'_>,
    position: usize,
    graph: &StepGraph<'_, '_>,
    terms: &[Option<CheckedTerm>],
    clock_unit: Option<&str>,
    meter: &mut WorkMeter,
    covered: &mut [bool],
    stamps: &mut Stamps,
) -> Result<(), ValidationFailure> {
    let catalog = context.catalog;
    let lock = context.lock;
    let node = &graph.nodes[position];
    meter.at_node(position, super::graph_storage::node_form(&node.node_tag));
    let Some(operation) = operation_of(terms.get(position).and_then(Option::as_ref)) else {
        return Ok(());
    };
    if catalog
        .entry(operation.identity.as_deref().unwrap_or_default())
        .map(|entry| entry.operator)
        != Some(ApplicationOperator::Temporal)
    {
        return Ok(());
    }
    let at_node = super::stage_storage::Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position);
    let at_body = at_node.key("body");
    let arguments: &[CheckedTerm] = match terms.get(position).and_then(Option::as_ref) {
        Some(CheckedTerm::Application { arguments, .. }) => arguments,
        _ => &[],
    };
    let hybrid_claim = arguments
        .first()
        .and_then(typed_reference)
        .is_some_and(|over| {
            graph.node(over).is_some_and(|(parameter, kind)| {
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
                return Err(temporal_fault(
                    meter,
                    position,
                    CheckedPackageRefusalCode::UnknownProfile,
                    cause,
                    at_body
                        .key("operation")
                        .key("laws")
                        .index(0)
                        .key("definition"),
                    &node.node_id,
                ));
            }
        }
    }

    // `over`: a declared `value`/`parameter` dependency, as a frame entry's
    // reference is (FR-040), refused with the path on the argument and, as a
    // frame entry's refusal is, the target's key as locus; a target that is no
    // node has none, so the clause's key stands.
    if let Some(over) = arguments.first().and_then(typed_reference) {
        let refusal = match graph.node(over) {
            None => Some((ModelRefusal::missing_name(), &node.node_id)),
            Some(_) if !node.dependencies.contains(over) => {
                Some((ModelRefusal::missing_name(), over))
            }
            Some((_, kind)) if kind != CheckedNodeKind::Value(ValueForm::Parameter) => {
                Some((ModelRefusal::malformed(), over))
            }
            Some(_) => None,
        };
        if let Some((refusal, locus)) = refusal {
            return Err(temporal_fault(
                meter,
                position,
                refusal.code,
                refusal.cause,
                at_body.key("arguments").index(0),
                locus,
            ));
        }
    }

    // Each fairness member resolves on its declaring node.
    let fairness: &[CheckedTerm] = arguments
        .get(4)
        .and_then(|term| match term {
            CheckedTerm::Aggregate { members } => Some(members.as_slice()),
            _ => None,
        })
        .unwrap_or(&[]);
    for reference in fairness {
        let Some((fair, member)) = fairness_member(reference, graph, terms, meter)? else {
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
                return Err(temporal_fault(
                    meter,
                    fair,
                    CheckedPackageRefusalCode::InvalidModelBinding,
                    CheckedPackageRefusalCause::MalformedDeclaration,
                    super::stage_storage::Path::root()
                        .key("semantic_graph")
                        .key("nodes")
                        .index(fair)
                        .key("body")
                        .key("operation")
                        .key("member")
                        .key("target")
                        .key("declaration"),
                    &graph.nodes[fair].node_id,
                ));
            }
            continue;
        };
        if let Err(failure) = resolve_fairness(declaration, name, graph, meter)? {
            return Err(temporal_fault(
                meter,
                fair,
                failure.refusal.code,
                failure.refusal.cause,
                super::stage_storage::Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(fair)
                    .key("body")
                    .key("operation")
                    .key("member")
                    .key("target")
                    .key(failure.member),
                if failure.locus_is_target {
                    declaration
                } else {
                    &graph.nodes[fair].node_id
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
        terms,
        meter,
        super::stage_storage::Site::new(
            super::PackageReadStage::Graph,
            position,
            "temporal",
            at_node,
        ),
        covered,
        stamps,
    )?;
    if let Some(profile) = profile {
        let fit = profile.interval_fit();
        for (formula, interval) in &intervals {
            if !interval_fits(fit, interval)
                || matches!(interval, Interval::Timed { upper: None, .. }) && !hybrid_claim
            {
                return Err(temporal_fault(
                    meter,
                    *formula,
                    CheckedPackageRefusalCode::InvalidPackage,
                    CheckedPackageRefusalCause::OperationMemberMismatch,
                    super::stage_storage::Path::root()
                        .key("semantic_graph")
                        .key("nodes")
                        .index(*formula)
                        .key("body"),
                    &graph.nodes[*formula].node_id,
                ));
            }
        }
        if !fairness.is_empty() && !profile.admits_fairness() {
            return Err(temporal_fault(
                meter,
                position,
                CheckedPackageRefusalCode::InvalidPackage,
                CheckedPackageRefusalCause::OperationMemberMismatch,
                at_body,
                &node.node_id,
            ));
        }
    }
    for (formula, interval) in &intervals {
        check_bounds(*formula, interval, graph, meter)?;
    }
    let mut aggregates = reached;
    aggregates.retain(|&formula| {
        operation_of(terms.get(formula).and_then(Option::as_ref)).is_some_and(|operation| {
            operation.identity.as_deref() == Some("quire.op.temporal.aggregate")
        })
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
        check_aggregate(formula, profile, clock_unit, graph, terms, meter)?;
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

#[ix_trace_rs::string_edge]
fn aggregate_literal<'a>(term: &'a CheckedTerm, kind: &str) -> Option<&'a str> {
    match term {
        CheckedTerm::Literal {
            value_kind, value, ..
        } if matches!(
            (kind, value_kind),
            ("text", LiteralKind::Text) | ("rational", LiteralKind::Rational)
        ) =>
        {
            value.as_str()
        }
        _ => None,
    }
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
fn aggregate_family(term: &CheckedTerm, graph: &StepGraph<'_, '_>) -> Option<AggregateFamily> {
    let (family, type_id) = match term {
        CheckedTerm::Literal {
            value_kind,
            type_id,
            ..
        } => {
            let spelling = match value_kind {
                LiteralKind::Integer => "integer",
                LiteralKind::Rational => "rational",
                LiteralKind::Quantity => "quantity",
                _ => return None,
            };
            (AggregateFamilyKind::from_wire(spelling)?, type_id.clone())
        }
        CheckedTerm::Reference { target } => {
            let (node, _) = graph.node(target)?;
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
#[ix_trace_rs::string_edge]
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
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    use CheckedPackageRefusalCause as Cause;
    use CheckedPackageRefusalCode as Code;

    let body = super::stage_storage::Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(formula)
        .key("body");
    let argument = body.key("arguments").index(0);
    let fail = |meter: &mut WorkMeter, code, cause, at: super::stage_storage::Path<'_>| {
        super::stage_storage::refuse(
            meter,
            &super::stage_storage::Site::new(
                super::PackageReadStage::Graph,
                formula,
                "temporal",
                at,
            ),
            super::stage_storage::Refusal::new(code, at)
                .because(cause)
                .at_node(&graph.nodes[formula].node_id),
        )
    };
    let window = match profile {
        Some(TemporalProfile::EventPositionFalseExtension) => AggregateWindow::Positions,
        Some(TemporalProfile::TimestampedEventFiniteWindow) => AggregateWindow::Duration,
        _ => return Err(fail(meter, Code::InvalidPackage, Cause::InvalidValue, body)),
    };
    let Some(CheckedTerm::Application { arguments, .. }) =
        terms.get(formula).and_then(Option::as_ref)
    else {
        return Ok(());
    };
    let Some(CheckedTerm::Aggregate { members }) = arguments.first() else {
        return Ok(());
    };

    let mut held = Held::default();
    let mut last_rank = None;
    for (at, candidate) in members.iter().enumerate() {
        let rank = match candidate {
            CheckedTerm::Binding { name, .. } => match name.kind() {
                super::typed_terms::CheckedBindingNameKind::Other(name) => {
                    AGGREGATE_NAMES.iter().position(|known| *known == name)
                }
                _ => None,
            },
            _ => None,
        };
        let pointer = argument.key("members").index(at);
        let Some(rank) = rank else {
            return Err(fail(
                meter,
                Code::InvalidPackage,
                Cause::UnknownMember,
                pointer,
            ));
        };
        let name = AGGREGATE_NAMES[rank];
        if held.contains_key(name) {
            return Err(fail(
                meter,
                Code::InvalidPackage,
                Cause::DuplicateMember,
                pointer,
            ));
        }
        if last_rank.is_some_and(|last| rank < last) {
            return Err(fail(
                meter,
                Code::InvalidPackage,
                Cause::InvalidValue,
                pointer,
            ));
        }
        held.insert(name, at);
        last_rank = Some(rank);
    }
    let Some(&kind_at) = held.get("kind") else {
        return Err(fail(
            meter,
            Code::InvalidPackage,
            Cause::MissingMember,
            argument,
        ));
    };
    let value_of = |name: &str| match &members[held[name]] {
        CheckedTerm::Binding { value, .. } => value.as_ref(),
        _ => unreachable!("the aggregate member was checked as a binding"),
    };
    let value_pointer = |at: usize| argument.key("members").index(at).key("value");
    let Some(kind_text) = aggregate_literal(value_of("kind"), "text") else {
        return Err(fail(
            meter,
            Code::InvalidPackage,
            Cause::WrongValueKind,
            value_pointer(kind_at),
        ));
    };
    let Some(kind) = AggregateKind::from_wire(kind_text) else {
        return Err(fail(
            meter,
            Code::InvalidPackage,
            Cause::InvalidValue,
            value_pointer(kind_at),
        ));
    };
    for (at, candidate) in members.iter().enumerate() {
        let name = match candidate {
            CheckedTerm::Binding { name, .. } => match name.kind() {
                super::typed_terms::CheckedBindingNameKind::Other(name) => name,
                _ => "",
            },
            _ => "",
        };
        if !aggregate_presence(kind, name).0 {
            return Err(fail(
                meter,
                Code::InvalidPackage,
                Cause::UnknownMember,
                argument.key("members").index(at),
            ));
        }
    }
    for name in AGGREGATE_NAMES {
        if aggregate_presence(kind, name).1 && !held.contains_key(name) {
            return Err(fail(
                meter,
                Code::InvalidPackage,
                Cause::MissingMember,
                argument,
            ));
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
                    return Err(fail(
                        meter,
                        Code::InvalidPackage,
                        Cause::WrongValueKind,
                        pointer,
                    ));
                };
                let (numerator, denominator) = magnitude(text);
                if numerator == "0"
                    || numerator.starts_with('-')
                    || compare_rationals(
                        (numerator, denominator),
                        ("1", "1"),
                        meter,
                        super::stage_storage::Site::new(
                            super::PackageReadStage::Graph,
                            formula,
                            "temporal",
                            pointer,
                        ),
                    )? != Ordering::Less
                {
                    return Err(fail(
                        meter,
                        Code::InvalidPackage,
                        Cause::InvalidValue,
                        pointer,
                    ));
                }
            }
            "predicate" | "filter" => {
                if aggregate_family(term, graph)
                    .as_ref()
                    .map(|family| family.kind)
                    != Some(AggregateFamilyKind::Boolean)
                {
                    return Err(fail(meter, Code::IllTyped, Cause::TypeMismatch, pointer));
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
                    return Err(fail(meter, Code::IllTyped, Cause::TypeMismatch, pointer));
                }
            }
            "window_kind" | "comparison" => {
                let Some(text) = aggregate_literal(term, "text") else {
                    return Err(fail(
                        meter,
                        Code::InvalidPackage,
                        Cause::WrongValueKind,
                        pointer,
                    ));
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
                    return Err(fail(
                        meter,
                        Code::InvalidPackage,
                        Cause::InvalidValue,
                        pointer,
                    ));
                }
                if name == "window_kind" && parsed_window != Some(window) {
                    return Err(fail(meter, Code::IllTyped, Cause::UnitMismatch, pointer));
                }
            }
            "lower" | "upper" => {
                let required = if window == AggregateWindow::Positions {
                    "integer"
                } else {
                    "quantity"
                };
                let Some(text) = aggregate_literal(term, required) else {
                    return Err(fail(
                        meter,
                        Code::InvalidPackage,
                        Cause::WrongValueKind,
                        pointer,
                    ));
                };
                let bound_type = match term {
                    CheckedTerm::Literal { type_id, .. } => Some(type_id),
                    _ => None,
                };
                if window == AggregateWindow::Duration
                    && !is_clock_unit_path(bound_type.map(|id| id.path.as_slice()), clock_unit)
                {
                    return Err(fail(meter, Code::IllTyped, Cause::UnitMismatch, pointer));
                }
                if text.starts_with('-') {
                    return Err(fail(
                        meter,
                        Code::InvalidPackage,
                        Cause::InvalidValue,
                        pointer,
                    ));
                }
                if name == "upper" {
                    let Some(lower) = aggregate_literal(value_of("lower"), required) else {
                        return Err(fail(
                            meter,
                            Code::InvalidPackage,
                            Cause::WrongValueKind,
                            value_pointer(held["lower"]),
                        ));
                    };
                    if compare_rationals(
                        magnitude(lower),
                        magnitude(text),
                        meter,
                        super::stage_storage::Site::new(
                            super::PackageReadStage::Graph,
                            formula,
                            "temporal",
                            argument,
                        ),
                    )? == Ordering::Greater
                    {
                        return Err(fail(
                            meter,
                            Code::InvalidPackage,
                            Cause::InvalidValue,
                            argument,
                        ));
                    }
                }
            }
            "min_count" => {
                let Some(text) = aggregate_literal(term, "integer") else {
                    return Err(fail(
                        meter,
                        Code::InvalidPackage,
                        Cause::WrongValueKind,
                        pointer,
                    ));
                };
                if text.starts_with('-')
                    || (text == "0" && !matches!(kind, AggregateKind::Count | AggregateKind::Sum))
                {
                    return Err(fail(
                        meter,
                        Code::InvalidPackage,
                        Cause::InvalidValue,
                        pointer,
                    ));
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
                    return Err(fail(meter, Code::IllTyped, Cause::UnitMismatch, pointer));
                }
                if expected != actual {
                    return Err(fail(meter, Code::IllTyped, Cause::TypeMismatch, pointer));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// The well-formed fairness member of the `temporal`/`fairness` node a clause's
/// fairness argument entry references, with that node's position.
fn fairness_member(
    entry: &CheckedTerm,
    graph: &StepGraph<'_, '_>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<Option<(usize, FairnessMember)>, ValidationFailure> {
    let Some(target) = typed_reference(entry) else {
        return Ok(None);
    };
    let Some(&position) = graph.index.get(target) else {
        return Ok(None);
    };
    if graph.kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::Fairness)) {
        return Ok(None);
    }
    let Some(operation) = operation_of(terms.get(position).and_then(Option::as_ref)) else {
        return Ok(None);
    };
    let Some(member) = operation.member.as_ref().map(TermNode::of) else {
        return Ok(None);
    };
    if !member_is_well_formed_node(OperationMemberKind::Fairness, member, &operation.member_ids) {
        return Ok(None);
    }
    let Some(target_node) = member.get("target") else {
        return Ok(None);
    };
    let Some(target) = super::CheckedFairnessTarget::from_validated_node(
        target_node,
        &operation.member_ids,
        meter,
    )?
    else {
        return Ok(None);
    };
    let Some(fairness_kind) = member
        .get("fairness_kind")
        .and_then(TermNode::as_str)
        .and_then(FairnessKind::from_wire)
    else {
        return Ok(None);
    };
    let Some(granularity) = member
        .get("granularity")
        .and_then(TermNode::as_str)
        .and_then(FairnessGranularity::from_wire)
    else {
        return Ok(None);
    };
    Ok(Some((
        position,
        FairnessMember {
            _fairness_kind: fairness_kind,
            _granularity: granularity,
            target,
        },
    )))
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

/// QSpec FR-402 "Reader order" for every `claim`/`hyperproperty` node in
/// ascending node-id order: steps 1 (shape), 2 (subjects), 3 (body) and 4
/// (fairness) run per node, each node's first defect the package's. The step runs after
/// the step-relation step and before the hybrid step.
pub(super) fn validate_hyperproperties(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    owners: &ModelOwners<'_>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StepGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    for &position in index.values() {
        if graph.kinds.get(position)
            == Some(&CheckedNodeKind::Claim(super::ClaimForm::Hyperproperty))
        {
            let decoded = meter.decode_body(position)?.map_err(|_| {
                ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    node_pointer(position).key("body"),
                )
            })?;
            check_hyper_shape(position, &graph, &decoded, meter)?;
            check_hyper_fairness(position, &graph, &decoded, terms, meter)?;
        }
    }
    Ok(())
}

/// FR-402 steps 1 and 2 of one node with a `behaviours` domain.
fn check_hyper_shape(
    position: usize,
    graph: &StepGraph<'_, '_>,
    body: &CheckedTerm,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    const NAMES: [&str; 7] = [
        "domain",
        "subjects",
        "parameters",
        "quantifiers",
        "match",
        "align_skip",
        "body",
    ];
    let node = &graph.nodes[position];
    let at_body = || node_pointer(position).key("body");
    let graph_defect = |at: JsonPointer| {
        ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at,
            None,
            node.node_id.clone(),
        )
    };
    let CheckedTerm::Aggregate { members } = body else {
        return Err(graph_defect(at_body()));
    };
    fn binding(term: &CheckedTerm) -> Option<(&str, &CheckedTerm)> {
        match term {
            // `parameters` and `body` are contextual kinds, not `Other`: the
            // shape is keyed on the raw wire name.
            CheckedTerm::Binding { name, value } => Some((name.as_ref(), value)),
            _ => None,
        }
    }
    for (at, member) in members.iter().enumerate().take(NAMES.len()) {
        if binding(member).map(|(name, _)| name) != Some(NAMES[at]) {
            return Err(graph_defect(at_body().key("members").index(at)));
        }
    }
    if members.len() != NAMES.len() {
        let at = members.len().min(NAMES.len());
        let pointer = if members.len() < NAMES.len() {
            at_body()
        } else {
            at_body().key("members").index(at)
        };
        return Err(graph_defect(pointer));
    }
    let value_of = |at: usize| binding(&members[at]).map(|(_, value)| value);
    let at_value = |at: usize| at_body().key("members").index(at).key("value");
    let domain = value_of(0).ok_or_else(|| graph_defect(at_value(0)))?;
    let finite = typed_reference(domain).is_some();
    let domain_text = match domain {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        } => value.as_str(),
        _ => None,
    };
    if !finite && domain_text != Some("behaviours") {
        return Err(graph_defect(at_value(0)));
    }
    let Some(CheckedTerm::Aggregate { members: subjects }) = value_of(1) else {
        return Err(graph_defect(at_value(1)));
    };
    let Some(CheckedTerm::Aggregate {
        members: quantifiers,
    }) = value_of(3)
    else {
        return Err(graph_defect(at_value(3)));
    };
    // Step 1: a finite domain has no subjects; each quantifier Group.
    if finite && !subjects.is_empty() {
        return Err(graph_defect(at_value(1).key("members").index(0)));
    }
    let bindings_in_group: usize = if finite { 2 } else { 3 };
    let mut aliases = meter.vec(quantifiers.len())?;
    for (group_at, group) in quantifiers.iter().enumerate() {
        let at_group = || at_value(3).key("members").index(group_at);
        let CheckedTerm::Aggregate {
            members: group_members,
        } = group
        else {
            return Err(graph_defect(at_group()));
        };
        for (at, member) in group_members.iter().enumerate() {
            let expected = ["variable", "quantifier", "alias"]
                .get(at)
                .copied()
                .filter(|_| at < bindings_in_group);
            let layout_ok = match (expected, binding(member)) {
                (Some(name), Some((found, _))) => name == found,
                // A finite domain holds no fairness reference.
                (None, None) => !finite && typed_reference(member).is_some(),
                _ => false,
            };
            if !layout_ok {
                return Err(graph_defect(at_group().key("members").index(at)));
            }
        }
        if group_members.len() < bindings_in_group {
            return Err(graph_defect(at_group()));
        }
        let quantifier = binding(&group_members[1]).and_then(|(_, value)| match value {
            CheckedTerm::Literal {
                value_kind: LiteralKind::Text,
                value,
                ..
            } => value.as_str(),
            _ => None,
        });
        if quantifier.is_none_or(|word| !matches!(word, "forall" | "exists")) {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                at_group().key("members").index(1).key("value"),
                Some(CheckedPackageRefusalCause::InvalidValue),
                node.node_id.clone(),
            ));
        }
        if !finite {
            let alias = binding(&group_members[2])
                .and_then(|(_, value)| match value {
                    CheckedTerm::Literal {
                        value_kind: LiteralKind::Text,
                        value,
                        ..
                    } => value.as_str(),
                    _ => None,
                })
                .unwrap_or_default();
            meter.push(&mut aliases, alias)?;
        }
    }
    // Step 2: subjects.
    let mut subject_names = meter.vec(subjects.len())?;
    for (subject_at, subject) in subjects.iter().enumerate() {
        let Some((name, value)) = binding(subject) else {
            return Err(graph_defect(at_value(1).key("members").index(subject_at)));
        };
        let target = typed_reference(value);
        let is_root = target
            .and_then(|target| graph.node(target))
            .is_some_and(|(_, kind)| kind == CheckedNodeKind::Model(ModelForm::ModelRoot));
        if !is_root {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidModelBinding,
                at_value(1)
                    .key("members")
                    .index(subject_at)
                    .key("value")
                    .key("target"),
                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                target.unwrap_or(&node.node_id).clone(),
            ));
        }
        meter.push(&mut subject_names, name)?;
    }
    if let Some(group_at) = aliases
        .iter()
        .position(|alias| !subject_names.contains(alias))
    {
        return Err(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            at_value(3)
                .key("members")
                .index(group_at)
                .key("members")
                .index(2),
            Some(CheckedPackageRefusalCause::MissingName),
            node.node_id.clone(),
        ));
    }
    if let Some(subject_at) = subject_names
        .iter()
        .position(|name| !aliases.contains(name))
    {
        return Err(graph_defect(at_value(1).key("members").index(subject_at)));
    }
    // Step 3: the body.
    let body_target = value_of(6).and_then(typed_reference).cloned();
    let body_node = body_target.as_ref().and_then(|target| graph.node(target));
    if finite {
        let boolean_expression = body_node.is_some_and(|(target, kind)| {
            matches!(kind, CheckedNodeKind::Expression(_))
                && graph.node(&target.semantic_type).is_some_and(|(_, kind)| {
                    kind == CheckedNodeKind::ScalarType(super::ScalarTypeForm::Boolean)
                })
        });
        if !boolean_expression {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::IllTyped,
                at_value(6).key("target"),
                Some(CheckedPackageRefusalCause::NonBooleanRoot),
                body_target.clone().unwrap_or_else(|| node.node_id.clone()),
            ));
        }
    } else {
        let Some((wrapper, CheckedNodeKind::Temporal(TemporalForm::TemporalClause))) = body_node
        else {
            return Err(ineligible(graph, position));
        };
        let wrapper_position = body_target
            .as_ref()
            .and_then(|target| graph.index.get(target).copied());
        let wrapper_body =
            wrapper_position.and_then(|position| meter.decode_body(position).ok()?.ok());
        let fairness_nonempty = wrapper_body
            .as_ref()
            .and_then(|body| match body {
                CheckedTerm::Application { arguments, .. } => arguments.get(4),
                _ => None,
            })
            .and_then(|term| match term {
                CheckedTerm::Aggregate { members } => Some(!members.is_empty()),
                _ => None,
            })
            .unwrap_or(false);
        if fairness_nonempty {
            let wrapper_at = body_target
                .as_ref()
                .and_then(|target| graph.index.get(target).copied())
                .unwrap_or(position);
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                node_pointer(wrapper_at)
                    .key("body")
                    .key("arguments")
                    .index(4),
                None,
                wrapper.node_id.clone(),
            ));
        }
    }
    Ok(())
}

/// QSpec FR-402 "Reader order" step 4, for a `claim`/`hyperproperty` node: each
/// fairness reference of a quantifier Group resolves as a clause's fairness
/// member does, located at the reference. Layout defects are not this check's;
/// a body it cannot read is skipped.
fn check_hyper_fairness(
    position: usize,
    graph: &StepGraph<'_, '_>,
    body: &CheckedTerm,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let CheckedTerm::Aggregate { members } = body else {
        return Ok(());
    };
    let binding_value = |at: usize, wanted: &str| match members.get(at) {
        Some(CheckedTerm::Binding { name, value })
            if name.kind() == super::typed_terms::CheckedBindingNameKind::Other(wanted) =>
        {
            Some(value.as_ref())
        }
        _ => None,
    };
    let (Some(quantifiers), Some(subjects)) = (
        binding_value(3, "quantifiers").and_then(|term| match term {
            CheckedTerm::Aggregate { members } => Some(members.as_slice()),
            _ => None,
        }),
        binding_value(1, "subjects").and_then(|term| match term {
            CheckedTerm::Aggregate { members } => Some(members.as_slice()),
            _ => None,
        }),
    ) else {
        return Ok(());
    };
    let profile = binding_value(6, "body")
        .and_then(typed_reference)
        .and_then(|body| {
            graph
                .index
                .get(body)
                .and_then(|position| operation_of(terms.get(*position).and_then(Option::as_ref)))
        })
        .and_then(|operation| match operation.laws.as_slice() {
            [law] if law.role_class() == Some(LawRole::TemporalProfile) => {
                TemporalProfile::from_wire(&law.definition.identity)
            }
            _ => None,
        });
    let reference_pointer = |group: usize, member: usize| {
        node_pointer(position)
            .key("body")
            .key("members")
            .index(3)
            .key("value")
            .key("members")
            .index(group)
            .key("members")
            .index(member)
    };
    let malformed = |group: usize, member: usize, fair: &CheckedNodeId| {
        ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            reference_pointer(group, member),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            fair.clone(),
        )
    };
    for (group_at, group) in quantifiers.iter().enumerate() {
        let CheckedTerm::Aggregate {
            members: group_members,
        } = group
        else {
            continue;
        };
        let alias = group_members.iter().find_map(|member| match member {
            CheckedTerm::Binding { name, value }
                if name.kind() == super::typed_terms::CheckedBindingNameKind::Other("alias") =>
            {
                match value.as_ref() {
                    CheckedTerm::Literal {
                        value_kind: LiteralKind::Text,
                        value,
                        ..
                    } => value.as_str(),
                    _ => None,
                }
            }
            _ => None,
        });
        let alias_root = alias.and_then(|alias| {
            subjects.iter().find_map(|subject| match subject {
                CheckedTerm::Binding { name, value } if AsRef::<str>::as_ref(name) == alias => {
                    typed_reference(value)
                }
                _ => None,
            })
        });
        let alias_package = alias_root.and_then(|root| match graph.node(root)? {
            (node, CheckedNodeKind::Model(ModelForm::ModelRoot)) => match node.owner.as_ref()? {
                super::CheckedNodeOwner::ModelRoot { identity } => Some(identity),
                _ => None,
            },
            _ => None,
        });
        for (member_at, member) in group_members.iter().enumerate() {
            let Some((_, fairness)) = fairness_member(member, graph, terms, meter)? else {
                continue;
            };
            let fair_id = &graph.nodes[position].node_id;
            let super::CheckedFairnessTarget::Operation { declaration, name } = &fairness.target
            else {
                return Err(malformed(group_at, member_at, fair_id));
            };
            if let Err(failure) = resolve_fairness(declaration, name, graph, meter)? {
                return Err(ValidationFailure::refused_at(
                    failure.refusal.code,
                    reference_pointer(group_at, member_at),
                    Some(failure.refusal.cause),
                    if failure.locus_is_target {
                        declaration.clone()
                    } else {
                        fair_id.clone()
                    },
                ));
            }
            let declaring_package = graph
                .node(declaration)
                .and_then(|(declaring, _)| graph.owners.recover(declaring).ok())
                .map(|owner| &owner.package.identity);
            if alias_package.is_some() && declaring_package != alias_package {
                return Err(malformed(group_at, member_at, fair_id));
            }
            if profile.is_some_and(|profile| !profile.admits_fairness()) {
                return Err(ValidationFailure::refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    reference_pointer(group_at, member_at),
                    Some(CheckedPackageRefusalCause::OperationMemberMismatch),
                    fair_id.clone(),
                ));
            }
        }
    }
    Ok(())
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
    root: Option<&CheckedTerm>,
    graph: &StepGraph<'_, '_>,
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
    at: super::stage_storage::Site<'_>,
    covered: &mut [bool],
    stamps: &mut Stamps,
) -> Result<FormulaReach, ValidationFailure> {
    stamps.next();
    let mut pending: Vec<usize> = meter.vec(graph.nodes.len())?;
    if let Some(start) = root
        .and_then(typed_reference)
        .and_then(|target| graph.index.get(target).copied())
    {
        meter.push(&mut pending, start)?;
    }
    let mut reached: Vec<usize> = meter.vec(graph.nodes.len())?;
    let mut intervals = meter.vec(graph.nodes.len())?;
    while let Some(position) = pending.pop() {
        if graph.kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::Formula))
            || !stamps.insert(position)
        {
            continue;
        }
        super::natural::work_at(meter, at, 1)?;
        meter.push(&mut reached, position)?;
        if let Some(mark) = covered.get_mut(position) {
            *mark = true;
        }
        if let Some(interval) = interval_of(catalog, position, terms, meter)? {
            meter.push(&mut intervals, (position, interval))?;
        }
        let operands = || {
            terms
                .get(position)
                .and_then(Option::as_ref)
                .and_then(|term| match term {
                    CheckedTerm::Application { arguments, .. } => Some(arguments.as_slice()),
                    _ => None,
                })
                .into_iter()
                .flatten()
                .filter_map(typed_reference)
                .filter_map(|target| graph.index.get(target).copied())
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

    #[trace("TC-472", "FR-038-AC-516", "FR-038-AC-5")]
    #[test]
    fn invalid_reference_id_refuses_in_the_graph_term_pass_before_temporal_views() {
        use crate::checked_package::common::{validate_term, Step, TermGrammar, Trail};
        let body = CheckedJson::from_value(&json!({"term":"reference","target":{
            "package":"", "path":[], "ordinal":0
        }}))
        .unwrap();
        let steps = [
            Step::Key("semantic_graph"),
            Step::Key("nodes"),
            Step::Index(7),
            Step::Key("body"),
        ];
        let mut budget =
            super::super::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let mut visited = 0;
        let result = validate_term(
            TermNode::of(&body),
            TermGrammar::V2,
            true,
            &Trail::Base(&steps),
            &mut |_, _, _, _, _| {
                visited += 1;
                Ok(())
            },
            super::super::read_charge::ids(
                &mut budget,
                super::super::PackageReadStage::Graph,
                7,
                "temporal",
                &[],
            ),
        );
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("invalid target must refuse before temporal interpretation")
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            "/semantic_graph/nodes/7/body/target"
        );
        assert_eq!(
            visited, 0,
            "invalid IDs never enter the authenticated target table"
        );
    }

    #[trace("TC-472", "FR-038-AC-510", "FR-038-AC-511")]
    #[test]
    fn profile_fault_contexts_are_paid_at_exact_and_short_storage() {
        use super::super::stage_storage::Path;
        let node: CheckedNodeId = serde_json::from_value(id('a')).unwrap();
        let body = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(7)
            .key("body");
        for (code, cause, at) in [
            (
                CheckedPackageRefusalCode::UnknownProfile,
                CheckedPackageRefusalCause::UnsupportedSelection,
                body.key("operation").key("laws").index(0).key("definition"),
            ),
            (
                CheckedPackageRefusalCode::UnknownProfile,
                CheckedPackageRefusalCause::WrongSelectionRole,
                body.key("operation").key("laws").index(0).key("definition"),
            ),
            (
                CheckedPackageRefusalCode::InvalidPackage,
                CheckedPackageRefusalCause::OperationMemberMismatch,
                body,
            ),
        ] {
            let run = |limit| {
                let mut budget =
                    super::super::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let allocated = allocation_counter::measure(|| {
                    result = Some(temporal_fault(
                        &mut WorkMeter::over(&mut budget),
                        7,
                        code,
                        cause,
                        at,
                        &node,
                    ));
                });
                (result.unwrap(), allocated.bytes_total, budget)
            };
            let (result, allocated, budget) = run(u64::MAX);
            let ValidationFailure::Refused(refusal) = &result else {
                panic!("profile fault")
            };
            assert_eq!(refusal.code, code);
            assert_eq!(refusal.cause, Some(cause));
            assert_eq!(refusal.loci[0].path.as_ref().unwrap(), &at.build());
            assert_eq!(allocated, budget.read_storage_consumed());
            assert_eq!(run(allocated).0, result);
            let (short, allocated, budget) = run(allocated - 1);
            assert!(matches!(short, ValidationFailure::Budget(_)));
            assert_eq!(allocated, budget.read_storage_consumed());
            let locus = budget.first_unpaid_locus().unwrap();
            assert_eq!(locus.form, "diagnostic");
            assert_eq!(
                locus.source,
                super::super::NativeJoinLocusSource::PackageRead(
                    super::super::PackageReadStage::Graph,
                    7
                )
            );
        }
    }

    fn id(fill: char) -> Value {
        json!({ "package": "test/temporal", "path": [fill.to_string()], "ordinal": 0 })
    }

    #[trace("TC-048", "FR-038-AC-22")]
    #[test]
    fn tc_048_diagnostic_reference_requires_a_closed_term() {
        let target: CheckedNodeId = serde_json::from_value(id('a')).expect("target");
        let refused = |value: &Value| {
            let json = CheckedJson::from_serializable(value).expect("document");
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
        let json = CheckedJson::from_serializable(&value).expect("document");
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
                Some(
                    CheckedTerm::decode_admitted(&node.body.to_value())
                        .expect("closed temporal fixture"),
                )
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
                Some(
                    CheckedTerm::decode_admitted(&node.body.to_value())
                        .expect("closed temporal fixture"),
                )
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
