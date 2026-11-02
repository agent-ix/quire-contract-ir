// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Authenticated schedulability claim carrier and FR-482 body admission.

use super::{CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CheckedTerm, ClaimForm,
    CheckedLiteralRead, CheckedLiteralValue, LiteralKind, RawSemanticNodeV2, WorkMeter,
    ScalarTypeForm};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{CheckedOccurrence, CheckedOccurrenceRole,
    CheckedPackageRefusalCode as Code, CheckedPackageRefusalCause as Cause, JsonPointer};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    match term { CheckedTerm::Aggregate { members } => Some(members), _ => None }
}
fn bound(term: &CheckedTerm, index: usize) -> Option<&CheckedTerm> {
    match members(term)?.get(index)? {
        CheckedTerm::Binding { value, .. } => Some(value), _ => None,
    }
}
fn admitted_bound(term: &CheckedTerm, index: usize) -> &CheckedTerm {
    bound(term, index).expect("admission established the ordered binding shape")
}
fn text(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal { value_kind: LiteralKind::Text, value, .. } => value.as_str(),
        _ => None,
    }
}
fn admitted_text(term: &CheckedTerm) -> &str {
    text(term).expect("admission established the text literal")
}
fn literal(term: &CheckedTerm) -> CheckedLiteralRead<'_> {
    term.literal_read().expect("admission established the exact literal")
}
fn empty(term: &CheckedTerm) -> bool { members(term).is_some_and(<[CheckedTerm]>::is_empty) }
fn kind(term: &CheckedTerm, kind: LiteralKind) -> bool {
    matches!(term, CheckedTerm::Literal { value_kind, .. } if *value_kind == kind)
}
impl<'a> CheckedSchedulabilityClaimRef<'a> {
    /// Exact admitted local declaration identity.
    pub fn declaration(&self) -> &'a CheckedNodeId { self.declaration }
    /// Exact owned claim occurrence.
    pub fn occurrence(&self) -> &'a CheckedOccurrence { self.occurrence }
    /// Complete admitted body, borrowed without copying.
    pub fn body(&self) -> &'a CheckedTerm { self.body }
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
    pub fn response_task(&self) -> Option<&'a str> { text(admitted_bound(self.body, 2)) }
    /// Exact response bound, present exactly for a response claim.
    pub fn response_bound(&self) -> Option<CheckedLiteralRead<'a>> {
        admitted_bound(self.body, 3).literal_read()
    }
    /// Tasks in the admitted task-name order.
    pub fn tasks(&self) -> impl ExactSizeIterator<Item = CheckedSchedulabilityTaskRef<'a>> + 'a {
        members(admitted_bound(self.body, 4)).expect("admitted tasks aggregate")
            .iter().map(|body| CheckedSchedulabilityTaskRef { body })
    }
}
impl<'a> CheckedSchedulabilityTaskRef<'a> {
    /// Exact task name.
    pub fn name(&self) -> &'a str { admitted_text(admitted_bound(self.body, 0)) }
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
    pub fn period(&self) -> CheckedLiteralRead<'a> { literal(admitted_bound(self.body, 2)) }
    /// Exact relative deadline.
    pub fn deadline(&self) -> CheckedLiteralRead<'a> { literal(admitted_bound(self.body, 3)) }
    /// Exact best-case work.
    pub fn bcet(&self) -> CheckedLiteralRead<'a> { literal(admitted_bound(self.body, 4)) }
    /// Exact low-mode worst-case work.
    pub fn wcet(&self) -> CheckedLiteralRead<'a> { literal(admitted_bound(self.body, 5)) }
    /// Exact high-mode work, present exactly for high-criticality tasks.
    pub fn wcet_hi(&self) -> Option<CheckedLiteralRead<'a>> { admitted_bound(self.body, 6).literal_read() }
    /// Declared priority, with absence preserved.
    pub fn priority(&self) -> Option<CheckedLiteralRead<'a>> { admitted_bound(self.body, 8).literal_read() }
    /// Exact release jitter.
    pub fn jitter(&self) -> CheckedLiteralRead<'a> { literal(admitted_bound(self.body, 9)) }
    /// Exact blocking time.
    pub fn blocking(&self) -> CheckedLiteralRead<'a> { literal(admitted_bound(self.body, 10)) }
}
impl CheckedPackageV2 {
    /// Borrow the retained producer identity rows for lossless complete encoding.
    pub fn schedulability_claim_rows(&self) -> Option<&[CheckedSchedulabilityClaimRow]> {
        self.wire.schedulability_claims.as_deref()
    }
    /// Authenticated claims in canonical index order, after complete admission.
    pub fn schedulability_claims(&self) -> impl ExactSizeIterator<Item = CheckedSchedulabilityClaimRef<'_>> + '_ {
        self.wire.schedulability_claims.as_deref().unwrap_or_default().iter().map(|row| {
            let position = self.node_index[&row.declaration];
            let node = &self.wire.semantic_graph.nodes[position];
            let occurrence = node.occurrences.iter().find(|occurrence| {
                occurrence.role == CheckedOccurrenceRole::Claim && occurrence.ordinal == row.occurrence
            }).expect("admitted occurrence join");
            CheckedSchedulabilityClaimRef { declaration: &node.node_id, occurrence, body: &node.body }
        })
    }
}

