// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Closed timed-model node wire shapes checked before identity validation.

use super::model_members::{
    Budget, MemberKind, ModelFailure, ModelOwners, Resolved, SelectedTypeRef,
};
use super::{
    CheckedNodeId, CheckedNodeOwner, RawSemanticNodeV2 as CheckedSemanticNodeV2, TimedModelForm,
};
use crate::checked_package::common::{
    exact_members, is_digest, node_pointer, ValidationFailure, NODE_DOMAIN,
};
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

fn node_key(value: &Value) -> bool {
    serde_json::from_value::<CheckedNodeId>(value.clone())
        .is_ok_and(|key| key.domain.as_ref() == NODE_DOMAIN && is_digest(&key.digest))
}

fn reference(value: &Value, at: &JsonPointer) -> Result<(), ValidationFailure> {
    if value
        .as_object()
        .is_some_and(|o| exact_members(o, &["term", "target"]))
        && value["term"] == "reference"
        && node_key(&value["target"])
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
        || !node_key(&value["type"])
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
                    "time_dimension",
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
            reference(b[2].0, &b[2].1)?;
            for (value, at) in &b[3..] {
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
                        &["kind", "unit", "model", "operation", "period"],
                        &body_at,
                    )?;
                    reference(b[1].0, &b[1].1)?;
                    reference(b[2].0, &b[2].1)?;
                    reference(b[3].0, &b[3].1)?;
                    magnitude(b[4].0, "rational", &b[4].1)?;
                }
                _ => return Err(malformed(kind_at.key("value"))),
            }
        }
        TimedModelForm::Place => {
            let b = bindings(
                &node.body,
                &["model", "path", "resolved_path", "guards"],
                &body_at,
            )?;
            reference(b[0].0, &b[0].1)?;
            path(b[1].0, &b[1].1)?;
            let rows = aggregate(b[2].0, &b[2].1)?;
            for (index, row) in rows.iter().enumerate() {
                let row_at = b[2].1.clone().key("members").index(index);
                let fields = bindings(
                    row,
                    &[
                        "path_step",
                        "declaring_type",
                        "member_identity",
                        "member_type",
                        "collection_kind",
                        "minimum",
                        "maximum",
                    ],
                    &row_at,
                )?;
                integer(fields[0].0, &fields[0].1)?;
                for (value, at) in &fields[1..] {
                    text(value, at)?;
                }
            }
            references(b[3].0, &b[3].1)?;
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

use super::{CheckedNodeKind, CheckedTerm, LiteralKind, ScalarTypeForm, WorkMeter};
use crate::checked_package::shared::CheckedPackageRefusalCause;
use std::collections::{BTreeMap, BTreeSet};

fn typed_members(term: &CheckedTerm) -> &[CheckedTerm] {
    match term {
        CheckedTerm::Aggregate { members } => members,
        _ => &[],
    }
}

fn bound(term: &CheckedTerm, index: usize) -> Option<&CheckedTerm> {
    match typed_members(term).get(index)? {
        CheckedTerm::Binding { value, .. } => Some(value),
        _ => None,
    }
}

fn typed_target(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

fn spelling(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal { value, .. } => value.as_str(),
        _ => None,
    }
}

fn integer_value(term: &CheckedTerm) -> Option<u64> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Integer,
            value,
            ..
        } => value.as_u64(),
        _ => None,
    }
}

fn scalar_text(term: &CheckedTerm) -> Option<String> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Integer,
            value,
            ..
        } => Some(value.as_u64()?.to_string()),
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        } => value.as_str().map(str::to_owned),
        _ => None,
    }
}

fn text_member(term: &CheckedTerm, index: usize) -> Option<&str> {
    bound(term, index).and_then(spelling)
}

