// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Closed tick-monitor admission and package-borrowed reads (FR-038-AC-421..429).

use super::model_members::DomainModel;
use super::native_join_storage as storage;
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageV2,
    CheckedRational, CheckedTerm, ClockConfiguration, CompositeTypeForm, FunctionForm, LiteralKind,
    NativeJoinBudget, NativeJoinChargeKind, RawSemanticNodeV2, ScalarTypeForm, TemporalForm,
    ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, JsonPointer,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[allow(dead_code)] // The complete selected closure reader will call this prerequisite.
mod selected_closure_walk;

const ENTRY_SLOTS: [usize; 4] = [5, 6, 7, 15];

const NAMES: [&str; 17] = [
    "clause",
    "state",
    "inputs",
    "scope",
    "results",
    "init",
    "on_event",
    "on_watermark",
    "counter_width_bits",
    "max_reading_gap",
    "max_event_rate",
    "buffers",
    "counter",
    "tick_unit",
    "uncertainty",
    "on_gap",
    "activation_capacity",
];

/// Candidate entry references are collected without deciding root shape. The
/// plan step owns that refusal, even when a named candidate is malformed.
pub(super) fn entry_candidates(nodes: &[RawSemanticNodeV2]) -> BTreeSet<CheckedNodeId> {
    let mut candidates = BTreeSet::new();
    for node in nodes {
        if super::CheckedNodeTag::from_wire(&node.node_tag)
            .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
            != Some(CheckedNodeKind::Temporal(TemporalForm::MonitorPlan))
        {
            continue;
        }
        let Some(super::CheckedTermView::Aggregate { members }) = CheckedTerm::view(&node.body)
        else {
            continue;
        };
        for member in members {
            if let Some(super::CheckedTermView::Binding { name, value }) = CheckedTerm::view(member)
            {
                if ENTRY_SLOTS.iter().any(|&slot| NAMES[slot] == name) {
                    if let Some(target) = super::structural::reference_target(value) {
                        candidates.insert(target);
                    }
                }
            }
        }
    }
    candidates
}

/// The ordinary executable body, independent of owner/model clause descriptors.
pub(super) fn anonymous_body_shape(
    body: &serde_json::Value,
    nodes: &BTreeMap<&CheckedNodeId, &RawSemanticNodeV2>,
) -> bool {
    let Some(super::CheckedTermView::Aggregate { members }) = CheckedTerm::view(body) else {
        return false;
    };
    let (parameters, body, measure) = match members {
        [parameters, body] => (parameters, body, None),
        [parameters, body, measure] => (parameters, body, Some(measure)),
        _ => return false,
    };
    let Some(super::CheckedTermView::Binding {
        name: "parameters",
        value,
    }) = CheckedTerm::view(parameters)
    else {
        return false;
    };
    let Some(super::CheckedTermView::Aggregate { members }) = CheckedTerm::view(value) else {
        return false;
    };
    let kind = |term: &serde_json::Value| {
        let id = super::structural::reference_target(term)?;
        let node = nodes.get(&id)?;
        super::CheckedNodeTag::from_wire(&node.node_tag)
            .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
    };
    members
        .iter()
        .all(|parameter| kind(parameter) == Some(CheckedNodeKind::Value(ValueForm::Parameter)))
        && matches!(CheckedTerm::view(body), Some(super::CheckedTermView::Binding { name: "body", value })
            if matches!(kind(value), Some(CheckedNodeKind::Expression(_))))
        && measure.is_none_or(|measure| matches!(CheckedTerm::view(measure), Some(super::CheckedTermView::Binding {name:"measure",value}) if kind(value).is_some()))
}

#[derive(Clone)]
pub(super) struct Plan {
    position: usize,
    references: [usize; 10],
    entries: [Entry; 4],
    width: u8,
    gap: u64,
    rate: CheckedRational,
    uncertainty: CheckedRational,
    buffers: Vec<Buffer>,
    activation_capacity: Box<str>,
}
#[derive(Clone)]
struct Entry {
    position: usize,
    parameters: Vec<usize>,
    result: usize,
}
#[derive(Clone)]
struct Buffer {
    subformula: usize,
    capacity: Box<str>,
}