fn invalid(code: Code, cause: Option<Cause>, at: JsonPointer, id: &CheckedNodeId) -> ValidationFailure {
    ValidationFailure::refused_at(code, at, cause, id.clone())
}
fn claim(kind: CheckedNodeKind) -> bool { kind == CheckedNodeKind::Claim(ClaimForm::Schedulability) }

pub(super) fn validate_carrier(
    package_name: &str, rows: Option<&[CheckedSchedulabilityClaimRow]>, nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind], index: &BTreeMap<&CheckedNodeId, usize>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>], meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let root = JsonPointer::root().key("schedulability_claims");
    let mut seen = BTreeSet::new();
    let mut previous = None;
    for (i, row) in rows.unwrap_or_default().iter().enumerate() {
        let at = root.clone().index(i);
        meter.charge(1, || at.clone())?;
        let Some(&position) = index.get(&row.declaration).filter(|&&p| row.declaration.package.as_ref() == package_name && claim(kinds[p])) else {
            return Err(invalid(Code::InvalidPackage, Some(Cause::InvalidValue), at.key("declaration"), &row.declaration));
        };
        if !nodes[position].occurrences.iter().any(|o| o.role == CheckedOccurrenceRole::Claim && o.ordinal == row.occurrence) {
            return Err(invalid(Code::InvalidPackage, Some(Cause::InvalidValue), at.key("occurrence"), &row.declaration));
        }
        let key = (structural_keys[position].as_ref(), row.occurrence);
        if !seen.insert(key) {
            return Err(invalid(Code::InvalidPackage, Some(Cause::ConflictingDefinition), at, &row.declaration));
        }
        if previous.is_some_and(|previous| previous > key) {
            return Err(invalid(Code::InvalidPackage, Some(Cause::InvalidValue), at, &row.declaration));
        }
        previous = Some(key);
    }
    let expected = nodes.iter().enumerate().filter(|(p, _)| claim(kinds[*p])).flat_map(|(p, node)| {
        node.occurrences.iter().filter(|o| o.role == CheckedOccurrenceRole::Claim)
            .map(move |o| (structural_keys[p].as_ref(), o.ordinal))
    });
    if expected.clone().any(|key| !seen.contains(&key)) {
        return Err(ValidationFailure::refused_because(Code::InvalidPackage, root, Cause::InvalidValue));
    }
    Ok(())
}

