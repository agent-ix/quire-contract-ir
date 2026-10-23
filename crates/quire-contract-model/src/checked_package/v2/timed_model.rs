// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Closed timed-model node wire shapes checked before identity validation.

use super::{CheckedNodeId, CheckedSemanticNodeV2, TimedModelForm};
use crate::checked_package::common::{exact_members, node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCode, JsonPointer,
};
use serde_json::Value;

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

fn aggregate<'a>(value: &'a Value, at: &JsonPointer) -> Result<&'a [Value], ValidationFailure> {
    let Some(object) = value.as_object() else {
        return Err(malformed(at.clone()));
    };
    if !exact_members(object, &["term", "members"]) || value["term"] != "aggregate" {
        return Err(malformed(at.clone()));
    }
    value["members"]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| malformed(at.clone().key("members")))
}

fn bindings<'a>(
    value: &'a Value,
    names: &[&str],
    at: &JsonPointer,
) -> Result<Vec<(&'a Value, JsonPointer)>, ValidationFailure> {
    let members = aggregate(value, at)?;
    if members.len() != names.len() {
        return Err(malformed(at.clone().key("members")));
    }
    members
        .iter()
        .zip(names)
        .enumerate()
        .map(|(index, (member, name))| {
            let member_at = at.clone().key("members").index(index);
            let valid = member
                .as_object()
                .is_some_and(|o| exact_members(o, &["term", "name", "value"]))
                && member["term"] == "binding"
                && member["name"] == *name;
            if !valid {
                return Err(malformed(member_at));
            }
            Ok((&member["value"], member_at.key("value")))
        })
        .collect()
}

fn reference(value: &Value, at: &JsonPointer) -> Result<(), ValidationFailure> {
    if value
        .as_object()
        .is_some_and(|o| exact_members(o, &["term", "target"]))
        && value["term"] == "reference"
        && serde_json::from_value::<CheckedNodeId>(value["target"].clone()).is_ok()
    {
        Ok(())
    } else {
        Err(malformed(at.clone()))
    }
}

fn literal<'a>(
    value: &'a Value,
    kind: &str,
    at: &JsonPointer,
) -> Result<&'a Value, ValidationFailure> {
    if !value
        .as_object()
        .is_some_and(|o| exact_members(o, &["term", "type", "value_kind", "value"]))
        || value["term"] != "literal"
        || value["value_kind"] != kind
        || serde_json::from_value::<CheckedNodeId>(value["type"].clone()).is_err()
    {
        return Err(malformed(at.clone()));
    }
    Ok(&value["value"])
}

fn text<'a>(value: &'a Value, at: &JsonPointer) -> Result<&'a str, ValidationFailure> {
    literal(value, "text", at)?
        .as_str()
        .ok_or_else(|| malformed(at.clone().key("value")))
}

fn word(value: &Value, words: &[&str], at: &JsonPointer) -> Result<(), ValidationFailure> {
    if words.contains(&text(value, at)?) {
        Ok(())
    } else {
        Err(malformed(at.clone().key("value")))
    }
}

fn integer(value: &Value, at: &JsonPointer) -> Result<(), ValidationFailure> {
    if literal(value, "integer", at)?.as_u64().is_some() {
        Ok(())
    } else {
        Err(malformed(at.clone().key("value")))
    }
}

fn magnitude(value: &Value, kind: &str, at: &JsonPointer) -> Result<(), ValidationFailure> {
    let Some(spelling) = literal(value, kind, at)?.as_str() else {
        return Err(malformed(at.clone().key("value")));
    };
    let natural = |s: &str| {
        s == "0"
            || s.as_bytes()
                .first()
                .is_some_and(|c| matches!(c, b'1'..=b'9'))
                && s.bytes().all(|c| c.is_ascii_digit())
    };
    let valid = match spelling.split_once('/') {
        Some((n, d)) => natural(n) && natural(d) && d != "0",
        None => natural(spelling),
    };
    if valid {
        Ok(())
    } else {
        Err(malformed(at.clone().key("value")))
    }
}

fn references(value: &Value, at: &JsonPointer) -> Result<(), ValidationFailure> {
    for (index, member) in aggregate(value, at)?.iter().enumerate() {
        reference(member, &at.clone().key("members").index(index))?;
    }
    Ok(())
}

fn path(value: &Value, at: &JsonPointer) -> Result<(), ValidationFailure> {
    let steps = aggregate(value, at)?;
    if steps.is_empty() {
        return Err(malformed(at.clone().key("members")));
    }
    for (index, step) in steps.iter().enumerate() {
        let step_at = at.clone().key("members").index(index);
        let members = aggregate(step, &step_at)?;
        let kind_at = step_at.clone().key("members").index(0).key("value");
        let kind = members
            .first()
            .and_then(|m| m.get("value"))
            .ok_or_else(|| malformed(step_at.clone()))?;
        match text(kind, &kind_at)? {
            "field" => {
                let b = bindings(step, &["kind", "name"], &step_at)?;
                text(b[1].0, &b[1].1)?;
            }
            "tuple" => {
                let b = bindings(step, &["kind", "index"], &step_at)?;
                integer(b[1].0, &b[1].1)?;
            }
            "union" => {
                let b = bindings(step, &["kind", "case"], &step_at)?;
                text(b[1].0, &b[1].1)?;
            }
            "slot" => {
                let b = bindings(
                    step,
                    &["kind", "collection", "selector", "minimum", "maximum"],
                    &step_at,
                )?;
                word(
                    b[1].0,
                    &["sequence", "map", "set", "bag", "ordered_set"],
                    &b[1].1,
                )?;
                if b[2].0["term"] == "reference" {
                    reference(b[2].0, &b[2].1)?;
                } else {
                    integer(b[2].0, &b[2].1)?;
                }
                integer(b[3].0, &b[3].1)?;
                if b[4].0["value_kind"] == "text" {
                    word(b[4].0, &["unbounded"], &b[4].1)?;
                } else {
                    integer(b[4].0, &b[4].1)?;
                }
            }
            _ => return Err(malformed(kind_at.key("value"))),
        }
    }
    Ok(())
}

