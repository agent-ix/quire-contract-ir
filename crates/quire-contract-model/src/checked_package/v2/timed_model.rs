// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Closed timed-model node wire shapes checked before identity validation.

use super::model_members::{
    Budget, CollectionKind, FieldDecl, MemberKind, ModelFailure, ModelOwners, Resolved,
    SelectedTypeRef,
};
use super::{
    CheckedNodeId, CheckedNodeOwner, CheckedTerm, CheckedTermView, LiteralKind,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, TimedModelForm,
};
use crate::checked_package::common::{
    body_term, exact_members, is_digest, node_pointer, TermField, ValidationFailure, WireTerm,
    NODE_DOMAIN,
};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCode, CheckedSourceMapEntry, JsonPointer,
};
use serde_json::Value;

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

fn aggregate<'a>(value: &'a Value, at: &JsonPointer) -> Result<&'a [Value], ValidationFailure> {
    let Some(object) = value.as_object() else {
        return Err(malformed(at.clone()));
    };
    if !exact_members(object, &["term", "members"])
        || body_term(value) != Some(super::BodyTerm::Aggregate)
    {
        return Err(malformed(at.clone()));
    }
    match CheckedTerm::view(value) {
        Some(CheckedTermView::Aggregate { members }) => Ok(members),
        _ => Err(malformed(at.clone().key("members"))),
    }
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
            match CheckedTerm::view(member) {
                Some(CheckedTermView::Binding {
                    name: actual,
                    value,
                }) if actual == *name => Ok((value, member_at.key("value"))),
                _ => Err(malformed(member_at)),
            }
        })
        .collect()
}

fn possible_binding_value(value: &Value) -> Option<&Value> {
    if let Some(CheckedTermView::Binding { value, .. }) = CheckedTerm::view(value) {
        return Some(value);
    }
    // The containing record may be malformed. Keep the original refusal
    // order by probing this member before the binding shape is diagnosed.
    WireTerm::decode(value).field(TermField::Value)
}

#[ix_trace_rs::string_edge]
fn node_key(value: &Value) -> Option<CheckedNodeId> {
    serde_json::from_value::<CheckedNodeId>(value.clone())
        .ok()
        .filter(|key| key.domain.as_ref() == NODE_DOMAIN && is_digest(&key.digest))
}

fn reference(value: &Value, at: &JsonPointer) -> Result<(), ValidationFailure> {
    match CheckedTerm::view(value) {
        Some(CheckedTermView::Reference { target }) if node_key(target).is_some() => Ok(()),
        _ => Err(malformed(at.clone())),
    }
}

fn literal<'a>(
    value: &'a Value,
    kind: &str,
    at: &JsonPointer,
) -> Result<&'a Value, ValidationFailure> {
    match CheckedTerm::view(value) {
        Some(CheckedTermView::Literal {
            type_id,
            value_kind,
            value,
        }) if Some(value_kind) == LiteralKind::from_wire(kind) && node_key(type_id).is_some() => {
            Ok(value)
        }
        _ => Err(malformed(at.clone())),
    }
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

struct ExactMagnitude<'a> {
    numerator: &'a str,
    denominator: &'a str,
    zero: bool,
}

impl<'a> ExactMagnitude<'a> {
    #[ix_trace_rs::string_edge]
    fn from_wire(spelling: &'a str) -> Option<Self> {
        let natural = |s: &str| {
            s == "0"
                || s.as_bytes()
                    .first()
                    .is_some_and(|c| matches!(c, b'1'..=b'9'))
                    && s.bytes().all(|c| c.is_ascii_digit())
        };
        let (numerator, denominator) = spelling.split_once('/').unwrap_or((spelling, "1"));
        (natural(numerator) && natural(denominator) && denominator != "0").then_some(Self {
            numerator,
            denominator,
            zero: numerator == "0",
        })
    }
}