/// An immutable monitor plan borrowing its selected admitted package.
///
/// ```compile_fail
/// use quire_contract_model::{CheckedPackageV2, CheckedMonitorPlanRef};
/// fn escape(package: CheckedPackageV2) -> CheckedMonitorPlanRef<'static> {
///     package.monitor_plans().next().unwrap()
/// }
/// ```
/// ```compile_fail
/// use quire_contract_model::CheckedMonitorPlanRef;
/// fn fabricate(view: CheckedMonitorPlanRef<'_>) {
///     let _ = CheckedMonitorPlanRef { ..view };
/// }
/// ```
///
/// ```compile_fail
/// use quire_contract_model::{CheckedMonitorPlanRef, CheckedTerm};
/// let _: CheckedMonitorPlanRef<'_> = CheckedTerm::aggregate(vec![]).into();
/// ```
#[derive(Clone, Copy)]
pub struct CheckedMonitorPlanRef<'a> {
    package: &'a CheckedPackageV2,
    plan: &'a Plan,
}
/// One anonymous, typed pure-function entry of an admitted monitor plan.
///
/// ```compile_fail
/// use quire_contract_model::{CheckedPackageV2, CheckedMonitorEntryRef};
/// fn escape(package: CheckedPackageV2) -> CheckedMonitorEntryRef<'static> {
///     package.monitor_plans().next().unwrap().on_gap()
/// }
/// ```
///
/// ```compile_fail
/// use quire_contract_model::CheckedMonitorEntryRef;
/// fn fabricate(view: CheckedMonitorEntryRef<'_>) {
///     let _ = CheckedMonitorEntryRef { ..view };
/// }
/// ```
///
/// ```compile_fail
/// use quire_contract_model::{CheckedMonitorEntryRef, CheckedTerm};
/// let _: CheckedMonitorEntryRef<'_> = CheckedTerm::aggregate(vec![]).into();
/// ```
#[derive(Clone, Copy)]
pub struct CheckedMonitorEntryRef<'a> {
    package: &'a CheckedPackageV2,
    plan: &'a CheckedNodeId,
    entry: &'a Entry,
}
/// One admitted buffer premise, with no host-width capacity restriction.
///
/// ```compile_fail
/// use quire_contract_model::{CheckedPackageV2, CheckedMonitorBufferRef};
/// fn escape(package: CheckedPackageV2) -> CheckedMonitorBufferRef<'static> {
///     package.monitor_plans().next().unwrap().buffers().next().unwrap()
/// }
/// ```
///
/// ```compile_fail
/// use quire_contract_model::CheckedMonitorBufferRef;
/// fn fabricate(view: CheckedMonitorBufferRef<'_>) {
///     let _ = CheckedMonitorBufferRef { ..view };
/// }
/// ```
///
/// ```compile_fail
/// use quire_contract_model::{CheckedMonitorBufferRef, CheckedTerm};
/// let _: CheckedMonitorBufferRef<'_> = CheckedTerm::aggregate(vec![]).into();
/// ```
#[derive(Clone, Copy)]
pub struct CheckedMonitorBufferRef<'a> {
    package: &'a CheckedPackageV2,
    buffer: &'a Buffer,
}

/// An admitted clock and claim name for one selected monitor clause occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedMonitorClockRead<'a> {
    root: &'a CheckedNodeId,
    clause: &'a CheckedNodeId,
    occurrence: CheckedOccurrence,
    clock: &'a ClockConfiguration,
    clock_name: &'a str,
    claim_name: &'a str,
    work: u64,
}

impl<'a> CheckedMonitorClockRead<'a> {
    /// The selected admitted monitor root.
    pub fn root(&self) -> &'a CheckedNodeId {
        self.root
    }
    /// The root's admitted temporal clause.
    pub fn clause(&self) -> &'a CheckedNodeId {
        self.clause
    }
    /// The exact selected claim occurrence.
    pub fn occurrence(&self) -> &CheckedOccurrence {
        &self.occurrence
    }
    /// The admitted closed clock configuration.
    pub fn clock(&self) -> &'a ClockConfiguration {
        self.clock
    }
    /// The clause's exact admitted clock name.
    pub fn clock_name(&self) -> &'a str {
        self.clock_name
    }
    /// The selected claim's original admitted name.
    pub fn claim_name(&self) -> &'a str {
        self.claim_name
    }
    /// Work charged by this read.
    pub fn work(&self) -> u64 {
        self.work
    }
}

/// Stable cause of a selected monitor clock refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedMonitorClockReadErrorCode {
    /// The selected clause is not this monitor root's clause.
    WrongClause,
    /// The selected occurrence is not a claim.
    WrongRole,
    /// The clause has no such admitted claim occurrence.
    UnknownOccurrence,
    /// The admitted claim has no clock binding.
    MissingClock,
    /// The next inspection exceeds the caller's allowance.
    WorkLimit,
    /// The next owned payload exceeds the caller's storage allowance.
    StorageLimit,
}

/// Typed first refusal, without a partial clock view.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("selected monitor clock refused: {code:?} at {at:?}")]
pub struct CheckedMonitorClockReadError {
    /// Stable cause.
    pub code: CheckedMonitorClockReadErrorCode,
    /// First selected clause or inspected row node.
    pub at: Option<Box<CheckedNodeId>>,
    /// Exact requested occurrence.
    pub occurrence: Option<CheckedOccurrence>,
    /// Work charged before refusal.
    pub work: u64,
    /// First refused charge, zero for semantic refusal.
    pub attempted_work: u64,
}

struct ClockBudget<'a> {
    occurrence: &'a CheckedOccurrence,
    budget: &'a mut NativeJoinBudget,
    start_work: u64,
}

