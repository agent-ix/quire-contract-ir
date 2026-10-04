//! The temporal step of the reader order (QSpec FR-370 "Reader order"): after
//! the state step and before any operation refusal, the reader checks
//!
//! 1. the placement of every application of the three temporal classes that
//!    stand only at one node form's body root (`temporal`, `temporal_formula`
//!    and `temporal_fairness`; `case` is placed by the operation step), and of every `reference` to a
//!    `temporal`/`formula` or `temporal`/`fairness` node, over the whole graph in
//!    ascending node-id digest order;
//! 2. then each `temporal`/`temporal_clause` node in ascending node-id digest
//!    order, each taken through its checks before the next clause is read:
//!    the profile identity (`unknown_profile`), the `over` reference, each
//!    fairness member's resolution, the profile fit of every interval and of the
//!    fairness argument, and the interval bounds,
//!
//! and reports the first defect. Operand family and count and `result_type`
//! stay the operation step's. The step reads only a well-formed member: a
//! member the operation step refuses as `operation-member-mismatch` is skipped
//! here.
//!
//! The module also owns the `temporal_interval` and `fairness` member shapes
//! ([`member_is_well_formed`]) and the unbounded-integer comparison of interval
//! bounds ([`IntegerString`]), which the operation step and the term walk read.

use super::frame::StepGraph;
use super::identity::is_identifier;
use super::model_members::{MemberKind, ModelOwners, ModelRefusal, Resolved};
use super::operation_catalog::operation_catalog;
use super::operations::OperationWire;
use super::structural::reference_target;
use super::{
    ApplicationOperator, BodyTerm, CheckedNodeKind, CheckedPackageLockV2, CheckedSelectionRole,
    CheckedSemanticNodeV2, FairnessGranularity, FairnessKind, IntervalFit, LawRole, ModelForm,
    OperationMemberKind, TemporalForm, TemporalProfile, ValueForm, WorkMeter,
};
use crate::checked_package::common::{
    application_operator, body_term, node_pointer, ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    JsonPointer,
};
use serde::Deserialize;
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

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
pub(in crate::checked_package) fn is_placed_class(operator: ApplicationOperator) -> bool {
    PLACED.iter().any(|(class, _)| *class == operator)
}

/// The pointer a diagnostics entry's `details` term is refused at, or `None`
/// when it is admitted. A `details` term is no node's body, so an application
/// of a temporal class in it, as its root or nested, is refused at that
/// application's `operator`, the first in pre-order (a `case` application there
/// was refused earlier, by the term walk, at the same pointer: it is no body
/// root); and a `reference` to a
/// node `refused_target` names (a `temporal`/`formula`, `temporal`/`fairness`
/// or `expression`/`case` node) is refused at the entry, `at`, the pointer of
/// `term` itself. A reference to a union or union value node is an ordinary one
/// (merged QSpec FR-370 and FR-440: a `details` term may reference a
/// `composite_type`/`union` or `value`/`union_value` node).
pub(in crate::checked_package) fn misplaced_in_details(
    term: &Value,
    at: JsonPointer,
    refused_target: impl Fn(&CheckedNodeId) -> bool,
) -> Option<JsonPointer> {
    let entry = at.clone();
    let mut pending = vec![(term, at)];
    while let Some((term, at)) = pending.pop() {
        let (member, children): (&str, Vec<&Value>) = match body_term(term) {
            Some(BodyTerm::Application) => {
                if application_operator(term).is_some_and(is_placed_class) {
                    return Some(at.key("operator"));
                }
                ("arguments", terms(term, "arguments").collect())
            }
            Some(BodyTerm::Aggregate) => ("members", terms(term, "members").collect()),
            Some(BodyTerm::Binding) => ("value", term.get("value").into_iter().collect()),
            Some(BodyTerm::Reference) => {
                if reference_target(term).is_some_and(|target| refused_target(&target)) {
                    return Some(entry);
                }
                continue;
            }
            Some(
                BodyTerm::Literal
                | BodyTerm::DependencyReference
                | BodyTerm::Frame
                | BodyTerm::AbstractionRelation,
            )
            | None => continue,
        };
        let below = at.key(member);
        for (child_at, child) in children.into_iter().enumerate().rev() {
            // A binding's one value sits directly under `value`; the others
            // are array elements.
            let pointer = if member == "value" {
                below.clone()
            } else {
                below.clone().index(child_at)
            };
            pending.push((child, pointer));
        }
    }
    None
}

