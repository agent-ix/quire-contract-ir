// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Closed timed-model node wire shapes checked before identity validation.

use super::graph_storage::{first_positions, GraphIndex, SortedTable};
use super::model_members::{
    Budget, CollectionKind, DomainModel, FieldDecl, MemberKind, ModelFailure, ModelOwners,
    Resolved, SelectedTypeRef,
};
use super::{
    CheckedNodeId, CheckedNodeOwner, CheckedTerm, CheckedTermView, LiteralKind,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, TimedModelForm,
};
use crate::checked_package::common::{
    body_term, exact_members, node_pointer, TermField, ValidationFailure, WireTerm,
};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCode, CheckedSourceMapEntry, JsonPointer,
};
use serde_json::Value;

fn malformed(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, at)
}

/// The selected model of a package identity.
pub(super) trait ModelLookup {
    fn model(&self, identity: &str) -> Option<&DomainModel>;
}

impl ModelLookup for BTreeMap<&str, &DomainModel> {
    fn model(&self, identity: &str) -> Option<&DomainModel> {
        self.get(identity).copied()
    }
}

impl ModelLookup for SortedTable<&str, &DomainModel> {
    fn model(&self, identity: &str) -> Option<&DomainModel> {
        self.get(&identity).copied()
    }
}

/// A source Time has a declared nominal body; a selected model Time is
/// identified by its FCD declaration and has no declaration or body terms.
#[ix_trace_rs::string_edge]
pub(super) fn is_base_time_dimension(
    node: &CheckedSemanticNodeV2,
    models: &impl ModelLookup,
) -> bool {
    match node.owner.as_ref() {
        Some(CheckedNodeOwner::Model {
            identity,
            node: fcd_id,
        }) => {
            node.declaration.is_none()
                && models
                    .model(identity.as_ref())
                    .and_then(|model| model.dimensions.get(fcd_id))
                    .is_some_and(|dimension| dimension.name.as_ref() == "Time" && dimension.base)
        }
        Some(CheckedNodeOwner::Source { .. }) => {
            super::identity::NominalBody::from_value(&node.body, JsonPointer::root()).is_ok_and(
                |body| {
                    matches!(
                        body.kind(),
                        super::identity::NominalBodyKind::Dimension { base: true, .. }
                    )
                },
            ) && node
                .declaration
                .as_ref()
                .and_then(|declaration| declaration.qualified_name.last())
                .is_some_and(|name| name.as_ref() == "Time")
        }
        _ => false,
    }
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
    CheckedNodeId::from_value(value).filter(CheckedNodeId::is_well_formed)
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
#[ix_trace_rs::string_edge]
pub(super) fn check_wire(
    node: &CheckedSemanticNodeV2,
    form: TimedModelForm,
    position: usize,
) -> Result<(), ValidationFailure> {
    let at = node_pointer(position);
    if node.recursion_group.is_some() {
        return Err(malformed(at.clone().key("recursion_group")));
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
            let b = bindings(&node.body, &["model", "path", "resolved_path"], &body_at)
                .map_err(|_| malformed(body_at.clone()))?;
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
        }
        TimedModelForm::Reset => {
            let members = aggregate(&node.body, &body_at)?;
            let kind_at = body_at.clone().key("members").index(0).key("value");
            let kind = members
                .first()
                .and_then(possible_binding_value)
                .ok_or_else(|| malformed(body_at.clone()))?;
            match text(kind, &kind_at)? {
                "place" => {
                    let b = bindings(
                        &node.body,
                        &[
                            "kind",
                            "operation",
                            "place",
                            "owner",
                            "slots",
                            "binders",
                            "condition",
                            "value",
                        ],
                        &body_at,
                    )
                    .map_err(|_| malformed(body_at.clone()))?;
                    reference(b[1].0, &b[1].1)?;
                    reference(b[2].0, &b[2].1)?;
                    reference(b[3].0, &b[3].1)?;
                    references(b[4].0, &b[4].1)?;
                    references(b[5].0, &b[5].1)?;
                    reference(b[6].0, &b[6].1)?;
                    magnitude(b[7].0, "quantity", &b[7].1)?;
                }
                "parameter" => {
                    let b = bindings(
                        &node.body,
                        &["kind", "operation", "parameter", "value"],
                        &body_at,
                    )
                    .map_err(|_| malformed(body_at.clone()))?;
                    reference(b[1].0, &b[1].1)?;
                    text(b[2].0, &b[2].1)?;
                    magnitude(b[3].0, "quantity", &b[3].1)?;
                }
                _ => return Err(malformed(kind_at.key("value"))),
            }
        }
        TimedModelForm::Guard => {
            let b = bindings(&node.body, &["operation", "parts"], &body_at)?;
            reference(b[0].0, &b[0].1)?;
            let parts = aggregate(b[1].0, &b[1].1)?;
            if parts.is_empty() {
                return Err(malformed(body_at));
            }
            for (index, part) in parts.iter().enumerate() {
                let at = b[1].1.clone().key("members").index(index);
                let members = aggregate(part, &at)?;
                if members.len() < 2 {
                    return Err(malformed(body_at));
                }
                // A part starts with its predicate, may bind quantified slots, then names constraints.
                let first_at = at.clone().key("members").index(0);
                let predicate = match CheckedTerm::view(&members[0]) {
                    Some(CheckedTermView::Binding {
                        name: "predicate",
                        value,
                    }) => value,
                    _ => return Err(malformed(first_at)),
                };
                reference(predicate, &first_at.key("value"))?;
                let first_constraint = match members.get(1).and_then(CheckedTerm::view) {
                    Some(CheckedTermView::Binding {
                        name: "binders",
                        value,
                    }) => {
                        let binders_at = at.clone().key("members").index(1).key("value");
                        if aggregate(value, &binders_at)?.is_empty() {
                            return Err(malformed(binders_at.key("members")));
                        }
                        references(value, &binders_at)?;
                        2
                    }
                    _ => 1,
                };
                if members.len() <= first_constraint {
                    return Err(malformed(body_at));
                }
                for (reference_index, member) in members.iter().enumerate().skip(first_constraint) {
                    reference(member, &at.clone().key("members").index(reference_index))?;
                }
            }
        }
        TimedModelForm::Invariant => {
            let members = aggregate(&node.body, &body_at)?;
            let quantified = matches!(
                members.get(1).and_then(CheckedTerm::view),
                Some(CheckedTermView::Binding {
                    name: "binders",
                    ..
                })
            );
            let b = if quantified {
                bindings(&node.body, &["predicate", "binders", "bounds"], &body_at)?
            } else {
                bindings(&node.body, &["predicate", "bounds"], &body_at)?
            };
            reference(b[0].0, &b[0].1)?;
            if quantified {
                if aggregate(b[1].0, &b[1].1)?.is_empty() {
                    return Err(malformed(b[1].1.clone().key("members")));
                }
                references(b[1].0, &b[1].1)?;
            }
            let bounds_index = if quantified { 2 } else { 1 };
            let bounds = aggregate(b[bounds_index].0, &b[bounds_index].1)?;
            if bounds.is_empty() {
                return Err(malformed(body_at));
            }
            for (index, member) in bounds.iter().enumerate() {
                reference(
                    member,
                    &b[bounds_index].1.clone().key("members").index(index),
                )?;
            }
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
            let members = aggregate(&node.body, &body_at)?;
            let kind_at = body_at.clone().key("members").index(0).key("value");
            let kind = members
                .first()
                .and_then(possible_binding_value)
                .ok_or_else(|| malformed(body_at.clone()))?;
            match text(kind, &kind_at)? {
                "place" => {
                    let b = bindings(
                        &node.body,
                        &["kind", "place", "owner", "slots", "comparison", "bound"],
                        &body_at,
                    )?;
                    reference(b[1].0, &b[1].1)?;
                    reference(b[2].0, &b[2].1)?;
                    references(b[3].0, &b[3].1)?;
                    word(b[4].0, &["lt", "le", "eq", "ge", "gt"], &b[4].1)
                        .map_err(|_| malformed(body_at.clone()))?;
                    magnitude(b[5].0, "quantity", &b[5].1)?;
                }
                "parameter" => {
                    let b = bindings(
                        &node.body,
                        &["kind", "operation", "parameter", "comparison", "bound"],
                        &body_at,
                    )?;
                    reference(b[1].0, &b[1].1)?;
                    text(b[2].0, &b[2].1)?;
                    word(b[3].0, &["lt", "le", "eq", "ge", "gt"], &b[3].1)
                        .map_err(|_| malformed(body_at.clone()))?;
                    magnitude(b[4].0, "quantity", &b[4].1)?;
                }
                _ => return Err(malformed(kind_at.key("value"))),
            }
        }
    }
    Ok(())
}

use super::{
    CheckedFrameModification, CheckedNodeKind, CompositeTypeForm, ScalarTypeForm, StateForm,
    TemporalForm, TemporalProfile, ValueForm, WorkMeter,
};
use crate::checked_package::shared::CheckedPackageRefusalCause;
use std::cmp::Ordering;
use std::collections::BTreeMap;

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

/// Whether the integer or text literal `term` spells `text`; a term that
/// is neither (or an integer past `u64`) spells nothing.
fn scalar_matches(term: Option<&CheckedTerm>, text: Option<&str>) -> bool {
    match term {
        Some(CheckedTerm::Literal {
            value_kind: LiteralKind::Integer,
            value,
            ..
        }) => match value.as_u64() {
            Some(number) => text.is_some_and(|text| decimal_is(text, number)),
            None => text.is_none(),
        },
        Some(CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        }) => value.as_str() == text,
        _ => text.is_none(),
    }
}

/// Whether `text` is `number` in decimal, compared without rendering it.
fn decimal_is(text: &str, number: u64) -> bool {
    let mut digits = [0_u8; 20];
    let mut at = digits.len();
    let mut rest = number;
    loop {
        at -= 1;
        digits[at] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    text.as_bytes() == &digits[at..]
}

/// Whether `text` is the concatenation of `parts`, compared without
/// building it.
fn concat_is(text: &str, parts: &[&str]) -> bool {
    let mut rest = text;
    for part in parts {
        match rest.strip_prefix(part) {
            Some(after) => rest = after,
            None => return false,
        }
    }
    rest.is_empty()
}

/// A qualified name written with `/` between its parts.
struct Slashed<'a>(&'a [Box<str>]);

