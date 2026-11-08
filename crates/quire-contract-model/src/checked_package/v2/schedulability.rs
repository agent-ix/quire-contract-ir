// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Authenticated schedulability claim carrier and FR-482 body admission.

use super::graph_storage::{GraphIndex, ResolvedKey, SortedTable};
use super::{
    CheckedLiteralRead, CheckedLiteralValue, CheckedNodeId, CheckedNodeKind, CheckedPackageV2,
    CheckedTerm, ClaimForm, LiteralKind, RawSemanticNodeV2, ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, JsonPointer,
};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// An untrusted producer row naming one local claim occurrence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSchedulabilityClaimRow {
    /// Local claim node identity.
    pub declaration: CheckedNodeId,
    /// Ordinal of its claim occurrence.
    pub occurrence: u64,
}
impl Encode for CheckedSchedulabilityClaimRow {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "declaration", &self.declaration)?;
        super::encode::member(writer, "occurrence", &self.occurrence)?;
        writer.end_object()
    }
}

/// The closed schedulability claim forms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedSchedulabilityKind {
    /// Whole task-set schedulability.
    Schedulable,
    /// One task's response bound.
    Response,
}
/// The closed scheduling policies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedSchedulingPolicy {
    /// Static fixed priority.
    FixedPriority,
    /// Earliest deadline first.
    Edf,
    /// Adaptive mixed criticality.
    Amc,
}
/// The closed task arrival modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedTaskArrival {
    /// Periodic releases.
    Periodic,
    /// Sporadic releases.
    Sporadic,
}
/// The closed task criticalities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedTaskCriticality {
    /// Low criticality.
    Lo,
    /// High criticality.
    Hi,
}

/// A package-borrowed authenticated claim; it cannot be constructed from a wire row.
///
/// ```compile_fail
/// use quire_contract_model::{CheckedSchedulabilityClaimRef, CheckedPackageV2};
/// fn escape(package: CheckedPackageV2) -> CheckedSchedulabilityClaimRef<'static> {
///     package.schedulability_claims().next().unwrap()
/// }
/// ```
///
/// ```compile_fail
/// use quire_contract_model::{CheckedSchedulabilityClaimRef, CheckedNodeId, CheckedOccurrence, CheckedTerm};
/// fn fabricate<'a>(declaration: &'a CheckedNodeId, occurrence: &'a CheckedOccurrence, body: &'a CheckedTerm) {
///     let _ = CheckedSchedulabilityClaimRef { declaration, occurrence, body };
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CheckedSchedulabilityClaimRef<'a> {
    declaration: &'a CheckedNodeId,
    occurrence: &'a CheckedOccurrence,
    body: &'a CheckedTerm,
}
/// A package-borrowed task whose binding shape and policy premises were admitted.
///
/// ```compile_fail
/// use quire_contract_model::{CheckedSchedulabilityTaskRef, CheckedTerm};
/// fn fabricate(body: &CheckedTerm) {
///     let _ = CheckedSchedulabilityTaskRef { body };
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CheckedSchedulabilityTaskRef<'a> {
    body: &'a CheckedTerm,
}

