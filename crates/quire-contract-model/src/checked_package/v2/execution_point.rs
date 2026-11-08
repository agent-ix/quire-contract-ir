// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The QSpec execution-point object is a closed bare value in the fourth
//! operation-anchor binding. It is not a semantic expression term.

use super::read_charge::{ReadStorage, Site};
use super::ExecutionPoint;
use crate::checked_package::common::{
    validate_term, Ids, ReferenceVisitor, TermGrammar, Trail, ValidationFailure,
};
use crate::checked_package::shared::CheckedPackageRefusalCode;
use crate::checked_package::terms::TermSource;
use crate::identity::ExecutionPointName;

/// Validate the anchor's term members and optional bare point in wire order.
pub(super) fn validate_anchor_body<'a, V: TermSource<'a>>(
    body: V,
    at: &Trail<'a>,
    visit: &mut ReferenceVisitor<'_>,
    mut ids: Ids<'_, '_>,
) -> Result<u64, ValidationFailure> {
    let members = || body.member("members").and_then(TermSource::items);
    if !body.is_object() || members().is_none() {
        return validate_term(body, TermGrammar::V2, true, at, visit, ids);
    }
    let body_refusal = || {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        )
    };
    fn is_point<'a>(member: impl TermSource<'a>) -> bool {
        member.member("name").and_then(TermSource::text) == Some("execution_point")
    }
    if members()
        .into_iter()
        .flatten()
        .enumerate()
        .any(|(index, member)| index != 3 && is_point(member))
    {
        return Err(body_refusal());
    }
    let count = members().map_or(0, Iterator::count);
    if count == 3 {
        return validate_term(body, TermGrammar::V2, true, at, visit, ids);
    }
    if count != 4
        || !body.exact_members(&["term", "members"])
        || body.member("term").and_then(TermSource::text) != Some("aggregate")
    {
        return Err(body_refusal());
    }
    let Some(fourth) = members().and_then(|mut items| items.nth(3)) else {
        return Err(body_refusal());
    };
    if !fourth.is_object()
        || !fourth.exact_members(&["term", "name", "value"])
        || fourth.member("term").and_then(TermSource::text) != Some("binding")
        || fourth.member("name").and_then(TermSource::text) != Some("execution_point")
    {
        return Err(body_refusal());
    }
    let mut work = 3_u64; // aggregate, fourth binding, and point object
    for (index, member) in members().into_iter().flatten().take(3).enumerate() {
        let members_at = at.key("members");
        let member_at = members_at.index(index);
        work = work.saturating_add(validate_term(
            member.shorter(),
            TermGrammar::V2,
            false,
            &member_at,
            visit,
            ids.reborrow(),
        )?);
    }
    let value = fourth.member("value");
    if !value.is_some_and(|value| value.is_object() && value.member("term").is_none()) {
        return Err(body_refusal());
    }
    let point = match value {
        Some(value) => read_point(value, &mut *ids.sink, ids.site)?,
        None => None,
    };
    if point.is_none() {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.key("members").index(3).key("value").pointer(),
        ));
    }
    Ok(work)
}

/// The closed point object `{"kind": ..., "name" | "operation": ...}` a value
/// holds, with its name charged to `sink` at `site` before it is allocated;
/// `None` when the value is no point.
fn read_point<'a, V: TermSource<'a>>(
    value: V,
    sink: &mut dyn ReadStorage,
    site: Site,
) -> Result<Option<ExecutionPoint>, ValidationFailure> {
    let (field, make): (&str, fn(ExecutionPointName) -> ExecutionPoint) =
        match value.member("kind").and_then(TermSource::text) {
            Some("initialization") => ("name", |name| ExecutionPoint::Initialization { name }),
            Some("handler") => ("name", |name| ExecutionPoint::Handler { name }),
            Some("pre") => ("operation", |operation| ExecutionPoint::Pre { operation }),
            Some("post") => ("operation", |operation| ExecutionPoint::Post { operation }),
            _ => return Ok(None),
        };
    if !value.exact_members(&["kind", field]) {
        return Ok(None);
    }
    let Some(name) = value.member(field).and_then(TermSource::text) else {
        return Ok(None);
    };
    sink.reserve(name.len() as u128, site)
        .map_err(|_| sink.denial())?;
    Ok(ExecutionPointName::new(name).ok().map(make))
}
