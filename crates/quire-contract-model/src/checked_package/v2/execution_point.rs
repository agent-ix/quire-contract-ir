// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The QSpec execution-point object is a closed bare value in the fourth
//! operation-anchor binding. It is not a semantic expression term.

use super::CheckedExecutionPoint;
use crate::checked_package::common::{
    exact_members, validate_term, ReferenceVisitor, TermGrammar, Trail, ValidationFailure,
};
use crate::checked_package::shared::CheckedPackageRefusalCode;
use serde_json::Value;

/// Validate the anchor's term members and optional bare point in wire order.
pub(super) fn validate_anchor_body(
    body: &Value,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    let Some(object) = body.as_object() else {
        return validate_term(body, TermGrammar::V2, true, at, visit);
    };
    let Some(members) = object.get("members").and_then(Value::as_array) else {
        return validate_term(body, TermGrammar::V2, true, at, visit);
    };
    let body_refusal = || {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at.pointer(),
        )
    };
    if members.iter().enumerate().any(|(index, member)| {
        index != 3 && member.get("name").and_then(Value::as_str) == Some("execution_point")
    }) {
        return Err(body_refusal());
    }
    if members.len() == 3 {
        return validate_term(body, TermGrammar::V2, true, at, visit);
    }
    if members.len() != 4
        || !exact_members(object, &["term", "members"])
        || object.get("term").and_then(Value::as_str) != Some("aggregate")
    {
        return Err(body_refusal());
    }
    let Some(fourth) = members.get(3).and_then(Value::as_object) else {
        return Err(body_refusal());
    };
    if !exact_members(fourth, &["term", "name", "value"])
        || fourth.get("term").and_then(Value::as_str) != Some("binding")
        || fourth.get("name").and_then(Value::as_str) != Some("execution_point")
    {
        return Err(body_refusal());
    }
    let mut work = 3_u64; // aggregate, fourth binding, and point object
    for (index, member) in members.iter().take(3).enumerate() {
        work = work.saturating_add(validate_term(
            member,
            TermGrammar::V2,
            false,
            &at.key("members").index(index),
            visit,
        )?);
    }
    let point_at = at.key("members").index(3).key("value");
    if fourth
        .get("value")
        .and_then(CheckedExecutionPoint::from_wire)
        .is_none()
    {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            point_at.pointer(),
        ));
    }
    Ok(work)
}