impl std::fmt::Display for Slashed<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (at, part) in self.0.iter().enumerate() {
            if at > 0 {
                formatter.write_str("/")?;
            }
            formatter.write_str(part)?;
        }
        Ok(())
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

/// The rendered type of a declared field. The element's own rendering comes
/// from `render`; each wrapper around it is written into a block charged
/// first.
fn declared_field_type(
    field: &FieldDecl,
    render: impl FnOnce(&SelectedTypeRef) -> Option<String>,
    meter: &mut WorkMeter,
) -> Result<Option<String>, ValidationFailure> {
    let Some(element) = field.slot.type_ref.as_ref().and_then(render) else {
        return Ok(None);
    };
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
        meter
            .formatted(format_args!("Option<{element}>"))?
            .into_string()
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
        meter
            .formatted(format_args!("{kind}<{element}>"))?
            .into_string()
    };
    Ok(Some(if field.optional && !optional_clock_element {
        meter
            .formatted(format_args!("Option<{value}>"))?
            .into_string()
    } else {
        value
    }))
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

#[ix_trace_rs::string_edge]
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
    let model_name: Option<Word<'_>> = match model_node.declaration.as_ref() {
        Some(declaration) => Some(match declaration.qualified_name.as_slice() {
            [only] => Word::Borrowed(only),
            parts => Word::Owned(meter.formatted(format_args!("{}", Slashed(parts)))?),
        }),
        None => match model_node.owner.as_ref() {
            Some(CheckedNodeOwner::Model { node, .. }) => Some(Word::Borrowed(node)),
            _ => None,
        },
    };
    let mut expected_declaring = model_name;
    for (index, (step, row)) in steps.iter().zip(rows).enumerate() {
        let at = rows_at.clone().key("members").index(index);
        meter.charge(1, || at.clone())?;
        let step_kind = path_step_kind(step);
        let parent_type = expected_declaring.as_ref().map(Word::as_str);
        let declaring_expected = if step_kind == Some(PathStepKind::Slot) {
            parent_type.and_then(|ty| match RenderedCollectionType::from_wire(ty) {
                RenderedCollectionType::Map(_) => Some(ty),
                RenderedCollectionType::OtherCollection(inner, _) => Some(inner),
                RenderedCollectionType::Other => None,
            })
        } else {
            parent_type
        };
        let step_index = bound(row, 0).and_then(integer_value);
        let declaring = text_member(row, 1);
        let identity = text_member(row, 2);
        let member_type = text_member(row, 3);
        let collection = text_member(row, 4);
        let minimum = text_member(row, 5);
        let maximum = text_member(row, 6);
        if step_index != Some(index as u64)
            || declaring != declaring_expected
            || identity.is_none()
            || member_type.is_none()
            || collection.is_none()
            || minimum.is_none()
            || maximum.is_none()
        {
            return Err(fail(at));
        }
        let declared = declaring.unwrap_or_default();
        let identity_matches = identity.is_some_and(|identity| match step_kind {
            Some(PathStepKind::Field) => {
                text_member(step, 1).is_some_and(|name| concat_is(identity, &[declared, "/", name]))
            }
            Some(PathStepKind::Tuple) => text_member(step, 1)
                .is_some_and(|slot| concat_is(identity, &[declared, "/tuple/", slot])),
            Some(PathStepKind::Union) => text_member(step, 1)
                .is_some_and(|case| concat_is(identity, &[declared, "/union/", case])),
            Some(PathStepKind::Slot) => {
                parent_type.is_some_and(|name| concat_is(identity, &[name, "[slot]"]))
            }
            _ => false,
        });
        if step_kind == Some(PathStepKind::Field) {
            let path_at = binding_at(position, 1).key("members").index(index);
            let model_owner = owners
                .recover(model_node)
                .map_err(|_| fail(path_at.clone()))?;
            let declaration = parent_type.unwrap_or_default();
            let mut budget =
                Budget::new(meter, model_owner.package.root, model_owner.package.bytes);
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
            let actual_type = declared_field_type(
                field,
                |reference| model_owner.package.clock_type_name(reference),
                meter,
            )?
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
                let exact_one = (multiplicity.lower, multiplicity.upper) == (1, Some(1));
                // The collection form and the bounds the slot must spell.
                let expected = match RenderedCollectionType::from_wire(unwrapped) {
                    RenderedCollectionType::Map(_) if exact_one => Some(("map", 0, None)),
                    _ if !exact_one => Some((
                        CollectionKind::of(multiplicity.ordered, multiplicity.unique).form(),
                        multiplicity.lower,
                        multiplicity.upper,
                    )),
                    _ => None,
                };
                if expected.is_none_or(|(kind, lower, upper)| {
                    collection != Some(kind)
                        || !minimum.is_some_and(|text| decimal_is(text, lower))
                        || !match upper {
                            None => maximum == Some("unbounded"),
                            Some(upper) => maximum.is_some_and(|text| decimal_is(text, upper)),
                        }
                }) {
                    return Err(fail(at));
                }
            }
        }
        if !identity_matches {
            return Err(fail(at));
        }
        if step_kind == Some(PathStepKind::Slot) {
            let step_at = binding_at(position, 1).key("members").index(index);
            let selector_kind = bound(step, 2)
                .and_then(typed_target)
                .and_then(|target| graph.index.get(target).copied())
                .and_then(|position| graph.kinds.get(position).copied());
            if !selector_fits(
                parent_type.unwrap_or_default(),
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
            if collection != expected_collection
                || !scalar_matches(bound(next, 3), minimum)
                || !scalar_matches(bound(next, 4), maximum)
            {
                return Err(fail(at));
            }
        } else if collection != Some("none") || minimum != Some("") || maximum != Some("") {
            return Err(fail(at));
        }
        expected_declaring = member_type.map(Word::Borrowed);
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
    index: &'a GraphIndex<'a>,
}

struct PlaceBindings {
    place: usize,
    owner: usize,
    slots: usize,
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
            matches!(kind, CheckedNodeKind::Expression(_))
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

    fn type_issue(
        &self,
        position: usize,
        cause: CheckedPackageRefusalCause,
        at: JsonPointer,
    ) -> ValidationFailure {
        ValidationFailure::refused_at(
            CheckedPackageRefusalCode::IllTyped,
            at,
            Some(cause),
            self.nodes[position].node_id.clone(),
        )
    }

    fn anchor(
        &self,
        position: usize,
        term: Option<&CheckedTerm>,
        at: JsonPointer,
        model: usize,
    ) -> Result<usize, ValidationFailure> {
        let anchor = self.target(position, term, at.clone(), |kind| {
            kind == CheckedNodeKind::State(StateForm::OperationAnchor)
        })?;
        let body = self.body(anchor)?;
        if matches!(body, CheckedTerm::OperationAnchorBody(body) if body.execution_point.is_some())
        {
            return Err(self.issue(position, CheckedPackageRefusalCause::TimedNodeReference, at));
        }
        let context = self.target(
            anchor,
            bound(body, 0),
            binding_at(anchor, 0).key("target"),
            |kind| matches!(kind, CheckedNodeKind::Model(_)),
        )?;
        if context != model {
            return Err(self.issue(position, CheckedPackageRefusalCause::TimedNodeReference, at));
        }
        Ok(anchor)
    }

    fn place_terms(
        &self,
        position: usize,
        body: &CheckedTerm,
        bindings: PlaceBindings,
        model: usize,
        owners: &ModelOwners<'_>,
        meter: &mut WorkMeter,
    ) -> Result<usize, ValidationFailure> {
        use CheckedPackageRefusalCause as Cause;
        let PlaceBindings {
            place: place_binding,
            owner: owner_binding,
            slots: slots_binding,
        } = bindings;
        let place = self.target(
            position,
            bound(body, place_binding),
            binding_at(position, place_binding).key("target"),
            |kind| kind == CheckedNodeKind::TimedModel(TimedModelForm::Place),
        )?;
        let place_body = self.body(place)?;
        let place_model = self.target(
            place,
            bound(place_body, 0),
            binding_at(place, 0).key("target"),
            |kind| matches!(kind, CheckedNodeKind::Model(_)),
        )?;
        if place_model != model {
            return Err(self.issue(
                position,
                Cause::TimedNodeReference,
                binding_at(position, place_binding).key("target"),
            ));
        }
        check_resolved_path(self, place, place_body, model, owners, meter)?;
        let owner = self.target(
            position,
            bound(body, owner_binding),
            binding_at(position, owner_binding).key("target"),
            |kind| {
                matches!(
                    kind,
                    CheckedNodeKind::Value(_) | CheckedNodeKind::Expression(_)
                )
            },
        )?;
        let owner_type = self.index.get(&self.nodes[owner].semantic_type).copied();
        let correct_owner = owner_type
            .and_then(|p| self.body(p).ok())
            .and_then(|body| typed_members(body).first())
            .and_then(typed_target)
            == Some(&self.nodes[model].node_id);
        if !correct_owner {
            return Err(self.type_issue(
                position,
                Cause::TypeMismatch,
                binding_at(position, owner_binding),
            ));
        }
        let slots = bound(body, slots_binding)
            .map(typed_members)
            .unwrap_or_default();
        // Walked twice (count, then pair) instead of collected.
        let selectors = || {
            bound(place_body, 1)
                .map(typed_members)
                .unwrap_or_default()
                .iter()
                .filter(|step| path_step_kind(step) == Some(PathStepKind::Slot))
                .filter_map(|step| bound(step, 2).and_then(typed_target))
        };
        if slots.len() != selectors().count() {
            return Err(self.type_issue(
                position,
                Cause::OperatorIneligible,
                binding_at(position, slots_binding),
            ));
        }
        for (index, (slot, selector)) in slots.iter().zip(selectors()).enumerate() {
            let at = binding_at(position, slots_binding)
                .key("members")
                .index(index);
            let target = self.target(position, Some(slot), at.clone(), |kind| {
                matches!(
                    kind,
                    CheckedNodeKind::Value(_) | CheckedNodeKind::Expression(_)
                )
            })?;
            if &self.nodes[target].semantic_type != selector {
                return Err(self.type_issue(position, Cause::TypeMismatch, at));
            }
        }
        Ok(place)
    }

