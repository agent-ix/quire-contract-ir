// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Closed timed-model node wire shapes checked before identity validation.

use super::{CheckedNodeId, CheckedSemanticNodeV2, TimedModelForm};
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
        for (member_index, member) in typed_members(body).iter().enumerate().skip(1) {
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
                node_pointer(position).key("body"),
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
                graph.target(position, bound(body, 0), binding_at(position, 0), |kind| {
                    matches!(kind, CheckedNodeKind::Model(_))
                })?;
                graph.target(position, bound(body, 1), binding_at(position, 1), |kind| {
                    kind == CheckedNodeKind::TimedModel(Form::Source)
                })?;
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
                    let member_index = member_index + 2;
                    let children = bound(body, member_index)
                        .map(typed_members)
                        .unwrap_or_default();
                    let mut seen = BTreeSet::new();
                    for (child_index, child) in children.iter().enumerate() {
                        let at = binding_at(position, member_index)
                            .key("members")
                            .index(child_index);
                        meter.charge(1, || at.clone())?;
                        let target = graph.target(position, Some(child), at.clone(), |kind| {
                            kind == CheckedNodeKind::TimedModel(expected)
                        })?;
                        if expected == Form::Place && !seen.insert(target) {
                            return Err(graph.issue(position, Cause::DuplicateClockPlace, at));
                        }
                    }
                }
            }
            Form::Source => {
                graph.target(position, bound(body, 1), binding_at(position, 1), |kind| {
                    kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
                })?;
                if bound(body, 0).and_then(spelling) == Some("tick") {
                    graph.target(position, bound(body, 2), binding_at(position, 2), |kind| {
                        matches!(
                            kind,
                            CheckedNodeKind::Function(_)
                                | CheckedNodeKind::State(super::StateForm::OperationAnchor)
                        )
                    })?;
                    graph.exact(
                        position,
                        bound(body, 3),
                        true,
                        binding_at(position, 3),
                        meter,
                    )?;
                }
            }
            Form::Place => {
                graph.target(position, bound(body, 0), binding_at(position, 0), |kind| {
                    matches!(kind, CheckedNodeKind::Model(_))
                })?;
                for (child_index, child) in bound(body, 2)
                    .map(typed_members)
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                {
                    let at = binding_at(position, 2).key("members").index(child_index);
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
