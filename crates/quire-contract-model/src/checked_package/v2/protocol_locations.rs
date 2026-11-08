// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation and borrowed reads of the canonical captured protocol locations.
//! The common checked-term-data module owns all wire types and decoding.

use super::checked_term_data::CheckedProtocolSemanticLocation;
use super::checked_term_data::{CheckedKeyOrdinal, CheckedProtocolLocationOrigin};
use super::dispatch_projection::{data_site, fail, invalid};
use super::stage_storage::{self, Path};
use super::{
    CheckedNodeKind, CheckedPackageV2, CheckedPackageWireV2, CheckedSemanticNodeV2, CheckedTerm,
    ExpressionForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrence, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedSourceMapEntry,
};

const MAX_SAFE: u64 = 9_007_199_254_740_991;

fn at(row: usize) -> Path<'static> {
    Path::root()
        .key("checked_term_data")
        .key("protocol_locations")
        .index(row)
}

/// A malformed-wire refusal at `path`, its diagnostic charged before it is built.
fn malformed(meter: &mut WorkMeter, row: usize, path: Path<'static>) -> ValidationFailure {
    fail(
        meter,
        row,
        CheckedPackageRefusalCode::MalformedWire,
        Some(CheckedPackageRefusalCause::InvalidValue),
        path,
    )
}

/// One unit of work for the value at `path` in row `row`.
fn work(
    meter: &mut WorkMeter,
    row: usize,
    path: Path<'static>,
    amount: u64,
) -> Result<(), ValidationFailure> {
    stage_storage::work(meter, &data_site(row, path), amount)
}