    fn frame_writes_place(&self, anchor: usize, place: usize, model: usize) -> bool {
        let Some(anchor_body) = self.terms.get(anchor).and_then(Option::as_ref) else {
            return false;
        };
        let Some(frame) = bound(anchor_body, 2)
            .and_then(typed_target)
            .and_then(|key| self.index.get(key))
            .copied()
        else {
            return false;
        };
        let Some(CheckedTerm::Frame { modifies, .. }) =
            self.terms.get(frame).and_then(Option::as_ref)
        else {
            return false;
        };
        let Some(place_body) = self.terms.get(place).and_then(Option::as_ref) else {
            return false;
        };
        let field = bound(place_body, 1)
            .map(typed_members)
            .and_then(|steps| steps.first())
            .and_then(|step| text_member(step, 1));
        modifies.iter().any(|entry| {
            matches!(entry, CheckedFrameModification::Field { declaration, name, .. }
            if declaration == &self.nodes[model].node_id && Some(name.as_ref()) == field)
        })
    }
}

fn frame_writes_clock(
    graph: &Graph<'_>,
    anchor: usize,
    model: usize,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let Some(body) = graph.terms.get(anchor).and_then(Option::as_ref) else {
        return Ok(false);
    };
    let Some(frame) = bound(body, 2)
        .and_then(typed_target)
        .and_then(|key| graph.index.get(key))
        .copied()
    else {
        return Ok(false);
    };
    let Some(CheckedTerm::Frame { modifies, .. }) = graph.terms.get(frame).and_then(Option::as_ref)
    else {
        return Ok(false);
    };
    let model_owner = owners.recover(&graph.nodes[model]).map_err(|_| {
        graph.issue(
            anchor,
            CheckedPackageRefusalCause::TimedNodeReference,
            binding_at(anchor, 0),
        )
    })?;
    for entry in modifies {
        let CheckedFrameModification::Field {
            declaration, name, ..
        } = entry
        else {
            continue;
        };
        if declaration != &graph.nodes[model].node_id {
            continue;
        }
        let mut budget = Budget::new(meter, model_owner.package.root, model_owner.package.bytes);
        match model_owner
            .package
            .resolve(model_owner.node, MemberKind::Field, name, &mut budget)
        {
            Ok(Resolved::Field(field))
                if matches!(
                    field.slot.type_ref,
                    Some(SelectedTypeRef::Native(
                        super::model_members::NativeTypeRef::Clock
                    ))
                ) =>
            {
                return Ok(true);
            }
            Err(ModelFailure::Limit(failure)) => return Err(failure),
            _ => {}
        }
    }
    Ok(false)
}

/// Compare the retained path terms exactly, charging each actual term visit.
fn same_place_path(
    left: &CheckedTerm,
    right: &CheckedTerm,
    meter: &mut WorkMeter,
    at: &JsonPointer,
) -> Result<bool, ValidationFailure> {
    let mut pending = meter.vec(1)?;
    pending.push((left, right));
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
                meter.room(&mut pending, left.len())?;
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
                meter.push(&mut pending, (&**left, &**right))?;
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

#[ix_trace_rs::string_edge]
fn is_quantified_binders(term: Option<&CheckedTerm>) -> bool {
    matches!(term, Some(CheckedTerm::Binding { name, .. }) if name.as_ref() == "binders")
}

#[ix_trace_rs::string_edge]
fn extend_place_slots<'a>(
    slots: &mut Vec<&'a CheckedNodeId>,
    constraint: &'a CheckedTerm,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if bound(constraint, 0).and_then(spelling) == Some("place") {
        let more = bound(constraint, 3)
            .map(typed_members)
            .unwrap_or_default()
            .iter()
            .filter_map(typed_target);
        meter.room(slots, more.clone().count())?;
        slots.extend(more);
    }
    Ok(())
}

fn check_quantified_binders(
    graph: &Graph<'_>,
    position: usize,
    binders: Option<&CheckedTerm>,
    at: JsonPointer,
    slots: &[&CheckedNodeId],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(binders) = binders else {
        return Ok(());
    };
    meter.charge(u64::try_from(slots.len()).unwrap_or(u64::MAX), || {
        at.clone()
    })?;
    let slot_positions = first_positions(slots.iter().copied(), meter)?;
    let mut previous_slot = None;
    for (index, binder) in typed_members(binders).iter().enumerate() {
        let binder_at = at.clone().key("members").index(index).key("target");
        meter.charge(1, || binder_at.clone())?;
        let target = graph.target(position, Some(binder), binder_at.clone(), |kind| {
            kind == CheckedNodeKind::Value(ValueForm::Parameter)
        })?;
        let Some(&slot_index) = slot_positions.get(&&graph.nodes[target].node_id) else {
            return Err(graph.issue(
                position,
                CheckedPackageRefusalCause::TimedNodeReference,
                binder_at,
            ));
        };
        if previous_slot.is_some_and(|previous| slot_index <= previous) {
            return Err(graph.issue(
                position,
                CheckedPackageRefusalCause::TimedNodeReference,
                binder_at,
            ));
        }
        previous_slot = Some(slot_index);
    }
    Ok(())
}

type SourceOrderKey<'a> = (&'a str, u64, u64);

/// A word of an order key: borrowed from the graph when the package spells
/// it, otherwise formatted into a block charged first.
#[derive(Debug)]
enum Word<'g> {
    Borrowed(&'g str),
    Owned(Box<str>),
}

impl Word<'_> {
    fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(text) => text,
            Self::Owned(text) => text,
        }
    }
}

#[derive(Debug)]
enum OrderPart<'g> {
    End,
    Word(Word<'g>),
    Number(u64),
    Infinity,
    Ratio(quire_exact::Integer, quire_exact::Integer),
    Start,
}

impl OrderPart<'_> {
    fn rank(&self) -> u8 {
        match self {
            Self::End => 0,
            Self::Number(_) => 1,
            Self::Infinity => 2,
            Self::Word(_) => 3,
            Self::Ratio(..) => 4,
            Self::Start => 5,
        }
    }

    /// The order of two parts. Two ratios compare by cross multiplication,
    /// each product's limb blocks charged before they are requested.
    fn compare(&self, other: &Self, meter: &mut WorkMeter) -> Result<Ordering, ValidationFailure> {
        Ok(match (self, other) {
            (Self::End, Self::End) | (Self::Start, Self::Start) => Ordering::Equal,
            (Self::Word(left), Self::Word(right)) => left.as_str().cmp(right.as_str()),
            (Self::Number(left), Self::Number(right)) => left.cmp(right),
            (Self::Infinity, Self::Infinity) => Ordering::Equal,
            (Self::Ratio(left_n, left_d), Self::Ratio(right_n, right_d)) => {
                let left = meter.exact_mul(left_n, right_d)?;
                let right = meter.exact_mul(right_n, left_d)?;
                left.cmp(&right)
            }
            _ => self.rank().cmp(&other.rank()),
        })
    }
}

/// Lexicographic order of two keys, a shorter prefix first.
fn compare_keys(
    left: &[OrderPart<'_>],
    right: &[OrderPart<'_>],
    meter: &mut WorkMeter,
) -> Result<Ordering, ValidationFailure> {
    for (left, right) in left.iter().zip(right) {
        match left.compare(right, meter)? {
            Ordering::Equal => {}
            order => return Ok(order),
        }
    }
    Ok(left.len().cmp(&right.len()))
}

fn quantity_key<'g>(
    term: Option<&CheckedTerm>,
    meter: &mut WorkMeter,
    at: &JsonPointer,
) -> Result<Option<OrderPart<'g>>, ValidationFailure> {
    let Some(value) = term.and_then(spelling) else {
        return Ok(None);
    };
    // Cross multiplication compares two values of this size. Charge its
    // quadratic upper bound before either decimal parse or multiplication.
    let digits = u64::try_from(value.len()).unwrap_or(u64::MAX);
    meter.charge(digits.saturating_mul(digits), || at.clone())?;
    let Some(exact) = ExactMagnitude::from_wire(value) else {
        return Ok(None);
    };
    let (Some(numerator), Some(denominator)) = (
        meter.exact_integer(exact.numerator)?,
        meter.exact_integer(exact.denominator)?,
    ) else {
        return Ok(None);
    };
    Ok(Some(OrderPart::Ratio(numerator, denominator)))
}

/// Flatten the nested structural term key with explicit list delimiters.
/// A reference is replaced by its target's form and body; node ids and
/// source regions therefore cannot decide the order of a shared term.
/// `active` marks the positions on the current reference path, all clear on
/// entry and on return.
fn term_key<'g>(
    graph: &Graph<'g>,
    term: Option<&'g CheckedTerm>,
    active: &mut [bool],
    meter: &mut WorkMeter,
    at: &JsonPointer,
) -> Result<Option<Vec<OrderPart<'g>>>, ValidationFailure> {
    enum Visit<'g> {
        Term(&'g CheckedTerm),
        Atom(OrderPart<'g>),
        Exit(usize),
    }
    let Some(term) = term else {
        return Ok(None);
    };
    let terms: &'g [Option<CheckedTerm>] = graph.terms;
    let mut key: Vec<OrderPart<'g>> = Vec::new();
    let mut pending: Vec<Visit<'g>> = meter.vec(1)?;
    pending.push(Visit::Term(term));
    let outcome = 'walk: {
        while let Some(visit) = pending.pop() {
            meter.charge(1, || at.clone())?;
            match visit {
                Visit::Atom(atom) => meter.push(&mut key, atom)?,
                Visit::Exit(position) => {
                    if let Some(mark) = active.get_mut(position) {
                        *mark = false;
                    }
                }
                Visit::Term(CheckedTerm::Reference { target }) => {
                    let Some(&position) = graph.index.get(target) else {
                        break 'walk None;
                    };
                    match active.get_mut(position) {
                        Some(mark) if !*mark => *mark = true,
                        _ => break 'walk None,
                    }
                    let Some(body) = terms.get(position).and_then(Option::as_ref) else {
                        break 'walk None;
                    };
                    let Some(kind) = graph.kinds.get(position) else {
                        break 'walk None;
                    };
                    let word = meter.formatted(format_args!("{kind:?}"))?;
                    meter.room(&mut pending, 5)?;
                    pending.push(Visit::Exit(position));
                    pending.push(Visit::Atom(OrderPart::End));
                    pending.push(Visit::Term(body));
                    pending.push(Visit::Atom(OrderPart::Word(Word::Owned(word))));
                    pending.push(Visit::Atom(OrderPart::Start));
                }
                Visit::Term(CheckedTerm::Aggregate { members }) => {
                    meter.room(&mut pending, members.len() + 2)?;
                    pending.push(Visit::Atom(OrderPart::End));
                    pending.extend(members.iter().rev().map(Visit::Term));
                    pending.push(Visit::Atom(OrderPart::Start));
                }
                Visit::Term(CheckedTerm::Binding { name, value }) => {
                    meter.room(&mut pending, 4)?;
                    pending.push(Visit::Atom(OrderPart::End));
                    pending.push(Visit::Term(value));
                    pending.push(Visit::Atom(OrderPart::Word(Word::Borrowed(name))));
                    pending.push(Visit::Atom(OrderPart::Start));
                }
                Visit::Term(CheckedTerm::Literal {
                    value_kind, value, ..
                }) => {
                    let spelled = match value.as_str() {
                        Some(text) => Word::Borrowed(text),
                        None => Word::Owned(meter.formatted(format_args!("{value}"))?),
                    };
                    let kind = meter.formatted(format_args!("{value_kind:?}"))?;
                    meter.room(&mut pending, 4)?;
                    pending.push(Visit::Atom(OrderPart::End));
                    pending.push(Visit::Atom(OrderPart::Word(spelled)));
                    pending.push(Visit::Atom(OrderPart::Word(Word::Owned(kind))));
                    pending.push(Visit::Atom(OrderPart::Start));
                }
                Visit::Term(CheckedTerm::Application {
                    operator,
                    arguments,
                    ..
                }) => {
                    let word = meter.formatted(format_args!("{operator:?}"))?;
                    meter.room(&mut pending, arguments.len() + 5)?;
                    pending.push(Visit::Atom(OrderPart::End));
                    pending.push(Visit::Atom(OrderPart::End));
                    pending.extend(arguments.iter().rev().map(Visit::Term));
                    pending.push(Visit::Atom(OrderPart::Start));
                    pending.push(Visit::Atom(OrderPart::Word(Word::Owned(word))));
                    pending.push(Visit::Atom(OrderPart::Start));
                }
                // These closed bodies cannot be owner, slot, binder or predicate
                // terms. Leave their typed refusal to the later per-node join.
                Visit::Term(_) => break 'walk None,
            }
        }
        Some(())
    };
    for visit in pending {
        if let Visit::Exit(position) = visit {
            if let Some(mark) = active.get_mut(position) {
                *mark = false;
            }
        }
    }
    Ok(outcome.map(|()| key))
}

