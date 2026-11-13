// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The runtime layout context of a selected monitor closure (FR-361).
//!
//! Every owned value is measured by a counting pass, paid as one container
//! plus one storage unit per encoded byte, and only then allocated and
//! written. The canonical writer's own scratch growth is paid through the
//! same budget by its reservation hook.

use super::authority::{refusal, SelectedMonitorAuthorityErrorCode};
use super::selected_type_closure::{
    native_error, SelectedMonitorClosureRows, SelectedMonitorTypeClosureError,
};
use super::{CheckedMonitorClock, CheckedMonitorClockRead, CheckedMonitorPlanRef};
use crate::checked_package::shared::CheckedSelection;
use crate::checked_package::v2::native_join_budget::read_charge;
use crate::checked_package::v2::timed_model::{
    admitted_time_source, anchor_operation_text, tick_source_references, AdmittedTimeSource,
};
use crate::checked_package::v2::{
    CheckedNodeId, CheckedNodeOwner, CheckedSelectionRole, NativeJoinBudget, NativeJoinChargeKind,
    NativeJoinLocus, NativeJoinLocusSource, SelectedExactIdentity,
};
use quire_canonical::{Encode, Error, Reservation, ReservationDenied, Sink, Writer};
use quire_exact::{EffectiveId, UnitId};

const FORM: &str = "selected_layout_context";
const CLOCK_FORM: &str = "selected_clock_binding";

/// The exact selection values the runtime `Layout` retains for one claim
/// occurrence of the selected monitor clause.
///
/// ```compile_fail
/// use quire_contract_model::SelectedMonitorLayoutContext;
/// fn fabricate(context: &SelectedMonitorLayoutContext<'_>) {
///     let _ = context.subject;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedMonitorLayoutContext<'a> {
    root_key: &'a CheckedNodeId,
    entry_keys: [&'a CheckedNodeId; 4],
    clause_key: &'a CheckedNodeId,
    package_lock: Vec<u8>,
    occurrence: Vec<u8>,
    profile: String,
    subject: String,
    clock_binding: Vec<u8>,
}

impl<'a> SelectedMonitorLayoutContext<'a> {
    /// The resolved structural key of the plan root.
    pub fn root_key(&self) -> &'a CheckedNodeId {
        self.root_key
    }
    /// The resolved keys of `init`, `on_event`, `on_watermark` and `on_gap`.
    pub fn entry_keys(&self) -> [&'a CheckedNodeId; 4] {
        self.entry_keys
    }
    /// The resolved structural key of the admitted clause.
    pub fn clause_key(&self) -> &'a CheckedNodeId {
        self.clause_key
    }
    /// The RFC 8785 encoding of the admitted package lock.
    pub fn package_lock(&self) -> &[u8] {
        &self.package_lock
    }
    /// The RFC 8785 encoding of the selected claim occurrence's source-map entry.
    pub fn occurrence(&self) -> &[u8] {
        &self.occurrence
    }
    /// The RFC 8785 text of the selected clause profile identity and role.
    pub fn profile(&self) -> &str {
        &self.profile
    }
    /// The canonical Exact identity text of the clause's model subject.
    pub fn subject(&self) -> &str {
        &self.subject
    }
    /// The canonical `model-time` clock-binding value: UTF-8 RFC 8785 JSON text.
    pub fn clock_binding(&self) -> &[u8] {
        &self.clock_binding
    }
}

struct Count(u64);

impl Sink for Count {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.0 = self.0.saturating_add(bytes.len() as u64);
        Ok(())
    }
}

struct Pay<'b>(&'b mut NativeJoinBudget);

impl Reservation for Pay<'_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), ReservationDenied> {
        if self.0.charge(NativeJoinChargeKind::Storage, bytes as u128) {
            Ok(())
        } else {
            Err(ReservationDenied)
        }
    }
}

/// `{"authority", "identity", "role"}` of the selected profile.
struct Profile<'a>(&'a CheckedSelection);

fn write_profile<S: Sink + ?Sized>(
    writer: &mut Writer<'_, S>,
    selection: &CheckedSelection,
) -> Result<(), Error> {
    writer.begin_object()?;
    writer.name("authority")?;
    writer.string(&selection.definition.authority)?;
    writer.name("identity")?;
    writer.string(&selection.definition.identity)?;
    writer.name("role")?;
    writer.string(selection.role.as_wire())?;
    writer.end_object()
}

