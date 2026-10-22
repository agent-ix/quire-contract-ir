//! FR-348 extension form body and declaration joins.

use super::identity::is_identifier;
use super::model_members::{Budget, DeclarationForm, ModelOwners};
use super::{
    CheckedNodeKind, CheckedPackageWireV2, CheckedSemanticNodeV2, CheckedTerm, ScalarTypeForm,
    WorkMeter,
};
use crate::checked_package::common::{is_digest, node_pointer, ValidationFailure, NODE_DOMAIN};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedExtensionRefusalDetail, CheckedNodeId,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, JsonPointer,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExtensionBody {
    term: Box<str>,
    definition: CheckedArtifactRef,
    node_kind: Box<str>,
    clauses: Vec<ExtensionClause>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtensionClause {
    term: Box<str>,
    name: Box<str>,
    value: ExtensionReference,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtensionReference {
    term: Box<str>,
    target: CheckedNodeId,
}

/// The body has its own closed grammar, checked before key authentication.
pub(super) fn read_body(body: &Value, at: JsonPointer) -> Result<ExtensionBody, ValidationFailure> {
    let parsed: ExtensionBody = serde_json::from_value(body.clone())
        .map_err(|_| ValidationFailure::refused(Code::MalformedWire, at.clone()))?;
    if parsed.term.as_ref() != "extension_form"
        || parsed.definition.authority.is_empty()
        || parsed.definition.identity.is_empty()
        || !is_identifier(&parsed.node_kind)
        || parsed.clauses.iter().any(|clause| {
            clause.term.as_ref() != "binding"
                || !is_identifier(&clause.name)
                || clause.value.term.as_ref() != "reference"
                || clause.value.target.domain.as_ref() != NODE_DOMAIN
                || !is_digest(&clause.value.target.digest)
        })
    {
        return Err(ValidationFailure::refused(Code::MalformedWire, at));
    }
    Ok(parsed)
}

pub(super) fn targets(body: &ExtensionBody) -> impl Iterator<Item = &CheckedNodeId> {
    body.clauses.iter().map(|clause| &clause.value.target)
}

pub(super) fn body_work(body: &ExtensionBody) -> u64 {
    u64::try_from(body.clauses.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(2)
        .saturating_add(1)
}

pub(super) fn validate_source_regions(
    wire: &CheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
) -> Result<(), ValidationFailure> {
    let extensions = wire
        .semantic_graph
        .nodes
        .iter()
        .zip(kinds)
        .filter(|(_, kind)| matches!(kind, CheckedNodeKind::Extension(_)))
        .map(|(node, _)| &node.node_id)
        .collect::<BTreeSet<_>>();
    for (entry_index, entry) in wire.source_map.iter().enumerate() {
        if extensions.contains(&entry.node_id) && entry.regions.len() != 1 {
            return Err(ValidationFailure::refused_at(
                Code::InvalidSourceMap,
                JsonPointer::root()
                    .key("source_map")
                    .index(entry_index)
                    .key("regions"),
                None,
                entry.node_id.clone(),
            ));
        }
    }
    Ok(())
}

fn refusal(
    node: &CheckedSemanticNodeV2,
    pointer: JsonPointer,
    code: Code,
    cause: Option<Cause>,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, pointer, cause, node.node_id.clone())
}

fn with_detail(
    mut failure: ValidationFailure,
    member: &str,
    expected: Option<&str>,
    actual: Option<String>,
    actual_semantic_type: Option<CheckedNodeId>,
) -> ValidationFailure {
    if let ValidationFailure::Refused(ref mut refusal) = failure {
        refusal.extension_detail = Some(Box::new(CheckedExtensionRefusalDetail {
            member: member.into(),
            expected_type: expected.map(Into::into),
            actual_type: actual.map(Into::into),
            actual_semantic_type,
        }));
    }
    failure
}

/// Joins selected and held definitions, node kinds, clauses and clause types.
pub(super) fn validate(
    wire: &CheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    evidence: &CheckedPackageEvidence,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let nodes = &wire.semantic_graph.nodes;
    for &position in index.values() {
        if !matches!(kinds.get(position), Some(CheckedNodeKind::Extension(_))) {
            continue;
        }
        let node = &nodes[position];
        let at = node_pointer(position).key("body");
        let Some(CheckedTerm::ExtensionForm {
            definition,
            node_kind,
            clauses,
        }) = terms.get(position).and_then(Option::as_ref)
        else {
            return Err(ValidationFailure::refused(Code::MalformedWire, at));
        };
        let clauses = clauses
            .iter()
            .map(|clause| match clause {
                CheckedTerm::Binding { name, value } => match value.as_ref() {
                    CheckedTerm::Reference { target } => Some((name.as_ref(), target)),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| ValidationFailure::refused(Code::MalformedWire, at.clone()))?;
        let def_at = at.clone().key("definition");
        let fail = |code, cause, pointer| refusal(node, pointer, code, cause);
        if !wire.lock.definition_selections.contains(definition) {
            return Err(fail(
                Code::MissingDeclaration,
                Some(Cause::MissingSelection),
                def_at,
            ));
        }
        let Some(definition) = evidence.extension_definition(definition) else {
            return Err(ValidationFailure::unknown_extension_definition(
                def_at,
                node.node_id.clone(),
            ));
        };
        let Some(schema) = definition
            .typed_node_schema
            .nodes
            .iter()
            .find(|entry| entry.node_kind.as_ref() == node_kind.as_ref())
        else {
            return Err(fail(
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                at.key("node_kind"),
            ));
        };
        for (ordinal, ((name, _), expected)) in clauses.iter().zip(&schema.clauses).enumerate() {
            if *name != expected.name.as_ref() {
                return Err(fail(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    at.clone().key("clauses").index(ordinal),
                ));
            }
        }
        if clauses.len() > schema.clauses.len() {
            return Err(fail(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                at.clone().key("clauses").index(schema.clauses.len()),
            ));
        }
        if clauses.len() < schema.clauses.len() {
            return Err(with_detail(
                fail(
                    Code::InvalidPackage,
                    Some(Cause::MissingMember),
                    at.clone().key("clauses"),
                ),
                &schema.clauses[clauses.len()].name,
                None,
                None,
                None,
            ));
        }
        for (ordinal, ((name, target_id), expected)) in
            clauses.iter().zip(&schema.clauses).enumerate()
        {
            let pointer = at.clone().key("clauses").index(ordinal).key("value");
            let Some(&target_position) = index.get(target_id) else {
                return Err(fail(
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    pointer,
                ));
            };
            let target = &nodes[target_position];
            if !type_matches(
                &expected.type_ref,
                nodes,
                kinds,
                terms,
                target_position,
                index,
                owners,
                meter,
            )? {
                return Err(with_detail(
                    fail(Code::IllTyped, Some(Cause::TypeMismatch), pointer),
                    name,
                    Some(&expected.type_ref),
                    Some(actual_type(
                        target,
                        kinds[target_position],
                        nodes,
                        terms,
                        target_position,
                        index,
                    )),
                    Some(target.semantic_type.clone()),
                ));
            }
        }
    }
    Ok(())
}

fn integer_range_bounds(term: &CheckedTerm) -> Option<(&str, &str)> {
    fn bound<'a>(term: &'a CheckedTerm, expected: &str) -> Option<&'a str> {
        let CheckedTerm::Binding { name, value } = term else {
            return None;
        };
        if name.as_ref() != expected {
            return None;
        }
        let CheckedTerm::Literal { value, .. } = value.as_ref() else {
            return None;
        };
        value.as_str()
    }
    let CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    let [lo, hi] = members.as_slice() else {
        return None;
    };
    Some((bound(lo, "min")?, bound(hi, "max")?))
}

fn type_matches(
    expected: &str,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    target_position: usize,
    index: &BTreeMap<&CheckedNodeId, usize>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let Some((target, &kind)) = nodes.get(target_position).zip(kinds.get(target_position)) else {
        return Ok(false);
    };
    if !matches!(
        kind,
        CheckedNodeKind::Value(_) | CheckedNodeKind::Expression(_) | CheckedNodeKind::Extension(_)
    ) {
        return Ok(false);
    }
    if let Some(bound) = expected
        .strip_prefix("Int[")
        .and_then(|tail| tail.strip_suffix(']'))
    {
        let Some((lo, hi)) = bound.split_once(',') else {
            return Ok(false);
        };
        let Some(&position) = index.get(&target.semantic_type) else {
            return Ok(false);
        };
        let type_node = &nodes[position];
        let bounds_match = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(integer_range_bounds)
            .is_some_and(|(lower, upper)| lower == lo.trim() && upper == hi.trim());
        return Ok(matches!(type_node.node_tag.as_ref(), "bounded_domain")
            && type_node.semantic_form.as_ref() == "integer_range"
            && bounds_match);
    }
    let scalar = match expected {
        "Boolean" => Some(ScalarTypeForm::Boolean),
        "Integer" | "Int" => Some(ScalarTypeForm::Integer),
        "Rational" => Some(ScalarTypeForm::Rational),
        "Decimal" => Some(ScalarTypeForm::Decimal),
        "Float32" => Some(ScalarTypeForm::Float32),
        "Float64" => Some(ScalarTypeForm::Float64),
        "Text" => Some(ScalarTypeForm::Text),
        "String" => Some(ScalarTypeForm::String),
        "UUID" => Some(ScalarTypeForm::Uuid),
        "Timestamp" => Some(ScalarTypeForm::Timestamp),
        _ => None,
    };
    if let Some(form) = scalar {
        return Ok(index
            .get(&target.semantic_type)
            .and_then(|&position| nodes.get(position))
            .is_some_and(|type_node| {
                type_node.node_tag.as_ref() == "scalar_type"
                    && type_node.semantic_form.as_ref() == form.as_wire()
            }));
    }
    if let Some(type_node) = index
        .get(&target.semantic_type)
        .and_then(|&position| nodes.get(position))
    {
        if type_node.node_tag.as_ref() == "model"
            && type_node.semantic_form.as_ref() == "record_value_type"
        {
            if let Ok(actual) = owners.recover(type_node) {
                if actual.form == DeclarationForm::RecordValueType
                    && actual.package.record_types.contains_key(expected)
                {
                    let mut budget = Budget::new(meter, actual.selection, actual.package.bytes);
                    return actual
                        .package
                        .conforms_to(actual.node, expected, &mut budget);
                }
            }
        }
    }
    Ok(matches!(kind, CheckedNodeKind::Extension(_))
        && matches!(terms.get(target_position).and_then(Option::as_ref),
            Some(CheckedTerm::ExtensionForm { node_kind, .. }) if node_kind.as_ref() == expected))
}

fn actual_type(
    target: &CheckedSemanticNodeV2,
    kind: CheckedNodeKind,
    nodes: &[CheckedSemanticNodeV2],
    terms: &[Option<CheckedTerm>],
    target_position: usize,
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> String {
    if matches!(kind, CheckedNodeKind::Extension(_)) {
        return match terms.get(target_position).and_then(Option::as_ref) {
            Some(CheckedTerm::ExtensionForm { node_kind, .. }) => node_kind.to_string(),
            _ => "extension".into(),
        };
    }
    let Some(&type_position) = index.get(&target.semantic_type) else {
        return target.semantic_type.digest.to_string();
    };
    let Some(type_node) = nodes.get(type_position) else {
        return target.semantic_type.digest.to_string();
    };
    if type_node.node_tag.as_ref() == "bounded_domain"
        && type_node.semantic_form.as_ref() == "integer_range"
    {
        if let Some((lo, hi)) = terms
            .get(type_position)
            .and_then(Option::as_ref)
            .and_then(integer_range_bounds)
        {
            return format!("Int[{lo}, {hi}]");
        }
    }
    match type_node.semantic_form.as_ref() {
        "boolean" => "Boolean".into(),
        "integer" => "Integer".into(),
        "rational" => "Rational".into(),
        "decimal" => "Decimal".into(),
        "float32" => "Float32".into(),
        "float64" => "Float64".into(),
        "text" => "Text".into(),
        "string" => "String".into(),
        "uuid" => "UUID".into(),
        "timestamp" => "Timestamp".into(),
        _ => type_node
            .declaration
            .as_ref()
            .map(|declaration| declaration.qualified_name.join("."))
            .unwrap_or_else(|| type_node.semantic_form.to_string()),
    }
}