impl ClockBudget<'_> {
    fn error(
        &mut self,
        code: CheckedMonitorClockReadErrorCode,
        at: &CheckedNodeId,
    ) -> CheckedMonitorClockReadError {
        let work = self.budget.consumed().work - self.start_work;
        let units = storage::node(at).saturating_add(1);
        if !self.budget.charge(NativeJoinChargeKind::Storage, units) {
            return CheckedMonitorClockReadError {
                code: CheckedMonitorClockReadErrorCode::StorageLimit,
                at: None,
                occurrence: None,
                work,
                attempted_work: 0,
            };
        }
        CheckedMonitorClockReadError {
            code,
            at: Some(Box::new(at.clone())),
            occurrence: Some(self.occurrence.clone()),
            work,
            attempted_work: 0,
        }
    }

    fn charge(&mut self, at: &CheckedNodeId) -> Result<(), CheckedMonitorClockReadError> {
        if !self.budget.charge(NativeJoinChargeKind::Work, 1) {
            let mut error = self.error(CheckedMonitorClockReadErrorCode::WorkLimit, at);
            error.attempted_work = 1;
            return Err(error);
        }
        Ok(())
    }

    fn storage(&mut self, amount: u128) -> Result<(), CheckedMonitorClockReadError> {
        if self.budget.charge(NativeJoinChargeKind::Storage, amount) {
            Ok(())
        } else {
            Err(CheckedMonitorClockReadError {
                code: CheckedMonitorClockReadErrorCode::StorageLimit,
                at: None,
                occurrence: None,
                work: self.budget.consumed().work - self.start_work,
                attempted_work: 0,
            })
        }
    }
}

fn selected_row<'a, T>(
    package: &CheckedPackageV2,
    rows: &'a [T],
    clause: &CheckedNodeId,
    ordinal: u64,
    key: impl Fn(&T) -> (&CheckedNodeId, u64),
    budget: &mut ClockBudget<'_>,
) -> Result<Option<&'a T>, CheckedMonitorClockReadError> {
    let selected_key = package
        .structural_key(clause)
        .expect("admitted monitor clause key");
    let mut low = 0;
    let mut high = rows.len();
    while low < high {
        let middle = low + (high - low) / 2;
        let row = &rows[middle];
        let (id, row_ordinal) = key(row);
        budget.charge(id)?;
        let row_key = package
            .structural_key(id)
            .expect("admitted temporal row key");
        match (row_key, row_ordinal).cmp(&(selected_key, ordinal)) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
            std::cmp::Ordering::Equal => return Ok(Some(row)),
        }
    }
    Ok(None)
}