fn members(term: &CheckedTerm) -> Option<&[CheckedTerm]> {
    match term {
        CheckedTerm::Aggregate { members } => Some(members),
        _ => None,
    }
}
fn bound(term: &CheckedTerm, index: usize) -> Option<&CheckedTerm> {
    match members(term)?.get(index)? {
        CheckedTerm::Binding { value, .. } => Some(value),
        _ => None,
    }
}
fn admitted_bound(term: &CheckedTerm, index: usize) -> &CheckedTerm {
    bound(term, index).expect("admission established the ordered binding shape")
}
fn text(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        } => value.as_str(),
        _ => None,
    }
}
fn admitted_text(term: &CheckedTerm) -> &str {
    text(term).expect("admission established the text literal")
}
fn literal(term: &CheckedTerm) -> CheckedLiteralRead<'_> {
    term.literal_read()
        .expect("admission established the exact literal")
}
fn empty(term: &CheckedTerm) -> bool {
    members(term).is_some_and(<[CheckedTerm]>::is_empty)
}
fn kind(term: &CheckedTerm, kind: LiteralKind) -> bool {
    matches!(term, CheckedTerm::Literal { value_kind, .. } if *value_kind == kind)
}
impl<'a> CheckedSchedulabilityClaimRef<'a> {
    /// Exact admitted local declaration identity.
    pub fn declaration(&self) -> &'a CheckedNodeId {
        self.declaration
    }
    /// Exact owned claim occurrence.
    pub fn occurrence(&self) -> &'a CheckedOccurrence {
        self.occurrence
    }
    /// Complete admitted body, borrowed without copying.
    pub fn body(&self) -> &'a CheckedTerm {
        self.body
    }
    /// Closed claim kind.
    #[ix_trace_rs::string_edge]
    pub fn kind(&self) -> CheckedSchedulabilityKind {
        match admitted_text(admitted_bound(self.body, 0)) {
            "schedulable" => CheckedSchedulabilityKind::Schedulable,
            "response" => CheckedSchedulabilityKind::Response,
            _ => unreachable!("admitted claim kind"),
        }
    }
    /// Closed declared policy.
    #[ix_trace_rs::string_edge]
    pub fn policy(&self) -> CheckedSchedulingPolicy {
        match admitted_text(admitted_bound(self.body, 1)) {
            "fixed-priority" => CheckedSchedulingPolicy::FixedPriority,
            "edf" => CheckedSchedulingPolicy::Edf,
            "amc" => CheckedSchedulingPolicy::Amc,
            _ => unreachable!("admitted scheduling policy"),
        }
    }
    /// Response task, present exactly for a response claim.
    pub fn response_task(&self) -> Option<&'a str> {
        text(admitted_bound(self.body, 2))
    }
    /// Exact response bound, present exactly for a response claim.
    pub fn response_bound(&self) -> Option<CheckedLiteralRead<'a>> {
        admitted_bound(self.body, 3).literal_read()
    }
    /// Tasks in the admitted task-name order.
    pub fn tasks(&self) -> impl ExactSizeIterator<Item = CheckedSchedulabilityTaskRef<'a>> + 'a {
        members(admitted_bound(self.body, 4))
            .expect("admitted tasks aggregate")
            .iter()
            .map(|body| CheckedSchedulabilityTaskRef { body })
    }
}
impl<'a> CheckedSchedulabilityTaskRef<'a> {
    /// Exact task name.
    pub fn name(&self) -> &'a str {
        admitted_text(admitted_bound(self.body, 0))
    }
    /// Closed arrival mode.
    #[ix_trace_rs::string_edge]
    pub fn arrival(&self) -> CheckedTaskArrival {
        match admitted_text(admitted_bound(self.body, 1)) {
            "periodic" => CheckedTaskArrival::Periodic,
            "sporadic" => CheckedTaskArrival::Sporadic,
            _ => unreachable!("admitted arrival"),
        }
    }
    /// Closed criticality.
    #[ix_trace_rs::string_edge]
    pub fn criticality(&self) -> CheckedTaskCriticality {
        match admitted_text(admitted_bound(self.body, 7)) {
            "lo" => CheckedTaskCriticality::Lo,
            "hi" => CheckedTaskCriticality::Hi,
            _ => unreachable!("admitted criticality"),
        }
    }
    /// Exact period or minimum inter-arrival time.
    pub fn period(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 2))
    }
    /// Exact relative deadline.
    pub fn deadline(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 3))
    }
    /// Exact best-case work.
    pub fn bcet(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 4))
    }
    /// Exact low-mode worst-case work.
    pub fn wcet(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 5))
    }
    /// Exact high-mode work, present exactly for high-criticality tasks.
    pub fn wcet_hi(&self) -> Option<CheckedLiteralRead<'a>> {
        admitted_bound(self.body, 6).literal_read()
    }
    /// Declared priority, with absence preserved.
    pub fn priority(&self) -> Option<CheckedLiteralRead<'a>> {
        admitted_bound(self.body, 8).literal_read()
    }
    /// Exact release jitter.
    pub fn jitter(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 9))
    }
    /// Exact blocking time.
    pub fn blocking(&self) -> CheckedLiteralRead<'a> {
        literal(admitted_bound(self.body, 10))
    }
}
impl CheckedPackageV2 {
    /// Borrow the retained producer identity rows for lossless complete encoding.
    pub fn schedulability_claim_rows(&self) -> Option<&[CheckedSchedulabilityClaimRow]> {
        self.wire.schedulability_claims.as_deref()
    }
    /// Authenticated claims in canonical index order, after complete admission.
    pub fn schedulability_claims(
        &self,
    ) -> impl ExactSizeIterator<Item = CheckedSchedulabilityClaimRef<'_>> + '_ {
        self.schedulability_claim_index
            .iter()
            .map(|&(node, occurrence)| {
                let node = &self.wire.semantic_graph.nodes[node];
                CheckedSchedulabilityClaimRef {
                    declaration: &node.node_id,
                    occurrence: &node.occurrences[occurrence],
                    body: &node.body,
                }
            })
    }
}

