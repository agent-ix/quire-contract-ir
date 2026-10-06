//! Closed node-owner wire and selected-evidence joins for FR-038-AC-153..155.

use super::{
    CheckedDeclaration, CheckedNodeKind, CheckedNodeOwner, CheckedPackageWireV2,
    CheckedSemanticNodeV2, DomainModel, WorkMeter,
};
use crate::checked_package::common::{first_difference, node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OwnerRequirement {
    Source,
    Model,
    None,
}

fn requirement(
    node_tag: &str,
    semantic_form: &str,
    declaration: Option<&CheckedDeclaration>,
    nominal: bool,
    body: &Value,
) -> OwnerRequirement {
    if nominal || body.get("term").and_then(Value::as_str) == Some("application") {
        return OwnerRequirement::None;
    }
    if declaration.is_some() {
        return OwnerRequirement::Source;
    }
    match (node_tag, semantic_form) {
        ("model", "object_type" | "systems_interface")
        | ("relation", "relationship")
        | ("function", _) => OwnerRequirement::Model,
        _ => OwnerRequirement::None,
    }
}

fn check_one(
    requirement: OwnerRequirement,
    owner: Option<&CheckedNodeOwner>,
    at: JsonPointer,
) -> Result<(), ValidationFailure> {
    let valid = match (requirement, owner) {
        (OwnerRequirement::None, None) => true,
        (
            OwnerRequirement::Source,
            Some(CheckedNodeOwner::Source {
                authority,
                identity,
            }),
        ) => !authority.is_empty() && !identity.is_empty(),
        (OwnerRequirement::Model, Some(CheckedNodeOwner::Model { identity, node })) => {
            !identity.is_empty() && !node.is_empty()
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        let at = if owner.is_some() { at.key("owner") } else { at };
        Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::MalformedWire,
            at,
        ))
    }
}

/// Checks owner presence and variant before package identity validation.
pub(super) fn validate_owner_schema(wire: &CheckedPackageWireV2) -> Result<(), ValidationFailure> {
    for (position, node) in wire.semantic_graph.nodes.iter().enumerate() {
        check_one(
            requirement(
                &node.node_tag,
                &node.semantic_form,
                node.declaration.as_ref(),
                node.nominal_identity_preimage.is_some(),
                &node.body,
            ),
            node.owner.as_ref(),
            node_pointer(position),
        )?;
    }
    for (position, projection) in wire
        .identity_preimage
        .identity_projection
        .iter()
        .enumerate()
    {
        let at = JsonPointer::root()
            .key("identity_preimage")
            .key("identity_projection")
            .index(position);
        check_one(
            requirement(
                &projection.node_tag,
                &projection.semantic_form,
                projection.declaration.as_ref(),
                projection.nominal_identity_preimage.is_some(),
                &projection.body,
            ),
            projection.owner.as_ref(),
            at.clone(),
        )?;
        if let Some(node) = wire.semantic_graph.nodes.get(position) {
            if projection.owner != node.owner {
                let owner_at = at.key("owner");
                let path = match (
                    serde_json::to_value(&projection.owner),
                    serde_json::to_value(&node.owner),
                ) {
                    (Ok(projection), Ok(node)) => first_difference(owner_at, &projection, &node),
                    _ => owner_at,
                };
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::StaleDependency,
                    path,
                ));
            }
        }
    }
    Ok(())
}

fn clause_kind(body: &Value) -> Option<&str> {
    if body.get("term")?.as_str()? != "aggregate" {
        return None;
    }
    let mut clauses = body.get("members")?.as_array()?.iter().filter(|member| {
        member.get("term").and_then(Value::as_str) == Some("binding")
            && member.get("name").and_then(Value::as_str) == Some("clause")
    });
    let clause = clauses.next()?;
    if clauses.next().is_some() {
        return None;
    }
    let value = clause.get("value")?;
    if value.get("term")?.as_str()? != "literal" || value.get("value_kind")?.as_str()? != "text" {
        return None;
    }
    value.get("value")?.as_str()
}

