// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Closed tick-monitor admission and package-borrowed reads (FR-038-AC-421..429).

use super::graph_storage::GraphIndex;
use super::model_members::DomainModel;
use super::native_join_budget::PackageReadStage;
use super::native_join_storage as storage;
use super::stage_storage::{self, Path, Refusal, Site as StorageSite};
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageV2,
    CheckedRational, CheckedSemanticNodeV2, CheckedTerm, ClockConfiguration, CompositeTypeForm,
    FunctionForm, LiteralKind, NativeJoinBudget, NativeJoinChargeKind, NativeJoinDeniedCharge,
    ScalarTypeForm, TemporalForm, TemporalProfile, ValueForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use std::collections::BTreeSet;
use std::sync::Arc;

#[allow(dead_code)] // The complete selected closure reader will call this prerequisite.
mod reached_model_read;
#[cfg(test)]
pub(in crate::checked_package::v2) use reached_model_read::fixture_support as model_document_test_fixture;
#[allow(dead_code)] // The complete selected closure reader will call this prerequisite.
mod selected_closure_walk;
mod selected_type_closure;
pub use selected_type_closure::{
    SelectedMonitorClosureRows, SelectedMonitorTypeClosure, SelectedMonitorTypeClosureError,
    SelectedMonitorTypeClosureErrorCode,
};

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

