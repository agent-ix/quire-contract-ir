//! Closed node-owner wire and selected-evidence joins for FR-038-AC-153..155.

use super::{
    CheckedDeclaration, CheckedNodeKind, CheckedNodeOwner, CheckedNodeTag, CheckedTerm,
    DomainModel, LiteralKind, ProtocolForm, RawCheckedPackageWireV2 as CheckedPackageWireV2,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, WorkMeter,
};
use crate::checked_package::common::{body_term, node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OwnerRequirement {
    Source,
    Model,
    ModelRoot,
    None,
}

#[ix_trace_rs::string_edge]
fn requirement(
    kind: Option<CheckedNodeKind>,
    node_tag: &str,
    semantic_form: &str,
    declaration: Option<&CheckedDeclaration>,
    body: &Value,
    self_typed: bool,
    owner: Option<&CheckedNodeOwner>,
    anonymous_function: bool,
) -> OwnerRequirement {
    if kind == Some(CheckedNodeKind::Model(super::ModelForm::ModelRoot)) {
        return OwnerRequirement::ModelRoot;
    }
    if body_term(body) == Some(super::BodyTerm::Application) {
        return OwnerRequirement::None;
    }
    if kind
        == Some(CheckedNodeKind::ScalarType(
            super::ScalarTypeForm::DimensionEquivalence,
        ))
    {
        return OwnerRequirement::Source;
    }
    if declaration.is_some() {
        return OwnerRequirement::Source;
    }
    // Only complete ordinary pure-function bodies are owner-free outside a
    // monitor. Candidate entries defer their closed body/signature refusal to
    // the plan step, which also owns a wrong function kind at its binding.
    if owner.is_none() && matches!(kind, Some(CheckedNodeKind::Function(_))) && anonymous_function {
        return OwnerRequirement::None;
    }

    if matches!(
        kind,
        Some(CheckedNodeKind::ScalarType(
            super::ScalarTypeForm::Enum
                | super::ScalarTypeForm::Dimension
                | super::ScalarTypeForm::Unit
        ))
    ) && matches!(owner, Some(CheckedNodeOwner::Model { .. }))
    {
        return OwnerRequirement::Model;
    }
    match (node_tag, semantic_form) {
        ("model", "record_value_type" | "tuple_value_type" | "variant_type") => {
            if self_typed || matches!(owner, Some(CheckedNodeOwner::Model { .. })) {
                OwnerRequirement::Model
            } else {
                OwnerRequirement::None
            }
        }
        (
            "model",
            "object_type"
            | "event_type"
            | "state_machine"
            | "process"
            | "persistence_interface"
            | "systems_interface",
        )
        | ("relation", "relationship" | "population")
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
        (OwnerRequirement::ModelRoot, Some(CheckedNodeOwner::ModelRoot { identity })) => {
            !identity.is_empty()
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

// This lookup is only for ordinary anonymous-function shape checks. Monitor
// entries defer those checks to plan admission; unrelated nodes need no index.
fn anonymous_function_nodes<'a>(
    nodes: &'a [CheckedSemanticNodeV2],
    monitor_entries: &BTreeSet<super::CheckedNodeId>,
) -> BTreeMap<&'a super::CheckedNodeId, &'a CheckedSemanticNodeV2> {
    let mut targets = BTreeSet::new();
    for node in nodes {
        if node.owner.is_some()
            || node.declaration.is_some()
            || monitor_entries.contains(&node.node_id)
            || CheckedNodeTag::from_wire(&node.node_tag)
                .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
                != Some(CheckedNodeKind::Function(super::FunctionForm::PureFunction))
        {
            continue;
        }
        let Some(super::CheckedTermView::Aggregate { members }) = CheckedTerm::view(&node.body)
        else {
            continue;
        };
        // Collect possible targets only; anonymous_body_shape remains the sole
        // authority for the closed syntax and the target kinds.
        for member in members {
            let Some(super::CheckedTermView::Binding { name, value }) = CheckedTerm::view(member)
            else {
                continue;
            };
            if name == "parameters" {
                if let Some(super::CheckedTermView::Aggregate { members }) =
                    CheckedTerm::view(value)
                {
                    targets.extend(
                        members
                            .iter()
                            .filter_map(super::structural::reference_target),
                    );
                }
            } else if matches!(name, "body" | "measure") {
                targets.extend(super::structural::reference_target(value));
            }
        }
    }
    let mut selected = BTreeMap::new();
    if !targets.is_empty() {
        for node in nodes {
            if targets.contains(&node.node_id) {
                // Preserve the raw lookup's last duplicate until ordinary
                // identity validation reports duplicate node IDs.
                selected.insert(&node.node_id, node);
            }
        }
    }
    selected
}

/// Checks owner presence and variant before package identity validation.
pub(super) fn validate_owner_schema(wire: &CheckedPackageWireV2) -> Result<(), ValidationFailure> {
    let monitor_entries = super::monitor_plan::entry_candidates(&wire.semantic_graph.nodes);
    let nodes = anonymous_function_nodes(&wire.semantic_graph.nodes, &monitor_entries);
    for (position, node) in wire.semantic_graph.nodes.iter().enumerate() {
        let kind = CheckedNodeTag::from_wire(&node.node_tag)
            .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form));
        let clause = kind == Some(CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause));
        if clause != node.owner_model.is_some() {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                if node.owner_model.is_some() {
                    node_pointer(position).key("owner_model")
                } else {
                    node_pointer(position)
                },
            ));
        }
        if let Some(owner_model) = &node.owner_model {
            super::validate_node_id(owner_model, || node_pointer(position).key("owner_model"))?;
        }
        if kind == Some(CheckedNodeKind::Protocol(ProtocolForm::ProtocolReference)) {
            // A local reference carrier may participate in a protocol SCC;
            // its group label is authenticated by structural-key and SCC validation.
            let forbidden = [
                ("declaration", node.declaration.is_some()),
                ("owner", node.owner.is_some()),
            ];
            if let Some((member, _)) = forbidden.into_iter().find(|(_, present)| *present) {
                return Err(ValidationFailure::refused(
                    CheckedPackageRefusalCode::MalformedWire,
                    node_pointer(position).key(member),
                ));
            }
        }
        check_one(
            requirement(
                kind,
                &node.node_tag,
                &node.semantic_form,
                node.declaration.as_ref(),
                &node.body,
                node.semantic_type == node.node_id,
                node.owner.as_ref(),
                node.owner.is_none()
                    && node.declaration.is_none()
                    && (monitor_entries.contains(&node.node_id)
                        || (kind
                            == Some(CheckedNodeKind::Function(super::FunctionForm::PureFunction))
                            && super::monitor_plan::anonymous_body_shape(&node.body, &nodes))),
            ),
            node.owner.as_ref(),
            node_pointer(position),
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum OwnerClauseKind {
    Invariant,
    Precondition,
    Body,
    Other,
}

impl OwnerClauseKind {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Self {
        match text {
            "invariant" => Self::Invariant,
            "precondition" => Self::Precondition,
            "body" => Self::Body,
            _ => Self::Other,
        }
    }
}

fn clause_kind(body: &CheckedTerm) -> Option<OwnerClauseKind> {
    let CheckedTerm::Aggregate { members } = body else {
        return None;
    };
    let mut clauses = members.iter().filter_map(|member| match member {
        CheckedTerm::Binding { name, value }
            if name.internal_kind() == super::TermBindingName::Clause =>
        {
            Some(value.as_ref())
        }
        _ => None,
    });
    let clause = clauses.next()?;
    if clauses.next().is_some() {
        return None;
    }
    match clause {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        } => value.as_str().map(OwnerClauseKind::from_wire),
        _ => None,
    }
}

