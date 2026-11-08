// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Typed clock configuration rows for temporal and compensation declarations.

use super::graph_storage::GraphIndex;
use super::term_view::TermNode;
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedOccurrenceRole, CheckedPackageLockV2,
    CheckedSelectionRole, LawRole, ProtocolForm, RawSemanticNodeV2, TemporalForm, TemporalProfile,
    WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// An exact number in a fixed-sample clock configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClockExactNumber {
    /// A canonical signed decimal integer.
    Integer {
        /// Decimal spelling without leading zeroes or negative zero.
        decimal: Box<str>,
    },
    /// A reduced rational with a positive denominator.
    Rational {
        /// Canonical signed numerator.
        numerator: Box<str>,
        /// Canonical positive denominator.
        denominator: Box<str>,
    },
}

/// The three closed clock configurations selected by temporal profiles.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClockConfiguration {
    /// Discrete positions from a named event sequence.
    EventPosition {
        /// Nonempty sequence authority.
        sequence_authority: Box<str>,
    },
    /// Exact samples at a fixed period from an epoch.
    FixedSample {
        /// Exact epoch.
        epoch: ClockExactNumber,
        /// Strictly positive exact period.
        period: ClockExactNumber,
        /// Nonempty unit name.
        unit: Box<str>,
    },
    /// Timestamps supplied by events.
    TimestampedEvent {
        /// Nonempty timestamp unit.
        timestamp_unit: Box<str>,
    },
}

/// One admitted clock selection keyed by a declaration and source occurrence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalBinding {
    /// Exact temporal clause or compensation declaration ID.
    pub declaration: CheckedNodeId,
    /// Ordinal of the clause's claim or compensation's declaration occurrence.
    pub occurrence: u64,
    /// Exact nonempty text of the declaration's clock argument.
    pub clock_name: Box<str>,
    /// Closed clock configuration.
    pub clock: ClockConfiguration,
}

impl Encode for TemporalBinding {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "declaration", &self.declaration)?;
        super::encode::member(writer, "occurrence", &self.occurrence)?;
        super::encode::member(writer, "clock_name", self.clock_name.as_ref())?;
        super::encode::member(writer, "clock", &self.clock)?;
        writer.end_object()
    }
}

fn row_pointer(index: usize) -> JsonPointer {
    JsonPointer::root().key("temporal_bindings").index(index)
}

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

fn invalid(at: JsonPointer, declaration: &CheckedNodeId) -> ValidationFailure {
    ValidationFailure::refused_at(
        CheckedPackageRefusalCode::InvalidPackage,
        at,
        Some(CheckedPackageRefusalCause::TimedNodeReference),
        declaration.clone(),
    )
}

#[ix_trace_rs::string_edge]
fn decimal_magnitude(text: &str) -> Option<(&str, bool)> {
    if text == "0" {
        return Some(("0", true));
    }
    let digits = text.strip_prefix('-').unwrap_or(text);
    let mut bytes = digits.bytes();
    (bytes
        .next()
        .is_some_and(|first| (b'1'..=b'9').contains(&first))
        && bytes.all(|byte| byte.is_ascii_digit()))
    .then_some((digits, false))
}

fn validate_number(
    number: &ClockExactNumber,
    positive: bool,
    at: JsonPointer,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    match number {
        ClockExactNumber::Integer { decimal } => {
            let Some((_, zero)) = decimal_magnitude(decimal) else {
                return Err(malformed(at.key("decimal")));
            };
            if positive && (zero || decimal.starts_with('-')) {
                return Err(malformed(at.key("decimal")));
            }
        }
        ClockExactNumber::Rational {
            numerator,
            denominator,
        } => {
            let Some((digits, zero)) = decimal_magnitude(numerator) else {
                return Err(malformed(at.clone().key("numerator")));
            };
            let valid_denominator = denominator
                .bytes()
                .next()
                .is_some_and(|first| (b'1'..=b'9').contains(&first))
                && denominator
                    .bytes()
                    .skip(1)
                    .all(|byte| byte.is_ascii_digit());
            if !valid_denominator {
                return Err(malformed(at.clone().key("denominator")));
            }
            if positive && (zero || numerator.starts_with('-')) {
                return Err(malformed(at.clone().key("numerator")));
            }
            if !super::natural::coprime(digits, denominator, meter, &|| at.clone())? {
                return Err(malformed(at));
            }
        }
    }
    Ok(())
}