const CLAIM_FIELDS: &[&str] = &["kind", "policy", "task", "bound", "tasks"];
const TASK_FIELDS: &[&str] = &["name", "arrival", "period", "deadline", "bcet", "wcet", "wcet_hi", "criticality", "priority", "jitter", "blocking"];
fn shape(term: &CheckedTerm, fields: &[&str]) -> bool {
    members(term).is_some_and(|members| members.len() == fields.len() && members.iter().zip(fields).all(|(term, expected)| {
        matches!(term, CheckedTerm::Binding { name, .. } if name.as_ref() == *expected)
    }))
}
fn binding_at(at: &JsonPointer, i: usize) -> JsonPointer { at.clone().key("members").index(i) }
fn nonnegative(term: &CheckedTerm) -> bool {
    match literal(term).value {
        CheckedLiteralValue::Integer(value) => value.sign() != num_bigint::Sign::Minus,
        CheckedLiteralValue::Rational { numerator, .. } => numerator.sign() != num_bigint::Sign::Minus,
        _ => false,
    }
}
fn zero(term: &CheckedTerm) -> bool {
    match literal(term).value {
        CheckedLiteralValue::Rational { numerator, .. } => numerator == num_bigint::BigInt::from(0),
        _ => false,
    }
}
fn magnitude(term: &CheckedTerm) -> (&str, &str) {
    let CheckedTerm::Literal { value, .. } = term else { unreachable!("admitted quantity") };
    let text = value.as_str().expect("admitted rational spelling");
    text.split_once('/').unwrap_or((text, "1"))
}
fn greater(left: &CheckedTerm, right: &CheckedTerm, meter: &mut WorkMeter, at: &JsonPointer) -> Result<bool, ValidationFailure> {
    Ok(super::natural::compare_rationals(magnitude(left), magnitude(right), meter, &|| at.clone())?.is_gt())
}