fn model_kind_matches(
    kind: CheckedNodeKind,
    node: &CheckedSemanticNodeV2,
    model: &DomainModel,
    declared: &str,
    operations: &BTreeSet<&str>,
) -> bool {
    match kind {
        CheckedNodeKind::Model(super::ModelForm::ObjectType) => model
            .object_types
            .get(declared)
            .is_some_and(|declared| !declared.interface),
        CheckedNodeKind::Model(super::ModelForm::SystemsInterface) => model
            .object_types
            .get(declared)
            .is_some_and(|declared| declared.interface),
        CheckedNodeKind::Relation(super::RelationForm::Relationship) => {
            model.relationships.contains_key(declared)
        }
        CheckedNodeKind::Function(_) => match clause_kind(&node.body) {
            Some("invariant") => model
                .object_types
                .get(declared)
                .is_some_and(|object| !object.interface),
            Some("precondition" | "body") => operations.contains(declared),
            _ => false,
        },
        _ => false,
    }
}

fn missing_owner(node: &CheckedSemanticNodeV2, at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused_at(
        CheckedPackageRefusalCode::MissingDeclaration,
        at,
        Some(CheckedPackageRefusalCause::MissingSelection),
        node.node_id.clone(),
    )
}

/// Joins every owner, including unreachable nodes, before structural key checks.
pub(super) fn validate_owner_joins(
    wire: &CheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    models: &[DomainModel],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let selected_sources: BTreeSet<_> = wire
        .lock
        .sources
        .iter()
        .map(|source| (source.authority.as_ref(), source.identity.as_ref()))
        .collect();
    let selected_models: BTreeMap<_, _> = models
        .iter()
        .map(|model| (model.identity.as_ref(), model))
        .collect();
    let model_operations: BTreeMap<_, BTreeSet<_>> = models
        .iter()
        .map(|model| {
            (
                model.identity.as_ref(),
                model
                    .object_types
                    .values()
                    .flat_map(|object| object.operations.iter())
                    .map(|operation| operation.identity.as_ref())
                    .collect(),
            )
        })
        .collect();
    let mut source_entries: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for (entry_index, entry) in wire.source_map.iter().enumerate() {
        if entry.role == CheckedOccurrenceRole::Declaration {
            source_entries
                .entry(&entry.node_id)
                .or_default()
                .push((entry_index, entry));
        }
    }
    let mut positions: Vec<_> = (0..wire.semantic_graph.nodes.len()).collect();
    positions.sort_unstable_by(|&left, &right| {
        wire.semantic_graph.nodes[left]
            .node_id
            .digest
            .cmp(&wire.semantic_graph.nodes[right].node_id.digest)
    });
    for position in positions {
        let node = &wire.semantic_graph.nodes[position];
        let at = node_pointer(position).key("node_id");
        match node.owner.as_ref() {
            Some(CheckedNodeOwner::Source {
                authority,
                identity,
            }) => {
                if !selected_sources.contains(&(authority.as_ref(), identity.as_ref())) {
                    return Err(missing_owner(node, at));
                }
                if let Some(entries) = source_entries.get(&node.node_id) {
                    for &(entry_index, entry) in entries {
                        if node.occurrences.iter().any(|occurrence| {
                            occurrence.role == entry.role && occurrence.ordinal == entry.ordinal
                        }) {
                            for region in &entry.regions {
                                if region.source.authority != *authority
                                    || region.source.identity != *identity
                                {
                                    return Err(ValidationFailure::refused_at(
                                        CheckedPackageRefusalCode::InvalidPackage,
                                        JsonPointer::root().key("source_map").index(entry_index),
                                        Some(CheckedPackageRefusalCause::InvalidValue),
                                        node.node_id.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Some(CheckedNodeOwner::Model {
                identity,
                node: declared,
            }) => {
                let Some(model) = selected_models.get(identity.as_ref()) else {
                    return Err(missing_owner(node, at));
                };
                meter.charge(1, || at.clone())?;
                let operations = model_operations.get(identity.as_ref());
                if !kinds.get(position).is_some_and(|kind| {
                    operations.is_some_and(|operations| {
                        model_kind_matches(*kind, node, model, declared, operations)
                    })
                }) {
                    return Err(missing_owner(node, at));
                }
            }
            None => {}
        }
    }
    Ok(())
}