impl CheckedPackageV2 {
    /// Admitted monitor roots, in ascending resolved structural-key order.
    pub fn monitor_plans(&self) -> impl ExactSizeIterator<Item = CheckedMonitorPlanRef<'_>> {
        self.monitor_plan_index
            .iter()
            .map(|plan| CheckedMonitorPlanRef {
                package: self,
                plan,
            })
    }
}
impl<'a> CheckedMonitorPlanRef<'a> {
    fn id(&self, slot: usize) -> &'a CheckedNodeId {
        &self.package.graph().nodes[self.plan.references[slot]].node_id
    }
    fn entry(&self, slot: usize) -> CheckedMonitorEntryRef<'a> {
        CheckedMonitorEntryRef {
            package: self.package,
            plan: self.node_id(),
            entry: &self.plan.entries[slot],
        }
    }
    /// The root's graph-local node ID.
    pub fn node_id(&self) -> &'a CheckedNodeId {
        &self.package.graph().nodes[self.plan.position].node_id
    }
    /// The resolved document-independent content key.
    pub fn structural_key(&self) -> &'a CheckedNodeId {
        self.package
            .structural_key_at(self.plan.position)
            .expect("admitted root key")
    }
    /// The selected temporal clause.
    pub fn clause(&self) -> &'a CheckedNodeId {
        self.id(0)
    }
    /// Join one independently selected claim occurrence to this root's admitted clock and name.
    ///
    /// The caller supplies a final-emission clause ID and its full occurrence;
    /// this read neither chooses a claim nor executes the monitor.
    pub fn clock_for_claim(
        &self,
        clause: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
        native_budget: &mut NativeJoinBudget,
    ) -> Result<CheckedMonitorClockRead<'a>, CheckedMonitorClockReadError> {
        let before = native_budget.begin();
        let mut budget = ClockBudget {
            occurrence,
            start_work: before.work,
            budget: native_budget,
        };
        budget.charge(self.node_id())?;
        budget.charge(clause)?;
        if clause != self.clause() {
            return Err(budget.error(CheckedMonitorClockReadErrorCode::WrongClause, clause));
        }
        if occurrence.role != CheckedOccurrenceRole::Claim {
            return Err(budget.error(CheckedMonitorClockReadErrorCode::WrongRole, clause));
        }
        let admitted = self.package.node(clause).expect("admitted monitor clause");
        let mut found = false;
        for candidate in &admitted.occurrences {
            budget.charge(clause)?;
            if candidate == occurrence {
                found = true;
                break;
            }
        }
        if !found {
            return Err(budget.error(CheckedMonitorClockReadErrorCode::UnknownOccurrence, clause));
        }
        let binding = selected_row(
            self.package,
            self.package.temporal_bindings().unwrap_or_default(),
            clause,
            occurrence.ordinal,
            |row| (&row.declaration, row.occurrence),
            &mut budget,
        )?
        .ok_or_else(|| budget.error(CheckedMonitorClockReadErrorCode::MissingClock, clause))?;
        let name = selected_row(
            self.package,
            self.package.temporal_clause_names().unwrap_or_default(),
            clause,
            occurrence.ordinal,
            |row| (&row.declaration, row.occurrence),
            &mut budget,
        )?
        .expect("strict admission requires a name for every claim");
        budget.storage(1)?;
        let work = budget.budget.consumed().work - before.work;
        budget.budget.finish(before);
        Ok(CheckedMonitorClockRead {
            root: self.node_id(),
            clause: self.clause(),
            occurrence: occurrence.clone(),
            clock: &binding.clock,
            clock_name: &binding.clock_name,
            claim_name: &name.name,
            work,
        })
    }
    /// The monitor State record type.
    pub fn state(&self) -> &'a CheckedNodeId {
        self.id(1)
    }
    /// The event Inputs record type.
    pub fn inputs(&self) -> &'a CheckedNodeId {
        self.id(2)
    }
    /// The progress Scope record type.
    pub fn scope(&self) -> &'a CheckedNodeId {
        self.id(3)
    }
    /// The Results sequence type.
    pub fn results(&self) -> &'a CheckedNodeId {
        self.id(4)
    }
    /// Complete root body, without execution.
    pub fn body(&self) -> &'a CheckedTerm {
        &self.package.terms[self.plan.position]
    }
    /// The state initializer.
    pub fn init(&self) -> CheckedMonitorEntryRef<'a> {
        self.entry(0)
    }
    /// The event transition.
    pub fn on_event(&self) -> CheckedMonitorEntryRef<'a> {
        self.entry(1)
    }
    /// The progress transition.
    pub fn on_watermark(&self) -> CheckedMonitorEntryRef<'a> {
        self.entry(2)
    }
    /// The checked reading-gap transition.
    pub fn on_gap(&self) -> CheckedMonitorEntryRef<'a> {
        self.entry(3)
    }
    /// Counter width, admitted in 1..=64.
    pub fn counter_width_bits(&self) -> u8 {
        self.plan.width
    }
    /// Maximum modular gap admitted under the counter width.
    pub fn max_reading_gap(&self) -> u64 {
        self.plan.gap
    }
    /// Exact positive canonical events per tick.
    pub fn max_event_rate(&self) -> &'a CheckedRational {
        &self.plan.rate
    }
    /// Exact nonnegative canonical tick uncertainty.
    pub fn uncertainty(&self) -> &'a CheckedRational {
        &self.plan.uncertainty
    }
    /// Exact nonempty counter name.
    pub fn counter(&self) -> &'a str {
        text(bound(self.body(), 12).expect("admitted counter binding")).expect("admitted text")
    }
    /// The tick unit of the exact base Time dimension.
    pub fn tick_unit(&self) -> &'a CheckedNodeId {
        self.id(8)
    }
    /// Exact nonnegative canonical activation capacity, without narrowing.
    pub fn activation_capacity(&self) -> &'a str {
        &self.plan.activation_capacity
    }
    /// Buffer premises in admitted subformula order.
    pub fn buffers(&self) -> impl ExactSizeIterator<Item = CheckedMonitorBufferRef<'a>> + 'a {
        self.plan
            .buffers
            .iter()
            .map(|buffer| CheckedMonitorBufferRef {
                package: self.package,
                buffer,
            })
    }
}
impl<'a> CheckedMonitorEntryRef<'a> {
    /// The anonymous function's graph-local ID.
    pub fn node_id(&self) -> &'a CheckedNodeId {
        &self.package.graph().nodes[self.entry.position].node_id
    }
    /// The function's resolved content key.
    pub fn structural_key(&self) -> &'a CheckedNodeId {
        self.package
            .structural_key_at(self.entry.position)
            .expect("admitted entry key")
    }
    /// The entry's exact admitted result type.
    pub fn result_type(&self) -> &'a CheckedNodeId {
        &self.package.graph().nodes[self.entry.result].node_id
    }
    /// Complete ordinary function body, retained without execution.
    pub fn body(&self) -> &'a CheckedTerm {
        &self.package.terms[self.entry.position]
    }
    /// Read this admitted entry's ordered parameter handles and expression body.
    ///
    /// The read charges only this entry's body and references. Its handles do
    /// not claim complete type closure, and no expression is executed.
    pub fn callable(
        &self,
        work_limit: u64,
    ) -> Result<super::CheckedCallableRead, super::CheckedCallableReadError> {
        super::callable::monitor_entry_callable(self.package, self.plan, self.node_id(), work_limit)
    }
    /// Exact parameter types in signature order.
    pub fn parameter_types(&self) -> impl ExactSizeIterator<Item = &'a CheckedNodeId> + 'a {
        self.entry
            .parameters
            .iter()
            .map(|&position| &self.package.graph().nodes[position].node_id)
    }
}
impl<'a> CheckedMonitorBufferRef<'a> {
    /// The buffered subformula's exact node ID.
    pub fn subformula(&self) -> &'a CheckedNodeId {
        &self.package.graph().nodes[self.buffer.subformula].node_id
    }
    /// Exact canonical positive decimal capacity, without numeric narrowing.
    pub fn capacity(&self) -> &'a str {
        &self.buffer.capacity
    }
}