fn check_resolved_path(
    graph: &Graph<'_>,
    position: usize,
    place: &CheckedTerm,
    root_model: usize,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    use CheckedPackageRefusalCause as Cause;
    let rows_at = binding_at(position, 2);
    let steps = bound(place, 1).map(typed_members).unwrap_or_default();
    let rows = bound(place, 2).map(typed_members).unwrap_or_default();
    let fail = |at| graph.issue(position, Cause::MalformedClockPlace, at);
    if steps.first().and_then(|step| text_member(step, 0)) == Some("slot") {
        return Err(fail(
            node_pointer(position).key("body").key("members").index(1),
        ));
    }
    if rows.len() != steps.len() {
        return Err(fail(rows_at.key("members")));
    }
    let model_node = &graph.nodes[root_model];
    let model_name = model_node
        .declaration
        .as_ref()
        .map(|declaration| {
            declaration
                .qualified_name
                .iter()
                .map(|part| part.as_ref())
                .collect::<Vec<_>>()
                .join("/")
        })
        .or_else(|| match model_node.owner.as_ref() {
            Some(CheckedNodeOwner::Model { node, .. }) => Some(node.to_string()),
            _ => None,
        });
    let mut expected_declaring = model_name;
    for (index, (step, row)) in steps.iter().zip(rows).enumerate() {
        let at = rows_at.clone().key("members").index(index);
        meter.charge(1, || at.clone())?;
        let step_kind = text_member(step, 0).unwrap_or_default();
        let parent_type = expected_declaring.clone();
        let declaring_expected = if step_kind == "slot" {
            parent_type.as_deref().and_then(|ty| {
                if ty.starts_with("Map<") && ty.ends_with('>') {
                    return Some(ty.to_owned());
                }
                ["Sequence<", "Set<", "Bag<", "OrderedSet<"]
                    .iter()
                    .find_map(|prefix| ty.strip_prefix(prefix))
                    .and_then(|inside| inside.strip_suffix('>'))
                    .map(str::to_owned)
            })
        } else {
            parent_type.clone()
        };
        let step_index = bound(row, 0).and_then(integer_value);
        let declaring = text_member(row, 1);
        let identity = text_member(row, 2);
        let member_type = text_member(row, 3);
        let collection = text_member(row, 4);
        let minimum = text_member(row, 5);
        let maximum = text_member(row, 6);
        if step_index != Some(index as u64)
            || declaring != declaring_expected.as_deref()
            || identity.is_none()
            || member_type.is_none()
            || collection.is_none()
            || minimum.is_none()
            || maximum.is_none()
        {
            return Err(fail(at));
        }
        let expected_identity = match step_kind {
            "field" => {
                text_member(step, 1).map(|name| format!("{}/{name}", declaring.unwrap_or_default()))
            }
            "tuple" => text_member(step, 1)
                .map(|slot| format!("{}/tuple/{slot}", declaring.unwrap_or_default())),
            "union" => text_member(step, 1)
                .map(|case| format!("{}/union/{case}", declaring.unwrap_or_default())),
            "slot" => parent_type.as_ref().map(|name| format!("{name}[slot]")),
            _ => None,
        };
        if step_kind == "field" {
            let path_at = binding_at(position, 1).key("members").index(index);
            let model_owner = owners
                .recover(model_node)
                .map_err(|_| fail(path_at.clone()))?;
            let declaration = expected_declaring.as_deref().unwrap_or_default();
            let mut budget = Budget::new(meter, model_owner.selection, model_owner.package.bytes);
            let resolved = model_owner
                .package
                .resolve(
                    declaration,
                    MemberKind::Field,
                    text_member(step, 1).unwrap_or_default(),
                    &mut budget,
                )
                .map_err(|failure| match failure {
                    ModelFailure::Refused(_) => fail(path_at),
                    ModelFailure::Limit(failure) => failure,
                })?;
            let Resolved::Field(field) = resolved else {
                return Err(fail(at));
            };
            let actual_declaring = field.identity.rsplit_once('/').map(|(owner, _)| owner);
            let actual_type = match field.slot.type_ref.as_ref() {
                Some(SelectedTypeRef::Native(super::model_members::NativeTypeRef::Clock)) => {
                    "ix://quire/native/Clock".to_owned()
                }
                Some(SelectedTypeRef::Native(super::model_members::NativeTypeRef::Text)) => {
                    "ix://quire/native/Text".to_owned()
                }
                Some(SelectedTypeRef::Declared(name) | SelectedTypeRef::Unit(name)) => {
                    name.to_string()
                }
                Some(SelectedTypeRef::Map(key, value)) => format!("Map<{key}, {value}>"),
                _ => return Err(fail(at)),
            };
            if actual_declaring != declaring
                || Some(field.identity.as_ref()) != identity
                || actual_type != member_type.unwrap_or_default()
            {
                return Err(fail(at));
            }
        }
        if expected_identity.as_deref() != identity {
            return Err(fail(at));
        }
        if step_kind == "slot" && collection == Some("map") {
            let Some(map_inner) = parent_type
                .as_deref()
                .and_then(|name| name.strip_prefix("Map<"))
                .and_then(|name| name.strip_suffix('>'))
            else {
                return Err(fail(at));
            };
            let Some((key_type, value_type)) = map_inner.split_once(", ") else {
                return Err(fail(at));
            };
            let text_node = graph
                .kinds
                .iter()
                .position(|kind| *kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Text))
                .ok_or_else(|| fail(at.clone()))?;
            let selector_target = bound(step, 2).and_then(typed_target);
            let selector_position = selector_target
                .and_then(|key| graph.index.get(key))
                .copied();
            let text_node_id = &graph.nodes[text_node].node_id;
            if key_type != "Text"
                || value_type != member_type.unwrap_or_default()
                || selector_position
                    .is_none_or(|selector| graph.nodes[selector].semantic_type != *text_node_id)
            {
                return Err(fail(at));
            }
        }
        let next_kind = steps.get(index + 1).and_then(|next| text_member(next, 0));
        if next_kind == Some("slot") {
            let next = &steps[index + 1];
            let expected_collection = text_member(next, 1);
            let expected_minimum = bound(next, 3).and_then(scalar_text);
            let expected_maximum = bound(next, 4).and_then(scalar_text);
            if collection != expected_collection
                || minimum.map(str::to_owned) != expected_minimum
                || maximum.map(str::to_owned) != expected_maximum
            {
                return Err(fail(at));
            }
        } else if collection != Some("none") || minimum != Some("") || maximum != Some("") {
            return Err(fail(at));
        }
        expected_declaring = member_type.map(str::to_owned);
    }
    if rows.last().and_then(|row| text_member(row, 3)) != Some("ix://quire/native/Clock") {
        return Err(fail(
            rows_at.key("members").index(rows.len().saturating_sub(1)),
        ));
    }
    Ok(())
}