/// Validate retained row keys, canonical ordinal shapes and local graph joins.
/// The envelope owner calls this after the typed node terms are available.
/// No graph interpretation or source-location reconstruction occurs here.
pub(super) fn validate<Body>(
    wire: &CheckedPackageWireV2<Body>,
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

fn validate_rows<Body>(
    name: &str,
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    source_map: &[CheckedSourceMapEntry],
    rows: &[CheckedProtocolSemanticLocation],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut previous: Option<&CheckedProtocolSemanticLocation> = None;
    let mut missing_predecessor = false;
    for (index, row) in rows.iter().enumerate() {
        let here = at(index);
        work(meter, index, here, 1)?;
        intake_local_application(&row.application, name, index, meter)?;
        let term_ordinal = intake_term_ordinal(row.term_ordinal, index, meter)?;
        if let Some(before) = previous {
            let key = (&row.application, &row.occurrence, term_ordinal);
            let prior = (&before.application, &before.occurrence, before.term_ordinal);
            if key == prior {
                return Err(fail(
                    meter,
                    index,
                    CheckedPackageRefusalCode::InvalidPackage,
                    Some(CheckedPackageRefusalCause::ConflictingDefinition),
                    here,
                ));
            }
            if key < prior {
                return Err(invalid(meter, index, here));
            }
        }
        let expected = previous
            .filter(|before| {
                before.application == row.application && before.occurrence == row.occurrence
            })
            .map_or(0, |before| before.term_ordinal + 1);
        missing_predecessor |= term_ordinal != expected;
        previous = Some(row);
        let mut selected = None;
        for (position, node) in nodes.iter().enumerate() {
            work(meter, index, here.key("application"), 1)?;
            if node.node_id == row.application {
                selected = Some((position, node));
                break;
            }
        }
        let Some((position, node)) = selected else {
            return Err(invalid(meter, index, here.key("application")));
        };
        // A leaf owns its location on its graph node. Its occurrence and a
        // matching enclosing application location cannot make a row valid.
        match (kinds.get(position), terms.get(position)) {
            (Some(CheckedNodeKind::Expression(form)), Some(CheckedTerm::Application { .. }))
                if *form != ExpressionForm::Reference => {}
            _ => return Err(invalid(meter, index, here.key("application"))),
        }
        let mut occurrence_found = false;
        for occurrence in &node.occurrences {
            work(meter, index, here.key("occurrence"), 1)?;
            occurrence_found |= occurrence == &row.occurrence;
        }
        if !occurrence_found {
            return Err(invalid(meter, index, here.key("occurrence")));
        }
        let mut mapped = false;
        for entry in source_map {
            work(meter, index, here.key("occurrence"), 1)?;
            mapped |= entry.node_id == row.application
                && entry.role == row.occurrence.role
                && entry.ordinal == row.occurrence.ordinal;
        }
        if !mapped {
            return Err(invalid(meter, index, here.key("occurrence")));
        }
        intake_location_value(row, index, meter)?;
    }
    // A retained successor witnesses the gap; a final row cannot witness an absent successor.
    // Existing row shape, join, duplicate and ordering refusals retain their precedence.
    if missing_predecessor {
        return Err(invalid(
            meter,
            0,
            Path::root()
                .key("checked_term_data")
                .key("protocol_locations"),
        ));
    }
    Ok(())
}

/// Convert an incoming numeric term coordinate to its safe wire range.
#[ix_trace_rs::string_edge]
fn intake_term_ordinal(
    wire: u64,
    index: usize,
    meter: &mut WorkMeter,
) -> Result<u64, ValidationFailure> {
    if wire > MAX_SAFE {
        return Err(malformed(meter, index, at(index).key("term_ordinal")));
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
    meter: &mut WorkMeter,
) -> Result<&'a CheckedNodeId, ValidationFailure> {
    if wire_id.package.as_ref() != root_package {
        return Err(invalid(meter, index, at(index).key("application")));
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
    let origin = at(index).key("location").key("origin");
    match &row.location.origin {
        CheckedProtocolLocationOrigin::TypeDeclaration { name } => {
            work(
                meter,
                index,
                origin,
                u64::try_from(name.len()).unwrap_or(u64::MAX),
            )?;
            if name.is_empty() {
                return Err(malformed(meter, index, origin));
            }
        }
        CheckedProtocolLocationOrigin::ProtocolAttempt { protocol, attempt } => {
            if *protocol > MAX_SAFE || *attempt > MAX_SAFE {
                return Err(malformed(meter, index, origin));
            }
        }
    }
    for (position, ordinal) in row.location.path.iter().enumerate() {
        let locus = at(index).key("location").key("path").index(position);
        work(meter, index, locus, 1)?;
        match ordinal {
            CheckedKeyOrdinal::Safe(value) if *value <= MAX_SAFE => {}
            CheckedKeyOrdinal::BeyondSafe(text) => {
                work(
                    meter,
                    index,
                    locus,
                    u64::try_from(text.len()).unwrap_or(u64::MAX),
                )?;
                let beyond =
                    text.len() > 16 || (text.len() == 16 && text.as_ref() > "9007199254740991");
                if !beyond || text.starts_with('0') || !text.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(malformed(meter, index, locus));
                }
            }
            CheckedKeyOrdinal::Safe(_) => return Err(malformed(meter, index, locus)),
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

#[cfg(test)]
mod tests {
    use super::super::ValueForm;
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[test]
    #[trace("TC-048", "FR-038-AC-448")]
    fn every_value_form_and_reference_is_ineligible_for_a_location_row() {
        let id = json!({"package":"example","path":["value","literal","leaf"],"ordinal":0});
        let node: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id": id, "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"value", "semantic_form":"literal", "semantic_type":id,
            "dependencies":[], "occurrences":[{"role":"expression","ordinal":0}],
            "body":{"term":"reference","target":id}
        }))
        .unwrap();
        let row: CheckedProtocolSemanticLocation = serde_json::from_value(json!({
            "application":id, "occurrence":{"role":"expression","ordinal":0},
            "term_ordinal":0,
            "location":{"origin":{"kind":"type_declaration","name":"protocol P"},"path":[]}
        }))
        .unwrap();
        let term = CheckedTerm::Reference {
            target: node.node_id.clone(),
        };
        for kind in ValueForm::ALL
            .iter()
            .copied()
            .map(CheckedNodeKind::Value)
            .chain([CheckedNodeKind::Expression(ExpressionForm::Reference)])
        {
            let result = validate_rows(
                "example",
                std::slice::from_ref(&node),
                &[kind],
                std::slice::from_ref(&term),
                &[],
                std::slice::from_ref(&row),
                &mut WorkMeter::new(u64::MAX),
            );
            let Err(ValidationFailure::Refused(refusal)) = result else {
                panic!("{kind:?} must refuse its location row: {result:?}");
            };
            assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
            assert_eq!(
                refusal.cause,
                Some(CheckedPackageRefusalCause::InvalidValue)
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().unwrap().as_str(),
                "/checked_term_data/protocol_locations/0/application",
                "{kind:?}"
            );
        }
    }
}