/// Whether `name` is the name of an entry slot of a monitor plan.
pub(super) fn is_entry_slot(name: &str) -> bool {
    ENTRY_SLOTS.iter().any(|&slot| NAMES[slot] == name)
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

/// One admitted monitor clock binding. Timed clauses do not carry a clock row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedMonitorClock<'a> {
    /// One of the three admitted temporal binding configurations.
    Configured(&'a ClockConfiguration),
    /// The checked model time binding and exact source unit.
    ModelTime {
        /// The checked timed-model root that owns the source.
        timed_root: &'a CheckedNodeId,
        /// The selected model subject of the clause.
        model: &'a CheckedNodeId,
        /// The checked timed-model time source.
        source: &'a CheckedNodeId,
        /// The exact checked time unit.
        unit: &'a CheckedNodeId,
    },
    /// Timestamped trace positions supplied to the temporal profile.
    TraceTime,
}

/// An admitted clock and claim name for one selected monitor clause occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedMonitorClockRead<'a> {
    root: &'a CheckedNodeId,
    clause: &'a CheckedNodeId,
    occurrence: CheckedOccurrence,
    clock: CheckedMonitorClock<'a>,
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
    /// The admitted configured, model-time or trace-time binding.
    pub fn clock(&self) -> CheckedMonitorClock<'a> {
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
    /// First selected clause or inspected row node, absent in compact exhaustion.
    pub at: Option<Box<CheckedNodeId>>,
    /// Exact requested occurrence, absent in compact exhaustion.
    pub occurrence: Option<CheckedOccurrence>,
    /// Work charged before refusal.
    pub work: u64,
    /// First refused charge, zero for semantic refusal.
    pub attempted_work: u64,
    /// The denied resource and its full attempted amount, if any.
    pub denied: Option<NativeJoinDeniedCharge>,
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
                denied: Some(NativeJoinDeniedCharge {
                    kind: NativeJoinChargeKind::Storage,
                    amount: units,
                }),
            };
        }
        CheckedMonitorClockReadError {
            code,
            at: Some(Box::new(at.clone())),
            occurrence: Some(self.occurrence.clone()),
            work,
            attempted_work: 0,
            denied: None,
        }
    }

    fn charge(&mut self) -> Result<(), CheckedMonitorClockReadError> {
        if !self.budget.charge(NativeJoinChargeKind::Work, 1) {
            return Err(CheckedMonitorClockReadError {
                code: CheckedMonitorClockReadErrorCode::WorkLimit,
                at: None,
                occurrence: None,
                work: self.budget.consumed().work - self.start_work,
                attempted_work: 1,
                denied: Some(NativeJoinDeniedCharge {
                    kind: NativeJoinChargeKind::Work,
                    amount: 1,
                }),
            });
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
                denied: Some(NativeJoinDeniedCharge {
                    kind: NativeJoinChargeKind::Storage,
                    amount,
                }),
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
        budget.charge()?;
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

fn admitted_clock_name(term: &CheckedTerm) -> Option<&str> {
    let CheckedTerm::Application { arguments, .. } = term else {
        return None;
    };
    let argument = match arguments.get(1)? {
        CheckedTerm::Binding { value, .. } => value.as_ref(),
        direct => direct,
    };
    let CheckedTerm::Literal { value, .. } = argument else {
        return None;
    };
    value.as_str()
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
        &self.package.resolved_graph().nodes[self.plan.references[slot]].node_id
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
        &self.package.resolved_graph().nodes[self.plan.position].node_id
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
    /// Join one independently selected claim occurrence to its admitted
    /// configured, model-time or trace-time clock and claim name.
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
        budget.charge()?;
        budget.charge()?;
        if clause != self.clause() {
            return Err(budget.error(CheckedMonitorClockReadErrorCode::WrongClause, clause));
        }
        if occurrence.role != CheckedOccurrenceRole::Claim {
            return Err(budget.error(CheckedMonitorClockReadErrorCode::WrongRole, clause));
        }
        let admitted = self.package.node(clause).expect("admitted monitor clause");
        let mut found = false;
        for candidate in &admitted.occurrences {
            budget.charge()?;
            if candidate == occurrence {
                found = true;
                break;
            }
        }
        if !found {
            return Err(budget.error(CheckedMonitorClockReadErrorCode::UnknownOccurrence, clause));
        }
        let clause_term = self
            .package
            .term(clause)
            .expect("admitted monitor clause body");
        let timed = matches!(clause_term, CheckedTerm::Application { operation, .. }
            if operation.law_definitions().ok().is_some_and(|laws|
                matches!(laws, [law] if TemporalProfile::from_wire(&law.identity)
                    == Some(TemporalProfile::Timed))));
        let clock = if timed {
            match super::timed_model::admitted_model_time(self.package, clause, |_| {
                budget.charge()
            })? {
                Some(binding) => CheckedMonitorClock::ModelTime {
                    timed_root: binding.timed_root,
                    model: binding.model,
                    source: binding.source,
                    unit: binding.unit,
                },
                None => CheckedMonitorClock::TraceTime,
            }
        } else {
            let binding = selected_row(
                self.package,
                self.package.temporal_bindings().unwrap_or_default(),
                clause,
                occurrence.ordinal,
                |row| (&row.declaration, row.occurrence),
                &mut budget,
            )?
            .ok_or_else(|| budget.error(CheckedMonitorClockReadErrorCode::MissingClock, clause))?;
            CheckedMonitorClock::Configured(&binding.clock)
        };
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
            clock,
            clock_name: admitted_clock_name(clause_term).expect("admitted clause clock"),
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
        &self.package.wire.semantic_graph.nodes[self.plan.position].body
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
        &self.package.resolved_graph().nodes[self.entry.position].node_id
    }
    /// The function's resolved content key.
    pub fn structural_key(&self) -> &'a CheckedNodeId {
        self.package
            .structural_key_at(self.entry.position)
            .expect("admitted entry key")
    }
    /// The entry's exact admitted result type.
    pub fn result_type(&self) -> &'a CheckedNodeId {
        &self.package.resolved_graph().nodes[self.entry.result].node_id
    }
    /// Complete ordinary function body, retained without execution.
    pub fn body(&self) -> &'a CheckedTerm {
        &self.package.wire.semantic_graph.nodes[self.entry.position].body
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
            .map(|&position| &self.package.resolved_graph().nodes[position].node_id)
    }
}
impl<'a> CheckedMonitorBufferRef<'a> {
    /// The buffered subformula's exact node ID.
    pub fn subformula(&self) -> &'a CheckedNodeId {
        &self.package.resolved_graph().nodes[self.buffer.subformula].node_id
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
fn integer(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal { value, .. } => value.as_str().or_else(|| value.number_text()),
        _ => None,
    }
}
fn suffix_canonical(name: &str, index: usize) -> bool {
    name.len()
        == "buffer_".len()
            + index
                .checked_ilog10()
                .map_or(1, |digits| digits as usize + 1)
}
fn positive(text: &str) -> bool {
    matches!(text.as_bytes(), [first, rest @ ..] if (b'1'..=b'9').contains(first) && rest.iter().all(u8::is_ascii_digit))
}
#[ix_trace_rs::string_edge]
fn natural(text: &str) -> bool {
    text == "0" || positive(text)
}