impl Encode for Profile<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        write_profile(writer, self.0)
    }
}

/// The FR-150 effective declaration identity, as IR's `CheckedEffectiveId` encodes it.
struct Effective<'a>(&'a EffectiveId);

impl Encode for Effective<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        let chain = self.0.owner_chain();
        for index in (0..chain.len()).rev() {
            writer.begin_object()?;
            writer.name("owner_effective_type")?;
            if index == 0 {
                writer.null()?;
            }
        }
        for original in chain {
            writer.name("original")?;
            writer.begin_object()?;
            writer.name("node")?;
            writer.string(original.node())?;
            writer.name("package")?;
            writer.string(original.package())?;
            writer.end_object()?;
            writer.end_object()?;
        }
        Ok(())
    }
}

/// The `model-time` clock-binding value (QSpec FR-252 key components).
struct Binding<'a> {
    profile: &'a CheckedSelection,
    model: &'a str,
    /// `(operation, period numerator, period denominator)` of a tick source.
    tick: Option<(&'a str, &'a str, &'a str)>,
    unit: &'a str,
}

impl Encode for Binding<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("binding_kind")?;
        writer.string("model-time")?;
        writer.name("selected_profile_identity")?;
        write_profile(writer, self.profile)?;
        writer.name("time_source")?;
        writer.begin_object()?;
        writer.name("kind")?;
        writer.string(if self.tick.is_some() { "tick" } else { "dense" })?;
        writer.name("model")?;
        writer.string(self.model)?;
        if let Some((operation, _, _)) = self.tick {
            writer.name("operation")?;
            writer.string(operation)?;
        }
        writer.end_object()?;
        writer.name("unit")?;
        writer.string(self.unit)?;
        if let Some((_, numerator, denominator)) = self.tick {
            writer.name("period")?;
            writer.begin_object()?;
            writer.name("denominator")?;
            writer.string(denominator)?;
            writer.name("numerator")?;
            writer.string(numerator)?;
            writer.end_object()?;
        }
        writer.name("sequence_authority")?;
        writer.string("step-order")?;
        writer.end_object()
    }
}

fn work(
    budget: &mut NativeJoinBudget,
    amount: u128,
) -> Result<(), SelectedMonitorTypeClosureError> {
    read_charge(budget, NativeJoinChargeKind::Work, amount).map_err(|_| native_error(budget))
}

/// Pay one container plus one unit per byte, then allocate it.
fn owned_bytes(
    budget: &mut NativeJoinBudget,
    length: u64,
) -> Result<Vec<u8>, SelectedMonitorTypeClosureError> {
    read_charge(
        budget,
        NativeJoinChargeKind::Storage,
        u128::from(length).saturating_add(1),
    )
    .map_err(|_| native_error(budget))?;
    Ok(Vec::with_capacity(
        usize::try_from(length).unwrap_or(usize::MAX),
    ))
}

/// The canonical encoding of `value`, measured, paid, allocated and written.
fn encoded<T: Encode + ?Sized>(
    value: &T,
    budget: &mut NativeJoinBudget,
) -> Result<Vec<u8>, SelectedMonitorTypeClosureError> {
    let limits = quire_canonical::Limits::new(u64::MAX);
    let mut count = Count(0);
    quire_canonical::encode_with_reservation(&mut count, value, limits, &mut Pay(budget))
        .map_err(|_| native_error(budget))?;
    let mut bytes = owned_bytes(budget, count.0)?;
    quire_canonical::encode_with_reservation(&mut bytes, value, limits, &mut Pay(budget))
        .map_err(|_| native_error(budget))?;
    Ok(bytes)
}

fn text(bytes: Vec<u8>) -> String {
    // The canonical writer emits only escaped `&str` content, so it is UTF-8.
    String::from_utf8(bytes).expect("canonical JSON is UTF-8")
}

/// Counts the bytes a `Display` produces without storing them.
struct Measure(u64);

impl std::fmt::Write for Measure {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.0 = self.0.saturating_add(text.len() as u64);
        Ok(())
    }
}