fn validate_shape(
    rows: &[TemporalBinding],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for (index, row) in rows.iter().enumerate() {
        let at = row_pointer(index);
        meter.charge(1, || at.clone())?;
        if row.clock_name.is_empty() {
            return Err(malformed(at.clone().key("clock_name")));
        }
        let clock_at = at.key("clock");
        match &row.clock {
            ClockConfiguration::EventPosition { sequence_authority } => {
                if sequence_authority.is_empty() {
                    return Err(malformed(clock_at.key("sequence_authority")));
                }
            }
            ClockConfiguration::FixedSample {
                epoch,
                period,
                unit,
            } => {
                if unit.is_empty() {
                    return Err(malformed(clock_at.clone().key("unit")));
                }
                validate_number(epoch, false, clock_at.clone().key("epoch"), meter)?;
                validate_number(period, true, clock_at.key("period"), meter)?;
            }
            ClockConfiguration::TimestampedEvent { timestamp_unit } => {
                if timestamp_unit.is_empty() {
                    return Err(malformed(clock_at.key("timestamp_unit")));
                }
            }
        }
    }
    Ok(())
}

/// Whether `value` is a string.
fn is_text(value: Option<TermNode<'_>>) -> bool {
    value.is_some_and(|value| value.as_str().is_some())
}

/// Whether `law` is an operation law: a `role` text and a `definition` of
/// exactly an `authority` and an `identity`.
fn is_operation_law(law: TermNode<'_>) -> bool {
    law.exact_members(&["role", "definition"])
        && is_text(law.get("role"))
        && law.get("definition").is_some_and(|definition| {
            definition.exact_members(&["authority", "identity"])
                && is_text(definition.get("authority"))
                && is_text(definition.get("identity"))
        })
}

/// Whether `mode` is null or exactly a `kind` and a `value` text.
fn is_operation_mode(mode: TermNode<'_>) -> bool {
    mode.is_null()
        || (mode.exact_members(&["kind", "value"])
            && is_text(mode.get("kind"))
            && is_text(mode.get("value")))
}

/// Whether `leaf` is an operation leaf: a `path` of texts, `laws`, and an
/// optional `mode`, with no other member.
fn is_operation_leaf(leaf: TermNode<'_>) -> bool {
    let Some(members) = leaf.members() else {
        return false;
    };
    let (mut path, mut laws) = (false, false);
    for (name, value) in members {
        let fine = match name {
            "path" => {
                path = true;
                value
                    .items()
                    .is_some_and(|mut items| items.all(|item| item.as_str().is_some()))
            }
            "laws" => {
                laws = true;
                value
                    .items()
                    .is_some_and(|mut items| items.all(is_operation_law))
            }
            "mode" => is_operation_mode(value),
            _ => false,
        };
        if !fine {
            return false;
        }
    }
    path && laws
}

/// Whether `operation` has the closed shape `OperationWire` decodes: the five
/// members, `mode` and `member` present (null allowed).
fn is_operation_wire(operation: TermNode<'_>) -> bool {
    operation.exact_members(&["identity", "laws", "mode", "member", "leaves"])
        && is_text(operation.get("identity"))
        && operation
            .get("laws")
            .and_then(TermNode::items)
            .is_some_and(|mut laws| laws.all(is_operation_law))
        && operation.get("mode").is_some_and(is_operation_mode)
        && operation
            .get("leaves")
            .and_then(TermNode::items)
            .is_some_and(|mut leaves| leaves.all(is_operation_leaf))
}

