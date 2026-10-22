//! FR-348 extension form body and declaration joins.

use super::identity::is_identifier;
use super::model_members::{Budget, DeclarationForm, ModelOwners};
use super::{
    CheckedNodeKind, CheckedPackageWireV2, CheckedSemanticNodeV2, ScalarTypeForm, WorkMeter,
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
        let body = read_body(&node.body, at.clone())?;
        let def_at = at.clone().key("definition");
        let fail = |code, cause, pointer| refusal(node, pointer, code, cause);
        if !wire.lock.definition_selections.contains(&body.definition) {
            return Err(fail(
                Code::MissingDeclaration,
                Some(Cause::MissingSelection),
                def_at,
            ));
        }
        let Some(definition) = evidence.extension_definition(&body.definition) else {
            return Err(ValidationFailure::unknown_extension_definition(
                def_at,
                node.node_id.clone(),
            ));
        };
        let Some(schema) = definition
            .typed_node_schema
            .nodes
            .iter()
            .find(|entry| entry.node_kind == body.node_kind)
        else {
            return Err(fail(
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                at.key("node_kind"),
            ));
        };
        for (ordinal, (clause, expected)) in body.clauses.iter().zip(&schema.clauses).enumerate() {
            if clause.name != expected.name {
                return Err(fail(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    at.clone().key("clauses").index(ordinal),
                ));
            }
        }
        if body.clauses.len() > schema.clauses.len() {
            return Err(fail(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                at.clone().key("clauses").index(schema.clauses.len()),
            ));
        }
        if body.clauses.len() < schema.clauses.len() {
            return Err(with_detail(
                fail(
                    Code::InvalidPackage,
                    Some(Cause::MissingMember),
                    at.clone().key("clauses"),
                ),
                &schema.clauses[body.clauses.len()].name,
                None,
                None,
                None,
            ));
        }
        for (ordinal, (clause, expected)) in body.clauses.iter().zip(&schema.clauses).enumerate() {
            let pointer = at.clone().key("clauses").index(ordinal).key("value");
            let Some(&target_position) = index.get(&clause.value.target) else {
                return Err(fail(
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    pointer,
                ));
            };
            let target = &nodes[target_position];
            if !type_matches(
                &expected.type_ref,
                target,
                kinds[target_position],
                nodes,
                index,
                owners,
                meter,
            )? {
                return Err(with_detail(
                    fail(Code::IllTyped, Some(Cause::TypeMismatch), pointer),
                    &clause.name,
                    Some(&expected.type_ref),
                    Some(actual_type(target, kinds[target_position], nodes, index)),
                    Some(target.semantic_type.clone()),
                ));
            }
        }
    }
    Ok(())
}

fn type_matches(
    expected: &str,
    target: &CheckedSemanticNodeV2,
    kind: CheckedNodeKind,
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
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
        let members = type_node.body.get("members").and_then(Value::as_array);
        let bounds_match = members.is_some_and(|members| {
            members.len() == 2
                && members[0]["name"] == "min"
                && members[0]["value"]["value"].as_str() == Some(lo.trim())
                && members[1]["name"] == "max"
                && members[1]["value"]["value"].as_str() == Some(hi.trim())
        });
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
        && target.body.get("node_kind").and_then(Value::as_str) == Some(expected))
}

fn actual_type(
    target: &CheckedSemanticNodeV2,
    kind: CheckedNodeKind,
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> String {
    if matches!(kind, CheckedNodeKind::Extension(_)) {
        return target
            .body
            .get("node_kind")
            .and_then(Value::as_str)
            .unwrap_or("extension")
            .to_owned();
    }
    let Some(type_node) = index
        .get(&target.semantic_type)
        .and_then(|&position| nodes.get(position))
    else {
        return target.semantic_type.digest.to_string();
    };
    if type_node.node_tag.as_ref() == "bounded_domain"
        && type_node.semantic_form.as_ref() == "integer_range"
    {
        if let Some(members) = type_node.body.get("members").and_then(Value::as_array) {
            if let [lo, hi] = members.as_slice() {
                if let (Some(lo), Some(hi)) =
                    (lo["value"]["value"].as_str(), hi["value"]["value"].as_str())
                {
                    return format!("Int[{lo}, {hi}]");
                }
            }
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