struct Graph<'a, Body> {
    nodes: &'a [CheckedSemanticNodeV2<Body>],
    kinds: &'a [CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a GraphIndex<'a>,
    keys: &'a [Arc<CheckedNodeId>],
}
impl<Body> Graph<'_, Body> {
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
    fn fields<const N: usize>(&self, position: usize) -> Option<[(&str, usize); N]> {
        let members = members(self.term(position)?)?;
        if members.len() != N {
            return None;
        }
        let mut fields = [("", 0); N];
        for (field, member) in fields.iter_mut().zip(members) {
            let CheckedTerm::Binding { name, value } = member else {
                return None;
            };
            *field = (name.as_ref(), self.resolve(value)?);
        }
        Some(fields)
    }
    fn positional<const N: usize>(&self, position: usize) -> Option<[usize; N]> {
        let members = members(self.term(position)?)?;
        if members.len() != N {
            return None;
        }
        let mut positions = [0; N];
        for (position, member) in positions.iter_mut().zip(members) {
            *position = self.resolve(member)?;
        }
        Some(positions)
    }
    fn parameters(
        &self,
        position: usize,
        meter: &mut WorkMeter,
    ) -> Result<Option<Vec<usize>>, ValidationFailure> {
        let prepared = || -> Option<&[CheckedTerm]> {
            let (parameters, body, measure) = match members(self.term(position)?)? {
                [parameters, body] => (parameters, body, None),
                [parameters, body, measure] => (parameters, body, Some(measure)),
                _ => return None,
            };
            if let Some(measure) = measure {
                self.resolve(named(measure, "measure")?)?;
            }
            let expression = self.resolve(named(body, "body")?)?;
            if !matches!(
                self.kinds.get(expression),
                Some(CheckedNodeKind::Expression(_))
            ) {
                return None;
            }
            members(named(parameters, "parameters")?)
        };
        let Some(parameters) = prepared() else {
            return Ok(None);
        };
        let mut positions = meter.vec(parameters.len())?;
        for parameter in parameters {
            let Some(p) = self
                .resolve(parameter)
                .filter(|&p| self.kind(p, CheckedNodeKind::Value(ValueForm::Parameter)))
            else {
                return Ok(None);
            };
            let Some(ty) = self.index.get(&self.nodes[p].semantic_type) else {
                return Ok(None);
            };
            positions.push(*ty);
        }
        Ok(Some(positions))
    }
}

