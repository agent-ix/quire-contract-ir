// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Read-time verification of QSpec V2 source-reference claims before closed-schema decode.

use super::intake::{raw_member, source_raw};
use super::WorkMeter;
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, JsonPointer,
};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

fn array_at<'a>(bytes: &'a [u8], at: &JsonPointer) -> Option<Vec<&'a RawValue>> {
    serde_json::from_str(source_raw(bytes, at)?).ok()
}

fn array_member<'a>(raw: &'a RawValue, member: &str) -> Option<Vec<&'a RawValue>> {
    serde_json::from_str(raw_member(Some(raw), member)?.get()).ok()
}

fn source_member<'a>(raw: &'a RawValue) -> Option<&'a RawValue> {
    raw_member(Some(raw), "source")
}

struct Verifier<'a> {
    evidence: &'a CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    meter: &'a mut WorkMeter,
    charged_pairs: BTreeSet<(Box<str>, Box<str>)>,
    source_bytes: u64,
}

impl Verifier<'_> {
    fn check(&mut self, raw: &RawValue, at: JsonPointer) -> Result<(), ValidationFailure> {
        let Some(authority) = raw_member(Some(raw), "authority")
            .and_then(|value| serde_json::from_str::<String>(value.get()).ok())
        else {
            return Ok(());
        };
        let Some(identity) = raw_member(Some(raw), "identity")
            .and_then(|value| serde_json::from_str::<String>(value.get()).ok())
        else {
            return Ok(());
        };

        self.meter.charge(1, || at.clone())?;
        let claim = raw_member(Some(raw), "source_integrity")
            .and_then(|value| serde_json::from_str::<String>(value.get()).ok());
        let Some(claim) = claim.filter(|claim| {
            claim.len() == 64
                && claim
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        }) else {
            return Err(refused(at));
        };
        let Some(source) = self.evidence.source_document(&authority, &identity) else {
            return Err(refused(at));
        };
        if self
            .charged_pairs
            .insert((authority.into_boxed_str(), identity.into_boxed_str()))
        {
            self.source_bytes = self
                .source_bytes
                .saturating_add(u64::try_from(source.len()).unwrap_or(u64::MAX));
            if self.source_bytes > self.limits.bytes {
                return Err(ValidationFailure::incomplete(
                    CheckedPackageLimit::Bytes,
                    self.limits.bytes,
                    self.source_bytes,
                    None,
                ));
            }
        }
        if format!("{:x}", Sha256::digest(source)) != claim {
            return Err(refused(at));
        }
        Ok(())
    }
}

fn refused(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused_because(
        CheckedPackageRefusalCode::InvalidPackage,
        at,
        CheckedPackageRefusalCause::SourceIntegrity,
    )
}

/// Verify only the claims at the four QSpec source-reference sites, in wire order.
pub(super) fn verify(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut verifier = Verifier {
        evidence,
        limits,
        meter,
        charged_pairs: BTreeSet::new(),
        source_bytes: 0,
    };
    let lock_sources = JsonPointer::root().key("lock").key("sources");
    if let Some(rows) = array_at(bytes, &lock_sources) {
        for (index, raw) in rows.into_iter().enumerate() {
            verifier.check(raw, lock_sources.clone().index(index))?;
        }
    }
    let source_map = JsonPointer::root().key("source_map");
    if let Some(rows) = array_at(bytes, &source_map) {
        for (index, row) in rows.into_iter().enumerate() {
            if let Some(regions) = array_member(row, "regions") {
                for (region_index, region) in regions.into_iter().enumerate() {
                    if let Some(source) = source_member(region) {
                        verifier.check(
                            source,
                            source_map
                                .clone()
                                .index(index)
                                .key("regions")
                                .index(region_index)
                                .key("source"),
                        )?;
                    }
                }
            }
        }
    }
    let refinement = JsonPointer::root().key("refinement_step_rows");
    if let Some(rows) = array_at(bytes, &refinement) {
        for (index, row) in rows.into_iter().enumerate() {
            let region = raw_member(Some(row), "region");
            if let Some(source) = region.and_then(|region| source_member(region)) {
                verifier.check(
                    source,
                    refinement.clone().index(index).key("region").key("source"),
                )?;
            }
        }
    }
    let diagnostics = JsonPointer::root().key("diagnostics").key("entries");
    if let Some(rows) = array_at(bytes, &diagnostics) {
        for (index, row) in rows.into_iter().enumerate() {
            if let Some(loci) = array_member(row, "loci") {
                for (locus_index, locus) in loci.into_iter().enumerate() {
                    if let Some(source) = source_member(locus) {
                        verifier.check(
                            source,
                            diagnostics
                                .clone()
                                .index(index)
                                .key("loci")
                                .index(locus_index)
                                .key("source"),
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}
