// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The QSpec execution-point object is a closed bare value in the fourth
//! operation-anchor binding. It is not a semantic expression term.

use super::typed_terms::TermConversionCause;
use super::{CheckedOperationAnchorBody, CheckedTerm, ExecutionPoint};
use crate::checked_package::common::{
    validate_term, Ids, ReferenceVisitor, TermGrammar, Trail, ValidationFailure,
};
use crate::checked_package::shared::CheckedPackageRefusalCode;
use crate::checked_package::terms::TermSource;
use serde_json::Value;

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
    // The point object is a small closed value read by `ExecutionPoint`'s own
    // wire reader, which takes a `Value`: an uncharged copy of the object.
    if value
        .map(TermSource::to_value_uncharged)
        .as_ref()
        .and_then(ExecutionPoint::from_wire)
        .is_none()
    {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.key("members").index(3).key("value").pointer(),
        ));
    }
    Ok(work)
}

/// Decode the optional point only for an operation-anchor body. The generic
/// expression decoder cannot create an anchor point or a pointed anchor body.
pub(super) fn decode_anchor_body(body: &Value) -> Result<CheckedTerm, TermConversionCause> {
    if body.get("term").and_then(Value::as_str) != Some("aggregate") {
        return CheckedTerm::decode_admitted(body);
    }
    let Some(members) = body.get("members").and_then(Value::as_array) else {
        return CheckedTerm::decode_admitted(body);
    };
    let (context, operation, frame, point) = match members.as_slice() {
        [context, operation, frame] => (context, operation, frame, None),
        [context, operation, frame, fourth] => {
            let point = fourth
                .get("value")
                .and_then(ExecutionPoint::from_wire)
                .ok_or(TermConversionCause::MalformedMember {
                    member: "execution_point",
                    category: serde_json::error::Category::Data,
                })?;
            (context, operation, frame, Some(point))
        }
        _ => return CheckedTerm::decode_admitted(body),
    };
    Ok(CheckedTerm::OperationAnchorBody(Box::new(
        CheckedOperationAnchorBody {
            members: [
                CheckedTerm::decode_admitted(context)?,
                CheckedTerm::decode_admitted(operation)?,
                CheckedTerm::decode_admitted(frame)?,
            ],
            execution_point: point,
        },
    )))
}