/// Gives a key `more` parts, charged first.
fn append<'g>(
    meter: &mut WorkMeter,
    key: &mut Vec<OrderPart<'g>>,
    more: Vec<OrderPart<'g>>,
) -> Result<(), ValidationFailure> {
    meter.room(key, more.len())?;
    key.extend(more);
    Ok(())
}

macro_rules! need {
    ($value:expr) => {
        match $value {
            Some(value) => value,
            None => return Ok(None),
        }
    };
}

fn place_path_key<'g>(
    graph: &Graph<'g>,
    place: usize,
    meter: &mut WorkMeter,
) -> Result<Option<Vec<OrderPart<'g>>>, ValidationFailure> {
    let terms: &'g [Option<CheckedTerm>] = graph.terms;
    let body = need!(terms.get(place).and_then(Option::as_ref));
    let steps = need!(bound(body, 1).map(typed_members));
    // At most five parts a step, between the opening and closing marks.
    let mut key: Vec<OrderPart<'g>> = meter.vec(steps.len().saturating_mul(5).saturating_add(2))?;
    key.push(OrderPart::Start);
    for step in steps {
        match need!(path_step_kind(step)) {
            PathStepKind::Field => {
                key.push(OrderPart::Number(0));
                key.push(OrderPart::Word(Word::Borrowed(need!(text_member(step, 1)))));
            }
            PathStepKind::Union => {
                key.push(OrderPart::Number(2));
                key.push(OrderPart::Word(Word::Borrowed(need!(text_member(step, 1)))));
            }
            PathStepKind::Tuple => {
                key.push(OrderPart::Number(1));
                key.push(OrderPart::Number(need!(
                    bound(step, 1).and_then(integer_value)
                )));
            }
            PathStepKind::Slot => {
                key.push(OrderPart::Number(3));
                key.push(OrderPart::Word(Word::Borrowed(need!(text_member(step, 1)))));
                key.push(OrderPart::Number(need!(
                    bound(step, 3).and_then(integer_value)
                )));
                key.push(if text_member(step, 4) == Some("unbounded") {
                    OrderPart::Infinity
                } else {
                    OrderPart::Number(need!(bound(step, 4).and_then(integer_value)))
                });
            }
        }
    }
    key.push(OrderPart::End);
    Ok(Some(key))
}

fn operation_name<'g>(graph: &Graph<'g>, term: Option<&CheckedTerm>) -> Option<&'g str> {
    let terms: &'g [Option<CheckedTerm>] = graph.terms;
    let position = *graph.index.get(typed_target(term?)?)?;
    let body = terms.get(position)?.as_ref()?;
    bound(body, 1).and_then(spelling)
}

#[derive(Clone, Copy)]
enum Shape {
    Term,
    Quantity,
}

/// What the key builders share: the graph, the path marks of the reference
/// walk, the meter and where the work is charged.
struct KeyWork<'w, 'g, 'b> {
    graph: &'w Graph<'g>,
    active: &'w mut [bool],
    meter: &'w mut WorkMeter<'b>,
    at: &'w JsonPointer,
}

impl<'g> KeyWork<'_, 'g, '_> {
    fn shape(
        &mut self,
        term: Option<&'g CheckedTerm>,
        shape: Shape,
    ) -> Result<Option<Vec<OrderPart<'g>>>, ValidationFailure> {
        match shape {
            Shape::Term => term_key(self.graph, term, self.active, self.meter, self.at),
            Shape::Quantity => match quantity_key(term, self.meter, self.at)? {
                Some(part) => {
                    let mut one = self.meter.vec(1)?;
                    one.push(part);
                    Ok(Some(one))
                }
                None => Ok(None),
            },
        }
    }

    /// Appends the key of `term` to `key`; `false` when it has none.
    fn extend(
        &mut self,
        key: &mut Vec<OrderPart<'g>>,
        term: Option<&'g CheckedTerm>,
        shape: Shape,
    ) -> Result<bool, ValidationFailure> {
        match self.shape(term, shape)? {
            Some(more) => {
                append(self.meter, key, more)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn push(
        &mut self,
        key: &mut Vec<OrderPart<'g>>,
        part: OrderPart<'g>,
    ) -> Result<(), ValidationFailure> {
        self.meter.push(key, part)
    }

    fn inner(&mut self, position: usize) -> Result<Option<Vec<OrderPart<'g>>>, ValidationFailure> {
        let graph = self.graph;
        let terms: &'g [Option<CheckedTerm>] = graph.terms;
        let body = need!(terms.get(position).and_then(Option::as_ref));
        let kind = need!(graph.kinds.get(position));
        let mut key: Vec<OrderPart<'g>> = Vec::new();
        macro_rules! got {
            ($call:expr) => {
                if !$call? {
                    return Ok(None);
                }
            };
        }
        match kind {
            CheckedNodeKind::TimedModel(TimedModelForm::Place) => {
                return place_path_key(graph, position, self.meter);
            }
            CheckedNodeKind::TimedModel(TimedModelForm::Constraint) => {
                if bound(body, 0).and_then(spelling) == Some("place") {
                    self.push(&mut key, OrderPart::Number(0))?;
                    let place = *need!(graph.index.get(need!(typed_target(need!(bound(body, 1))))));
                    let path = need!(place_path_key(graph, place, self.meter)?);
                    append(self.meter, &mut key, path)?;
                    got!(self.extend(&mut key, bound(body, 2), Shape::Term));
                    self.push(&mut key, OrderPart::Start)?;
                    for slot in need!(bound(body, 3).map(typed_members)) {
                        got!(self.extend(&mut key, Some(slot), Shape::Term));
                    }
                    self.push(&mut key, OrderPart::End)?;
                    let rank = need!(comparison_rank(need!(bound(body, 4).and_then(spelling))));
                    self.push(&mut key, OrderPart::Number(rank))?;
                    got!(self.extend(&mut key, bound(body, 5), Shape::Quantity));
                } else {
                    self.push(&mut key, OrderPart::Number(1))?;
                    let name = need!(operation_name(graph, bound(body, 1)));
                    self.push(&mut key, OrderPart::Word(Word::Borrowed(name)))?;
                    let member = need!(text_member(body, 2));
                    self.push(&mut key, OrderPart::Word(Word::Borrowed(member)))?;
                    let rank = need!(comparison_rank(need!(bound(body, 3).and_then(spelling))));
                    self.push(&mut key, OrderPart::Number(rank))?;
                    got!(self.extend(&mut key, bound(body, 4), Shape::Quantity));
                }
            }
            CheckedNodeKind::TimedModel(TimedModelForm::Reset) => {
                if bound(body, 0).and_then(spelling) == Some("place") {
                    self.push(&mut key, OrderPart::Number(0))?;
                    let place = *need!(graph.index.get(need!(typed_target(need!(bound(body, 2))))));
                    let path = need!(place_path_key(graph, place, self.meter)?);
                    append(self.meter, &mut key, path)?;
                    let name = need!(operation_name(graph, bound(body, 1)));
                    self.push(&mut key, OrderPart::Word(Word::Borrowed(name)))?;
                    got!(self.extend(&mut key, bound(body, 6), Shape::Term));
                    self.push(&mut key, OrderPart::Start)?;
                    for binder in need!(bound(body, 5).map(typed_members)) {
                        got!(self.extend(&mut key, Some(binder), Shape::Term));
                    }
                    self.push(&mut key, OrderPart::End)?;
                    got!(self.extend(&mut key, bound(body, 3), Shape::Term));
                    self.push(&mut key, OrderPart::Start)?;
                    for slot in need!(bound(body, 4).map(typed_members)) {
                        got!(self.extend(&mut key, Some(slot), Shape::Term));
                    }
                    self.push(&mut key, OrderPart::End)?;
                    got!(self.extend(&mut key, bound(body, 7), Shape::Quantity));
                } else {
                    self.push(&mut key, OrderPart::Number(1))?;
                    let name = need!(operation_name(graph, bound(body, 1)));
                    self.push(&mut key, OrderPart::Word(Word::Borrowed(name)))?;
                    let member = need!(text_member(body, 2));
                    self.push(&mut key, OrderPart::Word(Word::Borrowed(member)))?;
                    got!(self.extend(&mut key, bound(body, 3), Shape::Quantity));
                }
            }
            CheckedNodeKind::TimedModel(TimedModelForm::Guard) => {
                let name = need!(operation_name(graph, bound(body, 0)));
                self.push(&mut key, OrderPart::Word(Word::Borrowed(name)))?;
            }
            CheckedNodeKind::TimedModel(TimedModelForm::Invariant) => {
                self.push(&mut key, OrderPart::Start)?;
                let bounds_index = if is_quantified_binders(typed_members(body).get(1)) {
                    2
                } else {
                    1
                };
                for bound_term in need!(bound(body, bounds_index).map(typed_members)) {
                    let bound_position = *need!(graph.index.get(need!(typed_target(bound_term))));
                    let inner = need!(self.inner(bound_position)?);
                    append(self.meter, &mut key, inner)?;
                }
                self.push(&mut key, OrderPart::End)?;
                got!(self.extend(&mut key, bound(body, 0), Shape::Term));
            }
            CheckedNodeKind::TimedModel(TimedModelForm::Urgency) => {
                let first = need!(text_member(body, 0));
                self.push(&mut key, OrderPart::Word(Word::Borrowed(first)))?;
                let second = text_member(body, 2).unwrap_or_default();
                self.push(&mut key, OrderPart::Word(Word::Borrowed(second)))?;
            }
            _ => return Ok(None),
        }
        Ok(Some(key))
    }
}

fn structural_order_key<'g>(
    graph: &Graph<'g>,
    position: usize,
    active: &mut [bool],
    meter: &mut WorkMeter,
    at: &JsonPointer,
) -> Result<Option<Vec<OrderPart<'g>>>, ValidationFailure> {
    KeyWork {
        graph,
        active,
        meter,
        at,
    }
    .inner(position)
}

