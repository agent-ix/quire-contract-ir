// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-348 extension form body grammar, read from the package document's node
//! body through the term view: no `serde_json` value, parser or deserializer.
//! The targets are the node ids the body's clauses reference, each read (and
//! its heap charged) through the caller's sink before the table that holds
//! them is requested.

use super::identity::is_identifier;
use crate::checked_package::common::{Ids, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCode as Code, JsonPointer,
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
    at: JsonPointer,
    ids: Ids<'_, '_>,
) -> Result<ExtensionBody, ValidationFailure> {
    let Ids { sink, site, row } = ids;
    let malformed = || ValidationFailure::refused(Code::MalformedWire, at.clone());
    let named = |node: V, name: &str| node.member(name).and_then(V::text);
    if !body.exact_members(&["term", "definition", "node_kind", "clauses"])
        || named(body, "term") != Some("extension_form")
    {
        return Err(malformed());
    }
    let definition = body.member("definition").ok_or_else(malformed)?;
    if !definition.exact_members(&["authority", "identity"])
        || named(definition, "authority").is_none_or(str::is_empty)
        || named(definition, "identity").is_none_or(str::is_empty)
        || !named(body, "node_kind").is_some_and(is_identifier)
    {
        return Err(malformed());
    }
    let clauses = body.member("clauses").ok_or_else(malformed)?;
    let count = clauses.items().ok_or_else(malformed)?.count();
    let bytes = count.saturating_mul(size_of::<CheckedNodeId>());
    sink.reserve(bytes as u128, site)
        .map_err(|_| sink.denial())?;
    let mut targets = Vec::with_capacity(count);
    for clause in clauses.items().into_iter().flatten() {
        let value = clause.member("value").ok_or_else(malformed)?;
        if !clause.exact_members(&["term", "name", "value"])
            || named(clause, "term") != Some("binding")
            || !named(clause, "name").is_some_and(is_identifier)
            || !value.exact_members(&["term", "target"])
            || named(value, "term") != Some("reference")
        {
            return Err(malformed());
        }
        let target = value.member("target").ok_or_else(malformed)?;
        let read = target
            .node_id(&mut *sink, site, row)
            .map_err(|_| sink.denial())?;
        match read.spelled() {
            Some(id) if id.is_well_formed() => targets.push(id),
            _ => return Err(malformed()),
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