fn members(term: &CheckedTerm) -> Option<&[CheckedTerm]> {
    match term {
        CheckedTerm::Aggregate { members } => Some(members),
        _ => None,
    }
}
fn named<'a>(term: &'a CheckedTerm, expected: &str) -> Option<&'a CheckedTerm> {
    match term {
        CheckedTerm::Binding { name, value } if name.as_ref() == expected => Some(value),
        _ => None,
    }
}
fn bound(term: &CheckedTerm, slot: usize) -> Option<&CheckedTerm> {
    match members(term)?.get(slot)? {
        CheckedTerm::Binding { value, .. } => Some(value),
        _ => None,
    }
}
fn reference(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}
fn text(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal { value, .. } => value.as_str(),
        _ => None,
    }
}
fn literal(term: &CheckedTerm, kind: LiteralKind) -> bool {
    matches!(term, CheckedTerm::Literal { value_kind, .. } if *value_kind == kind)
}
fn integer(term: &CheckedTerm) -> Option<Box<str>> {
    match term {
        CheckedTerm::Literal { value, .. } => match value {
            serde_json::Value::String(s) => Some(s.as_str().into()),
            serde_json::Value::Number(n) => Some(n.to_string().into()),
            _ => None,
        },
        _ => None,
    }
}
fn positive(text: &str) -> bool {
    matches!(text.as_bytes(), [first, rest @ ..] if (b'1'..=b'9').contains(first) && rest.iter().all(u8::is_ascii_digit))
}
#[ix_trace_rs::string_edge]
fn natural(text: &str) -> bool {
    text == "0" || positive(text)
}

struct Graph<'a> {
    nodes: &'a [RawSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    keys: &'a [Arc<CheckedNodeId>],
}
impl Graph<'_> {
    fn term(&self, position: usize) -> Option<&CheckedTerm> {
        self.terms.get(position)?.as_ref()
    }
    fn kind(&self, position: usize, kind: CheckedNodeKind) -> bool {
        self.kinds.get(position) == Some(&kind)
    }
    fn resolve(&self, term: &CheckedTerm) -> Option<usize> {
        self.index.get(reference(term)?).copied()
    }
    fn scalar(&self, position: usize, form: ScalarTypeForm) -> bool {
        self.kind(position, CheckedNodeKind::ScalarType(form))
    }
    fn composite(&self, position: usize, form: CompositeTypeForm) -> bool {
        self.kind(position, CheckedNodeKind::CompositeType(form))
    }
    fn fields(&self, position: usize) -> Option<Vec<(&str, usize)>> {
        members(self.term(position)?)?
            .iter()
            .map(|member| {
                let CheckedTerm::Binding { name, value } = member else {
                    return None;
                };
                Some((name.as_ref(), self.resolve(value)?))
            })
            .collect()
    }
    fn positional(&self, position: usize) -> Option<Vec<usize>> {
        members(self.term(position)?)?
            .iter()
            .map(|member| self.resolve(member))
            .collect()
    }
    fn parameters(&self, position: usize) -> Option<Vec<usize>> {
        let (parameters, body, measure) = match members(self.term(position)?)? {
            [parameters, body] => (parameters, body, None),
            [parameters, body, measure] => (parameters, body, Some(measure)),
            _ => return None,
        };
        if let Some(measure) = measure {
            self.resolve(named(measure, "measure")?)?;
        }
        let parameters = members(named(parameters, "parameters")?)?;
        let expression = self.resolve(named(body, "body")?)?;
        if !matches!(
            self.kinds.get(expression),
            Some(CheckedNodeKind::Expression(_))
        ) {
            return None;
        }
        parameters
            .iter()
            .map(|parameter| {
                let p = self.resolve(parameter)?;
                self.kind(p, CheckedNodeKind::Value(ValueForm::Parameter))
                    .then_some(())?;
                self.index.get(&self.nodes.get(p)?.semantic_type).copied()
            })
            .collect()
    }
}

struct Site<'a> {
    position: usize,
    id: &'a CheckedNodeId,
}
impl Site<'_> {
    fn at(&self, slot: usize) -> JsonPointer {
        node_pointer(self.position)
            .key("body")
            .key("members")
            .index(slot)
    }
    fn refusal(&self, code: Code, cause: Option<Cause>, at: JsonPointer) -> ValidationFailure {
        ValidationFailure::refused_at(code, at, cause, self.id.clone())
    }
    fn shape(&self, slot: usize) -> ValidationFailure {
        self.refusal(Code::InvalidSemanticGraph, None, self.at(slot))
    }
    fn typed(&self, slot: usize) -> ValidationFailure {
        self.refusal(Code::IllTyped, Some(Cause::TypeMismatch), self.at(slot))
    }
    fn premise(&self, at: JsonPointer) -> ValidationFailure {
        self.refusal(Code::InvalidPackage, Some(Cause::InvalidValue), at)
    }
}