#[ix_trace_rs::string_edge]
pub(super) fn validate_bodies(
    nodes: &[RawSemanticNodeV2], kinds: &[CheckedNodeKind], terms: &[Option<CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>, structural_keys: &[std::sync::Arc<CheckedNodeId>], models: &[super::model_members::DomainModel], meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let models = models.iter().map(|model| (model.identity.as_ref(), model)).collect();
    let mut order = index.values().copied().collect::<Vec<_>>();
    order.sort_by(|&a, &b| structural_keys[a].cmp(&structural_keys[b]));
    for p in order {
        if !claim(kinds[p]) { continue; }
        let node = &nodes[p];
        let at = node_pointer(p).key("body");
        let fail = |code, cause, at| invalid(code, cause, at, &node.node_id);
        let body = terms[p].as_ref().expect("graph admission decoded the term");
        // Six complete sweeps preserve the normative first-defect order.
        if !shape(body, CLAIM_FIELDS) { return Err(fail(Code::InvalidSemanticGraph, None, at)); }
        let task_at = binding_at(&at, 4);
        let Some(tasks) = members(admitted_bound(body, 4)).filter(|tasks| !tasks.is_empty()) else {
            return Err(fail(Code::InvalidSemanticGraph, None, task_at));
        };
        for (i, task) in tasks.iter().enumerate() {
            if !shape(task, TASK_FIELDS) { return Err(fail(Code::InvalidSemanticGraph, None, task_at.clone().key("value").key("members").index(i))); }
        }
        for (i, allowed) in [(0, &["schedulable", "response"][..]), (1, &["fixed-priority", "edf", "amc"][..])] {
            if !text(admitted_bound(body, i)).is_some_and(|s| allowed.contains(&s)) {
                return Err(fail(Code::InvalidPackage, Some(Cause::InvalidValue), binding_at(&at, i)));
            }
        }
        for (i, task) in tasks.iter().enumerate() {
            let task_at = task_at.clone().key("value").key("members").index(i);
            for (j, allowed) in [(0, &[][..]), (1, &["periodic", "sporadic"][..]), (7, &["lo", "hi"][..])] {
                if !text(admitted_bound(task, j)).is_some_and(|s| !s.is_empty() && (allowed.is_empty() || allowed.contains(&s))) {
                    return Err(fail(Code::InvalidPackage, Some(Cause::InvalidValue), binding_at(&task_at, j)));
                }
            }
            let priority = admitted_bound(task, 8);
            if kind(priority, LiteralKind::Integer) && !nonnegative(priority) {
                return Err(fail(Code::InvalidPackage, Some(Cause::InvalidValue), binding_at(&task_at, 8)));
            }
        }
        let response = text(admitted_bound(body, 0)) == Some("response");
        let policy = admitted_text(admitted_bound(body, 1));
        if response && policy != "fixed-priority" { return Err(fail(Code::InvalidSemanticGraph, None, binding_at(&at, 1))); }
        for i in [2, 3] {
            let term = admitted_bound(body, i);
            let fits = if response { kind(term, if i == 2 { LiteralKind::Text } else { LiteralKind::Quantity }) } else { empty(term) };
            if !fits { return Err(fail(Code::InvalidSemanticGraph, None, binding_at(&at, i))); }
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
                if !fits { return Err(fail(Code::InvalidSemanticGraph, None, binding_at(&task_at, j))); }
            }
        }
        let mut previous = None;
        for task in tasks {
            let name = admitted_text(admitted_bound(task, 0));
            if previous.is_some_and(|prev| prev >= name) { return Err(fail(Code::InvalidSemanticGraph, None, task_at.clone())); }
            previous = Some(name);
        }
        if response && !tasks.iter().any(|task| text(admitted_bound(task, 0)) == text(admitted_bound(body, 2))) {
            return Err(fail(Code::InvalidSemanticGraph, None, binding_at(&at, 2)));
        }
        let mut common_unit = None;
        let times = (response.then(|| (admitted_bound(body, 3), binding_at(&at, 3).key("value")))).into_iter()
            .chain(tasks.iter().enumerate().flat_map(|(i, task)| {
                let task_at = task_at.clone().key("value").key("members").index(i);
                [2, 3, 4, 5, 6, 9, 10].into_iter().filter_map(move |j| {
                    let term = admitted_bound(task, j);
                    kind(term, LiteralKind::Quantity).then(|| (term, binding_at(&task_at, j).key("value")))
                })
            }));
        for (time, time_at) in times {
            let read = literal(time);
            let valid = index.get(read.type_id).is_some_and(|&u| {
                kinds[u] == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit) && index.get(&nodes[u].semantic_type).is_some_and(|&d| {
                    kinds[d] == CheckedNodeKind::ScalarType(ScalarTypeForm::Dimension) && super::timed_model::is_base_time_dimension(&nodes[d], &models)
                })
            });
            if !valid || common_unit.is_some_and(|unit| unit != read.type_id) {
                return Err(fail(Code::IllTyped, Some(Cause::TypeMismatch), time_at));
            }
            common_unit = Some(read.type_id);
        }
        let mut priorities = BTreeSet::new();
        for (i, task) in tasks.iter().enumerate() {
            let task_at = task_at.clone().key("value").key("members").index(i);
            let b = |j| admitted_bound(task, j);
            let hi = text(b(7)) == Some("hi");
            let positive = [2, 3, 5].into_iter().all(|j| nonnegative(b(j)) && !zero(b(j)));
            let nonnegative_times = [4, 9, 10].into_iter().all(|j| nonnegative(b(j))) && (!hi || nonnegative(b(6)));
            let mut valid = positive && nonnegative_times;
            if valid {
                valid = !greater(b(4), b(if hi {6} else {5}), meter, &task_at)? && (!hi || !greater(b(5), b(6), meter, &task_at)?);
            }
            if policy == "edf" { valid &= zero(b(9)) && zero(b(10)); }
            else if let Some(read) = b(8).literal_read() {
                if let CheckedLiteralValue::Integer(priority) = read.value { valid &= priorities.insert(priority); }
            } else { valid = false; }
            if valid && policy == "amc" { valid = !greater(b(3), b(2), meter, &task_at)?; }
            if !valid { return Err(fail(Code::InvalidTimedModel, Some(Cause::InvalidTaskSet), task_at)); }
        }
    }
    Ok(())
}

/// Graph premises precede body and carrier validation.
pub(super) fn validate_graph(nodes: &[RawSemanticNodeV2], kinds: &[CheckedNodeKind], index: &BTreeMap<&CheckedNodeId, usize>) -> Result<(), ValidationFailure> {
    for &p in index.values() {
        if !claim(kinds[p]) { continue; }
        let node = &nodes[p];
        for (bad, field) in [
            (!node.dependencies.is_empty(), "dependencies"),
            (node.owner.is_some(), "owner"),
            (!index.get(&node.semantic_type).is_some_and(|&t| kinds[t] == CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean)), "semantic_type"),
        ] {
            if bad { return Err(invalid(Code::InvalidSemanticGraph, None, node_pointer(p).key(field), &node.node_id)); }
        }
    }
    Ok(())
}