fn model_kind_matches(
    kind: CheckedNodeKind,
    term: &CheckedTerm,
    model: &DomainModel,
    declared: &str,
    operations: &BTreeSet<&str>,
) -> bool {
    match kind {
        CheckedNodeKind::ScalarType(super::ScalarTypeForm::Dimension) => {
            model.dimensions.contains_key(declared)
        }
        CheckedNodeKind::ScalarType(super::ScalarTypeForm::Unit) => {
            model.units.contains_key(declared)
        }
        CheckedNodeKind::Model(super::ModelForm::ObjectType) => {
            model.object_types.get(declared).is_some_and(|object| {
                !object.interface
                    && !model.state_machines.contains(declared)
                    && !model.processes.contains(declared)
            })
        }
        CheckedNodeKind::Model(super::ModelForm::StateMachine) => {
            model.state_machines.contains(declared)
        }
        CheckedNodeKind::Model(super::ModelForm::Process) => model.processes.contains(declared),
        CheckedNodeKind::Model(super::ModelForm::PersistenceInterface) => {
            model.persistence_interfaces.contains(declared)
        }
        CheckedNodeKind::Model(super::ModelForm::RecordValueType) => {
            model.record_types.contains_key(declared)
        }
        CheckedNodeKind::Model(super::ModelForm::EventType) => model.event_types.contains(declared),
        CheckedNodeKind::Model(super::ModelForm::TupleValueType) => {
            model.tuple_types.contains_key(declared)
        }
        CheckedNodeKind::Model(super::ModelForm::VariantType) => {
            model.variant_types.contains_key(declared)
        }
        CheckedNodeKind::Model(super::ModelForm::SystemsInterface) => model
            .object_types
            .get(declared)
            .is_some_and(|declared| declared.interface),
        CheckedNodeKind::Relation(super::RelationForm::Relationship) => {
            model.relationships.contains_key(declared) || model.relation_nodes.contains(declared)
        }
        CheckedNodeKind::Relation(super::RelationForm::Population) => {
            model.populations.contains(declared)
        }
        CheckedNodeKind::Function(_) => match clause_kind(term) {
            Some(OwnerClauseKind::Invariant) => {
                model.object_types.get(declared).is_some_and(|object| {
                    !object.interface && !model.state_machines.contains(declared)
                }) || model.record_types.contains_key(declared)
                    || model.event_types.contains(declared)
            }
            Some(OwnerClauseKind::Precondition | OwnerClauseKind::Body) => {
                operations.contains(declared)
            }
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
    terms: &[Option<CheckedTerm>],
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
            .cmp(&wire.semantic_graph.nodes[right].node_id)
    });
    for position in positions {
        let node = &wire.semantic_graph.nodes[position];
        let at = if matches!(
            kinds.get(position),
            Some(CheckedNodeKind::Model(
                super::ModelForm::TupleValueType | super::ModelForm::VariantType
            ))
        ) {
            node_pointer(position).key("owner")
        } else {
            node_pointer(position).key("node_id")
        };
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
                        terms
                            .get(position)
                            .and_then(Option::as_ref)
                            .is_some_and(|term| {
                                model_kind_matches(*kind, term, model, declared, operations)
                            })
                    })
                }) {
                    return Err(missing_owner(node, at));
                }
                // A selected model or relation declaration has a fixed,
                // self-typed shape. Check it at the declaration before an
                // application can report the same defect at its member.
                if kinds.get(position).is_some_and(|kind| {
                    matches!(
                        kind,
                        CheckedNodeKind::Model(_) | CheckedNodeKind::Relation(_)
                    )
                }) && (node.semantic_type != node.node_id
                    || node.declaration.is_some()
                    || node.recursion_group.is_some()
                    || !terms.get(position).and_then(Option::as_ref).is_some_and(
                        |term| matches!(term, CheckedTerm::Aggregate { members } if members.is_empty()),
                    ))
                {
                    return Err(ValidationFailure::refused_at(
                        CheckedPackageRefusalCode::InvalidPackage,
                        node_pointer(position).key("node_id"),
                        Some(CheckedPackageRefusalCause::InvalidDerivedShape),
                        node.node_id.clone(),
                    ));
                }
            }
            Some(CheckedNodeOwner::ModelRoot { .. }) | None => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod anonymous_lookup_tests {
    use super::*;
    use serde_json::json;

    fn node(index: usize, tag: &str, form: &str, body: Value) -> Value {
        let id = json!({"package":"test/owners", "path":[index.to_string()], "ordinal":0});
        json!({
            "schema_version":"quire.checked-semantic-graph/v2", "node_id":id,
            "node_tag":tag, "semantic_form":form, "semantic_type":id,
            "body":body, "dependencies":[], "occurrences":[]
        })
    }

    fn wire(nodes: Vec<Value>) -> CheckedPackageWireV2 {
        serde_json::from_value(json!({
            "contract_version":"quire.checked-package/v2", "name":"test/owners",
            "lock":{"sources":[], "edition":{"role":"edition", "definition":{"authority":"test", "identity":"edition"}},
                "profile_selections":[], "definition_selections":[], "required_features":[], "dependency_selections":[]},
            "semantic_graph":{"graph_version":"quire.checked-semantic-graph/v2", "nodes":nodes},
            "source_map":[], "capability_report":[],
            "diagnostics":{"catalog":{"authority":"test", "identity":"diagnostics"}, "entries":[]}
        })).expect("owner-schema input")
    }

    /// Trace: TC-048, FR-038-AC-425.
    #[test]
    fn anonymous_owner_lookup_preserves_last_duplicate_target() {
        let expression = node(
            1,
            "expression",
            "literal",
            json!({"term":"aggregate", "members":[]}),
        );
        let function = node(
            0,
            "function",
            "pure_function",
            json!({"term":"aggregate", "members":[
                {"term":"binding", "name":"parameters", "value":{"term":"aggregate", "members":[]}},
                {"term":"binding", "name":"body", "value":{"term":"reference", "target":expression["node_id"]}}
            ]}),
        );
        let wrong = node(
            1,
            "scalar_type",
            "integer",
            json!({"term":"aggregate", "members":[]}),
        );
        assert!(validate_owner_schema(&wire(vec![
            function.clone(),
            wrong.clone(),
            expression.clone()
        ]))
        .is_ok());
        let failure = validate_owner_schema(&wire(vec![function, expression, wrong]))
            .expect_err("last duplicate has wrong target kind");
        let ValidationFailure::Refused(refusal) = failure else {
            panic!("owner refusal")
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(
            refusal.loci[0].path,
            JsonPointer::parse("/semantic_graph/nodes/0")
        );
    }

    /// Trace: TC-048, FR-038-AC-425.
    #[test]
    fn owner_schema_irrelevant_nodes_scaling() {
        for count in [1_000, 100_000] {
            let input = wire(
                (0..count)
                    .map(|i| {
                        node(
                            i,
                            "scalar_type",
                            "integer",
                            json!({"term":"aggregate", "members":[]}),
                        )
                    })
                    .collect(),
            );
            let started = std::time::Instant::now();
            assert!(validate_owner_schema(&input).is_ok());
            eprintln!(
                "owner schema {count} irrelevant nodes: {:?}",
                started.elapsed()
            );
        }
    }
}