#[ix_trace_rs::string_edge]
fn rational(
    term: &CheckedTerm,
    allow_zero: bool,
    site: &Site<'_>,
    slot: usize,
    meter: &mut WorkMeter,
) -> Result<CheckedRational, ValidationFailure> {
    let at = site.at(slot).key("value");
    let raw = text(term).ok_or_else(|| site.premise(at.clone()))?;
    let (numerator, denominator) = raw.split_once('/').unwrap_or((raw, "1"));
    if !(if allow_zero {
        natural(numerator)
    } else {
        positive(numerator)
    }) || !positive(denominator)
        || (raw.contains('/') && denominator == "1")
    {
        return Err(site.premise(at));
    }
    super::identity::validate_rational_parts(numerator, denominator, meter, &|| at.clone())
        .map_err(|failure| match failure {
            ValidationFailure::Refused(_) => site.premise(at.clone()),
            other => other,
        })?;
    Ok(CheckedRational {
        numerator: numerator.into(),
        denominator: denominator.into(),
    })
}

#[ix_trace_rs::string_edge]
fn outcome(
    graph: &Graph<'_>,
    results: usize,
    meter: &mut WorkMeter,
    site: &Site<'_>,
) -> Result<bool, ValidationFailure> {
    let valid = || -> Option<bool> {
        let elements = graph.positional(results)?;
        let [result] = elements.as_slice() else {
            return Some(false);
        };
        if !graph.composite(*result, CompositeTypeForm::Record) {
            return Some(false);
        }
        let fields = graph.fields(*result)?;
        let [("instance", instance), ("outcome", outcome)] = fields.as_slice() else {
            return Some(false);
        };
        if !graph.composite(*instance, CompositeTypeForm::Record)
            || !graph.composite(*outcome, CompositeTypeForm::Union)
        {
            return Some(false);
        }
        let instance_fields = graph.fields(*instance)?;
        let [("subject", subject), ("subformula", subformula), ("origin", origin)] =
            instance_fields.as_slice()
        else {
            return Some(false);
        };
        if !graph.scalar(*subject, ScalarTypeForm::String)
            || !graph.scalar(*subformula, ScalarTypeForm::Integer)
            || !graph.scalar(*origin, ScalarTypeForm::Integer)
        {
            return Some(false);
        }
        let cases = super::structural::typed_union_type_body(graph.term(*outcome)?)?;
        if cases.len() != 6 {
            return Some(false);
        }
        for (case, expected) in cases.iter().zip([
            "Failed",
            "False",
            "Incomplete",
            "Indeterminate",
            "Pending",
            "True",
        ]) {
            if case.name != expected {
                return Some(false);
            }
            if matches!(expected, "Failed" | "Incomplete") {
                let [payload] = case.payload.as_slice() else {
                    return Some(false);
                };
                let p = *graph.index.get(payload)?;
                if !graph.composite(p, CompositeTypeForm::Record) {
                    return Some(false);
                }
                let fields = graph.fields(p)?;
                if expected == "Failed" {
                    let [("gap", gap), ("max_reading_gap", max)] = fields.as_slice() else {
                        return Some(false);
                    };
                    if !graph.scalar(*gap, ScalarTypeForm::Integer)
                        || !graph.scalar(*max, ScalarTypeForm::Integer)
                    {
                        return Some(false);
                    }
                } else {
                    let [("limit", limit), ("value", value), ("setting", setting)] =
                        fields.as_slice()
                    else {
                        return Some(false);
                    };
                    if !graph.scalar(*limit, ScalarTypeForm::String)
                        || !graph.scalar(*value, ScalarTypeForm::Integer)
                        || !graph.scalar(*setting, ScalarTypeForm::String)
                    {
                        return Some(false);
                    }
                }
            } else if !case.payload.is_empty() {
                return Some(false);
            }
        }
        Some(true)
    };
    meter.charge(1, || site.at(4))?;
    Ok(valid().unwrap_or(false))
}

fn activation_type(
    graph: &Graph<'_>,
    sequence: usize,
    meter: &mut WorkMeter,
    site: &Site<'_>,
) -> Result<bool, ValidationFailure> {
    meter.charge(1, || site.at(1))?;
    let valid = || -> Option<bool> {
        if !graph.composite(sequence, CompositeTypeForm::Sequence) {
            return Some(false);
        }
        let elements = graph.positional(sequence)?;
        let [record] = elements.as_slice() else {
            return Some(false);
        };
        if !graph.composite(*record, CompositeTypeForm::Record) {
            return Some(false);
        }
        let fields = graph.fields(*record)?;
        let [("subformula", subformula), ("origin", origin), ("status", _)] = fields.as_slice()
        else {
            return Some(false);
        };
        Some(
            graph.scalar(*subformula, ScalarTypeForm::Integer)
                && graph.scalar(*origin, ScalarTypeForm::Integer),
        )
    };
    Ok(valid().unwrap_or(false))
}