#[ix_trace_rs::string_edge]
fn comparison_rank(value: &str) -> Option<u64> {
    Some(match value {
        "lt" => 0,
        "le" => 1,
        "eq" => 2,
        "ge" => 3,
        "gt" => 4,
        _ => return None,
    })
}

/// A source-map entry may add an earlier region to an existing node. Keep
/// the minimum across every occurrence and region, independently of node id.
/// The result is one entry per node, in id order.
fn source_order_keys<'a>(
    source_map: &'a [CheckedSourceMapEntry],
    meter: &mut WorkMeter,
) -> Result<Vec<(&'a CheckedNodeId, SourceOrderKey<'a>)>, ValidationFailure> {
    let regions = source_map.iter().fold(0_usize, |sum, entry| {
        sum.saturating_add(entry.regions.len())
    });
    let mut keys: Vec<(&CheckedNodeId, SourceOrderKey<'_>)> = meter.vec(regions)?;
    for (at, entry) in source_map.iter().enumerate() {
        meter.at_row("source_map", at);
        for region in &entry.regions {
            meter.charge(1, || JsonPointer::root().key("source_map"))?;
            keys.push((
                &entry.node_id,
                (region.source.identity.as_ref(), region.start, region.end),
            ));
        }
    }
    keys.sort_unstable_by(|left, right| left.0.cmp(right.0).then(left.1.cmp(&right.1)));
    keys.dedup_by(|later, earlier| later.0 == earlier.0);
    Ok(keys)
}

fn check_reference_order<'g>(
    graph: &Graph<'g>,
    members: &'g [CheckedTerm],
    at: &JsonPointer,
    first_index: usize,
    keys: &[(&CheckedNodeId, SourceOrderKey<'_>)],
    active: &mut [bool],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut previous: Option<(SourceOrderKey<'_>, Vec<OrderPart<'g>>)> = None;
    for (index, member) in members.iter().enumerate() {
        let member_at = at.clone().key("members").index(index + first_index);
        meter.charge(1, || member_at.clone())?;
        let Some(target) = typed_target(member) else {
            continue;
        };
        let member_key = match graph.index.get(target) {
            Some(&target) => structural_order_key(graph, target, active, meter, &member_at)?,
            None => None,
        };
        let source = keys
            .binary_search_by(|(id, _)| (*id).cmp(target))
            .ok()
            .map(|at| keys[at].1);
        let key = source.zip(member_key);
        // Graph-shape and source-map checks have already validated every
        // reference and region. Their refusal has precedence over this step.
        let Some(key) = key else {
            continue;
        };
        meter.charge(u64::try_from(key.1.len()).unwrap_or(u64::MAX), || {
            member_at.clone()
        })?;
        if let Some(before) = previous.as_ref() {
            let order = match key.0.cmp(&before.0) {
                Ordering::Equal => compare_keys(&key.1, &before.1, meter)?,
                order => order,
            };
            if order != Ordering::Greater {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    member_at,
                ));
            }
        }
        previous = Some(key);
    }
    Ok(())
}

fn check_timed_reference_order(
    graph: &Graph<'_>,
    source_map: &[CheckedSourceMapEntry],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let keys = source_order_keys(source_map, meter)?;
    // The positions on the reference path the key walk is on.
    let mut active = meter.filled(graph.nodes.len(), false)?;
    for position in 0..graph.nodes.len() {
        let Some(body) = graph.terms.get(position).and_then(Option::as_ref) else {
            continue;
        };
        let bindings: &[usize] = match graph.kinds.get(position) {
            Some(CheckedNodeKind::TimedModel(TimedModelForm::Model)) => &[3, 4, 5, 6, 7, 8],
            Some(CheckedNodeKind::TimedModel(TimedModelForm::Invariant))
                if is_quantified_binders(typed_members(body).get(1)) =>
            {
                &[2]
            }
            Some(CheckedNodeKind::TimedModel(TimedModelForm::Invariant)) => &[1],
            _ => &[],
        };
        for &binding in bindings {
            check_reference_order(
                graph,
                bound(body, binding).map(typed_members).unwrap_or_default(),
                &binding_at(position, binding),
                0,
                &keys,
                &mut active,
                meter,
            )?;
        }
        if graph.kinds.get(position) != Some(&CheckedNodeKind::TimedModel(TimedModelForm::Guard)) {
            continue;
        }
        for (part_index, part) in bound(body, 1)
            .map(typed_members)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let at = binding_at(position, 1).key("members").index(part_index);
            let first_constraint = if is_quantified_binders(typed_members(part).get(1)) {
                2
            } else {
                1
            };
            if let Some(members) = typed_members(part).get(first_constraint..) {
                check_reference_order(
                    graph,
                    members,
                    &at,
                    first_constraint,
                    &keys,
                    &mut active,
                    meter,
                )?;
            }
        }
    }
    Ok(())
}

/// Resolve a clause's model subject, leaving trace subjects without a model join.
fn clause_model_subject<'a>(graph: &'a Graph<'_>, position: usize) -> Option<&'a CheckedNodeId> {
    let CheckedTerm::Application { arguments, .. } = graph.terms.get(position)?.as_ref()? else {
        return None;
    };
    let parameter = *graph.index.get(typed_target(arguments.first()?)?)?;
    if graph.kinds.get(parameter) != Some(&CheckedNodeKind::Value(ValueForm::Parameter)) {
        return None;
    }
    let type_id = &graph.nodes[parameter].semantic_type;
    let type_position = *graph.index.get(type_id)?;
    match graph.kinds.get(type_position)? {
        CheckedNodeKind::Model(_) => Some(type_id),
        CheckedNodeKind::CompositeType(CompositeTypeForm::Reference) => {
            let target = typed_members(graph.terms.get(type_position)?.as_ref()?).first()?;
            let model = typed_target(target)?;
            let model_position = *graph.index.get(model)?;
            matches!(
                graph.kinds.get(model_position),
                Some(CheckedNodeKind::Model(_))
            )
            .then_some(model)
        }
        _ => None,
    }
}

/// The four admitted identities of a selected model-time binding.
#[derive(Clone, Copy)]
pub(super) struct ModelTimeBinding<'a> {
    /// The checked timed-model root.
    pub timed_root: &'a CheckedNodeId,
    /// The clause's model subject.
    pub model: &'a CheckedNodeId,
    /// The checked time source.
    pub source: &'a CheckedNodeId,
    /// The source's exact unit.
    pub unit: &'a CheckedNodeId,
}

/// Find the admitted model-time source without allocating or inspecting an
/// uncharged timed root. A trace subject has no checked-model source.
pub(super) fn admitted_model_time<'a, E>(
    package: &'a super::CheckedPackageV2,
    clause: &CheckedNodeId,
    mut charge: impl FnMut(&CheckedNodeId) -> Result<(), E>,
) -> Result<Option<ModelTimeBinding<'a>>, E> {
    let Some(CheckedTerm::Application { arguments, .. }) = package.term(clause) else {
        return Ok(None);
    };
    let Some(parameter) = arguments.first().and_then(typed_target) else {
        return Ok(None);
    };
    let Some(parameter_node) = package.node(parameter) else {
        return Ok(None);
    };
    let type_id = &parameter_node.semantic_type;
    let model = match package.node_kind(type_id) {
        Some(CheckedNodeKind::Model(_)) => Some(type_id),
        Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Reference)) => package
            .term(type_id)
            .and_then(|term| typed_members(term).first())
            .and_then(typed_target)
            .filter(|target| matches!(package.node_kind(target), Some(CheckedNodeKind::Model(_)))),
        _ => None,
    };
    let Some(model) = model else {
        return Ok(None);
    };
    for &position in &package.timed_model_roots {
        let root = &package.resolved_graph().nodes[position];
        charge(&root.node_id)?;
        let Some(body) = package.term(&root.node_id) else {
            continue;
        };
        if bound(body, 0).and_then(typed_target) != Some(model) {
            continue;
        }
        let Some(source) = bound(body, 2).and_then(typed_target) else {
            continue;
        };
        charge(source)?;
        let Some(unit) = package
            .term(source)
            .and_then(|body| bound(body, 1))
            .and_then(typed_target)
        else {
            continue;
        };
        charge(unit)?;
        return Ok(Some(ModelTimeBinding {
            timed_root: &root.node_id,
            model,
            source,
            unit,
        }));
    }
    Ok(None)
}

/// A checked model-time binding borrowed from one admitted Timed clause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedModelTimeRead<'a> {
    clause: &'a CheckedNodeId,
    timed_root: &'a CheckedNodeId,
    model: &'a CheckedNodeId,
    source: &'a CheckedNodeId,
    unit: &'a CheckedNodeId,
    work: u64,
}