/// Validate the complete closed outer record and its ordered nested body.
pub(super) fn check_wire(
    node: &CheckedSemanticNodeV2,
    form: TimedModelForm,
    position: usize,
) -> Result<(), ValidationFailure> {
    let at = node_pointer(position);
    for (name, present) in [
        ("recursion_group", node.recursion_group.is_some()),
        (
            "nominal_identity_preimage",
            node.nominal_identity_preimage.is_some(),
        ),
    ] {
        if present {
            return Err(malformed(at.clone().key(name)));
        }
    }
    let root = form == TimedModelForm::Model;
    if root {
        if node.owner.is_none() || node.declaration.is_none() {
            return Err(malformed(at.clone()));
        }
        if node.occurrences.len() != 1 {
            return Err(malformed(at.clone().key("occurrences")));
        }
    } else {
        if node.owner.is_some() {
            return Err(malformed(at.clone().key("owner")));
        }
        if node.declaration.is_some() {
            return Err(malformed(at.clone().key("declaration")));
        }
        if node.occurrences.is_empty() {
            return Err(malformed(at.clone().key("occurrences")));
        }
    }
    let role = if root {
        CheckedOccurrenceRole::Declaration
    } else {
        CheckedOccurrenceRole::Expression
    };
    for (index, occurrence) in node.occurrences.iter().enumerate() {
        if occurrence.role != role {
            return Err(malformed(
                at.clone().key("occurrences").index(index).key("role"),
            ));
        }
    }
    let body_at = at.key("body");
    match form {
        TimedModelForm::Model => {
            let b = bindings(
                &node.body,
                &[
                    "model",
                    "source",
                    "clock_places",
                    "resets",
                    "guards",
                    "invariants",
                    "urgencies",
                    "constraints",
                ],
                &body_at,
            )?;
            reference(b[0].0, &b[0].1)?;
            reference(b[1].0, &b[1].1)?;
            for (value, at) in &b[2..] {
                references(value, at)?;
            }
        }
        TimedModelForm::Source => {
            let members = aggregate(&node.body, &body_at)?;
            let kind_at = body_at.clone().key("members").index(0).key("value");
            let kind = members
                .first()
                .and_then(|m| m.get("value"))
                .ok_or_else(|| malformed(body_at.clone()))?;
            match text(kind, &kind_at)? {
                "dense" => {
                    let b = bindings(&node.body, &["kind", "unit"], &body_at)?;
                    reference(b[1].0, &b[1].1)?;
                }
                "tick" => {
                    let b = bindings(
                        &node.body,
                        &["kind", "unit", "operation", "period"],
                        &body_at,
                    )?;
                    reference(b[1].0, &b[1].1)?;
                    reference(b[2].0, &b[2].1)?;
                    magnitude(b[3].0, "rational", &b[3].1)?;
                }
                _ => return Err(malformed(kind_at.key("value"))),
            }
        }
        TimedModelForm::Place => {
            let b = bindings(&node.body, &["model", "path", "guards"], &body_at)?;
            reference(b[0].0, &b[0].1)?;
            path(b[1].0, &b[1].1)?;
            references(b[2].0, &b[2].1)?;
        }
        TimedModelForm::Reset => {
            let b = bindings(&node.body, &["place", "value"], &body_at)?;
            reference(b[0].0, &b[0].1)?;
            magnitude(b[1].0, "quantity", &b[1].1)?;
        }
        TimedModelForm::Guard | TimedModelForm::Invariant => {
            let b = bindings(&node.body, &["predicate"], &body_at)?;
            reference(b[0].0, &b[0].1)?;
        }
        TimedModelForm::Urgency => {
            let members = aggregate(&node.body, &body_at)?;
            let kind_at = body_at.clone().key("members").index(0).key("value");
            let kind = members
                .first()
                .and_then(|m| m.get("value"))
                .ok_or_else(|| malformed(body_at.clone()))?;
            match text(kind, &kind_at)? {
                "when" => {
                    let b = bindings(&node.body, &["kind", "predicate"], &body_at)?;
                    reference(b[1].0, &b[1].1)?;
                }
                "operation" => {
                    let b = bindings(&node.body, &["kind", "model", "operation_name"], &body_at)?;
                    reference(b[1].0, &b[1].1)?;
                    text(b[2].0, &b[2].1)?;
                }
                _ => return Err(malformed(kind_at.key("value"))),
            }
        }
        TimedModelForm::Constraint => {
            let b = bindings(&node.body, &["place", "comparison", "bound"], &body_at)?;
            reference(b[0].0, &b[0].1)?;
            word(b[1].0, &["lt", "le", "gt", "ge"], &b[1].1)?;
            magnitude(b[2].0, "quantity", &b[2].1)?;
        }
    }
    Ok(())
}