/// The one temporal-profile law of the operation of graph node `position`,
/// read from the retained document; its two texts are charged before they are
/// copied. `None` when the body is not a decodable operation, or does not
/// carry exactly one such law.
pub(super) fn profile_law(
    position: usize,
    meter: &mut WorkMeter,
) -> Result<Option<super::CheckedArtifactRef>, ValidationFailure> {
    let Some((body, _)) = meter.body(position) else {
        return Ok(None);
    };
    let Some(operation) = body.get("operation").filter(|op| is_operation_wire(*op)) else {
        return Ok(None);
    };
    let Some(mut laws) = operation.get("laws").and_then(TermNode::items) else {
        return Ok(None);
    };
    let (Some(law), None) = (laws.next(), laws.next()) else {
        return Ok(None);
    };
    let role = law.get("role").and_then(TermNode::as_str);
    if role.and_then(LawRole::from_wire) != Some(LawRole::TemporalProfile) {
        return Ok(None);
    }
    let Some(definition) = law.get("definition") else {
        return Ok(None);
    };
    let (Some(authority), Some(identity)) = (
        definition.get("authority").and_then(TermNode::as_str),
        definition.get("identity").and_then(TermNode::as_str),
    ) else {
        return Ok(None);
    };
    meter.at_node(position, "profile_law");
    let authority = meter.boxed_str(authority)?;
    let identity = meter.boxed_str(identity)?;
    Ok(Some(super::CheckedArtifactRef {
        authority,
        identity,
    }))
}

fn clock_configured(profile: TemporalProfile) -> Option<TemporalProfile> {
    matches!(
        profile,
        TemporalProfile::EventPositionFalseExtension
            | TemporalProfile::FixedSampleFalseExtension
            | TemporalProfile::TimestampedEventFiniteWindow
    )
    .then_some(profile)
}

fn configured_profile(
    position: usize,
    meter: &mut WorkMeter,
) -> Result<Option<TemporalProfile>, ValidationFailure> {
    Ok(profile_law(position, meter)?
        .and_then(|law| TemporalProfile::from_wire(&law.identity))
        .and_then(clock_configured))
}

fn selected_profile(lock: &CheckedPackageLockV2) -> Option<TemporalProfile> {
    let selection = lock
        .profile_selections
        .iter()
        .find(|selection| selection.role == CheckedSelectionRole::TemporalProfile)?;
    clock_configured(TemporalProfile::from_wire(&selection.definition.identity)?)
}

/// The text of argument `argument` of the clause at `position`, a literal
/// text or a binding of one, borrowed from the retained document.
#[ix_trace_rs::string_edge]
pub(super) fn clock_name<'b>(
    meter: &WorkMeter<'b>,
    position: usize,
    argument: usize,
) -> Option<&'b str> {
    let (body, _) = meter.body(position)?;
    let term = body.get("arguments")?.items()?.nth(argument)?;
    let term = if term.get("term")?.as_str()? == "binding" {
        term.get("value")?
    } else {
        term
    };
    (term.get("term")?.as_str()? == "literal" && term.get("value_kind")?.as_str()? == "text")
        .then(|| term.get("value")?.as_str())?
}

fn clock_matches(profile: Option<TemporalProfile>, clock: &ClockConfiguration) -> bool {
    matches!(
        (profile, clock),
        (
            Some(TemporalProfile::EventPositionFalseExtension),
            ClockConfiguration::EventPosition { .. }
        ) | (
            Some(TemporalProfile::FixedSampleFalseExtension),
            ClockConfiguration::FixedSample { .. }
        ) | (
            Some(TemporalProfile::TimestampedEventFiniteWindow),
            ClockConfiguration::TimestampedEvent { .. }
        )
    )
}