impl<'a> CheckedModelTimeRead<'a> {
    /// The Timed clause read by this call.
    pub fn clause(&self) -> &'a CheckedNodeId {
        self.clause
    }
    /// The checked timed-model root.
    pub fn timed_root(&self) -> &'a CheckedNodeId {
        self.timed_root
    }
    /// The clause's selected model subject.
    pub fn model(&self) -> &'a CheckedNodeId {
        self.model
    }
    /// The root's checked time source.
    pub fn source(&self) -> &'a CheckedNodeId {
        self.source
    }
    /// The source's exact admitted time unit.
    pub fn unit(&self) -> &'a CheckedNodeId {
        self.unit
    }
    /// Work charged by this read.
    pub fn work(&self) -> u64 {
        self.work
    }
}

/// Stable refusal of a bounded model-time read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedModelTimeReadErrorCode {
    /// The supplied node is not an admitted Timed clause.
    WrongClause,
    /// The next inspection exceeds the caller's finite allowance.
    WorkLimit,
}

/// No partial model-time binding is returned on a read refusal.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("model-time read refused: {code:?} at {at:?}")]
pub struct CheckedModelTimeReadError {
    /// Stable refusal code.
    pub code: CheckedModelTimeReadErrorCode,
    /// The first uncharged node, or the wrong clause.
    pub at: Box<CheckedNodeId>,
    /// Work consumed before refusal.
    pub work: u64,
    /// First refused charge, zero for a semantic refusal.
    pub attempted_work: u64,
}

impl super::CheckedPackageV2 {
    /// Read the checked source for one Timed model clause under a finite work limit.
    /// A Timed trace clause returns `None`: its timestamps come from profile input.
    pub fn model_time_for_clause(
        &self,
        clause: &CheckedNodeId,
        work_limit: u64,
    ) -> Result<Option<CheckedModelTimeRead<'_>>, CheckedModelTimeReadError> {
        let mut work = 0;
        let mut charge = |at: &CheckedNodeId| {
            if work >= work_limit {
                return Err(CheckedModelTimeReadError {
                    code: CheckedModelTimeReadErrorCode::WorkLimit,
                    at: Box::new(at.clone()),
                    work,
                    attempted_work: 1,
                });
            }
            work += 1;
            Ok(())
        };
        charge(clause)?;
        let timed = self.node_kind(clause)
            == Some(CheckedNodeKind::Temporal(TemporalForm::TemporalClause))
            && self.term(clause).is_some_and(|term| {
                matches!(term, CheckedTerm::Application { operation, .. }
                    if operation.law_definitions().ok().is_some_and(|laws|
                        matches!(laws, [law] if TemporalProfile::from_wire(&law.identity)
                            == Some(TemporalProfile::Timed))))
            });
        if !timed {
            return Err(CheckedModelTimeReadError {
                code: CheckedModelTimeReadErrorCode::WrongClause,
                at: Box::new(clause.clone()),
                work,
                attempted_work: 0,
            });
        }
        let selected = admitted_model_time(self, clause, &mut charge)?;
        Ok(selected.map(|binding| CheckedModelTimeRead {
            clause: &self.node(clause).expect("admitted Timed clause").node_id,
            timed_root: binding.timed_root,
            model: binding.model,
            source: binding.source,
            unit: binding.unit,
            work,
        }))
    }
}

