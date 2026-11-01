// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation and borrowed reads of the canonical captured protocol locations.
//! The common checked-term-data module owns all wire types and decoding.

use super::checked_term_data::CheckedProtocolSemanticLocation;
use super::checked_term_data::{CheckedKeyOrdinal, CheckedProtocolLocationOrigin};
use super::{CheckedPackageV2, CheckedTerm, RawCheckedPackageWireV2, RawSemanticNodeV2, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodeKind, CheckedOccurrence, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedSourceMapEntry, JsonPointer,
};

const MAX_SAFE: u64 = 9_007_199_254_740_991;

fn at(row: usize) -> JsonPointer {
    JsonPointer::root()
        .key("checked_term_data")
        .key("protocol_locations")
        .index(row)
}
fn malformed(path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused_because(
        CheckedPackageRefusalCode::MalformedWire,
        path,
        CheckedPackageRefusalCause::InvalidValue,
    )
}
fn invalid(path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused_because(
        CheckedPackageRefusalCode::InvalidPackage,
        path,
        CheckedPackageRefusalCause::InvalidValue,
    )
}

/// Validate retained row keys, canonical ordinal shapes and local graph joins.
/// The envelope owner calls this after the typed node terms are available.
/// No graph interpretation or source-location reconstruction occurs here.
pub(super) fn validate(
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(data) = wire.checked_term_data.as_ref() else {
        return Ok(());
    };
    validate_rows(
        &wire.name,
        &wire.semantic_graph.nodes,
        kinds,
        terms,
        &wire.source_map,
        &data.protocol_locations,
        meter,
    )
}

fn validate_rows(
    name: &str,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    source_map: &[CheckedSourceMapEntry],
    rows: &[CheckedProtocolSemanticLocation],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut previous: Option<&CheckedProtocolSemanticLocation> = None;
    for (index, row) in rows.iter().enumerate() {
        meter.charge(1, || at(index))?;
        intake_local_application(&row.application, name, index)?;
        let term_ordinal = intake_term_ordinal(row.term_ordinal, index)?;
        if let Some(before) = previous {
            let key = (&row.application, &row.occurrence, term_ordinal);
            let prior = (&before.application, &before.occurrence, before.term_ordinal);
            if key == prior {
                return Err(ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::InvalidPackage,
                    at(index),
                    CheckedPackageRefusalCause::ConflictingDefinition,
                ));
            }
            if key < prior {
                return Err(invalid(at(index)));
            }
        }
        previous = Some(row);
        let mut selected = None;
        for (position, node) in nodes.iter().enumerate() {
            meter.charge(1, || at(index).key("application"))?;
            if node.node_id == row.application {
                selected = Some((position, node));
                break;
            }
        }
        let (position, node) = selected.ok_or_else(|| invalid(at(index).key("application")))?;
        if !matches!(kinds.get(position), Some(CheckedNodeKind::Expression(_)))
            || !matches!(terms.get(position), Some(CheckedTerm::Application { .. }))
        {
            return Err(invalid(at(index).key("application")));
        }
        let mut occurrence_found = false;
        for occurrence in &node.occurrences {
            meter.charge(1, || at(index).key("occurrence"))?;
            occurrence_found |= occurrence == &row.occurrence;
        }
        if !occurrence_found {
            return Err(invalid(at(index).key("occurrence")));
        }
        let mut mapped = false;
        for entry in source_map {
            meter.charge(1, || at(index).key("occurrence"))?;
            mapped |= entry.node_id == row.application
                && entry.role == row.occurrence.role
                && entry.ordinal == row.occurrence.ordinal;
        }
        if !mapped {
            return Err(invalid(at(index).key("occurrence")));
        }
        intake_location_value(row, index, meter)?;
    }
    Ok(())
}

/// Convert an incoming numeric term coordinate to its safe wire range.
#[ix_trace_rs::string_edge]
fn intake_term_ordinal(wire: u64, index: usize) -> Result<u64, ValidationFailure> {
    if wire > MAX_SAFE {
        return Err(malformed(at(index).key("term_ordinal")));
    }
    Ok(wire)
}