struct Graph<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
}

impl Graph<'_> {
    fn issue(
        &self,
        position: usize,
        cause: CheckedPackageRefusalCause,
        at: JsonPointer,
    ) -> ValidationFailure {
        let code = if cause == CheckedPackageRefusalCause::InvalidTimeQuantity {
            CheckedPackageRefusalCode::IllTyped
        } else {
            CheckedPackageRefusalCode::InvalidPackage
        };
        ValidationFailure::refused_at(code, at, Some(cause), self.nodes[position].node_id.clone())
    }

    fn body(&self, position: usize) -> Result<&CheckedTerm, ValidationFailure> {
        self.terms
            .get(position)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                self.issue(
                    position,
                    CheckedPackageRefusalCause::TimedNodeReference,
                    node_pointer(position).key("body"),
                )
            })
    }

    fn target(
        &self,
        position: usize,
        term: Option<&CheckedTerm>,
        at: JsonPointer,
        accepts: impl FnOnce(CheckedNodeKind) -> bool,
    ) -> Result<usize, ValidationFailure> {
        let target = term
            .and_then(typed_target)
            .and_then(|key| self.index.get(key))
            .copied();
        match target {
            Some(target) if self.kinds.get(target).is_some_and(|kind| accepts(*kind)) => Ok(target),
            _ => Err(self.issue(position, CheckedPackageRefusalCause::TimedNodeReference, at)),
        }
    }

    fn boolean(
        &self,
        position: usize,
        term: Option<&CheckedTerm>,
        at: JsonPointer,
    ) -> Result<(), ValidationFailure> {
        let target = self.target(position, term, at.clone(), |kind| {
            matches!(
                kind,
                CheckedNodeKind::Expression(_) | CheckedNodeKind::Value(_)
            )
        })?;
        let boolean = self
            .index
            .get(&self.nodes[target].semantic_type)
            .and_then(|&p| self.kinds.get(p))
            == Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean));
        if boolean {
            Ok(())
        } else {
            Err(self.issue(position, CheckedPackageRefusalCause::TimedNodeReference, at))
        }
    }

    fn exact(
        &self,
        position: usize,
        term: Option<&CheckedTerm>,
        positive: bool,
        at: JsonPointer,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let invalid = || {
            self.issue(
                position,
                CheckedPackageRefusalCause::InvalidTimeQuantity,
                at.clone(),
            )
        };
        let value = term.and_then(spelling).ok_or_else(invalid)?;
        let (numerator, denominator) = value.split_once('/').unwrap_or((value, "1"));
        // The strict shape has checked nonnegative decimal syntax. Shared
        // natural arithmetic accounts for the exact reduction work.
        if positive && numerator == "0"
            || !super::natural::coprime(numerator, denominator, meter, &|| at.clone())?
        {
            return Err(invalid());
        }
        Ok(())
    }
}