/// Join the retained timed roots, members and local typed targets in structural ID order.
#[ix_trace_rs::string_edge]
pub(super) fn validate_models<'a>(
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a GraphIndex<'a>,
    source_map: &[CheckedSourceMapEntry],
    owners: &ModelOwners<'_>,
    models: &[DomainModel],
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
    // Index selected FCD packages once; looking one up for each timed root
    // must not rescan every selected document.
    // Of two models with one identity the later one is kept, as a map built
    // by insertion would hold; the slice address orders them.
    let mut identified = meter.vec::<(&str, &DomainModel)>(models.len())?;
    identified.extend(models.iter().map(|model| (model.identity.as_ref(), model)));
    let models_by_identity = SortedTable::from_vec(identified, |left, right| {
        std::ptr::from_ref(*right).cmp(&std::ptr::from_ref(*left))
    });
    let root_count = index
        .values()
        .filter(|&&position| kinds.get(position) == Some(&CheckedNodeKind::TimedModel(Form::Model)))
        .count();
    let mut roots: Vec<usize> = meter.vec(root_count)?;
    // Each root names itself and every member node of its bindings.
    let mut named = root_count;
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::TimedModel(Form::Model)) {
            continue;
        }
        for member in typed_members(graph.body(position)?).iter().skip(2) {
            if let CheckedTerm::Binding { value, .. } = member {
                named = named
                    .saturating_add(usize::from(typed_target(value).is_some()))
                    .saturating_add(typed_members(value).iter().filter_map(typed_target).count());
            }
        }
    }
    let mut represented = meter.vec::<&CheckedNodeId>(named)?;
    // The root a node belongs to, by position; `usize::MAX` for none.
    let mut root_of = meter.filled(nodes.len(), usize::MAX)?;
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::TimedModel(Form::Model)) {
            continue;
        }
        let body = graph.body(position)?;
        roots.push(position);
        represented.push(&nodes[position].node_id);
        root_of[position] = position;
        for (member_index, member) in typed_members(body).iter().enumerate().skip(2) {
            meter.charge(1, || binding_at(position, member_index))?;
            let CheckedTerm::Binding { value, .. } = member else {
                continue;
            };
            if let Some(key) = typed_target(value) {
                represented.push(key);
                if let Some(&child) = index.get(key) {
                    root_of[child] = position;
                }
            }
            for child in typed_members(value) {
                if let Some(key) = typed_target(child) {
                    represented.push(key);
                    if let Some(&child) = index.get(key) {
                        root_of[child] = position;
                    }
                }
            }
        }
    }
    represented.sort_unstable();
    represented.dedup();
    if !roots.is_empty() {
        check_timed_reference_order(&graph, source_map, meter)?;
    }
    for &position in index.values() {
        let Some(CheckedNodeKind::TimedModel(_)) = kinds.get(position).copied() else {
            continue;
        };
        if roots.is_empty() {
            return Err(graph.issue(
                position,
                Cause::MissingTimedModel,
                JsonPointer::root().key("semantic_graph").key("nodes"),
            ));
        }
        if represented
            .binary_search(&&nodes[position].node_id)
            .is_err()
        {
            return Err(graph.issue(
                position,
                Cause::TimedNodeReference,
                node_pointer(position).key("body"),
            ));
        }
    }
    for &position in index.values() {
        let Some(CheckedNodeKind::TimedModel(form)) = kinds.get(position).copied() else {
            continue;
        };
        meter.charge(1, || node_pointer(position))?;
        let body = graph.body(position)?;
        let root = match root_of[position] {
            usize::MAX => position,
            root => root,
        };
        let root_body = graph.body(root)?;
        let root_model = graph.target(
            root,
            bound(root_body, 0),
            binding_at(root, 0).key("target"),
            |kind| matches!(kind, CheckedNodeKind::Model(_)),
        )?;
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
                let is_base_time = is_base_time_dimension(&nodes[dimension], &models_by_identity);
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
                    let mut seen_places: Vec<(&CheckedNodeId, &CheckedTerm)> =
                        meter.vec(children.len())?;
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
            }
            Form::Reset | Form::Constraint => {
                let place = bound(body, 0).and_then(spelling) == Some("place");
                if place {
                    let place_index = if form == Form::Reset { 2 } else { 1 };
                    let (owner_index, slots_index) =
                        if form == Form::Reset { (3, 4) } else { (2, 3) };
                    let anchor = if form == Form::Reset {
                        Some(graph.anchor(
                            position,
                            bound(body, 1),
                            binding_at(position, 1).key("target"),
                            root_model,
                        )?)
                    } else {
                        None
                    };
                    let place_position = graph.place_terms(
                        position,
                        body,
                        PlaceBindings {
                            place: place_index,
                            owner: owner_index,
                            slots: slots_index,
                        },
                        root_model,
                        owners,
                        meter,
                    )?;
                    if let Some(anchor) = anchor {
                        if !graph.frame_writes_place(anchor, place_position, root_model) {
                            return Err(graph.issue(
                                position,
                                Cause::TimedNodeReference,
                                binding_at(position, place_index).key("target"),
                            ));
                        }
                        let slots = bound(body, 4).map(typed_members).unwrap_or_default();
                        let binders = bound(body, 5).map(typed_members).unwrap_or_default();
                        let mut previous_slot = None;
                        for (binder_index, binder) in binders.iter().enumerate() {
                            let at = binding_at(position, 5)
                                .key("members")
                                .index(binder_index)
                                .key("target");
                            let binder_target =
                                graph.target(position, Some(binder), at.clone(), |kind| {
                                    kind == CheckedNodeKind::Value(ValueForm::Parameter)
                                })?;
                            let Some(slot_index) = slots.iter().position(|slot| {
                                typed_target(slot) == Some(&nodes[binder_target].node_id)
                            }) else {
                                return Err(graph.issue(position, Cause::TimedNodeReference, at));
                            };
                            if previous_slot.is_some_and(|previous| slot_index <= previous) {
                                return Err(graph.issue(position, Cause::TimedNodeReference, at));
                            }
                            previous_slot = Some(slot_index);
                        }
                        graph.boolean(
                            position,
                            bound(body, 6),
                            binding_at(position, 6).key("target"),
                        )?;
                    }
                } else {
                    let anchor = graph.anchor(
                        position,
                        bound(body, 1),
                        binding_at(position, 1).key("target"),
                        root_model,
                    )?;
                    let parameter_name = bound(body, 2).and_then(spelling).unwrap_or_default();
                    let operation_name = graph
                        .body(anchor)
                        .ok()
                        .and_then(|body| bound(body, 1))
                        .and_then(spelling)
                        .unwrap_or_default();
                    let model_owner = owners.recover(&nodes[root_model]).map_err(|_| {
                        graph.issue(position, Cause::TimedNodeReference, binding_at(position, 2))
                    })?;
                    let mut budget =
                        Budget::new(meter, model_owner.package.root, model_owner.package.bytes);
                    let declared = model_owner.package.resolve(
                        model_owner.node,
                        MemberKind::Operation,
                        operation_name,
                        &mut budget,
                    );
                    let parameter = match declared {
                        Ok(Resolved::Operation(operation)) => operation
                            .parameters
                            .iter()
                            .find(|parameter| parameter.name.as_deref() == Some(parameter_name))
                            .cloned(),
                        Err(ModelFailure::Limit(failure)) => return Err(failure),
                        _ => None,
                    };
                    if !parameter.is_some_and(|parameter| {
                        matches!(
                            parameter.type_ref,
                            Some(SelectedTypeRef::Native(
                                super::model_members::NativeTypeRef::Clock
                            ))
                        )
                    }) {
                        return Err(graph.issue(
                            position,
                            Cause::TimedNodeReference,
                            binding_at(position, 2),
                        ));
                    }
                    if form == Form::Reset
                        && !frame_writes_clock(&graph, anchor, root_model, owners, meter)?
                    {
                        return Err(graph.issue(
                            position,
                            Cause::TimedNodeReference,
                            binding_at(position, 1).key("target"),
                        ));
                    }
                }
                let amount = match (form, place) {
                    (Form::Reset, true) => 7,
                    (Form::Reset, false) => 3,
                    (Form::Constraint, true) => 5,
                    (Form::Constraint, false) => 4,
                    _ => unreachable!(),
                };
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
            Form::Guard => {
                let anchor = graph.anchor(
                    position,
                    bound(body, 0),
                    binding_at(position, 0).key("target"),
                    root_model,
                )?;
                for (part_index, part) in bound(body, 1)
                    .map(typed_members)
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                {
                    let at = binding_at(position, 1).key("members").index(part_index);
                    graph.boolean(
                        position,
                        bound(part, 0),
                        at.clone()
                            .key("members")
                            .index(0)
                            .key("value")
                            .key("target"),
                    )?;
                    let first_constraint = if is_quantified_binders(typed_members(part).get(1)) {
                        2
                    } else {
                        1
                    };
                    let mut slots = Vec::new();

                    for (constraint_index, constraint) in typed_members(part)
                        .iter()
                        .enumerate()
                        .skip(first_constraint)
                    {
                        let at = at
                            .clone()
                            .key("members")
                            .index(constraint_index)
                            .key("target");
                        let target =
                            graph.target(position, Some(constraint), at.clone(), |kind| {
                                kind == CheckedNodeKind::TimedModel(Form::Constraint)
                            })?;
                        let constraint_body = graph.body(target)?;
                        extend_place_slots(&mut slots, constraint_body, meter)?;
                        if bound(constraint_body, 0).and_then(spelling) == Some("parameter")
                            && bound(constraint_body, 1).and_then(typed_target)
                                != Some(&nodes[anchor].node_id)
                        {
                            return Err(graph.issue(position, Cause::TimedNodeReference, at));
                        }
                    }
                    if first_constraint == 2 {
                        check_quantified_binders(
                            &graph,
                            position,
                            bound(part, 1),
                            at.key("members").index(1).key("value"),
                            &slots,
                            meter,
                        )?;
                    }
                }
            }
            Form::Invariant => {
                graph.boolean(
                    position,
                    bound(body, 0),
                    binding_at(position, 0).key("target"),
                )?;
                let bounds_index = if is_quantified_binders(typed_members(body).get(1)) {
                    2
                } else {
                    1
                };
                let mut slots = Vec::new();

                for (bound_index, constraint) in bound(body, bounds_index)
                    .map(typed_members)
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                {
                    let at = binding_at(position, bounds_index)
                        .key("members")
                        .index(bound_index)
                        .key("target");
                    let target = graph.target(position, Some(constraint), at.clone(), |kind| {
                        kind == CheckedNodeKind::TimedModel(Form::Constraint)
                    })?;
                    let constraint_body = graph.body(target)?;
                    extend_place_slots(&mut slots, constraint_body, meter)?;
                    if bound(constraint_body, 0).and_then(spelling) != Some("place")
                        || !matches!(
                            bound(constraint_body, 4).and_then(spelling),
                            Some("lt" | "le")
                        )
                    {
                        return Err(graph.issue(position, Cause::TimedNodeReference, at));
                    }
                }
                if bounds_index == 2 {
                    check_quantified_binders(
                        &graph,
                        position,
                        bound(body, 1),
                        binding_at(position, 1),
                        &slots,
                        meter,
                    )?;
                }
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
    for &root in &roots {
        let body = graph.body(root)?;
        let guards = bound(body, 5).map(typed_members).unwrap_or_default();
        let mut anchors: Vec<&CheckedNodeId> = meter.vec(guards.len())?;
        let mut used_constraints: Vec<&CheckedNodeId> = Vec::new();
        let listed = bound(body, 8).map(typed_members).unwrap_or_default();
        let mut listed_constraints: Vec<&CheckedNodeId> =
            meter.vec(listed.iter().filter_map(typed_target).count())?;
        listed_constraints.extend(listed.iter().filter_map(typed_target));
        listed_constraints.sort_unstable();
        listed_constraints.dedup();
        for guard in bound(body, 5).map(typed_members).unwrap_or_default() {
            let Some(&guard) = typed_target(guard).and_then(|id| index.get(id)) else {
                continue;
            };
            let guard_body = graph.body(guard)?;
            if let Some(anchor) = bound(guard_body, 0).and_then(typed_target) {
                if !meter.insert_sorted(&mut anchors, anchor)? {
                    return Err(graph.issue(
                        guard,
                        Cause::TimedNodeReference,
                        binding_at(guard, 0).key("target"),
                    ));
                }
            }
            for (part_index, part) in bound(guard_body, 1)
                .map(typed_members)
                .unwrap_or_default()
                .iter()
                .enumerate()
            {
                let first_constraint = if is_quantified_binders(typed_members(part).get(1)) {
                    2
                } else {
                    1
                };
                for (constraint_index, member) in typed_members(part)
                    .iter()
                    .enumerate()
                    .skip(first_constraint)
                {
                    if let Some(target) = typed_target(member) {
                        if listed_constraints.binary_search(&target).is_err() {
                            return Err(graph.issue(
                                guard,
                                Cause::TimedNodeReference,
                                binding_at(guard, 1)
                                    .key("members")
                                    .index(part_index)
                                    .key("members")
                                    .index(constraint_index)
                                    .key("target"),
                            ));
                        }
                        meter.insert_sorted(&mut used_constraints, target)?;
                    }
                }
            }
        }
        for invariant in bound(body, 6).map(typed_members).unwrap_or_default() {
            let Some(&invariant) = typed_target(invariant).and_then(|id| index.get(id)) else {
                continue;
            };
            let invariant_body = graph.body(invariant)?;
            let bounds_index = if is_quantified_binders(typed_members(invariant_body).get(1)) {
                2
            } else {
                1
            };
            for (bound_index, member) in bound(invariant_body, bounds_index)
                .map(typed_members)
                .unwrap_or_default()
                .iter()
                .enumerate()
            {
                if let Some(target) = typed_target(member) {
                    if listed_constraints.binary_search(&target).is_err() {
                        return Err(graph.issue(
                            invariant,
                            Cause::TimedNodeReference,
                            binding_at(invariant, bounds_index)
                                .key("members")
                                .index(bound_index)
                                .key("target"),
                        ));
                    }
                    meter.insert_sorted(&mut used_constraints, target)?;
                }
            }
        }
        for (position, constraint) in bound(body, 8)
            .map(typed_members)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            if typed_target(constraint)
                .is_some_and(|target| used_constraints.binary_search(&target).is_err())
            {
                return Err(graph.issue(
                    root,
                    Cause::TimedNodeReference,
                    binding_at(root, 8)
                        .key("members")
                        .index(position)
                        .key("target"),
                ));
            }
        }
    }
    for (position, node) in nodes.iter().enumerate() {
        if kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::TemporalClause))
            || super::temporal_binding::profile_law(node)
                .as_ref()
                .and_then(|law| TemporalProfile::from_wire(&law.identity))
                != Some(TemporalProfile::Timed)
        {
            continue;
        }
        let at = node_pointer(position).key("body");
        meter.charge(1, || at.clone())?;
        if super::temporal_binding::clock_name(node, 1) != Some("model-time") {
            return Err(graph.issue(position, Cause::TimedNodeReference, at));
        }
        let Some(subject) = clause_model_subject(&graph, position) else {
            continue;
        };
        let mut matched = false;
        for &root in &roots {
            meter.charge(1, || at.clone())?;
            let root_body = graph.body(root)?;
            if bound(root_body, 0).and_then(typed_target) == Some(subject)
                && bound(root_body, 2)
                    .and_then(typed_target)
                    .and_then(|source| index.get(source))
                    .is_some_and(|&source| {
                        kinds.get(source) == Some(&CheckedNodeKind::TimedModel(Form::Source))
                    })
            {
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(graph.issue(position, Cause::TimedNodeReference, at));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NativeJoinBudget;
    use ix_trace_rs::trace;
    use serde_json::json;
    use std::collections::BTreeSet;
    use std::path::Path;

    /// Trace: FR-038-AC-323, TC-367.
    #[trace("TC-367", "FR-038-AC-323")]
    #[test]
    fn selected_model_base_time_uses_fcd_identity_without_a_declaration() {
        let document = json!({
            "contractVersion":"2.0.0",
            "package":{"identity":"acme/rpc","version":"1.0.0"},
            "constructs":[], "types":[],
            "dimensions":[
                {"identity":"ix://acme/rpc/dimension/Time","name":"Time","base":true,"terms":[]},
                {"identity":"ix://acme/rpc/dimension/Length","name":"Length","base":true,"terms":[]}
            ]
        });
        let mut meter = WorkMeter::new(u64::MAX);
        let mut budget = Budget::new(&mut meter, 0, 1 << 20);
        let model = super::super::model_members::read_semantic_ir(
            &document,
            "example",
            &mut budget,
            &std::sync::Arc::new(BTreeMap::new()),
        )
        .expect("selected FCD dimensions");
        let models = BTreeMap::from([(model.identity.as_ref(), &model)]);
        let dimension = |owner: &str| {
            serde_json::from_value::<CheckedSemanticNodeV2>(json!({
                "node_id":{"package":"acme/rpc","path":["scalar_type","dimension",owner],"ordinal":0},
                "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"scalar_type", "semantic_form":"dimension",
                "semantic_type":{"package":"acme/rpc","path":["scalar_type","dimension",owner],"ordinal":0},
                "dependencies":[], "occurrences":[],
                "owner":{"kind":"model","identity":"acme/rpc","node":owner},
                "body":{"term":"aggregate","members":[]}
            }))
            .expect("model-owned dimension node")
        };
        let time = dimension("ix://acme/rpc/dimension/Time");
        assert!(is_base_time_dimension(&time, &models));
        assert!(!is_base_time_dimension(
            &dimension("ix://acme/rpc/dimension/Length"),
            &models
        ));
        let mut wrong_package = time.clone();
        wrong_package.owner = Some(CheckedNodeOwner::Model {
            identity: "acme/other".into(),
            node: "ix://acme/rpc/dimension/Time".into(),
        });
        assert!(!is_base_time_dimension(&wrong_package, &models));
    }

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
        let expected: BTreeSet<_> = [
            "SL-01", "SL-02", "SL-03", "SL-04", "SL-05", "SL-06", "SL-07", "SR-01", "SR-02",
            "SR-03", "SR-04",
        ]
        .into_iter()
        .collect();
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
                let selector_kind = match selector.get("target") {
                    Some(target) if target == &vectors["type_nodes"]["integer"] => {
                        Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Integer))
                    }
                    Some(target) if target == &vectors["type_nodes"]["text"] => {
                        Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Text))
                    }
                    Some(_) => Some(CheckedNodeKind::Expression(
                        super::super::ExpressionForm::Query,
                    )),
                    None => None,
                };
                let fits = selector_fits(parent, collection, member, selector_kind);
                assert_eq!(fits, case["expected"] == "admitted", "{id}: selector fit");
                if id == "SL-07" {
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
        assert_eq!(seen, expected, "published slot selector IDs");
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
            declared_field_type(
                &field,
                |reference| match reference {
                    SelectedTypeRef::Native(NativeTypeRef::Clock) => {
                        Some("ix://quire/native/Clock".to_owned())
                    }
                    _ => None,
                },
                &mut WorkMeter::new(u64::MAX)
            )
            .unwrap()
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
            declared_field_type(
                &field,
                |reference| match reference {
                    SelectedTypeRef::Native(NativeTypeRef::Clock) => {
                        Some("ix://quire/native/Clock".to_owned())
                    }
                    _ => None,
                },
                &mut WorkMeter::new(u64::MAX)
            )
            .unwrap()
            .as_deref(),
            Some("Sequence<ix://quire/native/Clock>")
        );
    }

    fn node(name: &str) -> CheckedNodeId {
        CheckedNodeId {
            package: "p".into(),
            path: vec![crate::checked_package::shared::CheckedNodePathSegment::Name(name.into())],
            ordinal: 0,
        }
    }

    /// The structural key of a reference to a body holding a text and a
    /// numeric literal, read under `limit` bytes of read storage.
    fn key_under(limit: u64) -> (Option<Vec<String>>, Option<u128>, u64) {
        let (a, b) = (node("a"), node("b"));
        let nodes = [a.clone(), b.clone()];
        let index: GraphIndex<'_> = [(&nodes[0], 0), (&nodes[1], 1)].into_iter().collect();
        let kinds = [
            CheckedNodeKind::TimedModel(TimedModelForm::Constraint),
            CheckedNodeKind::TimedModel(TimedModelForm::Constraint),
        ];
        let terms = [
            Some(CheckedTerm::Reference { target: b.clone() }),
            Some(CheckedTerm::Aggregate {
                members: vec![
                    CheckedTerm::Binding {
                        name: "label".into(),
                        value: Box::new(CheckedTerm::Literal {
                            type_id: a.clone(),
                            value_kind: LiteralKind::Text,
                            value: json!("on"),
                        }),
                    },
                    CheckedTerm::Literal {
                        type_id: a.clone(),
                        value_kind: LiteralKind::Text,
                        value: json!(7),
                    },
                ],
            }),
        ];
        let graph = Graph {
            nodes: &[],
            kinds: &kinds,
            terms: &terms,
            index: &index,
        };
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
        let mut meter = WorkMeter::over(&mut budget);
        let mut active = vec![false; 2];
        let at = JsonPointer::root();
        let key = term_key(&graph, terms[0].as_ref(), &mut active, &mut meter, &at);
        drop(meter);
        let words = key.ok().flatten().map(|key| {
            key.iter()
                .map(|part| match part {
                    OrderPart::Word(word) => word.as_str().to_owned(),
                    other => format!("{other:?}"),
                })
                .collect()
        });
        (
            words,
            budget.first_unpaid_read_storage(),
            budget.read_storage_consumed(),
        )
    }

    /// A structural term key is built in blocks charged before they are
    /// requested: it is admitted at exactly the storage it consumed and
    /// denied one byte short (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn a_structural_term_key_charges_every_block_before_it_allocates() {
        let (words, unpaid, spent) = key_under(u64::MAX);
        let words = words.expect("key");
        assert!(unpaid.is_none());
        assert!(spent > 0, "the key's vectors and words are charged");
        assert!(words.iter().any(|word| word.contains("Constraint")));
        assert!(words.iter().any(|word| word == "label" || word == "on"));
        assert!(words.iter().any(|word| word == "7"));
        let (again, unpaid, _) = key_under(spent);
        assert_eq!(again.as_ref(), Some(&words));
        assert!(unpaid.is_none());
        let (short, unpaid, _) = key_under(spent - 1);
        assert!(short.is_none());
        assert!(unpaid.is_some(), "the short run was denied a request");
    }

    fn ratio(meter: &mut WorkMeter, numerator: &str, denominator: &str) -> OrderPart<'static> {
        OrderPart::Ratio(
            meter.exact_integer(numerator).unwrap().unwrap(),
            meter.exact_integer(denominator).unwrap().unwrap(),
        )
    }

    /// Parses two ratios of `wide` digits and, when asked, compares them,
    /// under `limit` bytes of read storage; the storage consumed with it.
    fn compare_under(limit: u64, compare: bool) -> (Result<Ordering, ValidationFailure>, u64) {
        let wide = "9".repeat(80);
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
        let mut meter = WorkMeter::over(&mut budget);
        let outcome = (|| {
            let big = ratio(&mut meter, &wide, "1");
            let bigger = ratio(&mut meter, &wide, "1");
            if compare {
                big.compare(&bigger, &mut meter)
            } else {
                Ok(Ordering::Equal)
            }
        })();
        drop(meter);
        (outcome, budget.read_storage_consumed())
    }

    /// Two ratios compare by cross multiplication through the quire-exact
    /// reserved entries: equal fractions are equal, and the products' limb
    /// blocks are charged to the budget (FR-038-AC-504, FR-038-AC-516).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn ratio_parts_compare_exactly_and_charge_the_products() {
        let mut budget = NativeJoinBudget::new(u64::MAX, 0);
        let mut meter = WorkMeter::over(&mut budget);
        let third = ratio(&mut meter, "1", "3");
        let two_sixths = ratio(&mut meter, "2", "6");
        let half = ratio(&mut meter, "1", "2");
        assert_eq!(
            third.compare(&two_sixths, &mut meter).unwrap(),
            Ordering::Equal
        );
        assert_eq!(third.compare(&half, &mut meter).unwrap(), Ordering::Less);
        assert_eq!(half.compare(&third, &mut meter).unwrap(), Ordering::Greater);

        let (parsed, parse_only) = compare_under(u64::MAX, false);
        assert!(parsed.is_ok());
        let (compared, spent) = compare_under(u64::MAX, true);
        assert_eq!(compared.unwrap(), Ordering::Equal);
        assert!(spent > parse_only, "the products are charged");
        assert!(compare_under(spent, true).0.is_ok());
        let (denied, _) = compare_under(spent - 1, true);
        assert!(matches!(denied, Err(ValidationFailure::Budget(_))));
    }

    /// The wrappers of a declared field's type are written into blocks
    /// charged first: denied one byte short, admitted at exactly what the
    /// run consumed (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn declared_field_type_wrappers_are_charged_before_they_are_written() {
        use super::super::model_members::{Multiplicity, NativeTypeRef, TypedSlot};
        let field = FieldDecl {
            identity: "ix://acme/orders/Order/v".into(),
            slot: TypedSlot {
                name: Some("v".into()),
                type_ref: Some(SelectedTypeRef::Native(NativeTypeRef::Clock)),
                multiplicity: Multiplicity {
                    lower: 0,
                    upper: Some(3),
                    ordered: true,
                    unique: false,
                },
            },
            optional: true,
            redefines: None,
        };
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut meter = WorkMeter::over(&mut budget);
            let rendered = declared_field_type(
                &field,
                |_| Some("ix://quire/native/Clock".to_owned()),
                &mut meter,
            );
            drop(meter);
            (rendered, budget.read_storage_consumed())
        };
        let (rendered, spent) = run(u64::MAX);
        assert_eq!(
            rendered.unwrap().as_deref(),
            Some("Sequence<Option<ix://quire/native/Clock>>")
        );
        assert!(spent > 0);
        assert!(run(spent).0.is_ok());
        assert!(matches!(
            run(spent - 1).0,
            Err(ValidationFailure::Budget(_))
        ));
    }

    /// The comparisons that replaced rendering a string to compare with
    /// agree with the rendering they replaced.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn unrendered_comparisons_agree_with_the_rendering_they_replace() {
        for number in [0_u64, 7, 10, 99, 100, 12345, u64::MAX] {
            assert!(decimal_is(&number.to_string(), number));
            assert!(!decimal_is(&(number / 2 + 1).to_string(), number) || number / 2 + 1 == number);
        }
        assert!(!decimal_is("", 0) && !decimal_is("00", 0) && !decimal_is("-1", 1));
        assert!(concat_is("a/b/c", &["a", "/", "b/c"]) && concat_is("", &[]));
        assert!(!concat_is("a/b", &["a", "/"]) && !concat_is("a", &["a", "b"]));
        let integer = |value| CheckedTerm::Literal {
            type_id: node("t"),
            value_kind: LiteralKind::Integer,
            value,
        };
        assert!(scalar_matches(Some(&integer(json!(12))), Some("12")));
        assert!(!scalar_matches(Some(&integer(json!(12))), Some("012")));
        assert!(!scalar_matches(Some(&integer(json!(12))), None));
        assert!(scalar_matches(Some(&integer(json!("x"))), None));
        assert!(scalar_matches(None, None) && !scalar_matches(None, Some("")));
    }
}