fn invalid(
    code: Code,
    cause: Option<Cause>,
    at: JsonPointer,
    id: &CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, at, cause, id.clone())
}
fn claim(kind: CheckedNodeKind) -> bool {
    kind == CheckedNodeKind::Claim(ClaimForm::Schedulability)
}

pub(super) fn validate_carrier(
    package_name: &str,
    rows: Option<&[CheckedSchedulabilityClaimRow]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<Vec<(usize, usize)>, ValidationFailure> {
    let root = || JsonPointer::root().key("schedulability_claims");
    // Per claim node (ascending position): its claim occurrences as
    // `(ordinal, occurrence position)` sorted, the last of equal ordinals
    // being the one a later insertion would have kept.
    let claims = nodes
        .iter()
        .enumerate()
        .filter(|(p, _)| claim(kinds[*p]))
        .count();
    let mut occurrences: Vec<(usize, Vec<(u64, usize)>)> = meter.vec(claims)?;
    for (p, node) in nodes.iter().enumerate().filter(|(p, _)| claim(kinds[*p])) {
        let held = node
            .occurrences
            .iter()
            .filter(|occurrence| occurrence.role == CheckedOccurrenceRole::Claim)
            .count();
        let mut by_ordinal: Vec<(u64, usize)> = meter.vec(held)?;
        for (position, occurrence) in node.occurrences.iter().enumerate() {
            meter.charge(1, || node_pointer(p).key("occurrences").index(position))?;
            if occurrence.role == CheckedOccurrenceRole::Claim {
                by_ordinal.push((occurrence.ordinal, position));
            }
        }
        by_ordinal.sort_unstable();
        occurrences.push((p, by_ordinal));
    }
    let mut joined = meter.vec(rows.map_or(0, <[CheckedSchedulabilityClaimRow]>::len))?;
    let mut seen: Vec<(ResolvedKey<'_>, u64)> = Vec::new();
    let mut previous: Option<(ResolvedKey<'_>, u64)> = None;
    for (i, row) in rows.unwrap_or_default().iter().enumerate() {
        let at = || root().index(i);
        meter.charge(1, at)?;
        let Some(&position) = index
            .get(&row.declaration)
            .filter(|&&p| row.declaration.package.as_ref() == package_name && claim(kinds[p]))
        else {
            return Err(invalid(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                at().key("declaration"),
                &row.declaration,
            ));
        };
        let Some(&occurrence_position) = occurrences
            .binary_search_by_key(&position, |(held, _)| *held)
            .ok()
            .and_then(|at| {
                let held = &occurrences[at].1;
                let after = held.partition_point(|(ordinal, _)| *ordinal <= row.occurrence);
                held.get(after.checked_sub(1)?)
                    .filter(|(ordinal, _)| *ordinal == row.occurrence)
                    .map(|(_, position)| position)
            })
        else {
            return Err(invalid(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                at().key("occurrence"),
                &row.declaration,
            ));
        };
        let key = (index.resolved_at(position, structural_keys), row.occurrence);
        let slot = match search_keys(&seen, &key, meter)? {
            Ok(_) => {
                return Err(invalid(
                    Code::InvalidPackage,
                    Some(Cause::ConflictingDefinition),
                    at(),
                    &row.declaration,
                ));
            }
            Err(slot) => slot,
        };
        meter.room(&mut seen, 1)?;
        seen.insert(slot, key);
        if let Some(previous) = previous {
            if compare_keys(&previous, &key, meter)? == Ordering::Greater {
                return Err(invalid(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    at(),
                    &row.declaration,
                ));
            }
        }
        previous = Some(key);
        joined.push((position, occurrence_position));
    }
    for (p, node) in nodes.iter().enumerate().filter(|(p, _)| claim(kinds[*p])) {
        for occurrence in node
            .occurrences
            .iter()
            .filter(|o| o.role == CheckedOccurrenceRole::Claim)
        {
            let key = (index.resolved_at(p, structural_keys), occurrence.ordinal);
            if search_keys(&seen, &key, meter)?.is_err() {
                return Err(ValidationFailure::refused_because(
                    Code::InvalidPackage,
                    root(),
                    Cause::InvalidValue,
                ));
            }
        }
    }
    Ok(joined)
}

/// The order of two `(structural key, ordinal)` rows: ranks when the graph
/// has them, else a comparison that charges its frame list.
fn compare_keys(
    left: &(ResolvedKey<'_>, u64),
    right: &(ResolvedKey<'_>, u64),
    meter: &mut WorkMeter,
) -> Result<Ordering, ValidationFailure> {
    Ok(left
        .0
        .order(&right.0, meter)?
        .then_with(|| left.1.cmp(&right.1)))
}

/// Binary search of sorted `rows` for `key` (`Ok` position, or `Err` the
/// insertion slot), each comparison charged.
fn search_keys(
    rows: &[(ResolvedKey<'_>, u64)],
    key: &(ResolvedKey<'_>, u64),
    meter: &mut WorkMeter,
) -> Result<Result<usize, usize>, ValidationFailure> {
    let (mut low, mut high) = (0, rows.len());
    while low < high {
        let middle = low + (high - low) / 2;
        match compare_keys(&rows[middle], key, meter)? {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => high = middle,
            Ordering::Equal => return Ok(Ok(middle)),
        }
    }
    Ok(Err(low))
}

const CLAIM_FIELDS: &[&str] = &["kind", "policy", "task", "bound", "tasks"];
const TASK_FIELDS: &[&str] = &[
    "name",
    "arrival",
    "period",
    "deadline",
    "bcet",
    "wcet",
    "wcet_hi",
    "criticality",
    "priority",
    "jitter",
    "blocking",
];
fn shape(term: &CheckedTerm, fields: &[&str]) -> bool {
    members(term).is_some_and(|members| {
        members.len() == fields.len() && members.iter().zip(fields).all(|(term, expected)| {
        matches!(term, CheckedTerm::Binding { name, .. } if name.as_ref() == *expected)
    })
    })
}
fn binding_at(at: &JsonPointer, i: usize) -> JsonPointer {
    at.clone().key("members").index(i)
}
fn nonnegative(term: &CheckedTerm) -> bool {
    match term.literal_read().map(|read| read.value) {
        Some(CheckedLiteralValue::Integer(value)) => value.sign() != num_bigint::Sign::Minus,
        Some(CheckedLiteralValue::Rational { numerator, .. }) => {
            numerator.sign() != num_bigint::Sign::Minus
        }
        _ => false,
    }
}
fn zero(term: &CheckedTerm) -> bool {
    match term.literal_read().map(|read| read.value) {
        Some(CheckedLiteralValue::Rational { numerator, .. }) => {
            numerator == num_bigint::BigInt::from(0)
        }
        _ => false,
    }
}
fn magnitude(term: &CheckedTerm) -> (&str, &str) {
    let CheckedTerm::Literal { value, .. } = term else {
        unreachable!("admitted quantity")
    };
    let text = value.as_str().expect("admitted rational spelling");
    text.split_once('/').unwrap_or((text, "1"))
}
fn greater(
    left: &CheckedTerm,
    right: &CheckedTerm,
    meter: &mut WorkMeter,
    at: &JsonPointer,
) -> Result<bool, ValidationFailure> {
    Ok(
        super::natural::compare_rationals(magnitude(left), magnitude(right), meter, &|| {
            at.clone()
        })?
        .is_gt(),
    )
}

#[ix_trace_rs::string_edge]
pub(super) fn validate_bodies(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    models: &[super::model_members::DomainModel],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    // Of two models with one identity the later one is kept, as a map built
    // by insertion would hold; the slice address orders them.
    let mut identified = meter.vec::<(&str, &super::model_members::DomainModel)>(models.len())?;
    identified.extend(models.iter().map(|model| (model.identity.as_ref(), model)));
    let models = SortedTable::from_vec(identified, |left, right| {
        std::ptr::from_ref(*right).cmp(&std::ptr::from_ref(*left))
    });
    let claims = index.values().filter(|&&p| claim(kinds[p])).count();
    let mut order: Vec<usize> = meter.vec(claims)?;
    order.extend(index.values().copied().filter(|&p| claim(kinds[p])));
    // Ranks order the keys without a comparison that allocates; positions
    // are unique, so the unstable sort is deterministic.
    order.sort_unstable_by_key(|&p| index.resolved_at(p, structural_keys));
    for p in order {
        let node = &nodes[p];
        if node
            .occurrences
            .iter()
            .any(|occurrence| occurrence.role != CheckedOccurrenceRole::Claim)
        {
            return Err(invalid(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(p).key("occurrences"),
                &node.node_id,
            ));
        }
        let at = node_pointer(p).key("body");
        let fail = |code, cause, at| invalid(code, cause, at, &node.node_id);
        let body = terms[p].as_ref().expect("graph admission decoded the term");
        // Six complete sweeps preserve the normative first-defect order.
        if !shape(body, CLAIM_FIELDS) {
            return Err(fail(Code::InvalidSemanticGraph, None, at));
        }
        let task_at = binding_at(&at, 4);
        let Some(tasks) = members(admitted_bound(body, 4)).filter(|tasks| !tasks.is_empty()) else {
            return Err(fail(Code::InvalidSemanticGraph, None, task_at));
        };
        for (i, task) in tasks.iter().enumerate() {
            if !shape(task, TASK_FIELDS) {
                return Err(fail(
                    Code::InvalidSemanticGraph,
                    None,
                    task_at.clone().key("value").key("members").index(i),
                ));
            }
        }
        for (i, allowed) in [
            (0, &["schedulable", "response"][..]),
            (1, &["fixed-priority", "edf", "amc"][..]),
        ] {
            if !text(admitted_bound(body, i)).is_some_and(|s| allowed.contains(&s)) {
                return Err(fail(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    binding_at(&at, i),
                ));
            }
        }
        for (i, task) in tasks.iter().enumerate() {
            let task_at = task_at.clone().key("value").key("members").index(i);
            for (j, allowed) in [
                (0, &[][..]),
                (1, &["periodic", "sporadic"][..]),
                (7, &["lo", "hi"][..]),
            ] {
                if !text(admitted_bound(task, j))
                    .is_some_and(|s| !s.is_empty() && (allowed.is_empty() || allowed.contains(&s)))
                {
                    return Err(fail(
                        Code::InvalidPackage,
                        Some(Cause::InvalidValue),
                        binding_at(&task_at, j),
                    ));
                }
            }
            let priority = admitted_bound(task, 8);
            if kind(priority, LiteralKind::Integer) && !nonnegative(priority) {
                return Err(fail(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    binding_at(&task_at, 8),
                ));
            }
        }
        let response = text(admitted_bound(body, 0)) == Some("response");
        let policy = admitted_text(admitted_bound(body, 1));
        if response && policy != "fixed-priority" {
            return Err(fail(Code::InvalidSemanticGraph, None, binding_at(&at, 1)));
        }
        for i in [2, 3] {
            let term = admitted_bound(body, i);
            let fits = if response {
                kind(
                    term,
                    if i == 2 {
                        LiteralKind::Text
                    } else {
                        LiteralKind::Quantity
                    },
                )
            } else {
                empty(term)
            };
            if !fits {
                return Err(fail(Code::InvalidSemanticGraph, None, binding_at(&at, i)));
            }
        }
        for (i, task) in tasks.iter().enumerate() {
            let task_at = task_at.clone().key("value").key("members").index(i);
            for j in [2, 3, 4, 5, 6, 8, 9, 10] {
                let term = admitted_bound(task, j);
                let fits = match j {
                    6 if text(admitted_bound(task, 7)) == Some("lo") => empty(term),
                    8 => empty(term) || kind(term, LiteralKind::Integer),
                    _ => kind(term, LiteralKind::Quantity),
                };
                if !fits {
                    return Err(fail(
                        Code::InvalidSemanticGraph,
                        None,
                        binding_at(&task_at, j),
                    ));
                }
            }
        }
        let mut previous = None;
        for task in tasks {
            let name = admitted_text(admitted_bound(task, 0));
            if previous.is_some_and(|prev| prev >= name) {
                return Err(fail(Code::InvalidSemanticGraph, None, task_at.clone()));
            }
            previous = Some(name);
        }
        if response
            && !tasks
                .iter()
                .any(|task| text(admitted_bound(task, 0)) == text(admitted_bound(body, 2)))
        {
            return Err(fail(Code::InvalidSemanticGraph, None, binding_at(&at, 2)));
        }
        let mut common_unit = None;
        let times = (response.then(|| (admitted_bound(body, 3), binding_at(&at, 3).key("value"))))
            .into_iter()
            .chain(tasks.iter().enumerate().flat_map(|(i, task)| {
                let task_at = task_at.clone().key("value").key("members").index(i);
                [2, 3, 4, 5, 6, 9, 10].into_iter().filter_map(move |j| {
                    let term = admitted_bound(task, j);
                    kind(term, LiteralKind::Quantity)
                        .then(|| (term, binding_at(&task_at, j).key("value")))
                })
            }));
        for (time, time_at) in times {
            let read = literal(time);
            let valid = index.get(read.type_id).is_some_and(|&u| {
                kinds[u] == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
                    && index.get(&nodes[u].semantic_type).is_some_and(|&d| {
                        kinds[d] == CheckedNodeKind::ScalarType(ScalarTypeForm::Dimension)
                            && super::timed_model::is_base_time_dimension(&nodes[d], &models)
                    })
            });
            if !valid || common_unit.is_some_and(|unit| unit != read.type_id) {
                return Err(fail(Code::IllTyped, Some(Cause::TypeMismatch), time_at));
            }
            common_unit = Some(read.type_id);
        }
        let mut priorities = Vec::new();
        for (i, task) in tasks.iter().enumerate() {
            let task_at = task_at.clone().key("value").key("members").index(i);
            let b = |j| admitted_bound(task, j);
            let hi = text(b(7)) == Some("hi");
            let positive = [2, 3, 5]
                .into_iter()
                .all(|j| nonnegative(b(j)) && !zero(b(j)));
            let nonnegative_times =
                [4, 9, 10].into_iter().all(|j| nonnegative(b(j))) && (!hi || nonnegative(b(6)));
            let mut valid = positive && nonnegative_times;
            if valid {
                valid = !greater(b(4), b(if hi { 6 } else { 5 }), meter, &task_at)?
                    && (!hi || !greater(b(5), b(6), meter, &task_at)?);
            }
            if policy == "edf" {
                valid &= zero(b(9)) && zero(b(10));
            } else if let Some(read) = b(8).literal_read() {
                if let CheckedLiteralValue::Integer(priority) = read.value {
                    valid &= meter.insert_sorted(&mut priorities, priority)?;
                }
            } else {
                valid = false;
            }
            if valid && policy == "amc" {
                valid = !greater(b(3), b(2), meter, &task_at)?;
            }
            if !valid {
                return Err(fail(
                    Code::InvalidTimedModel,
                    Some(Cause::InvalidTaskSet),
                    task_at,
                ));
            }
        }
    }
    Ok(())
}

/// Graph premises precede body and carrier validation.
pub(super) fn validate_graph(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for &p in index.values() {
        if !claim(kinds[p]) {
            continue;
        }
        let node = &nodes[p];
        for (bad, field) in [
            (!node.dependencies.is_empty(), "dependencies"),
            (node.owner.is_some(), "owner"),
            (
                !index.get(&node.semantic_type).is_some_and(|&t| {
                    kinds[t] == CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean)
                }),
                "semantic_type",
            ),
        ] {
            if bad {
                return Err(invalid(
                    Code::InvalidSemanticGraph,
                    None,
                    node_pointer(p).key(field),
                    &node.node_id,
                ));
            }
        }
        if let Some(body) = terms[p].as_ref() {
            validate_literal_values(body, node_pointer(p).key("body"), &node.node_id, meter)?;
        }
    }
    Ok(())
}

fn validate_literal_values(
    body: &CheckedTerm,
    at: JsonPointer,
    id: &CheckedNodeId,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut pending = vec![(body, at)];
    while let Some((term, at)) = pending.pop() {
        match term {
            CheckedTerm::Literal {
                value_kind: LiteralKind::Quantity,
                value,
                ..
            } => {
                let at = at.key("value");
                let bad = || {
                    invalid(
                        Code::InvalidPackage,
                        Some(Cause::InvalidValue),
                        at.clone(),
                        id,
                    )
                };
                let spelling = value.as_str().ok_or_else(bad)?;
                let (numerator, denominator) = spelling.split_once('/').unwrap_or((spelling, "1"));
                if let Err(failure) =
                    super::identity::validate_rational_parts(numerator, denominator, meter, &|| {
                        at.clone()
                    })
                {
                    return Err(match failure {
                        ValidationFailure::Refused(_) => bad(),
                        incomplete => incomplete,
                    });
                }
            }
            CheckedTerm::Literal {
                value_kind: LiteralKind::Integer,
                value,
                ..
            } => {
                let at = at.key("value");
                let bad = || {
                    invalid(
                        Code::InvalidPackage,
                        Some(Cause::InvalidValue),
                        at.clone(),
                        id,
                    )
                };
                if let Some(spelling) = value.as_str() {
                    if let Err(failure) =
                        super::identity::validate_rational_parts(spelling, "1", meter, &|| {
                            at.clone()
                        })
                    {
                        return Err(match failure {
                            ValidationFailure::Refused(_) => bad(),
                            incomplete => incomplete,
                        });
                    }
                } else if term.literal_read().is_none() {
                    return Err(bad());
                }
            }
            CheckedTerm::Aggregate { members } => {
                for (i, term) in members.iter().enumerate().rev() {
                    pending.push((term, at.clone().key("members").index(i)));
                }
            }
            CheckedTerm::Binding { value, .. } => pending.push((value, at.key("value"))),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NativeJoinBudget;
    use ix_trace_rs::trace;
    use serde_json::json;

    /// The claim carrier join requests every table before it makes it: the
    /// bytes the allocator is asked for equal the bytes charged, the measured
    /// requirement admits and one byte below is denied (FR-038-AC-504). The
    /// carrier holds only flat wire IDs, so the join's comparisons never
    /// allocate.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn claim_carrier_join_allocates_exactly_what_it_charges() {
        let claim = |name: &str, occurrences: u64| -> super::super::RawSemanticNodeV2 {
            serde_json::from_value(json!({
                "node_id":{"package":"example","path":[name],"ordinal":0},
                "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"claim", "semantic_form":"schedulability",
                "semantic_type":{"package":"example","path":["boolean"],"ordinal":0},
                "dependencies":[],
                "occurrences":(0..occurrences)
                    .map(|ordinal| json!({"role":"claim","ordinal":ordinal}))
                    .collect::<Vec<_>>(),
                "body":{"term":"aggregate","members":[]}
            }))
            .expect("claim node")
        };
        let nodes = [claim("a", 2), claim("b", 1), claim("c", 3)];
        let kinds = [CheckedNodeKind::Claim(ClaimForm::Schedulability); 3];
        let index = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect::<GraphIndex<'_>>();
        let keys: Vec<std::sync::Arc<CheckedNodeId>> = nodes
            .iter()
            .map(|node| std::sync::Arc::new(node.node_id.clone()))
            .collect();
        let rows: Vec<CheckedSchedulabilityClaimRow> =
            [(0, 0), (0, 1), (1, 0), (2, 0), (2, 1), (2, 2)]
                .into_iter()
                .map(|(position, occurrence)| CheckedSchedulabilityClaimRow {
                    declaration: nodes[position].node_id.clone(),
                    occurrence,
                })
                .collect();
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut joined = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                joined = Some(validate_carrier(
                    "example",
                    Some(&rows),
                    &nodes,
                    &kinds,
                    &index,
                    &keys,
                    &mut meter,
                ));
            });
            (
                joined.expect("ran").ok(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (joined, allocated, charged) = run(u64::MAX);
        assert_eq!(
            joined,
            Some(vec![(0, 0), (0, 1), (1, 0), (2, 0), (2, 1), (2, 2)])
        );
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert_eq!(run(charged).0, joined, "the measured limit admits");
        assert_eq!(run(charged - 1).0, None, "one byte short is denied");
    }

    /// The model table is one charged exact block the table keeps without
    /// copying: for several models, two sharing an identity, the bytes the
    /// allocator is asked for equal the bytes charged, the measured
    /// requirement admits and one byte below is denied (FR-038-AC-504). The
    /// claim order and priority set are charged the same way and are reached
    /// by the graph-stage exactness fixtures, not by this model-only call.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn model_table_allocates_exactly_what_it_charges() {
        let document = |identity: &str| {
            json!({
                "contractVersion":"2.0.0",
                "package":{"identity":identity,"version":"1.0.0"},
                "constructs":[], "types":[], "dimensions":[], "units":[]
            })
        };
        let unit_refs = std::sync::Arc::new(std::collections::BTreeMap::new());
        let read = |identity: &str| {
            let mut meter = WorkMeter::new(u64::MAX);
            let mut budget = super::super::model_members::Budget::new(&mut meter, 0, 1 << 20);
            super::super::model_members::read_semantic_ir(
                &document(identity),
                "example",
                &mut budget,
                &unit_refs,
            )
            .expect("selected declarations")
        };
        let models = [read("acme/one"), read("acme/two"), read("acme/one")];
        let index = std::iter::empty::<(&CheckedNodeId, usize)>().collect::<GraphIndex<'_>>();
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut outcome = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                outcome = Some(validate_bodies(
                    &[],
                    &[],
                    &[],
                    &index,
                    &[],
                    &models,
                    &mut meter,
                ));
            });
            (
                outcome.expect("ran").ok(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (outcome, allocated, charged) = run(u64::MAX);
        assert_eq!(outcome, Some(()));
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert_eq!(run(charged).0, outcome, "the measured limit admits");
        assert_eq!(run(charged - 1).0, None, "one byte short is denied");
    }
}