/// Compare the retained path terms exactly, charging each actual term visit.
/// Guards are a separate place binding and never enter this comparison.
fn same_place_path(
    left: &CheckedTerm,
    right: &CheckedTerm,
    meter: &mut WorkMeter,
    at: &JsonPointer,
) -> Result<bool, ValidationFailure> {
    let mut pending = vec![(left, right)];
    while let Some((left, right)) = pending.pop() {
        meter.charge(1, || at.clone())?;
        match (left, right) {
            (
                CheckedTerm::Aggregate { members: left },
                CheckedTerm::Aggregate { members: right },
            ) => {
                if left.len() != right.len() {
                    return Ok(false);
                }
                pending.extend(left.iter().zip(right));
            }
            (
                CheckedTerm::Binding {
                    name: left_name,
                    value: left,
                },
                CheckedTerm::Binding {
                    name: right_name,
                    value: right,
                },
            ) => {
                if left_name != right_name {
                    return Ok(false);
                }
                pending.push((left, right));
            }
            _ if left != right => return Ok(false),
            _ => {}
        }
    }
    Ok(true)
}

fn binding_at(position: usize, index: usize) -> JsonPointer {
    node_pointer(position)
        .key("body")
        .key("members")
        .index(index)
        .key("value")
}

/// Join the retained timed roots, members and local typed targets in digest order.
pub(super) fn validate_models<'a>(
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    use CheckedPackageRefusalCause as Cause;
    use TimedModelForm as Form;
    let graph = Graph {
        nodes,
        kinds,
        terms,
        index,
    };
    let mut roots = Vec::new();
    let mut represented = BTreeSet::new();
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::TimedModel(Form::Model)) {
            continue;
        }
        let body = graph.body(position)?;
        roots.push(position);
        represented.insert(&nodes[position].node_id);
        for (member_index, member) in typed_members(body).iter().enumerate().skip(2) {
            meter.charge(1, || binding_at(position, member_index))?;
            let CheckedTerm::Binding { value, .. } = member else {
                continue;
            };
            if let Some(key) = typed_target(value) {
                represented.insert(key);
            }
            for child in typed_members(value) {
                if let Some(key) = typed_target(child) {
                    represented.insert(key);
                }
            }
        }
    }
    for &position in index.values() {
        let Some(CheckedNodeKind::TimedModel(form)) = kinds.get(position).copied() else {
            continue;
        };
        meter.charge(1, || node_pointer(position))?;
        let body = graph.body(position)?;
        if roots.is_empty() {
            return Err(graph.issue(
                position,
                Cause::MissingTimedModel,
                JsonPointer::root().key("semantic_graph").key("nodes"),
            ));
        }
        if !represented.contains(&nodes[position].node_id) {
            return Err(graph.issue(
                position,
                Cause::TimedNodeReference,
                node_pointer(position).key("body"),
            ));
        }
        match form {
            Form::Model => {
                let model =
                    graph.target(position, bound(body, 0), binding_at(position, 0), |kind| {
                        matches!(kind, CheckedNodeKind::Model(_))
                    })?;
                let dimension =
                    graph.target(position, bound(body, 1), binding_at(position, 1), |kind| {
                        kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Dimension)
                    })?;
                let source =
                    graph.target(position, bound(body, 2), binding_at(position, 2), |kind| {
                        kind == CheckedNodeKind::TimedModel(Form::Source)
                    })?;
                let source_body = graph.body(source)?;
                let unit = graph.target(
                    source,
                    bound(source_body, 1),
                    binding_at(source, 1),
                    |kind| kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit),
                )?;
                let is_base_time = matches!(
                    nodes[dimension].nominal_identity_preimage.as_ref(),
                    Some(super::NominalIdentityPreimage::Dimension(preimage))
                        if preimage.terms.is_empty()
                            && preimage.qualified_declaration.last().is_some_and(|name| name.as_ref() == "Time")
                );
                if !is_base_time || nodes[unit].semantic_type != nodes[dimension].node_id {
                    return Err(graph.issue(
                        source,
                        Cause::InvalidTimeQuantity,
                        binding_at(source, 1).key("target"),
                    ));
                }
                if bound(source_body, 0).and_then(spelling) == Some("tick") {
                    let source_model = graph.target(
                        source,
                        bound(source_body, 2),
                        binding_at(source, 2).key("target"),
                        |kind| matches!(kind, CheckedNodeKind::Model(_)),
                    )?;
                    if source_model != model {
                        return Err(graph.issue(
                            source,
                            Cause::TimedNodeReference,
                            binding_at(source, 2).key("target"),
                        ));
                    }
                    let operation = graph.target(
                        source,
                        bound(source_body, 3),
                        binding_at(source, 3).key("target"),
                        |kind| kind == CheckedNodeKind::State(super::StateForm::OperationAnchor),
                    )?;
                    let operation_body = graph.body(operation)?;
                    let context = graph.target(
                        operation,
                        bound(operation_body, 0),
                        binding_at(operation, 0).key("target"),
                        |kind| matches!(kind, CheckedNodeKind::Model(_)),
                    )?;
                    if context != source_model {
                        return Err(graph.issue(
                            source,
                            Cause::TimedNodeReference,
                            binding_at(source, 3).key("target"),
                        ));
                    }
                }
                for (member_index, expected) in [
                    Form::Place,
                    Form::Reset,
                    Form::Guard,
                    Form::Invariant,
                    Form::Urgency,
                    Form::Constraint,
                ]
                .into_iter()
                .enumerate()
                {
                    let member_index = member_index + 3;
                    let children = bound(body, member_index)
                        .map(typed_members)
                        .unwrap_or_default();
                    let mut seen_places: Vec<(&CheckedNodeId, &CheckedTerm)> = Vec::new();
                    for (child_index, child) in children.iter().enumerate() {
                        let at = binding_at(position, member_index)
                            .key("members")
                            .index(child_index);
                        meter.charge(1, || at.clone())?;
                        let target = graph.target(position, Some(child), at.clone(), |kind| {
                            kind == CheckedNodeKind::TimedModel(expected)
                        })?;
                        if expected == Form::Place
                            && bound(source_body, 0).and_then(spelling) == Some("tick")
                        {
                            return Err(graph.issue(
                                target,
                                Cause::ClockWithoutDenseTime,
                                binding_at(target, 0).key("target"),
                            ));
                        }
                        if expected == Form::Place {
                            let place = graph.body(target)?;
                            let place_model =
                                bound(place, 0).and_then(typed_target).ok_or_else(|| {
                                    graph.issue(
                                        target,
                                        Cause::TimedNodeReference,
                                        binding_at(target, 0),
                                    )
                                })?;
                            let place_model =
                                index.get(&place_model).copied().ok_or_else(|| {
                                    graph.issue(
                                        target,
                                        Cause::TimedNodeReference,
                                        binding_at(target, 0),
                                    )
                                })?;
                            if place_model != model {
                                return Err(graph.issue(
                                    target,
                                    Cause::TimedNodeReference,
                                    binding_at(target, 0),
                                ));
                            }
                            let path = bound(place, 1).ok_or_else(|| {
                                graph.issue(
                                    target,
                                    Cause::MalformedClockPlace,
                                    binding_at(target, 1),
                                )
                            })?;
                            check_resolved_path(&graph, target, place, model, owners, meter)?;
                            for (seen_model, seen_path) in &seen_places {
                                meter.charge(1, || at.clone())?;
                                if *seen_model == &nodes[model].node_id
                                    && same_place_path(seen_path, path, meter, &at)?
                                {
                                    return Err(graph.issue(
                                        position,
                                        Cause::DuplicateClockPlace,
                                        at,
                                    ));
                                }
                            }
                            seen_places.push((&nodes[model].node_id, path));
                        }
                    }
                }
            }
            Form::Source => {
                graph.target(position, bound(body, 1), binding_at(position, 1), |kind| {
                    kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
                })?;
                if bound(body, 0).and_then(spelling) == Some("tick") {
                    graph.target(
                        position,
                        bound(body, 3),
                        binding_at(position, 3).key("target"),
                        |kind| kind == CheckedNodeKind::State(super::StateForm::OperationAnchor),
                    )?;
                    graph.exact(
                        position,
                        bound(body, 4),
                        true,
                        binding_at(position, 4).key("value"),
                        meter,
                    )?;
                }
            }
            Form::Place => {
                graph.target(position, bound(body, 0), binding_at(position, 0), |kind| {
                    matches!(kind, CheckedNodeKind::Model(_))
                })?;
                for (child_index, child) in bound(body, 3)
                    .map(typed_members)
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                {
                    let at = binding_at(position, 3).key("members").index(child_index);
                    meter.charge(1, || at.clone())?;
                    graph.boolean(position, Some(child), at)?;
                }
            }
            Form::Reset | Form::Constraint => {
                graph.target(position, bound(body, 0), binding_at(position, 0), |kind| {
                    kind == CheckedNodeKind::TimedModel(Form::Place)
                })?;
                let amount = if form == Form::Reset { 1 } else { 2 };
                graph.exact(
                    position,
                    bound(body, amount),
                    false,
                    binding_at(position, amount),
                    meter,
                )?;
                let unit = match bound(body, amount) {
                    Some(CheckedTerm::Literal {
                        type_id,
                        value_kind: LiteralKind::Quantity,
                        ..
                    }) => Some(type_id),
                    _ => None,
                };
                if !unit
                    .and_then(|id| index.get(id))
                    .and_then(|&p| kinds.get(p))
                    .is_some_and(|kind| *kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
                {
                    return Err(graph.issue(
                        position,
                        Cause::InvalidTimeQuantity,
                        binding_at(position, amount).key("type"),
                    ));
                }
            }
            Form::Guard | Form::Invariant => {
                graph.boolean(position, bound(body, 0), binding_at(position, 0))?
            }
            Form::Urgency => {
                if bound(body, 0).and_then(spelling) == Some("when") {
                    graph.boolean(position, bound(body, 1), binding_at(position, 1))?;
                } else {
                    graph.target(position, bound(body, 1), binding_at(position, 1), |kind| {
                        matches!(kind, CheckedNodeKind::Model(_))
                    })?;
                }
            }
        }
    }
    Ok(())
}