/// An integer string: the schema's `IntegerString`, `^(0|-?[1-9][0-9]*)$`.
/// Ordered as the integer it spells, of unbounded size: by sign, then digit
/// count, then digits, never as text and never through a fixed-width integer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::checked_package) struct IntegerString {
    negative: bool,
    /// The magnitude's digits, with no leading zero unless the magnitude is `0`.
    digits: Box<str>,
}

impl IntegerString {
    /// The integer `text` spells, `None` when it is not an `IntegerString`.
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
            digits: digits.into(),
        })
    }

    fn magnitude_cmp(&self, other: &Self) -> Ordering {
        self.digits
            .len()
            .cmp(&other.digits.len())
            .then_with(|| self.digits.cmp(&other.digits))
    }
}

impl Ord for IntegerString {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => self.magnitude_cmp(other),
            (true, true) => other.magnitude_cmp(self),
        }
    }
}

impl PartialOrd for IntegerString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
}

/// The interval of a `{kind: temporal_interval, interval}` member; `None` for
/// any other shape: a `null` member, another kind, a third member, an
/// `interval` that is neither `null` nor exactly `{lower, upper}`, or a bound
/// that is not an integer string.
fn read_interval_member(member: &Value) -> Option<Interval> {
    let object = member.as_object()?;
    let kind = object.get("kind")?.as_str()?;
    if object.len() != 2
        || OperationMemberKind::from_wire(kind) != Some(OperationMemberKind::TemporalInterval)
    {
        return None;
    }
    match object.get("interval")? {
        Value::Null => Some(Interval::Unbounded),
        Value::Object(interval) if interval.len() == 2 => {
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
        _ => None,
    }
}

/// The name of the first of `lower` and `upper`, in member order, of a
/// `temporal_interval` member of an application's `operation` that lies outside
/// the schema's non-negative integer-string pattern `^(0|[1-9][0-9]*)$`: a
/// negative bound, a malformed one (`"1.5"`, `"01"`, `"+1"`, `""`, `"3x"`) and a
/// bound that is no string. An `upper` of `null` is the lower-bounded form, not a
/// bound. The term walk refuses it `invalid-value` at that bound, in the early
/// stage, never `operation-member-mismatch` (merged QSpec FR-370).
pub(in crate::checked_package) fn interval_bound_outside_pattern(
    operation: &Value,
) -> Option<&'static str> {
    let member = operation.get("member")?;
    let kind = member.get("kind")?.as_str()?;
    if OperationMemberKind::from_wire(kind) != Some(OperationMemberKind::TemporalInterval) {
        return None;
    }
    let interval = member.get("interval")?.as_object()?;
    ["lower", "upper"].into_iter().find(|bound| {
        let Some(value) = interval.get(*bound) else {
            // A missing bound is a defect of the member's shape.
            return false;
        };
        if *bound == "upper" && value.is_null() {
            return false;
        }
        !value
            .as_str()
            .and_then(IntegerString::parse)
            .is_some_and(|bound| !bound.negative)
    })
}

/// A `{kind: fairness, fairness_kind, granularity, declaration, name}`
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
    declaration: CheckedNodeId,
    name: Box<str>,
}

/// The fairness member `member` is, `None` for any other shape.
fn read_fairness_member(member: &Value) -> Option<FairnessMember> {
    let member = FairnessMember::deserialize(member).ok()?;
    (member.kind == OperationMemberKind::Fairness && is_identifier(&member.name)).then_some(member)
}

