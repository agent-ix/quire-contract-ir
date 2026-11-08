// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-348 extension form body grammar, read from the package document's node
//! body through the term view: no `serde_json` value, parser or deserializer.
//! The targets are the node ids the body's clauses reference, each read (and
//! its heap charged) through the caller's sink before the table that holds
//! them is requested.

use super::identity::is_identifier;
use crate::checked_package::common::{Ids, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusal, CheckedPackageRefusalCode as Code,
    CheckedPackageRefusalLocation,
};
use crate::checked_package::terms::TermSource;
use std::mem::size_of;

/// The node ids an extension body's clauses reference, in clause order.
pub(super) struct ExtensionBody {
    targets: Vec<CheckedNodeId>,
}

/// The body has its own closed grammar, checked before key authentication.
#[ix_trace_rs::string_edge]
pub(super) fn read_body<'a, V: TermSource<'a>>(
    body: V,
    at: super::stage_storage::Position<'_>,
    ids: Ids<'_, '_>,
) -> Result<ExtensionBody, ValidationFailure> {
    let Ids { sink, site, row } = ids;
    let malformed = |sink: &mut dyn super::read_charge::ReadStorage| {
        let bytes = size_of::<CheckedPackageRefusal>()
            + size_of::<CheckedPackageRefusalLocation>()
            + at.pointer_len();
        match sink.reserve_diagnostic(
            bytes as u128,
            site,
            super::CheckedPackagePendingOutcome::Refused(Code::MalformedWire),
        ) {
            Ok(()) => ValidationFailure::refused(Code::MalformedWire, at.build()),
            Err(_) => sink.denial(),
        }
    };
    let named = |node: V, name: &str| node.member(name).and_then(V::text);
    if !body.exact_members(&["term", "definition", "node_kind", "clauses"])
        || named(body, "term") != Some("extension_form")
    {
        return Err(malformed(&mut *sink));
    }
    let definition = body
        .member("definition")
        .ok_or_else(|| malformed(&mut *sink))?;
    if !definition.exact_members(&["authority", "identity"])
        || named(definition, "authority").is_none_or(str::is_empty)
        || named(definition, "identity").is_none_or(str::is_empty)
        || !named(body, "node_kind").is_some_and(is_identifier)
    {
        return Err(malformed(&mut *sink));
    }
    let clauses = body
        .member("clauses")
        .ok_or_else(|| malformed(&mut *sink))?;
    let count = clauses
        .items()
        .ok_or_else(|| malformed(&mut *sink))?
        .count();
    let bytes = count.saturating_mul(size_of::<CheckedNodeId>());
    sink.reserve(bytes as u128, site)
        .map_err(|_| sink.denial())?;
    let mut targets = Vec::with_capacity(count);
    for clause in clauses.items().into_iter().flatten() {
        let value = clause
            .member("value")
            .ok_or_else(|| malformed(&mut *sink))?;
        if !clause.exact_members(&["term", "name", "value"])
            || named(clause, "term") != Some("binding")
            || !named(clause, "name").is_some_and(is_identifier)
            || !value.exact_members(&["term", "target"])
            || named(value, "term") != Some("reference")
        {
            return Err(malformed(&mut *sink));
        }
        let target = value
            .member("target")
            .ok_or_else(|| malformed(&mut *sink))?;
        let read = target
            .node_id(&mut *sink, site, row)
            .map_err(|_| sink.denial())?;
        match read.spelled() {
            Some(id) if id.is_well_formed() => targets.push(id),
            _ => return Err(malformed(&mut *sink)),
        }
    }
    Ok(ExtensionBody { targets })
}

pub(super) fn targets(body: &ExtensionBody) -> impl Iterator<Item = &CheckedNodeId> {
    body.targets.iter()
}

pub(super) fn body_work(body: &ExtensionBody) -> u64 {
    u64::try_from(body.targets.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(2)
        .saturating_add(1)
}

#[cfg(test)]
mod storage_tests {
    use super::*;
    use crate::checked_package::shared::JsonPointer;
    use crate::checked_package::v2::{CheckedJson, NativeJoinBudget, PackageReadStage, TermNode};
    use ix_trace_rs::trace;
    #[test]
    #[trace("TC-472", "FR-038-AC-511")]
    fn extension_shape_diagnostic_is_prepaid_and_healthy_body_borrows_its_path() {
        let healthy = CheckedJson::from_value(&serde_json::json!({"term":"extension_form","definition":{"authority":"test","identity":"quota"},"node_kind":"Quota","clauses":[]})).expect("document");
        let malformed = CheckedJson::from_value(&serde_json::json!({"term":"extension_form","definition":{"authority":"test","identity":"quota"},"clauses":[]})).expect("document");
        let at = super::super::stage_storage::Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(4)
            .key("body");
        let run = |body: &CheckedJson, limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let allocated = allocation_counter::measure(|| {
                result = Some(read_body(
                    TermNode::of(body),
                    at.into(),
                    Ids {
                        sink: &mut budget,
                        site: super::super::read_charge::Site::new(
                            PackageReadStage::WireChecks,
                            4,
                            "extension",
                        ),
                        row: &[],
                    },
                ));
            });
            (result.expect("read"), budget, allocated.bytes_total)
        };
        let (result, budget, allocated) = run(&healthy, 0);
        assert!(result.is_ok());
        assert_eq!(allocated, 0);
        assert_eq!(budget.read_storage_consumed(), 0);
        let (result, mut budget, allocated) = run(&malformed, u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("closed shape refuses");
        };
        assert_eq!(refusal.code, Code::MalformedWire);
        assert_eq!(
            refusal.primary_path().map(JsonPointer::as_str),
            Some("/semantic_graph/nodes/4/body")
        );
        assert_eq!(allocated, budget.read_storage_consumed());
        assert_eq!(
            budget.take_paid(),
            Some(super::super::CheckedPackagePendingOutcome::Refused(
                Code::MalformedWire
            ))
        );
        let exact = budget.read_storage_consumed();
        let (result, budget, allocated) = run(&malformed, exact);
        assert!(matches!(result, Err(ValidationFailure::Refused(_))));
        assert_eq!(allocated, exact);
        assert_eq!(budget.read_storage_consumed(), exact);
        let (result, mut budget, allocated) = run(&malformed, exact - 1);
        assert!(matches!(result, Err(ValidationFailure::Budget(_))));
        assert_eq!(allocated, 0);
        assert_eq!(budget.read_storage_consumed(), 0);
        assert_eq!(budget.first_unpaid_read_storage(), Some(u128::from(exact)));
        assert_eq!(
            budget.take_pending(),
            Some(super::super::CheckedPackagePendingOutcome::Refused(
                Code::MalformedWire
            ))
        );
    }
}