fn magnitude(value: &Value, kind: &str, at: &JsonPointer) -> Result<(), ValidationFailure> {
    let Some(spelling) = literal(value, kind, at)?.as_str() else {
        return Err(malformed(at.clone().key("value")));
    };
    if ExactMagnitude::from_wire(spelling).is_some() {
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
            .and_then(possible_binding_value)
            .ok_or_else(|| malformed(step_at.clone()))?;
        match PathStepKind::from_wire(text(kind, &kind_at)?) {
            Some(PathStepKind::Field) => {
                let b = bindings(step, &["kind", "name"], &step_at)?;
                text(b[1].0, &b[1].1)?;
            }
            Some(PathStepKind::Tuple) => {
                let b = bindings(step, &["kind", "index"], &step_at)?;
                integer(b[1].0, &b[1].1)?;
            }
            Some(PathStepKind::Union) => {
                let b = bindings(step, &["kind", "case"], &step_at)?;
                text(b[1].0, &b[1].1)?;
            }
            Some(PathStepKind::Slot) => {
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
                if matches!(
                    CheckedTerm::view(b[2].0),
                    Some(CheckedTermView::Reference { .. })
                ) {
                    reference(b[2].0, &b[2].1)?;
                } else {
                    integer(b[2].0, &b[2].1)?;
                }
                integer(b[3].0, &b[3].1)?;
                if matches!(
                    CheckedTerm::view(b[4].0),
                    Some(CheckedTermView::Literal {
                        value_kind: LiteralKind::Text,
                        ..
                    })
                ) {
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
                .and_then(possible_binding_value)
                .ok_or_else(|| malformed(body_at.clone()))?;
            match TimeSourceKind::from_wire(text(kind, &kind_at)?) {
                Some(TimeSourceKind::Dense) => {
                    let b = bindings(&node.body, &["kind", "unit"], &body_at)?;
                    reference(b[1].0, &b[1].1)?;
                }
                Some(TimeSourceKind::Tick) => {
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
                .and_then(possible_binding_value)
                .ok_or_else(|| malformed(body_at.clone()))?;
            match UrgencyKind::from_wire(text(kind, &kind_at)?) {
                Some(UrgencyKind::When) => {
                    let b = bindings(&node.body, &["kind", "predicate"], &body_at)?;
                    reference(b[1].0, &b[1].1)?;
                }
                Some(UrgencyKind::Operation) => {
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

use super::{CheckedNodeKind, ScalarTypeForm, WorkMeter};
use crate::checked_package::shared::CheckedPackageRefusalCause;
use std::collections::{BTreeMap, BTreeSet};

fn typed_members(term: &CheckedTerm) -> &[CheckedTerm] {
    match term {
        CheckedTerm::Aggregate { members } => members,
        CheckedTerm::OperationAnchorBody(body) => &body.members,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathStepKind {
    Field,
    Tuple,
    Union,
    Slot,
}

enum TimeSourceKind {
    Dense,
    Tick,
}

impl TimeSourceKind {
    #[ix_trace_rs::string_edge]
    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "dense" => Some(Self::Dense),
            "tick" => Some(Self::Tick),
            _ => None,
        }
    }
}

enum UrgencyKind {
    When,
    Operation,
}

impl UrgencyKind {
    #[ix_trace_rs::string_edge]
    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "when" => Some(Self::When),
            "operation" => Some(Self::Operation),
            _ => None,
        }
    }
}

impl PathStepKind {
    #[ix_trace_rs::string_edge]
    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "field" => Some(Self::Field),
            "tuple" => Some(Self::Tuple),
            "union" => Some(Self::Union),
            "slot" => Some(Self::Slot),
            _ => None,
        }
    }
}

fn path_step_kind(step: &CheckedTerm) -> Option<PathStepKind> {
    text_member(step, 0).and_then(PathStepKind::from_wire)
}

/// The selected model and selector type nodes of an already validated place template.
pub(super) fn place_template(place: &CheckedTerm) -> Option<(&CheckedNodeId, Vec<&CheckedNodeId>)> {
    let model = bound(place, 0).and_then(typed_target)?;
    let steps = bound(place, 1).map(typed_members)?;
    let selectors = steps
        .iter()
        .filter(|step| path_step_kind(step) == Some(PathStepKind::Slot))
        .map(|step| bound(step, 2).and_then(typed_target))
        .collect::<Option<Vec<_>>>()?;
    Some((model, selectors))
}

enum RenderedCollectionType<'a> {
    Map(&'a str),
    OtherCollection(&'a str, &'static str),
    Other,
}

impl<'a> RenderedCollectionType<'a> {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &'a str) -> Self {
        if let Some(inner) = text.strip_prefix("Map<").and_then(|s| s.strip_suffix('>')) {
            return Self::Map(inner);
        }
        for (prefix, kind) in [
            ("Sequence<", "sequence"),
            ("Set<", "set"),
            ("Bag<", "bag"),
            ("OrderedSet<", "ordered_set"),
        ] {
            if let Some(inner) = text.strip_prefix(prefix).and_then(|s| s.strip_suffix('>')) {
                return Self::OtherCollection(inner, kind);
            }
        }
        Self::Other
    }
}

fn declared_field_type(
    field: &FieldDecl,
    render: impl FnOnce(&SelectedTypeRef) -> Option<String>,
) -> Option<String> {
    let element = render(field.slot.type_ref.as_ref()?)?;
    let multiplicity = field.slot.multiplicity;
    let optional_clock_element = field.optional
        && matches!(
            field.slot.type_ref.as_ref(),
            Some(SelectedTypeRef::Native(
                super::model_members::NativeTypeRef::Clock
            ))
        )
        && (multiplicity.lower, multiplicity.upper) != (1, Some(1));
    let element = if optional_clock_element {
        format!("Option<{element}>")
    } else {
        element
    };
    let value = if (multiplicity.lower, multiplicity.upper) == (1, Some(1)) {
        element
    } else {
        let kind = match CollectionKind::of(multiplicity.ordered, multiplicity.unique) {
            CollectionKind::Set => "Set",
            CollectionKind::Bag => "Bag",
            CollectionKind::Sequence => "Sequence",
            CollectionKind::OrderedSet => "OrderedSet",
        };
        format!("{kind}<{element}>")
    };
    Some(if field.optional && !optional_clock_element {
        format!("Option<{value}>")
    } else {
        value
    })
}

#[ix_trace_rs::string_edge]
fn terminal_clock(value: &str) -> bool {
    let inner = value
        .strip_prefix("Option<")
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(value);
    inner == "ix://quire/native/Clock"
}

#[ix_trace_rs::string_edge]
fn selector_fits(
    parent_type: &str,
    collection: &str,
    member_type: &str,
    selector_kind: Option<CheckedNodeKind>,
) -> bool {
    match RenderedCollectionType::from_wire(parent_type) {
        RenderedCollectionType::Map(inner) if collection == "map" => {
            inner.split_once(", ").is_some_and(|(key, value)| {
                key == "Text"
                    && value == member_type
                    && selector_kind == Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Text))
            })
        }
        RenderedCollectionType::OtherCollection(inner, kind) if collection == kind => {
            inner == member_type
                && selector_kind == Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Integer))
        }
        _ => false,
    }
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
    if steps.first().and_then(path_step_kind) == Some(PathStepKind::Slot) {
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
        let step_kind = path_step_kind(step);
        let parent_type = expected_declaring.clone();
        let declaring_expected = if step_kind == Some(PathStepKind::Slot) {
            parent_type
                .as_deref()
                .and_then(|ty| match RenderedCollectionType::from_wire(ty) {
                    RenderedCollectionType::Map(_) => Some(ty.to_owned()),
                    RenderedCollectionType::OtherCollection(inner, _) => Some(inner.to_owned()),
                    RenderedCollectionType::Other => None,
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
            Some(PathStepKind::Field) => {
                text_member(step, 1).map(|name| format!("{}/{name}", declaring.unwrap_or_default()))
            }
            Some(PathStepKind::Tuple) => text_member(step, 1)
                .map(|slot| format!("{}/tuple/{slot}", declaring.unwrap_or_default())),
            Some(PathStepKind::Union) => text_member(step, 1)
                .map(|case| format!("{}/union/{case}", declaring.unwrap_or_default())),
            Some(PathStepKind::Slot) => parent_type.as_ref().map(|name| format!("{name}[slot]")),
            _ => None,
        };
        if step_kind == Some(PathStepKind::Field) {
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
            let actual_type = declared_field_type(field, |reference| {
                model_owner.package.clock_type_name(reference)
            })
            .ok_or_else(|| fail(at.clone()))?;
            if actual_declaring != declaring
                || Some(field.identity.as_ref()) != identity
                || actual_type != member_type.unwrap_or_default()
            {
                return Err(fail(at));
            }
            if steps.get(index + 1).and_then(path_step_kind) == Some(PathStepKind::Slot) {
                let multiplicity = field.slot.multiplicity;
                let unwrapped = actual_type
                    .strip_prefix("Option<")
                    .and_then(|inner| inner.strip_suffix('>'))
                    .unwrap_or(&actual_type);
                let expected = match RenderedCollectionType::from_wire(unwrapped) {
                    RenderedCollectionType::Map(_)
                        if (multiplicity.lower, multiplicity.upper) == (1, Some(1)) =>
                    {
                        Some(("map", "0".to_owned(), "unbounded".to_owned()))
                    }
                    _ if (multiplicity.lower, multiplicity.upper) != (1, Some(1)) => {
                        let kind =
                            CollectionKind::of(multiplicity.ordered, multiplicity.unique).form();
                        Some((
                            kind,
                            multiplicity.lower.to_string(),
                            multiplicity
                                .upper
                                .map_or_else(|| "unbounded".to_owned(), |upper| upper.to_string()),
                        ))
                    }
                    _ => None,
                };
                if expected.as_ref().is_none_or(|(kind, lower, upper)| {
                    collection != Some(*kind)
                        || minimum != Some(lower.as_str())
                        || maximum != Some(upper.as_str())
                }) {
                    return Err(fail(at));
                }
            }
        }
        if expected_identity.as_deref() != identity {
            return Err(fail(at));
        }
        if step_kind == Some(PathStepKind::Slot) {
            let step_at = binding_at(position, 1).key("members").index(index);
            let selector_kind = bound(step, 2)
                .and_then(typed_target)
                .and_then(|target| graph.index.get(target).copied())
                .and_then(|position| graph.kinds.get(position).copied());
            if !selector_fits(
                parent_type.as_deref().unwrap_or_default(),
                text_member(step, 1).unwrap_or_default(),
                member_type.unwrap_or_default(),
                selector_kind,
            ) {
                return Err(fail(step_at));
            }
        }
        let next_kind = steps.get(index + 1).and_then(path_step_kind);
        if next_kind == Some(PathStepKind::Slot) {
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
    if !rows
        .last()
        .and_then(|row| text_member(row, 3))
        .is_some_and(terminal_clock)
    {
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
        let exact = ExactMagnitude::from_wire(value).ok_or_else(invalid)?;
        // The strict shape has checked nonnegative decimal syntax. Shared
        // natural arithmetic accounts for the exact reduction work.
        if positive && exact.zero
            || !super::natural::coprime(exact.numerator, exact.denominator, meter, &|| at.clone())?
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

type SourceOrderKey<'a> = (&'a str, u64, u64);

/// A source-map entry may add an earlier region to an existing node. Keep
/// the minimum across every occurrence and region, independently of node id.
fn source_order_keys<'a>(
    source_map: &'a [CheckedSourceMapEntry],
    meter: &mut WorkMeter,
) -> Result<BTreeMap<&'a CheckedNodeId, SourceOrderKey<'a>>, ValidationFailure> {
    let mut keys = BTreeMap::new();
    for entry in source_map {
        for region in &entry.regions {
            meter.charge(1, || JsonPointer::root().key("source_map"))?;
            let key = (region.source.identity.as_ref(), region.start, region.end);
            keys.entry(&entry.node_id)
                .and_modify(|minimum| {
                    if key < *minimum {
                        *minimum = key;
                    }
                })
                .or_insert(key);
        }
    }
    Ok(keys)
}

fn check_reference_order(
    body: &CheckedTerm,
    position: usize,
    binding: usize,
    keys: &BTreeMap<&CheckedNodeId, SourceOrderKey<'_>>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut previous = None;
    let mut seen = BTreeSet::new();
    for (index, member) in bound(body, binding)
        .map(typed_members)
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        let at = binding_at(position, binding).key("members").index(index);
        meter.charge(1, || at.clone())?;
        let Some(target) = typed_target(member) else {
            continue;
        };
        // A repeated node is the existing duplicate-place defect. Preserve
        // that more specific refusal; equal order keys of distinct nodes are
        // still checked here.
        if !seen.insert(target) {
            continue;
        }
        let key = keys.get(target).copied();
        // Graph-shape and source-map checks have already validated every
        // reference and region. Their refusal has precedence over this step.
        let Some(key) = key else {
            continue;
        };
        if previous.is_some_and(|before| key <= before) {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at,
            ));
        }
        previous = Some(key);
    }
    Ok(())
}

fn check_timed_reference_order(
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    source_map: &[CheckedSourceMapEntry],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let keys = source_order_keys(source_map, meter)?;
    for &position in index.values() {
        let bindings: &[usize] = match kinds.get(position) {
            Some(CheckedNodeKind::TimedModel(TimedModelForm::Model)) => &[3, 4, 5, 6, 7, 8],
            Some(CheckedNodeKind::TimedModel(TimedModelForm::Place)) => &[3],
            _ => continue,
        };
        let Some(body) = terms.get(position).and_then(Option::as_ref) else {
            continue;
        };
        for &binding in bindings {
            check_reference_order(body, position, binding, &keys, meter)?;
        }
    }
    Ok(())
}

/// Join the retained timed roots, members and local typed targets in digest order.
pub(super) fn validate_models<'a>(
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    source_map: &[CheckedSourceMapEntry],
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
    if !roots.is_empty() {
        check_timed_reference_order(kinds, terms, index, source_map, meter)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Provenance: IR-1083, QSpec STD-526 SL-01..08 and SR-01..04.
    #[test]
    fn ir1083_published_slot_selectors_match_their_type_nodes() {
        let Ok(root) = std::env::var("QUIRE_SPECIFICATION_DIR") else {
            return;
        };
        let path =
            Path::new(&root).join("proposals/checked-package-v2/timed-slot-selector-vectors.json");
        let bytes = std::fs::read(path).expect("published timed slot vectors");
        let vectors: Value = serde_json::from_slice(&bytes).expect("published vector JSON");
        let cases = vectors["cases"].as_array().expect("published slot cases");
        assert_eq!(cases.len(), 12);
        let mut seen = BTreeSet::new();
        for case in cases {
            let id = case["id"].as_str().expect("case id");
            assert!(seen.insert(id));
            let steps = case["node"]["body"]["members"][1]["value"]["members"]
                .as_array()
                .expect("place path");
            let rows = case["node"]["body"]["members"][2]["value"]["members"]
                .as_array()
                .expect("resolved path");
            assert_eq!(steps.len(), rows.len(), "{id}: resolved row count");
            let mut checked = false;
            for (index, step) in steps.iter().enumerate() {
                if step["members"][0]["value"]["value"] != "slot" {
                    continue;
                }
                checked = true;
                let parent = rows[index - 1]["members"][3]["value"]["value"]
                    .as_str()
                    .expect("parent type");
                let collection = step["members"][1]["value"]["value"]
                    .as_str()
                    .expect("collection kind");
                let member = rows[index]["members"][3]["value"]["value"]
                    .as_str()
                    .expect("selected member type");
                let selector = &step["members"][2]["value"];
                let selector_kind = match selector["target"]["digest"].as_str() {
                    Some(digest) if Some(digest) == vectors["type_nodes"]["integer"].as_str() => {
                        Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Integer))
                    }
                    Some(digest) if Some(digest) == vectors["type_nodes"]["text"].as_str() => {
                        Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Text))
                    }
                    Some(_) => Some(CheckedNodeKind::Expression(
                        super::super::ExpressionForm::Query,
                    )),
                    None => None,
                };
                let fits = selector_fits(parent, collection, member, selector_kind);
                assert_eq!(fits, case["expected"] == "admitted", "{id}: selector fit");
                if id == "SL-08" {
                    let forged = format!("Option<{member}>");
                    assert!(
                        terminal_clock(&forged),
                        "mutant must retain a Clock terminal"
                    );
                    assert!(
                        !selector_fits(parent, collection, &forged, selector_kind),
                        "{id}: slot row must match the collection element type"
                    );
                }
            }
            assert!(checked, "{id}: slot case has a slot step");
        }
        for ordinal in 1..=8 {
            assert!(seen.contains(format!("SL-{ordinal:02}").as_str()));
        }
        for ordinal in 1..=4 {
            assert!(seen.contains(format!("SR-{ordinal:02}").as_str()));
        }
    }

    #[test]
    fn ir1083_optional_clock_terminal_unwraps_once() {
        assert!(terminal_clock("Option<ix://quire/native/Clock>"));
        assert!(terminal_clock("ix://quire/native/Clock"));
        assert!(!terminal_clock("Option<Option<ix://quire/native/Clock>>"));
    }

    #[test]
    fn ir1083_selected_field_presence_and_multiplicity_render_the_clock_path_type() {
        use super::super::model_members::{Multiplicity, NativeTypeRef, TypedSlot};
        let mut field = FieldDecl {
            identity: "ix://acme/orders/Order/v".into(),
            slot: TypedSlot {
                name: Some("v".into()),
                type_ref: Some(SelectedTypeRef::Native(NativeTypeRef::Clock)),
                multiplicity: Multiplicity {
                    lower: 1,
                    upper: Some(1),
                    ordered: false,
                    unique: true,
                },
            },
            optional: true,
            redefines: None,
        };
        assert_eq!(
            declared_field_type(&field, |reference| match reference {
                SelectedTypeRef::Native(NativeTypeRef::Clock) => {
                    Some("ix://quire/native/Clock".to_owned())
                }
                _ => None,
            })
            .as_deref(),
            Some("Option<ix://quire/native/Clock>")
        );
        field.optional = false;
        field.slot.multiplicity = Multiplicity {
            lower: 0,
            upper: Some(3),
            ordered: true,
            unique: false,
        };
        assert_eq!(
            declared_field_type(&field, |reference| match reference {
                SelectedTypeRef::Native(NativeTypeRef::Clock) => {
                    Some("ix://quire/native/Clock".to_owned())
                }
                _ => None,
            })
            .as_deref(),
            Some("Sequence<ix://quire/native/Clock>")
        );
    }
}
