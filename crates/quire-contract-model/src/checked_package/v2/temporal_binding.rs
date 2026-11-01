// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Typed clock configuration rows for temporal clause declarations.

use super::{
    CheckedNodeId, CheckedNodeKind, CheckedPackageLockV2, CheckedSelectionRole, LawRole,
    RawSemanticNodeV2, TemporalForm, TemporalProfile, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

/// One admitted clock selection keyed by a temporal clause's structural ID.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalBinding {
    /// Exact temporal clause declaration ID.
    pub declaration: CheckedNodeId,
    /// Exact nonempty text of the clause clock argument.
    pub clock_name: Box<str>,
    /// Closed clock configuration.
    pub clock: ClockConfiguration,
}

impl Encode for TemporalBinding {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "declaration", &self.declaration)?;
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

fn profile_law(node: &RawSemanticNodeV2) -> Option<super::CheckedArtifactRef> {
    let operation: super::operations::OperationWire =
        serde_json::from_value(node.body.get("operation")?.clone()).ok()?;
    let [law] = operation.laws.as_slice() else {
        return None;
    };
    (law.role_class() == Some(LawRole::TemporalProfile)).then(|| law.definition.clone())
}

fn configured_profile(node: &RawSemanticNodeV2) -> Option<TemporalProfile> {
    let law = profile_law(node)?;
    let profile = TemporalProfile::from_wire(&law.identity)?;
    matches!(
        profile,
        TemporalProfile::EventPositionFalseExtension
            | TemporalProfile::FixedSampleFalseExtension
            | TemporalProfile::TimestampedEventFiniteWindow
    )
    .then_some(profile)
}

fn clause_clock_name(node: &RawSemanticNodeV2) -> Option<&str> {
    let term = node.body.get("arguments")?.as_array()?.get(1)?;
    (term.get("term")?.as_str()? == "literal" && term.get("value_kind")?.as_str()? == "text")
        .then(|| term.get("value")?.as_str())?
}

/// Validate row coverage before per-node joins and compare names before kinds.
pub(super) fn validate(
    rows: Option<&[TemporalBinding]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let rows = rows.unwrap_or_default();
    validate_shape(rows, meter)?;
    let mut seen = BTreeSet::new();
    let mut previous = None;
    let mut resolved = Vec::with_capacity(rows.len());
    for (row_index, row) in rows.iter().enumerate() {
        let at = row_pointer(row_index);
        if !seen.insert(&row.declaration) {
            return Err(invalid(at, &row.declaration));
        }
        let Some(&position) = index.get(&row.declaration) else {
            return Err(invalid(at.key("declaration"), &row.declaration));
        };
        if kinds[position] != CheckedNodeKind::Temporal(TemporalForm::TemporalClause) {
            return Err(invalid(at.key("declaration"), &row.declaration));
        }
        let structural = structural_keys[position].as_ref();
        if previous.is_some_and(|id: &CheckedNodeId| id > structural) {
            return Err(invalid(at.key("declaration"), &row.declaration));
        }
        previous = Some(structural);
        resolved.push(position);
    }
    let missing = index.values().copied().filter(|&position| {
        kinds[position] == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
            && configured_profile(&nodes[position]).is_some()
            && !seen.contains(&nodes[position].node_id)
    });
    if let Some(position) = missing.min_by_key(|&position| structural_keys[position].as_ref()) {
        return Err(invalid(
            node_pointer(position).key("body"),
            &nodes[position].node_id,
        ));
    }
    for (row_index, (row, &position)) in rows.iter().zip(&resolved).enumerate() {
        let node = &nodes[position];
        let at = row_pointer(row_index);
        if clause_clock_name(node) != Some(row.clock_name.as_ref()) {
            return Err(invalid(at.clone().key("clock_name"), &row.declaration));
        }
        let law = profile_law(node);
        let profile = configured_profile(node);
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
        let kind_matches = matches!(
            (profile, &row.clock),
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
        );
        if !selected || !kind_matches {
            return Err(invalid(at.key("clock").key("kind"), &row.declaration));
        }
    }
    Ok(())
}