struct Site<'a> {
    position: usize,
    id: &'a CheckedNodeId,
}
impl Site<'_> {
    fn work(&self, meter: &mut WorkMeter, at: Path<'_>) -> Result<(), ValidationFailure> {
        stage_storage::work(
            meter,
            &StorageSite::new(PackageReadStage::Graph, self.position, "temporal", at),
            1,
        )
    }

    fn at(&self, slot: usize) -> Path<'_> {
        Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(self.position)
            .key("body")
            .key("members")
            .index(slot)
    }
    fn refusal(
        &self,
        code: Code,
        cause: Option<Cause>,
        at: Path<'_>,
        meter: &mut WorkMeter,
    ) -> ValidationFailure {
        let refusal = Refusal::new(code, at).at_node(self.id);
        let refusal = if let Some(cause) = cause {
            refusal.because(cause)
        } else {
            refusal
        };
        stage_storage::refuse(
            meter,
            &StorageSite::new(PackageReadStage::Graph, self.position, "temporal", at),
            refusal,
        )
    }
    fn shape(&self, slot: usize, meter: &mut WorkMeter) -> ValidationFailure {
        self.refusal(Code::InvalidSemanticGraph, None, self.at(slot), meter)
    }
    fn typed(&self, slot: usize, meter: &mut WorkMeter) -> ValidationFailure {
        self.refusal(
            Code::IllTyped,
            Some(Cause::TypeMismatch),
            self.at(slot),
            meter,
        )
    }
    fn premise(&self, at: Path<'_>, meter: &mut WorkMeter) -> ValidationFailure {
        self.refusal(Code::InvalidPackage, Some(Cause::InvalidValue), at, meter)
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
    let raw = text(term).ok_or_else(|| site.premise(at, meter))?;
    let (numerator, denominator) = raw.split_once('/').unwrap_or((raw, "1"));
    if !(if allow_zero {
        natural(numerator)
    } else {
        positive(numerator)
    }) || !positive(denominator)
        || (raw.contains('/') && denominator == "1")
    {
        return Err(site.premise(at, meter));
    }
    if !super::schedulability::rational_parts_are_canonical(
        numerator,
        denominator,
        meter,
        StorageSite::new(PackageReadStage::Graph, site.position, "temporal", at),
    )? {
        return Err(site.premise(at, meter));
    }
    Ok(CheckedRational {
        numerator: meter.boxed_str(numerator)?,
        denominator: meter.boxed_str(denominator)?,
    })
}

#[ix_trace_rs::string_edge]
fn outcome<Body>(
    graph: &Graph<'_, Body>,
    results: usize,
    meter: &mut WorkMeter,
    site: &Site<'_>,
) -> Result<bool, ValidationFailure> {
    let valid = || -> Option<bool> {
        let elements = graph.positional::<1>(results)?;
        let [result] = elements.as_slice() else {
            return Some(false);
        };
        if !graph.composite(*result, CompositeTypeForm::Record) {
            return Some(false);
        }
        let fields = graph.fields::<2>(*result)?;
        let [("instance", instance), ("outcome", outcome)] = fields.as_slice() else {
            return Some(false);
        };
        if !graph.composite(*instance, CompositeTypeForm::Record)
            || !graph.composite(*outcome, CompositeTypeForm::Union)
        {
            return Some(false);
        }
        let instance_fields = graph.fields::<3>(*instance)?;
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
        let cases = members(graph.term(*outcome)?)?;
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
            let CheckedTerm::Binding { name, value } = case else {
                return None;
            };
            if name.as_ref() != expected {
                return Some(false);
            }
            let payloads = members(value)?;
            if matches!(expected, "Failed" | "Incomplete") {
                let [payload] = payloads else {
                    return Some(false);
                };
                let p = *graph.index.get(reference(payload)?)?;
                if !graph.composite(p, CompositeTypeForm::Record) {
                    return Some(false);
                }
                if expected == "Failed" {
                    let fields = graph.fields::<2>(p)?;
                    let [("gap", gap), ("max_reading_gap", max)] = fields.as_slice() else {
                        return Some(false);
                    };
                    if !graph.scalar(*gap, ScalarTypeForm::Integer)
                        || !graph.scalar(*max, ScalarTypeForm::Integer)
                    {
                        return Some(false);
                    }
                } else {
                    let fields = graph.fields::<3>(p)?;
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
            } else if !payloads.is_empty() {
                return Some(false);
            }
        }
        Some(true)
    };
    site.work(meter, site.at(4))?;
    Ok(valid().unwrap_or(false))
}