/// Writes into a pre-paid `String` and refuses to grow it past its payment.
struct Prepaid<'s>(&'s mut String);

impl std::fmt::Write for Prepaid<'_> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        if self.0.len() + text.len() > self.0.capacity() {
            return Err(std::fmt::Error);
        }
        self.0.push_str(text);
        Ok(())
    }
}

/// An identity's `Display` text: its length is measured, exactly that is paid,
/// and only then is it built. No fixed length is assumed.
pub(super) fn hex_text(
    identity: &impl std::fmt::Display,
    at: &CheckedNodeId,
    budget: &mut NativeJoinBudget,
) -> Result<String, SelectedMonitorTypeClosureError> {
    use std::fmt::Write as _;
    let mut measure = Measure(0);
    let unavailable = |budget: &mut NativeJoinBudget| {
        refusal(
            SelectedMonitorAuthorityErrorCode::LayoutIdentityUnavailable,
            at,
            None,
            budget,
        )
    };
    if write!(measure, "{identity}").is_err() {
        return Err(unavailable(budget));
    }
    let mut out = text(owned_bytes(budget, measure.0)?);
    if write!(Prepaid(&mut out), "{identity}").is_err() || out.len() as u64 != measure.0 {
        return Err(unavailable(budget));
    }
    Ok(out)
}

/// A model or operation node's name: its declaration path joined by '.', or
/// the original identity of its owning model declaration, verbatim.
pub(super) fn declared_name<'p>(
    package: &'p crate::CheckedPackageV2,
    node: &CheckedNodeId,
    budget: &mut NativeJoinBudget,
) -> Result<std::borrow::Cow<'p, str>, SelectedMonitorTypeClosureError> {
    work(budget, 1)?;
    let Some(admitted) = package.node(node) else {
        return Err(refusal(
            SelectedMonitorAuthorityErrorCode::LayoutIdentityUnavailable,
            node,
            None,
            budget,
        ));
    };
    if let Some(declaration) = &admitted.declaration {
        let mut length = declaration.qualified_name.len().saturating_sub(1);
        for segment in &declaration.qualified_name {
            work(budget, 1)?;
            length += segment.len();
        }
        let mut joined = owned_bytes(budget, length as u64)?;
        for (index, segment) in declaration.qualified_name.iter().enumerate() {
            if index > 0 {
                joined.push(b'.');
            }
            joined.extend_from_slice(segment.as_bytes());
        }
        return Ok(std::borrow::Cow::Owned(text(joined)));
    }
    if let Some(CheckedNodeOwner::Model { node, .. }) = &admitted.owner {
        return Ok(std::borrow::Cow::Borrowed(node));
    }
    // An `operation_anchor` has no declaration: its own `operation` text names it.
    work(budget, 1)?;
    if let Some(name) = anchor_operation_text(package, node) {
        return Ok(std::borrow::Cow::Borrowed(name));
    }
    Err(refusal(
        SelectedMonitorAuthorityErrorCode::LayoutIdentityUnavailable,
        node,
        None,
        budget,
    ))
}