/// Bind the incoming sidecar NodeRef's namespace before any graph join.
/// This checks untrusted wire selection; it does not classify an admitted node.
#[ix_trace_rs::string_edge]
fn intake_local_application<'a>(
    wire_id: &'a CheckedNodeId,
    root_package: &str,
    index: usize,
) -> Result<&'a CheckedNodeId, ValidationFailure> {
    if wire_id.package.as_ref() != root_package {
        return Err(invalid(at(index).key("application")));
    }
    Ok(wire_id)
}

/// Check the captured origin and ordinal wire grammar, without graph decisions.
#[ix_trace_rs::string_edge]
fn intake_location_value(
    row: &CheckedProtocolSemanticLocation,
    index: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    match &row.location.origin {
        CheckedProtocolLocationOrigin::TypeDeclaration { name } => {
            meter.charge(u64::try_from(name.len()).unwrap_or(u64::MAX), || {
                at(index).key("location").key("origin")
            })?;
            if name.is_empty() {
                return Err(malformed(at(index).key("location").key("origin")));
            }
        }
        CheckedProtocolLocationOrigin::ProtocolAttempt { protocol, attempt } => {
            if *protocol > MAX_SAFE || *attempt > MAX_SAFE {
                return Err(malformed(at(index).key("location").key("origin")));
            }
        }
    }
    for (position, ordinal) in row.location.path.iter().enumerate() {
        let locus = || at(index).key("location").key("path").index(position);
        meter.charge(1, locus)?;
        match ordinal {
            CheckedKeyOrdinal::Safe(value) if *value <= MAX_SAFE => {}
            CheckedKeyOrdinal::BeyondSafe(text) => {
                meter.charge(u64::try_from(text.len()).unwrap_or(u64::MAX), locus)?;
                let beyond =
                    text.len() > 16 || (text.len() == 16 && text.as_ref() > "9007199254740991");
                if !beyond || text.starts_with('0') || !text.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(malformed(locus()));
                }
            }
            CheckedKeyOrdinal::Safe(_) => return Err(malformed(locus())),
        }
    }
    Ok(())
}

/// Why a selected captured semantic location could not be returned.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedProtocolLocationReadError {
    /// The admitted package retained no checked-term-data sidecar.
    #[error("protocol semantic location data is unavailable")]
    Unavailable,
    /// The requested exact application/occurrence/term key has no captured row.
    #[error("the selected protocol term has no captured location")]
    Missing,
    /// The first uncharged row or child-path entry exceeded the caller limit.
    #[error("protocol semantic location read exhausted its work allowance")]
    WorkExhausted {
        /// Caller-supplied finite work limit.
        limit: u64,
        /// Attempted work including the denied charge.
        consumed: u64,
        /// Sidecar row where work stopped.
        row: usize,
        /// Child-path position when that entry could not be charged.
        child: Option<usize>,
    },
}

impl CheckedPackageV2 {
    /// Borrow the captured row for one exact local application occurrence and
    /// semantic term coordinate. An absent mapping is never inferred from a
    /// source-map region or another term's mapping.
    pub fn protocol_location(
        &self,
        application: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
        term_ordinal: u64,
        work_limit: u64,
    ) -> Result<&CheckedProtocolSemanticLocation, CheckedProtocolLocationReadError> {
        let data = self
            .wire
            .checked_term_data
            .as_ref()
            .ok_or(CheckedProtocolLocationReadError::Unavailable)?;
        let mut consumed = 0_u64;
        for (index, row) in data.protocol_locations.iter().enumerate() {
            charge(&mut consumed, work_limit, index, None)?;
            if &row.application != application
                || &row.occurrence != occurrence
                || row.term_ordinal != term_ordinal
            {
                continue;
            }
            for child in 0..row.location.path.len() {
                charge(&mut consumed, work_limit, index, Some(child))?;
            }
            return Ok(row);
        }
        Err(CheckedProtocolLocationReadError::Missing)
    }
}

fn charge(
    consumed: &mut u64,
    limit: u64,
    row: usize,
    child: Option<usize>,
) -> Result<(), CheckedProtocolLocationReadError> {
    let next = consumed.checked_add(1);
    *consumed = next.unwrap_or(u64::MAX);
    if next.is_none() || *consumed > limit {
        return Err(CheckedProtocolLocationReadError::WorkExhausted {
            limit,
            consumed: *consumed,
            row,
            child,
        });
    }
    Ok(())
}