#[ix_trace_rs::string_edge]
pub(super) fn validate(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    keys: &[Arc<CheckedNodeId>],
    models: &[DomainModel],
    meter: &mut WorkMeter,
) -> Result<Vec<Plan>, ValidationFailure> {
    let graph = Graph {
        nodes,
        kinds,
        terms,
        index,
        keys,
    };
    let model_index = models
        .iter()
        .map(|model| (model.identity.as_ref(), model))
        .collect::<BTreeMap<_, _>>();
    let mut positions = index
        .values()
        .copied()
        .filter(|&p| graph.kind(p, CheckedNodeKind::Temporal(TemporalForm::MonitorPlan)))
        .collect::<Vec<_>>();
    positions.sort_by(|&a, &b| keys[a].cmp(&keys[b]));
    let mut plans = Vec::with_capacity(positions.len());
    for position in positions {
        let node = &nodes[position];
        let site = Site {
            position,
            id: &node.node_id,
        };
        meter.charge(1, || node_pointer(position).key("body"))?;
        let body = graph.term(position).ok_or_else(|| site.shape(0))?;
        let body_members = members(body).ok_or_else(|| site.shape(0))?;
        let mut values = Vec::with_capacity(NAMES.len());
        // Complete shape precedes all reference-kind and signature checks.
        for (slot, name) in NAMES.iter().enumerate() {
            meter.charge(1, || site.at(slot))?;
            let value = body_members
                .get(slot)
                .and_then(|member| named(member, name))
                .ok_or_else(|| site.shape(slot))?;
            let valid = match slot {
                8 | 9 | 16 => literal(value, LiteralKind::Integer),
                10 | 14 => literal(value, LiteralKind::Rational),
                12 => literal(value, LiteralKind::Text),
                11 => {
                    let buffers = members(value).ok_or_else(|| site.shape(slot))?;
                    for (i, buffer) in buffers.iter().enumerate() {
                        meter.charge(1, || site.at(slot).key("value").key("members").index(i))?;
                        if !matches!(members(buffer), Some([subformula, capacity]) if named(subformula, "subformula").and_then(reference).is_some()
                            && named(capacity, "capacity").is_some_and(|t| literal(t, LiteralKind::Integer)))
                        {
                            return Err(site.shape(slot));
                        }
                    }
                    true
                }
                _ => reference(value).is_some(),
            };
            if !valid {
                return Err(site.shape(slot));
            }
            values.push(value);
        }
        if body_members.len() != NAMES.len() {
            return Err(site.shape(NAMES.len()));
        }
        let slots = [0, 1, 2, 3, 4, 5, 6, 7, 13, 15];
        let mut references = [0; 10];
        let mut buffers = Vec::new();
        for (i, slot) in slots.into_iter().enumerate() {
            if slot == 13 {
                for (i, buffer) in members(values[11])
                    .expect("checked buffers")
                    .iter()
                    .enumerate()
                {
                    meter.charge(1, || site.at(11).key("value").key("members").index(i))?;
                    let subformula = graph
                        .resolve(bound(buffer, 0).expect("checked buffer"))
                        .filter(|&p| {
                            graph.kind(p, CheckedNodeKind::Temporal(TemporalForm::Formula))
                        })
                        .ok_or_else(|| site.shape(11))?;
                    buffers.push(Buffer {
                        subformula,
                        capacity: integer(bound(buffer, 1).expect("checked capacity"))
                            .ok_or_else(|| site.shape(11))?,
                    });
                }
            }
            meter.charge(1, || site.at(slot))?;
            let p = graph
                .resolve(values[slot])
                .ok_or_else(|| site.shape(slot))?;
            let valid = match slot {
                0 => graph.kind(p, CheckedNodeKind::Temporal(TemporalForm::TemporalClause)),
                1..=3 => graph.composite(p, CompositeTypeForm::Record),
                4 => graph.composite(p, CompositeTypeForm::Sequence),
                5..=7 | 15 => {
                    graph.kind(p, CheckedNodeKind::Function(FunctionForm::PureFunction))
                        && nodes[p].declaration.is_none()
                }
                13 => {
                    graph.scalar(p, ScalarTypeForm::Unit) && {
                        // Nominal admission has joined this exact dimension for
                        // both source units and selected model-owned units.
                        let dimension = index.get(&nodes[p].semantic_type).copied();
                        dimension.is_some_and(|d| {
                            graph.scalar(d, ScalarTypeForm::Dimension)
                                && super::timed_model::is_base_time_dimension(
                                    &nodes[d],
                                    &model_index,
                                )
                        })
                    }
                }
                _ => unreachable!("closed reference slots"),
            };
            if !valid {
                return Err(site.shape(slot));
            }
            references[i] = p;
        }
        let state = references[1];
        if node.semantic_type != nodes[state].node_id {
            return Err(site.refusal(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(position).key("semantic_type"),
            ));
        }
        let mut entries = Vec::with_capacity(4);
        for (entry_index, slot) in ENTRY_SLOTS.into_iter().enumerate() {
            meter.charge(1, || site.at(slot))?;
            let p = references[if entry_index == 3 { 9 } else { 5 + entry_index }];
            let parameters = graph.parameters(p).ok_or_else(|| site.typed(slot))?;
            let result = index
                .get(&nodes[p].semantic_type)
                .copied()
                .ok_or_else(|| site.typed(slot))?;
            let signature = match entry_index {
                0 => parameters.is_empty() && result == state,
                1 | 2 => {
                    matches!(parameters.as_slice(), [s, delta, values] if *s == state && graph.scalar(*delta, ScalarTypeForm::Integer) && *values == references[if entry_index == 1 { 2 } else { 3 }])
                }
                3 => {
                    matches!(parameters.as_slice(), [s, delta] if *s == state && graph.scalar(*delta, ScalarTypeForm::Integer))
                }
                _ => unreachable!("four entries"),
            };
            let result_valid = entry_index == 0
                || (graph.composite(result, CompositeTypeForm::Tuple)
                    && graph.positional(result).as_deref() == Some(&[state, references[4]]));
            if !signature || !result_valid {
                return Err(site.typed(slot));
            }
            entries.push(Entry {
                position: p,
                parameters,
                result,
            });
        }
        if !outcome(&graph, references[4], meter, &site)? {
            return Err(site.typed(4));
        }
        let width = integer(values[8])
            .filter(|s| natural(s))
            .and_then(|s| s.parse::<u8>().ok())
            .filter(|&w| (1..=64).contains(&w))
            .ok_or_else(|| site.premise(site.at(8).key("value")))?;
        let gap = integer(values[9])
            .filter(|s| natural(s))
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&g| width == 64 || g < (1_u64 << width))
            .ok_or_else(|| site.premise(site.at(9).key("value")))?;
        let rate = rational(values[10], false, &site, 10, meter)?;
        for (i, buffer) in buffers.iter().enumerate() {
            if !positive(&buffer.capacity) {
                return Err(site.premise(
                    site.at(11)
                        .key("value")
                        .key("members")
                        .index(i)
                        .key("members")
                        .index(1)
                        .key("value"),
                ));
            }
        }
        if text(values[12]).is_none_or(str::is_empty) {
            return Err(site.premise(site.at(12).key("value")));
        }
        let uncertainty = rational(values[14], true, &site, 14, meter)?;
        let activation_capacity = integer(values[16])
            .filter(|value| natural(value))
            .ok_or_else(|| site.premise(site.at(16).key("value")))?;
        let mut reached = BTreeSet::new();
        let mut pending = vec![references[0]];
        while let Some(p) = pending.pop() {
            meter.charge(1, || site.at(11))?;
            if !reached.insert(p) {
                continue;
            }
            if let Some(CheckedTerm::Application { arguments, .. }) = graph.term(p) {
                for argument in arguments {
                    if let Some(target) = graph.resolve(argument) {
                        if graph.kind(target, CheckedNodeKind::Temporal(TemporalForm::Formula)) {
                            pending.push(target);
                        }
                    }
                }
            }
        }
        let mut previous = None;
        for buffer in &buffers {
            let key = graph
                .keys
                .get(buffer.subformula)
                .expect("resolved buffer key");
            if !reached.contains(&buffer.subformula) || previous.is_some_and(|last| last >= key) {
                return Err(site.shape(11));
            }
            previous = Some(key);
        }
        let fields = graph.fields(state).ok_or_else(|| site.shape(1))?;
        if fields.len() != buffers.len() + 4 {
            return Err(site.shape(1));
        }
        for (i, (name, ty)) in fields.iter().enumerate() {
            meter.charge(1, || site.at(1))?;
            let valid = match i {
                0 => *name == "now" && graph.scalar(*ty, ScalarTypeForm::Integer),
                1 => *name == "history_start" && graph.scalar(*ty, ScalarTypeForm::Integer),
                i if i == fields.len() - 2 => {
                    *name == "activations" && activation_type(&graph, *ty, meter, &site)?
                }
                i if i == fields.len() - 1 => {
                    *name == "event_position" && graph.scalar(*ty, ScalarTypeForm::Integer)
                }
                _ => {
                    *name == format!("buffer_{}", i - 2)
                        && graph.composite(*ty, CompositeTypeForm::Sequence)
                }
            };
            if !valid {
                return Err(site.shape(1));
            }
        }
        let entries = entries
            .try_into()
            .unwrap_or_else(|_| unreachable!("four admitted entries"));
        plans.push(Plan {
            position,
            references,
            entries,
            width,
            gap,
            rate,
            uncertainty,
            buffers,
            activation_capacity,
        });
    }
    Ok(plans)
}

/// The root's dependency set includes body targets and its State semantic type.
pub(super) fn validate_dependencies(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    references: &[Vec<super::BodyReference>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    keys: &[Arc<CheckedNodeId>],
) -> Result<(), ValidationFailure> {
    for (position, node) in nodes.iter().enumerate() {
        if kinds[position] != CheckedNodeKind::Temporal(TemporalForm::MonitorPlan) {
            continue;
        }
        let key = |id: &CheckedNodeId| {
            index
                .get(id)
                .and_then(|&p| keys.get(p))
                .map(Arc::as_ref)
                .cloned()
        };
        let expected = references[position]
            .iter()
            .filter(|(_, site)| site.member == super::ReferenceMember::Target)
            .map(|(target, _)| target)
            .chain(std::iter::once(&node.semantic_type))
            .filter_map(key)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let actual = node.dependencies.iter().filter_map(key).collect::<Vec<_>>();
        if actual != expected {
            return Err(ValidationFailure::refused_at(
                Code::InvalidSemanticGraph,
                node_pointer(position).key("dependencies"),
                None,
                node.node_id.clone(),
            ));
        }
    }
    Ok(())
}