impl<'a> CheckedMonitorPlanRef<'a> {
    /// Read the layout context of the selected claim occurrence at `index` in `clocks()`.
    pub(super) fn layout_context(
        &self,
        rows: &SelectedMonitorClosureRows<'_>,
        clock: &CheckedMonitorClockRead<'a>,
        index: usize,
        budget: &mut NativeJoinBudget,
    ) -> Result<SelectedMonitorLayoutContext<'a>, SelectedMonitorTypeClosureError> {
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::MonitorSeed(index),
            form: FORM,
        }));
        let unavailable = |at: &CheckedNodeId, budget: &mut NativeJoinBudget| {
            refusal(
                SelectedMonitorAuthorityErrorCode::LayoutIdentityUnavailable,
                at,
                None,
                budget,
            )
        };
        let CheckedMonitorClock::ModelTime {
            model,
            source,
            unit,
            ..
        } = clock.clock()
        else {
            return Err(unavailable(self.clause(), budget));
        };
        let package = self.package;
        // The borrowed structural keys: one inspection each.
        work(budget, 6)?;
        let entry_keys = [
            self.init().structural_key(),
            self.on_event().structural_key(),
            self.on_watermark().structural_key(),
            self.on_gap().structural_key(),
        ];
        let package_lock = encoded(package.lock(), budget)?;
        // The selected claim occurrence's own source-map entry.
        let mut entry = None;
        for candidate in package.source_map() {
            work(budget, 1)?;
            if candidate.node_id == *self.clause()
                && candidate.role == clock.occurrence().role
                && candidate.ordinal == clock.occurrence().ordinal
            {
                entry = Some(candidate);
                break;
            }
        }
        let Some(entry) = entry else {
            return Err(unavailable(self.clause(), budget));
        };
        let occurrence = encoded(entry, budget)?;
        // The clause profile: its one law joined to the lock's selection row.
        work(budget, 1)?;
        let law = match package.term(self.clause()) {
            Some(crate::CheckedTerm::Application { operation, .. }) => {
                operation.law_definitions().ok().and_then(<[_]>::first)
            }
            _ => None,
        };
        let Some(law) = law else {
            return Err(unavailable(self.clause(), budget));
        };
        let mut selection = None;
        for candidate in &package.lock().profile_selections {
            work(budget, 1)?;
            if candidate.role == CheckedSelectionRole::TemporalProfile
                && candidate.definition == *law
            {
                selection = Some(candidate);
                break;
            }
        }
        let Some(selection) = selection else {
            return Err(unavailable(self.clause(), budget));
        };
        let profile = text(encoded(&Profile(selection), budget)?);
        // The subject: one declaration row of the clause's model.
        let mut found = None;
        let mut count = 0_u64;
        for row in rows.declarations {
            work(budget, 1)?;
            if row.structural_node == *model {
                count += 1;
                found = Some(&row.exact_identity);
            }
        }
        let subject = match (count, found) {
            (1, Some(SelectedExactIdentity::Record(key))) => hex_text(key, model, budget)?,
            (1, Some(SelectedExactIdentity::Object(effective))) => {
                text(encoded(&Effective(effective), budget)?)
            }
            _ => return Err(unavailable(model, budget)),
        };
        // The unit: exactly one Declared-domain row, among declarations and source units.
        let mut found = None;
        let mut count = 0_u64;
        for row in rows.declarations {
            work(budget, 1)?;
            if row.structural_node == *unit {
                if let SelectedExactIdentity::DeclaredUnit(id) = &row.exact_identity {
                    count += 1;
                    found = Some(id);
                } else {
                    count += 1;
                }
            }
        }
        for row in rows.source_units {
            work(budget, 1)?;
            if row.structural_node == *unit {
                count += 1;
                found = Some(&row.unit_id);
            }
        }
        let unit_bytes = match (count, found) {
            (1, Some(UnitId::Declared(key))) => key,
            _ => return Err(unavailable(unit, budget)),
        };
        let unit_text = hex_text(unit_bytes, unit, budget)?;
        let unit_text = unit_text.as_str();
        // The clock-binding value.
        budget.set_locus_form(CLOCK_FORM);
        work(budget, 1)?;
        let Some(admitted) = admitted_time_source(package, source) else {
            return Err(unavailable(source, budget));
        };
        let (model_name, tick) = match admitted {
            AdmittedTimeSource::Dense => (declared_name(package, model, budget)?, None),
            AdmittedTimeSource::Tick { period } => {
                work(budget, 1)?;
                let Some((source_model, operation)) = tick_source_references(package, source)
                else {
                    return Err(unavailable(source, budget));
                };
                let model_name = declared_name(package, source_model, budget)?;
                let operation = declared_name(package, operation, budget)?;
                let (numerator, denominator) = period.split_once('/').unwrap_or((period, "1"));
                (model_name, Some((operation, numerator, denominator)))
            }
        };
        let clock_binding = encoded(
            &Binding {
                profile: selection,
                model: &model_name,
                tick: tick.as_ref().map(|(operation, numerator, denominator)| {
                    (operation.as_ref(), *numerator, *denominator)
                }),
                unit: unit_text,
            },
            budget,
        )?;
        let Some(clause_key) = package
            .node_with_position(self.clause())
            .and_then(|(position, _)| package.structural_key_at(position))
        else {
            return Err(unavailable(self.clause(), budget));
        };
        Ok(SelectedMonitorLayoutContext {
            root_key: self.structural_key(),
            entry_keys,
            clause_key,
            package_lock,
            occurrence,
            profile,
            subject,
            clock_binding,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2};
    use ix_trace_rs::trace;

    fn selection() -> CheckedSelection {
        CheckedSelection {
            role: CheckedSelectionRole::TemporalProfile,
            definition: crate::CheckedArtifactRef {
                authority: "agent-ix".into(),
                identity: "quire.temporal.timed/v1".into(),
            },
        }
    }

    const PROFILE: &str = r#"{"authority":"agent-ix","identity":"quire.temporal.timed/v1","role":"temporal_profile"}"#;

    fn bytes(value: &impl Encode) -> String {
        let mut out = Vec::new();
        quire_canonical::encode(&mut out, value, quire_canonical::Limits::new(u64::MAX)).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[trace("TC-471", "FR-361-AC-3")]
    #[test]
    fn the_clock_binding_has_exactly_the_specified_members_and_each_change_alters_it() {
        let selection = selection();
        let tick = |model, operation, numerator, denominator, unit| Binding {
            profile: &selection,
            model,
            tick: Some((operation, numerator, denominator)),
            unit,
        };
        let base = bytes(&tick("m.M", "m.M.op", "2", "1", "u"));
        assert_eq!(
            base,
            format!(
                r#"{{"binding_kind":"model-time","period":{{"denominator":"1","numerator":"2"}},"selected_profile_identity":{PROFILE},"sequence_authority":"step-order","time_source":{{"kind":"tick","model":"m.M","operation":"m.M.op"}},"unit":"u"}}"#
            )
        );
        for changed in [
            tick("m.N", "m.M.op", "2", "1", "u"),
            tick("m.M", "m.M.other", "2", "1", "u"),
            tick("m.M", "m.M.op", "3", "1", "u"),
            tick("m.M", "m.M.op", "2", "2", "u"),
            tick("m.M", "m.M.op", "2", "1", "v"),
        ] {
            assert_ne!(bytes(&changed), base);
        }
        let dense = Binding {
            profile: &selection,
            model: "m.M",
            tick: None,
            unit: "u",
        };
        assert_ne!(bytes(&dense), base);
        let mut other = selection.clone();
        other.definition.identity = "quire.temporal.other/v1".into();
        let changed = Binding {
            profile: &other,
            model: "m.M",
            tick: None,
            unit: "u",
        };
        assert_ne!(bytes(&changed), bytes(&dense));
    }

    #[trace("TC-471", "FR-361-AC-3")]
    #[test]
    fn the_genuine_tick_source_names_its_model_operation_and_period() {
        let mut evidence = CheckedPackageEvidence::new();
        evidence.support_feature("quire.value.complete/v1");
        evidence
            .insert_domain_package_document(
                "example/config-version",
                include_bytes!(
                    "../../../../../../tests/fixtures/fold-binders/config-version-domain.json"
                )
                .to_vec(),
            )
            .unwrap();
        let limits = CheckedPackageReadLimits {
            bytes: 1 << 26,
            nodes: 1_000_000,
            edges: 10_000_000,
            occurrences: 10_000_000,
            diagnostics: 10_000,
            work: 1_000_000_000,
        };
        let crate::CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
            include_bytes!("../../../../../../tests/fixtures/fold-binders/monitor-base.json"),
            limits,
            &evidence,
        ) else {
            panic!("the genuine monitor package admits");
        };
        let source = package
            .graph()
            .nodes
            .iter()
            .find(|node| node.semantic_form.as_ref() == "time_source")
            .expect("genuine time source")
            .node_id
            .clone();
        let Some(AdmittedTimeSource::Tick { period }) = admitted_time_source(&package, &source)
        else {
            panic!("the genuine source is a tick source");
        };
        assert_eq!(period.split_once('/'), Some(("2", "1")));
        let (model, operation) = tick_source_references(&package, &source).unwrap();
        let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        assert_eq!(
            declared_name(&package, model, &mut budget).unwrap(),
            "ix://example/config-version/ConfigVersion"
        );
        let operation = declared_name(&package, operation, &mut budget).unwrap();
        assert!(
            operation == "attemptUpdate",
            "the operation's own identity: {operation}"
        );
    }
}