/// Validate row coverage before per-node joins and compare names before kinds.
pub(super) fn validate(
    rows: Option<&[TemporalBinding]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let rows = rows.unwrap_or_default();
    validate_shape(rows, meter)?;
    let compensation_profile = selected_profile(lock);
    let mut seen = BTreeSet::new();
    let mut previous: Option<(super::graph_storage::ResolvedKey<'_>, u64)> = None;
    let mut resolved = Vec::with_capacity(rows.len());
    for (row_index, row) in rows.iter().enumerate() {
        let at = row_pointer(row_index);
        let Some(&position) = index.get(&row.declaration) else {
            return Err(invalid(at.key("declaration"), &row.declaration));
        };
        if !matches!(
            kinds[position],
            CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
                | CheckedNodeKind::Protocol(ProtocolForm::Compensation)
        ) {
            return Err(invalid(at.key("declaration"), &row.declaration));
        }
        if kinds[position] == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
            && profile_law(position, meter)?
                .as_ref()
                .and_then(|law| TemporalProfile::from_wire(&law.identity))
                == Some(TemporalProfile::Timed)
        {
            return Err(invalid(at, &row.declaration));
        }
        let role = if kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::Compensation) {
            CheckedOccurrenceRole::Declaration
        } else {
            CheckedOccurrenceRole::Claim
        };
        let mut owned = false;
        for occurrence in &nodes[position].occurrences {
            meter.charge(1, || at.clone().key("occurrence"))?;
            if occurrence.role == role && occurrence.ordinal == row.occurrence {
                owned = true;
                break;
            }
        }
        if !owned {
            return Err(invalid(at.key("occurrence"), &row.declaration));
        }
        if !seen.insert((&row.declaration, row.occurrence)) {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                at,
                Some(CheckedPackageRefusalCause::ConflictingDefinition),
                row.declaration.clone(),
            ));
        }
        let structural = index.resolved_at(position, structural_keys);
        if let Some((previous_id, previous_occurrence)) = previous {
            if previous_id > structural {
                return Err(invalid(at.key("declaration"), &row.declaration));
            }
            if previous_id == structural && previous_occurrence > row.occurrence {
                return Err(invalid(at.key("occurrence"), &row.declaration));
            }
        }
        previous = Some((structural, row.occurrence));
        resolved.push(position);
    }
    let mut missing = None;
    for (position, node) in nodes.iter().enumerate() {
        let role = if kinds[position] == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
            && configured_profile(position, meter)?.is_some()
        {
            CheckedOccurrenceRole::Claim
        } else if kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::Compensation)
            && compensation_profile.is_some()
        {
            CheckedOccurrenceRole::Declaration
        } else {
            continue;
        };
        for occurrence in &node.occurrences {
            meter.charge(1, || node_pointer(position).key("body"))?;
            if occurrence.role == role && !seen.contains(&(&node.node_id, occurrence.ordinal)) {
                missing = Some(position);
                break;
            }
        }
        if missing.is_some() {
            break;
        }
    }
    if let Some(position) = missing {
        return Err(invalid(
            node_pointer(position).key("body"),
            &nodes[position].node_id,
        ));
    }
    for (row_index, (row, &position)) in rows.iter().zip(&resolved).enumerate() {
        let node = &nodes[position];
        let at = row_pointer(row_index);
        if kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::Compensation) {
            if clock_name(meter, position, 4) != Some(row.clock_name.as_ref()) {
                return Err(invalid(at.clone().key("clock_name"), &row.declaration));
            }
            if !clock_matches(compensation_profile, &row.clock) {
                return Err(invalid(at.key("clock").key("kind"), &row.declaration));
            }
            continue;
        }
        let law = profile_law(position, meter)?;
        if law
            .as_ref()
            .is_some_and(|law| TemporalProfile::from_wire(&law.identity).is_none())
        {
            // The temporal step owns unknown-profile classification at the
            // law definition. Do not let this row's profile join preempt it.
            continue;
        }
        if clock_name(meter, position, 1) != Some(row.clock_name.as_ref()) {
            return Err(invalid(at.clone().key("clock_name"), &row.declaration));
        }
        let profile = configured_profile(position, meter)?;
        let mut selected = false;
        if let Some(law) = law.as_ref() {
            for selection in &lock.profile_selections {
                meter.charge(1, || at.clone().key("clock").key("kind"))?;
                if selection.role == CheckedSelectionRole::TemporalProfile
                    && selection.definition == *law
                {
                    selected = true;
                    break;
                }
            }
        }
        if !selected || !clock_matches(profile, &row.clock) {
            return Err(invalid(at.key("clock").key("kind"), &row.declaration));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::graph_storage::OwnedBodies;
    use super::super::native_join_budget::NativeJoinBudget;
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn node_body(operation: serde_json::Value, clock: &str) -> serde_json::Value {
        json!({"term":"application","operation":operation,"arguments":[
            {"term":"binding","name":"clock","value":
                {"term":"literal","kind":"text","value_kind":"text","value":clock}},
            {"term":"literal","value_kind":"text","value":clock}]})
    }

    fn operation(laws: serde_json::Value) -> serde_json::Value {
        json!({"identity":"quire.op.temporal.always","laws":laws,"mode":null,"member":null,"leaves":[]})
    }

    fn law(role: &str, identity: &str) -> serde_json::Value {
        json!({"role":role,"definition":{"authority":"quire","identity":identity}})
    }

    /// The profile law and the clock name read from the document agree with
    /// the typed decode of the same bodies, and the law's two texts are
    /// allocated exactly as charged (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn profile_law_and_clock_name_read_the_document_as_the_decode_does() {
        let profile = law("temporal_profile", "quire.profile.timed");
        let cases = [
            operation(json!([profile.clone()])),
            operation(json!([])),
            operation(json!([profile.clone(), profile.clone()])),
            operation(json!([law("other", "x")])),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"leaves":[]}),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"member":null,"leaves":[],"extra":1}),
            json!({"identity":"x","laws":[profile.clone()],"mode":{"kind":"k","value":3},"member":null,"leaves":[]}),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"member":null,
                   "leaves":[{"path":["a"],"laws":[],"mode":null}]}),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"member":null,
                   "leaves":[{"path":["a"]}]}),
            json!({"identity":"x","laws":[profile.clone()],"mode":null,"member":null,
                   "leaves":[{"path":[1],"laws":[]}]}),
            json!({"identity":"x","laws":[{"role":"temporal_profile","definition":
                   {"authority":"q","identity":"i","more":1}}],"mode":null,"member":null,"leaves":[]}),
        ];
        for (n, case) in cases.iter().enumerate() {
            let body = node_body(case.clone(), "model-time");
            let expected =
                serde_json::from_value::<super::super::operations::OperationWire>(case.clone())
                    .ok()
                    .and_then(|wire| {
                        let [law] = wire.laws.as_slice() else {
                            return None;
                        };
                        (law.role_class() == Some(LawRole::TemporalProfile))
                            .then(|| law.definition.clone())
                    });
            let bodies = OwnedBodies::of_values(&[body], vec![Vec::new()]);
            let mut meter = WorkMeter::new(u64::MAX);
            bodies.attach(&mut meter);
            let read = profile_law(0, &mut meter).expect("an unlimited meter admits");
            assert_eq!(read, expected, "case {n}");
            assert_eq!(clock_name(&meter, 0, 1), Some("model-time"), "case {n}");
            assert_eq!(clock_name(&meter, 0, 0), Some("model-time"), "case {n}");
            assert_eq!(clock_name(&meter, 0, 2), None, "case {n}");
        }

        // The law's two texts are charged before they are copied, to the byte.
        let body = node_body(cases[0].clone(), "model-time");
        let bodies = OwnedBodies::of_values(&[body], vec![Vec::new()]);
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut outcome = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                bodies.attach(&mut meter);
                outcome = Some(profile_law(0, &mut meter));
            });
            (
                outcome.expect("ran").ok().flatten(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (law, allocated, charged) = run(u64::MAX);
        assert!(law.is_some() && charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert!(run(charged).0.is_some(), "the measured limit admits");
        assert!(run(charged - 1).0.is_none(), "one byte short is denied");
    }
}