fn activation_type<Body>(
    graph: &Graph<'_, Body>,
    sequence: usize,
    meter: &mut WorkMeter,
    site: &Site<'_>,
) -> Result<bool, ValidationFailure> {
    site.work(meter, site.at(1))?;
    let valid = || -> Option<bool> {
        if !graph.composite(sequence, CompositeTypeForm::Sequence) {
            return Some(false);
        }
        let elements = graph.positional::<1>(sequence)?;
        let [record] = elements.as_slice() else {
            return Some(false);
        };
        if !graph.composite(*record, CompositeTypeForm::Record) {
            return Some(false);
        }
        let fields = graph.fields::<3>(*record)?;
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
pub(super) fn validate<Body>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    keys: &[Arc<CheckedNodeId>],
    models: &[DomainModel],
    meter: &mut WorkMeter,
) -> Result<Vec<Plan>, ValidationFailure> {
    meter.at_node(0, "temporal");
    let graph = Graph {
        nodes,
        kinds,
        terms,
        index,
        keys,
    };
    // Of two models with one identity the later one is kept, as a map built
    // by insertion would hold; the slice address orders them.
    let mut identified = meter.vec::<(&str, &DomainModel)>(models.len())?;
    identified.extend(models.iter().map(|model| (model.identity.as_ref(), model)));
    let model_index = super::graph_storage::SortedTable::from_vec(identified, |left, right| {
        std::ptr::from_ref(*right).cmp(&std::ptr::from_ref(*left))
    });
    let count = index
        .values()
        .filter(|&&p| graph.kind(p, CheckedNodeKind::Temporal(TemporalForm::MonitorPlan)))
        .count();
    let mut positions = meter.vec(count)?;
    positions.extend(
        index
            .values()
            .copied()
            .filter(|&p| graph.kind(p, CheckedNodeKind::Temporal(TemporalForm::MonitorPlan))),
    );
    positions.sort_unstable_by(|&a, &b| keys[a].cmp(&keys[b]));
    let mut plans = meter.vec(positions.len())?;
    for position in positions {
        meter.at_node(position, "temporal");
        let node = &nodes[position];
        let site = Site {
            position,
            id: &node.node_id,
        };
        site.work(
            meter,
            Path::root()
                .key("semantic_graph")
                .key("nodes")
                .index(position)
                .key("body"),
        )?;
        let body = graph.term(position).ok_or_else(|| site.shape(0, meter))?;
        let body_members = members(body).ok_or_else(|| site.shape(0, meter))?;
        let mut values = meter.vec(NAMES.len())?;
        // Complete shape precedes all reference-kind and signature checks.
        for (slot, name) in NAMES.iter().enumerate() {
            site.work(meter, site.at(slot))?;
            let value = body_members
                .get(slot)
                .and_then(|member| named(member, name))
                .ok_or_else(|| site.shape(slot, meter))?;
            let valid = match slot {
                8 | 9 | 16 => literal(value, LiteralKind::Integer),
                10 | 14 => literal(value, LiteralKind::Rational),
                12 => literal(value, LiteralKind::Text),
                11 => {
                    let buffers = members(value).ok_or_else(|| site.shape(slot, meter))?;
                    for (i, buffer) in buffers.iter().enumerate() {
                        site.work(meter, site.at(slot).key("value").key("members").index(i))?;
                        if !matches!(members(buffer), Some([subformula, capacity]) if named(subformula, "subformula").and_then(reference).is_some()
                            && named(capacity, "capacity").is_some_and(|t| literal(t, LiteralKind::Integer)))
                        {
                            return Err(site.shape(slot, meter));
                        }
                    }
                    true
                }
                _ => reference(value).is_some(),
            };
            if !valid {
                return Err(site.shape(slot, meter));
            }
            values.push(value);
        }
        if body_members.len() != NAMES.len() {
            return Err(site.shape(NAMES.len(), meter));
        }
        let slots = [0, 1, 2, 3, 4, 5, 6, 7, 13, 15];
        let mut references = [0; 10];
        let mut buffers = meter.vec(members(values[11]).expect("checked buffers").len())?;
        for (i, slot) in slots.into_iter().enumerate() {
            if slot == 13 {
                for (i, buffer) in members(values[11])
                    .expect("checked buffers")
                    .iter()
                    .enumerate()
                {
                    site.work(meter, site.at(11).key("value").key("members").index(i))?;
                    let subformula = graph
                        .resolve(bound(buffer, 0).expect("checked buffer"))
                        .filter(|&p| {
                            graph.kind(p, CheckedNodeKind::Temporal(TemporalForm::Formula))
                        })
                        .ok_or_else(|| site.shape(11, meter))?;
                    let capacity = integer(bound(buffer, 1).expect("checked capacity"))
                        .ok_or_else(|| site.shape(11, meter))?;
                    buffers.push(Buffer {
                        subformula,
                        capacity: meter.boxed_str(capacity)?,
                    });
                }
            }
            site.work(meter, site.at(slot))?;
            let p = graph
                .resolve(values[slot])
                .ok_or_else(|| site.shape(slot, meter))?;
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
                                    graph.term(d),
                                    &model_index,
                                )
                        })
                    }
                }
                _ => unreachable!("closed reference slots"),
            };
            if !valid {
                return Err(site.shape(slot, meter));
            }
            references[i] = p;
        }
        let state = references[1];
        if node.semantic_type != nodes[state].node_id {
            return Err(site.refusal(
                Code::InvalidSemanticGraph,
                None,
                Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(position)
                    .key("semantic_type"),
                meter,
            ));
        }
        let mut entries = meter.vec(4)?;
        for (entry_index, slot) in ENTRY_SLOTS.into_iter().enumerate() {
            site.work(meter, site.at(slot))?;
            let p = references[if entry_index == 3 { 9 } else { 5 + entry_index }];
            let parameters = graph
                .parameters(p, meter)?
                .ok_or_else(|| site.typed(slot, meter))?;
            let result = index
                .get(&nodes[p].semantic_type)
                .copied()
                .ok_or_else(|| site.typed(slot, meter))?;
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
                    && graph.positional::<2>(result) == Some([state, references[4]]));
            if !signature || !result_valid {
                return Err(site.typed(slot, meter));
            }
            entries.push(Entry {
                position: p,
                parameters,
                result,
            });
        }
        if !outcome(&graph, references[4], meter, &site)? {
            return Err(site.typed(4, meter));
        }
        let width = integer(values[8])
            .filter(|s| natural(s))
            .and_then(|s| s.parse::<u8>().ok())
            .filter(|&w| (1..=64).contains(&w))
            .ok_or_else(|| site.premise(site.at(8).key("value"), meter))?;
        let gap = integer(values[9])
            .filter(|s| natural(s))
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&g| width == 64 || g < (1_u64 << width))
            .ok_or_else(|| site.premise(site.at(9).key("value"), meter))?;
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
                    meter,
                ));
            }
        }
        if text(values[12]).is_none_or(str::is_empty) {
            return Err(site.premise(site.at(12).key("value"), meter));
        }
        let uncertainty = rational(values[14], true, &site, 14, meter)?;
        let activation_capacity = integer(values[16])
            .filter(|value| natural(value))
            .ok_or_else(|| site.premise(site.at(16).key("value"), meter))?;
        let mut reached = meter.filled(nodes.len(), false)?;
        let mut pending = meter.vec(1)?;
        pending.push(references[0]);
        while let Some(p) = pending.pop() {
            site.work(meter, site.at(11))?;
            if reached[p] {
                continue;
            }
            reached[p] = true;
            if let Some(CheckedTerm::Application { arguments, .. }) = graph.term(p) {
                for argument in arguments {
                    if let Some(target) = graph.resolve(argument) {
                        if graph.kind(target, CheckedNodeKind::Temporal(TemporalForm::Formula)) {
                            meter.push(&mut pending, target)?;
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
            if !reached[buffer.subformula] || previous.is_some_and(|last| last >= key) {
                return Err(site.shape(11, meter));
            }
            previous = Some(key);
        }
        let fields = graph
            .term(state)
            .and_then(members)
            .ok_or_else(|| site.shape(1, meter))?;
        if fields.iter().any(|field| match field {
            CheckedTerm::Binding { value, .. } => graph.resolve(value).is_none(),
            _ => true,
        }) {
            return Err(site.shape(1, meter));
        }
        if fields.len() != buffers.len() + 4 {
            return Err(site.shape(1, meter));
        }
        for (i, field) in fields.iter().enumerate() {
            let CheckedTerm::Binding { name, value } = field else {
                return Err(site.shape(1, meter));
            };
            let ty = graph.resolve(value).ok_or_else(|| site.shape(1, meter))?;
            site.work(meter, site.at(1))?;
            let valid = match i {
                0 => name.as_ref() == "now" && graph.scalar(ty, ScalarTypeForm::Integer),
                1 => name.as_ref() == "history_start" && graph.scalar(ty, ScalarTypeForm::Integer),
                i if i == fields.len() - 2 => {
                    name.as_ref() == "activations" && activation_type(&graph, ty, meter, &site)?
                }
                i if i == fields.len() - 1 => {
                    name.as_ref() == "event_position" && graph.scalar(ty, ScalarTypeForm::Integer)
                }
                _ => {
                    name.strip_prefix("buffer_")
                        .and_then(|suffix| suffix.parse::<usize>().ok())
                        .is_some_and(|index| index == i - 2 && suffix_canonical(name, index))
                        && graph.composite(ty, CompositeTypeForm::Sequence)
                }
            };
            if !valid {
                return Err(site.shape(1, meter));
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
            activation_capacity: meter.boxed_str(activation_capacity)?,
        });
    }
    Ok(plans)
}

/// The root's dependency set includes body targets and its State semantic type.
pub(super) fn validate_dependencies<Body>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    references: &[Vec<super::BodyReference>],
    index: &GraphIndex<'_>,
    keys: &[Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for (position, node) in nodes.iter().enumerate() {
        if kinds[position] != CheckedNodeKind::Temporal(TemporalForm::MonitorPlan) {
            continue;
        }
        let key = |id: &CheckedNodeId| index.get(id).and_then(|&p| keys.get(p)).map(Arc::as_ref);
        meter.at_node(position, "temporal");
        let mut expected = meter.vec(references[position].len().saturating_add(1))?;
        expected.extend(
            references[position]
                .iter()
                .filter(|(_, site)| site.member == super::ReferenceMember::Target)
                .map(|(target, _)| target)
                .chain(std::iter::once(&node.semantic_type))
                .filter_map(key),
        );
        expected.sort_unstable();
        expected.dedup();
        let mut actual = meter.vec(node.dependencies.len())?;
        actual.extend(node.dependencies.iter().filter_map(key));
        if actual != expected {
            let site = Site {
                position,
                id: &node.node_id,
            };
            return Err(site.refusal(
                Code::InvalidSemanticGraph,
                None,
                Path::root()
                    .key("semantic_graph")
                    .key("nodes")
                    .index(position)
                    .key("dependencies"),
                meter,
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod admission_storage_tests {
    use super::super::{test_support::node_id, CheckedJson};
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn node(fill: char) -> CheckedSemanticNodeV2<CheckedJson> {
        let id = node_id(fill);
        serde_json::from_value(json!({
            "node_id":id, "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"temporal", "semantic_form":"monitor_plan", "semantic_type":id,
            "dependencies":[], "occurrences":[], "body":{"term":"aggregate","members":[]}
        }))
        .expect("retained document metadata")
    }

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn monitor_shape_refusal_prepays_actual_tables_and_held_diagnostic() {
        let nodes = [node('a')];
        let kinds = [CheckedNodeKind::Temporal(TemporalForm::MonitorPlan)];
        let terms = [Some(CheckedTerm::Aggregate {
            members: Vec::new(),
        })];
        let index = nodes
            .iter()
            .enumerate()
            .map(|(p, n)| (&n.node_id, p))
            .collect::<GraphIndex<'_>>();
        let keys = [Arc::new(nodes[0].node_id.clone())];
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                result = Some(validate(
                    &nodes,
                    &kinds,
                    &terms,
                    &index,
                    &keys,
                    &[],
                    &mut WorkMeter::over(&mut budget),
                ));
            });
            (
                result.expect("the monitor validator ran"),
                measured.bytes_total,
                budget,
            )
        };
        let check = |result| {
            let Err(ValidationFailure::Refused(refusal)) = result else {
                panic!("missing first slot must refuse");
            };
            assert_eq!(refusal.code, Code::InvalidSemanticGraph);
            assert_eq!(refusal.cause, None);
            assert_eq!(refusal.locus.as_ref(), Some(&nodes[0].node_id));
            assert_eq!(
                refusal.loci[0].path.as_ref().expect("path").as_str(),
                "/semantic_graph/nodes/0/body/members/0"
            );
        };
        let (result, allocated, budget) = run(u64::MAX);
        check(result);
        let required = budget.read_storage_consumed();
        assert!(required > 0);
        assert_eq!(allocated, required);
        check(run(required).0);
        assert!(matches!(
            run(required - 1).0,
            Err(ValidationFailure::Budget(_))
        ));
    }

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn monitor_dependency_join_canonicalizes_expected_keys_but_preserves_actual_sequence() {
        let mut nodes = [node('a'), node('b'), node('c')];
        nodes[0].semantic_type = nodes[1].node_id.clone();
        nodes[0].dependencies = vec![nodes[1].node_id.clone(), nodes[2].node_id.clone()];
        let kinds = [
            CheckedNodeKind::Temporal(TemporalForm::MonitorPlan),
            CheckedNodeKind::ScalarType(ScalarTypeForm::Integer),
            CheckedNodeKind::ScalarType(ScalarTypeForm::Integer),
        ];
        let target = |id: &CheckedNodeId| {
            (
                id.clone(),
                super::super::ReferenceSite {
                    member: super::super::ReferenceMember::Target,
                    is_body_root: false,
                },
            )
        };
        let references = [
            vec![
                target(&nodes[2].node_id),
                target(&nodes[1].node_id),
                target(&nodes[2].node_id),
            ],
            vec![],
            vec![],
        ];
        let run = |nodes: &[CheckedSemanticNodeV2<CheckedJson>], limit| {
            let index = nodes
                .iter()
                .enumerate()
                .map(|(p, n)| (&n.node_id, p))
                .collect::<GraphIndex<'_>>();
            let keys: Vec<_> = nodes.iter().map(|n| Arc::new(n.node_id.clone())).collect();
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                result = Some(validate_dependencies(
                    nodes,
                    &kinds,
                    &references,
                    &index,
                    &keys,
                    &mut WorkMeter::over(&mut budget),
                ));
            });
            (
                result.expect("the dependency join ran"),
                measured.bytes_total,
                budget,
            )
        };
        let (result, allocated, budget) = run(&nodes, u64::MAX);
        assert!(result.is_ok());
        let required = budget.read_storage_consumed();
        assert!(required > 0);
        assert_eq!(allocated, required);
        assert!(run(&nodes, required).0.is_ok());
        let (short, allocated, budget) = run(&nodes, required - 1);
        assert!(matches!(short, Err(ValidationFailure::Budget(_))));
        assert!(
            allocated <= budget.read_storage_consumed(),
            "a denied request cannot allocate an unpaid pointer"
        );
        for duplicate in [false, true] {
            let mut changed = nodes.clone();
            if duplicate {
                changed[0]
                    .dependencies
                    .push(changed[0].dependencies[1].clone());
            } else {
                changed[0].dependencies.reverse();
            }
            let Err(ValidationFailure::Refused(refusal)) = run(&changed, u64::MAX).0 else {
                panic!("actual order and duplicates remain validated");
            };
            assert_eq!(refusal.code, Code::InvalidSemanticGraph);
            assert_eq!(
                refusal.loci[0].path.as_ref().expect("path").as_str(),
                "/semantic_graph/nodes/0/dependencies"
            );
        }
    }
}