/// Whether `member` has the closed shape its catalogued member `kind`
/// requires; the other member kinds are checked elsewhere.
pub(in crate::checked_package) fn member_is_well_formed(
    kind: OperationMemberKind,
    member: &Value,
) -> bool {
    match kind {
        OperationMemberKind::TemporalInterval => read_interval_member(member).is_some(),
        OperationMemberKind::Fairness => read_fairness_member(member).is_some(),
        OperationMemberKind::Field
        | OperationMemberKind::Position
        | OperationMemberKind::Element
        | OperationMemberKind::RelationshipEnd
        | OperationMemberKind::Operation
        | OperationMemberKind::TypeArgument
        | OperationMemberKind::ProfileOperator
        | OperationMemberKind::StateClause => true,
    }
}

/// The temporal step. See the module documentation for the order.
pub(super) fn validate_temporal(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    owners: &ModelOwners<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StepGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    for &position in index.values() {
        if let Some(failure) = placement_defect(position, &graph) {
            return Err(failure);
        }
    }
    let mut covered = BTreeSet::new();
    for &position in index.values() {
        if graph.kinds.get(position)
            == Some(&CheckedNodeKind::Temporal(TemporalForm::TemporalClause))
        {
            check_clause(position, &graph, lock, meter, &mut covered)?;
        }
    }
    // A formula node no clause reaches is no clause's to fit, but its
    // interval is an interval all the same.
    for &position in index.values() {
        if graph.kinds.get(position) == Some(&CheckedNodeKind::Temporal(TemporalForm::Formula))
            && !covered.contains(&position)
        {
            if let Some(failure) = interval_of(position, &graph)
                .and_then(|interval| bounds_defect(position, &interval, &graph))
            {
                return Err(failure);
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

fn terms<'a>(term: &'a Value, member: &str) -> impl Iterator<Item = &'a Value> {
    term.get(member)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// The placement defect of one node: its body root against its form, any
/// application of a placed class nested in its body, and any `reference` to a
/// formula or fairness node from a place that may not name one.
fn placement_defect(position: usize, graph: &StepGraph<'_, '_>) -> Option<ValidationFailure> {
    let node = graph.nodes.get(position)?;
    let kind = *graph.kinds.get(position)?;
    let root_class = (body_term(&node.body) == Some(BodyTerm::Application))
        .then(|| application_operator(&node.body))
        .flatten();
    let required = PLACED
        .iter()
        .find(|(_, form)| *form == kind)
        .map(|(class, _)| *class);
    match (required, root_class) {
        (Some(required), Some(root)) if root == required => {}
        (Some(_), _) => return Some(ineligible(graph, position)),
        (None, Some(root)) if is_placed_class(root) => return Some(ineligible(graph, position)),
        (None, _) => {}
    }
    let mut pending = vec![(&node.body, true, Slot::Other)];
    while let Some((term, is_root, slot)) = pending.pop() {
        match body_term(term) {
            Some(BodyTerm::Application) => {
                if !is_root && application_operator(term).is_some_and(is_placed_class) {
                    return Some(ineligible(graph, position));
                }
                let arguments: Vec<&Value> = terms(term, "arguments").collect();
                for (at, argument) in arguments.into_iter().enumerate().rev() {
                    let slot = if is_root {
                        argument_slot(kind, at)
                    } else {
                        Slot::Other
                    };
                    pending.push((argument, false, slot));
                }
            }
            Some(BodyTerm::Aggregate) => {
                let member = if slot == Slot::FairnessList {
                    Slot::Fairness
                } else {
                    Slot::Other
                };
                pending
                    .extend(terms(term, "members").map(|member_term| (member_term, false, member)));
            }
            Some(BodyTerm::Binding) => {
                pending.extend(term.get("value").map(|value| (value, false, Slot::Other)));
            }
            Some(BodyTerm::Reference) => {
                let target = reference_target(term).and_then(|target| graph.node(&target));
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
                    return Some(ineligible(graph, position));
                }
            }
            Some(
                BodyTerm::Literal
                | BodyTerm::DependencyReference
                | BodyTerm::Frame
                | BodyTerm::AbstractionRelation,
            )
            | None => {}
        }
    }
    None
}

/// The wire `operation` of the application at the root of `node`'s body.
fn operation_of(node: &CheckedSemanticNodeV2) -> Option<OperationWire> {
    OperationWire::deserialize(node.body.get("operation")?).ok()
}

/// The interval of the formula node at `position`: the member of a
/// catalogued interval operator, when it is well formed.
fn interval_of(position: usize, graph: &StepGraph<'_, '_>) -> Option<Interval> {
    let operation = operation_of(graph.nodes.get(position)?)?;
    let entry = operation_catalog().entry(&operation.identity)?;
    if entry.operator != ApplicationOperator::TemporalFormula
        || entry.member != Some(OperationMemberKind::TemporalInterval)
    {
        return None;
    }
    read_interval_member(operation.member.as_ref()?)
}

/// `invalid_package`/`invalid-value` at the application when a closed interval
/// has `lower > upper`.
fn bounds_defect(
    position: usize,
    interval: &Interval,
    graph: &StepGraph<'_, '_>,
) -> Option<ValidationFailure> {
    let Interval::Closed { lower, upper } = interval else {
        return None;
    };
    (lower > upper).then(|| {
        ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidPackage,
            node_pointer(position).key("body"),
            Some(CheckedPackageRefusalCause::InvalidValue),
            graph.nodes[position].node_id.clone(),
        )
    })
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
/// resolution, profile fit, interval bounds. A part of the clause the
/// operation step refuses (an unreadable `operation`, a missing law, a
/// missing argument) is skipped here.
fn check_clause(
    position: usize,
    graph: &StepGraph<'_, '_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
    covered: &mut BTreeSet<usize>,
) -> Result<(), ValidationFailure> {
    let node = &graph.nodes[position];
    let Some(operation) = operation_of(node) else {
        return Ok(());
    };
    if operation_catalog()
        .entry(&operation.identity)
        .map(|entry| entry.operator)
        != Some(ApplicationOperator::Temporal)
    {
        return Ok(());
    }
    let at_node = node_pointer(position);
    let at_body = at_node.clone().key("body");
    let arguments: Vec<&Value> = terms(&node.body, "arguments").collect();

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
    if let Some(over) = arguments.first().copied().and_then(reference_target) {
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
    let fairness: Vec<&Value> = arguments
        .get(4)
        .into_iter()
        .flat_map(|aggregate| terms(aggregate, "members"))
        .collect();
    for reference in &fairness {
        let Some((fair, member)) = fairness_member(reference, graph) else {
            continue;
        };
        if let Err(failure) = resolve_fairness(&member, graph, meter)? {
            return Err(ValidationFailure::refused_at(
                failure.refusal.code,
                node_pointer(fair)
                    .key("body")
                    .key("operation")
                    .key("member")
                    .key(failure.member),
                Some(failure.refusal.cause),
                if failure.locus_is_target {
                    member.declaration.clone()
                } else {
                    graph.nodes[fair].node_id.clone()
                },
            ));
        }
    }

    // Profile fit of every interval, then of the fairness argument, then the
    // bounds of every interval.
    let intervals = formula_intervals(arguments.get(5).copied(), graph, meter, &at_node, covered)?;
    if let Some(profile) = profile {
        let fit = profile.interval_fit();
        for (formula, interval) in &intervals {
            if !interval_fits(fit, interval) {
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
        if let Some(failure) = bounds_defect(*formula, interval, graph) {
            return Err(failure);
        }
    }
    Ok(())
}

fn interval_fits(fit: IntervalFit, interval: &Interval) -> bool {
    match (fit, interval) {
        (IntervalFit::Any, _)
        | (IntervalFit::NullOnly, Interval::Unbounded)
        | (IntervalFit::ClosedOnly, Interval::Closed { .. }) => true,
        (IntervalFit::NullOnly, Interval::LowerBounded | Interval::Closed { .. })
        | (IntervalFit::ClosedOnly, Interval::Unbounded | Interval::LowerBounded) => false,
    }
}

/// The well-formed fairness member of the `temporal`/`fairness` node a clause's
/// fairness argument entry references, with that node's position.
fn fairness_member(entry: &Value, graph: &StepGraph<'_, '_>) -> Option<(usize, FairnessMember)> {
    let target = reference_target(entry)?;
    let position = *graph.index.get(&target)?;
    if graph.kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::Fairness)) {
        return None;
    }
    let member = read_fairness_member(
        graph
            .nodes
            .get(position)?
            .body
            .get("operation")?
            .get("member")?,
    )?;
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
    member: &FairnessMember,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<Result<(), FairnessRefusal>, ValidationFailure> {
    // A `declaration` that names no node is a target that is no
    // `model`/`object_type` node (merged FR-370 "Fairness resolution" step 1),
    // located at the key as named.
    let Some((declaring, kind)) = graph.node(&member.declaration) else {
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
            .resolve_member(declaring, MemberKind::Operation, &member.name, meter)?
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
fn formula_intervals(
    root: Option<&Value>,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
    at: &JsonPointer,
    covered: &mut BTreeSet<usize>,
) -> Result<Vec<(usize, Interval)>, ValidationFailure> {
    let mut pending: Vec<usize> = root
        .and_then(reference_target)
        .and_then(|target| graph.index.get(&target).copied())
        .into_iter()
        .collect();
    let mut seen = BTreeSet::new();
    let mut intervals = Vec::new();
    while let Some(position) = pending.pop() {
        if graph.kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::Formula))
            || !seen.insert(position)
        {
            continue;
        }
        meter.charge(1, || at.clone())?;
        covered.insert(position);
        if let Some(interval) = interval_of(position, graph) {
            intervals.push((position, interval));
        }
        let operands: Vec<usize> = terms(&graph.nodes[position].body, "arguments")
            .filter_map(reference_target)
            .filter_map(|target| graph.index.get(&target).copied())
            .collect();
        pending.extend(operands.into_iter().rev());
    }
    Ok(intervals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::common::NODE_DOMAIN;
    use crate::checked_package::shared::CheckedSelection;
    use crate::checked_package::v2::CheckedNodeTag;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn id(fill: char) -> Value {
        json!({ "domain": NODE_DOMAIN, "digest": fill.to_string().repeat(64) })
    }

    /// FR-038-AC-97: an interval bound is an integer string of unbounded size,
    /// ordered as the integer it spells and never as text.
    ///
    /// Tracing: TC-048, FR-038-AC-97
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
            model_selections: Vec::new(),
            required_features: Vec::new(),
            dependency_selections: Vec::new(),
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
        let kinds: Vec<CheckedNodeKind> = nodes
            .iter()
            .map(|node| {
                let tag = CheckedNodeTag::from_wire(&node.node_tag).expect("test node tag");
                CheckedNodeKind::decode(tag, &node.semantic_form).expect("test node form")
            })
            .collect();
        let index: BTreeMap<&CheckedNodeId, usize> = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        validate_temporal(
            nodes,
            &kinds,
            &index,
            &ModelOwners::default(),
            &lock(),
            &mut WorkMeter::new(1_000),
        )
    }

    /// FR-038-AC-103: an `over` that names no declared dependency of its clause
    /// refuses `missing_declaration`/`missing-name` at the clause node, and one
    /// that does admits. (A package read cannot reach this: the application
    /// dependency join makes every referenced node a dependency first.)
    ///
    /// Tracing: TC-048, FR-038-AC-103
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
            refusal.path.as_ref().map(JsonPointer::as_str),
            Some("/semantic_graph/nodes/0/body/arguments/0")
        );
        // The path is on the argument and the locus is the target (a node that
        // is no declared dependency).
        assert_eq!(refusal.locus.as_ref(), Some(&nodes[1].node_id));
    }
}
