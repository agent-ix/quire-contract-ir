//! Closed node-owner wire and selected-evidence joins for FR-038-AC-153..155.

use super::native_join_budget::{NativeJoinBudget, PackageReadStage};
use super::read_charge::{self, Site};
use super::CheckedJson;
use super::{
    CheckedDeclaration, CheckedNodeKind, CheckedNodeOwner, CheckedNodeTag, CheckedTerm,
    DomainModel, LiteralKind, ProtocolForm, RawCheckedPackageWireV2 as CheckedPackageWireV2,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, TermItems, TermNode, TermView, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};

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
    body_is_application: bool,
    self_typed: bool,
    owner: Option<&CheckedNodeOwner>,
    anonymous_function: bool,
) -> OwnerRequirement {
    if kind == Some(CheckedNodeKind::Model(super::ModelForm::ModelRoot)) {
        return OwnerRequirement::ModelRoot;
    }
    if body_is_application {
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

/// The charge site of the owner-schema check at graph node `index`.
fn owner_site(index: usize) -> Site {
    Site::new(PackageReadStage::WireChecks, index, "owner_schema")
}

/// The target of a `reference` term, as an owned node ID read through the
/// operand row of the node holding it, or `None` when the term is no reference
/// or its target is no valid ID. The ID is charged before it is allocated.
fn reference_target<'d>(
    term: TermNode<'d>,
    row: &[u64],
    budget: &mut NativeJoinBudget,
    site: Site,
) -> Result<Option<super::CheckedNodeId>, ValidationFailure> {
    match term.view() {
        Some(TermView::Reference { target }) => Ok(target
            .node_id_in(budget, site, row)
            .map_err(|_| super::budgeted_read::denial(budget))?
            .valid()),
        _ => Ok(None),
    }
}

/// The `name` and `value` of a `binding` term.
fn binding<'d>(term: TermNode<'d>) -> Option<(&'d str, TermNode<'d>)> {
    match term.view() {
        Some(TermView::Binding { name, value }) => Some((name, value)),
        _ => None,
    }
}

/// The members of an `aggregate` term.
fn aggregate<'d>(term: TermNode<'d>) -> Option<TermItems<'d>> {
    match term.view() {
        Some(TermView::Aggregate { members }) => Some(members),
        _ => None,
    }
}

/// Candidate entry references are collected without deciding root shape. The
/// plan step owns that refusal, even when a named candidate is malformed.
/// The result is sorted and without repeats; each target and the list are
/// charged before they are allocated.
fn entry_candidates(
    nodes: &[CheckedSemanticNodeV2],
    bodies: &[CheckedJson],
    rows: &[Vec<u64>],
    budget: &mut NativeJoinBudget,
) -> Result<Vec<super::CheckedNodeId>, ValidationFailure> {
    let mut candidates = Vec::new();
    for (position, node) in nodes.iter().enumerate() {
        if CheckedNodeTag::from_wire(&node.node_tag)
            .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
            != Some(CheckedNodeKind::Temporal(super::TemporalForm::MonitorPlan))
        {
            continue;
        }
        let Some(members) = aggregate(TermNode::of(&bodies[position])) else {
            continue;
        };
        for member in members {
            if let Some((name, value)) = binding(member) {
                if super::monitor_plan::is_entry_slot(name) {
                    let site = owner_site(position);
                    if let Some(target) = reference_target(value, &rows[position], budget, site)? {
                        read_charge::push(budget, site, &mut candidates, target)?;
                    }
                }
            }
        }
    }
    candidates.sort_unstable();
    candidates.dedup();
    Ok(candidates)
}

/// The nodes an anonymous-function shape check can name, found by node ID. A
/// duplicate ID resolves to its last node until identity validation reports it.
struct AnonymousNodes<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    /// Graph positions by node ID, sorted by ID then position.
    positions: Vec<(&'a super::CheckedNodeId, usize)>,
}

impl<'a> AnonymousNodes<'a> {
    fn get(&self, id: &super::CheckedNodeId) -> Option<&'a CheckedSemanticNodeV2> {
        let upper = self.positions.partition_point(|(key, _)| *key <= id);
        let &(key, position) = self.positions.get(upper.checked_sub(1)?)?;
        (key == id).then(|| self.nodes.get(position)).flatten()
    }
}

// This lookup is only for ordinary anonymous-function shape checks. Monitor
// entries defer those checks to plan admission; unrelated nodes need no index.
#[ix_trace_rs::string_edge]
fn anonymous_function_nodes<'a>(
    nodes: &'a [CheckedSemanticNodeV2],
    bodies: &[CheckedJson],
    rows: &[Vec<u64>],
    monitor_entries: &[super::CheckedNodeId],
    budget: &mut NativeJoinBudget,
) -> Result<AnonymousNodes<'a>, ValidationFailure> {
    let mut targets: Vec<super::CheckedNodeId> = Vec::new();
    for (position, node) in nodes.iter().enumerate() {
        if node.owner.is_some()
            || node.declaration.is_some()
            || monitor_entries.binary_search(&node.node_id).is_ok()
            || CheckedNodeTag::from_wire(&node.node_tag)
                .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
                != Some(CheckedNodeKind::Function(super::FunctionForm::PureFunction))
        {
            continue;
        }
        let Some(members) = aggregate(TermNode::of(&bodies[position])) else {
            continue;
        };
        if !matches!(members.len(), 2 | 3) {
            continue;
        }
        // Collect possible targets only; anonymous_body_shape remains the sole
        // authority for the closed syntax and the target kinds.
        let (site, row) = (owner_site(position), rows[position].as_slice());
        for member in members {
            let Some((name, value)) = binding(member) else {
                continue;
            };
            if name == "parameters" {
                if let Some(parameters) = aggregate(value) {
                    for parameter in parameters {
                        if let Some(target) = reference_target(parameter, row, budget, site)? {
                            read_charge::push(budget, site, &mut targets, target)?;
                        }
                    }
                }
            } else if matches!(name, "body" | "measure") {
                if let Some(target) = reference_target(value, row, budget, site)? {
                    read_charge::push(budget, site, &mut targets, target)?;
                }
            }
        }
    }
    targets.sort_unstable();
    targets.dedup();
    let mut positions = Vec::new();
    for (position, node) in nodes.iter().enumerate() {
        if targets.binary_search(&node.node_id).is_ok() {
            read_charge::push(
                budget,
                owner_site(position),
                &mut positions,
                (&node.node_id, position),
            )?;
        }
    }
    positions.sort_unstable();
    Ok(AnonymousNodes { nodes, positions })
}

/// The ordinary executable body, independent of owner/model clause descriptors.
fn anonymous_body_shape(
    body: TermNode<'_>,
    row: &[u64],
    nodes: &AnonymousNodes<'_>,
    budget: &mut NativeJoinBudget,
    site: Site,
) -> Result<bool, ValidationFailure> {
    let Some(mut members) = aggregate(body) else {
        return Ok(false);
    };
    let (Some(parameters), Some(body), measure) = (members.next(), members.next(), members.next())
    else {
        return Ok(false);
    };
    if members.next().is_some() {
        return Ok(false);
    }
    let Some(("parameters", value)) = binding(parameters) else {
        return Ok(false);
    };
    let Some(parameters) = aggregate(value) else {
        return Ok(false);
    };
    let mut kind = |term: TermNode<'_>| -> Result<Option<CheckedNodeKind>, ValidationFailure> {
        let Some(id) = reference_target(term, row, budget, site)? else {
            return Ok(None);
        };
        Ok(nodes.get(&id).and_then(|node| {
            CheckedNodeTag::from_wire(&node.node_tag)
                .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
        }))
    };
    for parameter in parameters {
        if kind(parameter)? != Some(CheckedNodeKind::Value(super::ValueForm::Parameter)) {
            return Ok(false);
        }
    }
    let Some(("body", value)) = binding(body) else {
        return Ok(false);
    };
    if !matches!(kind(value)?, Some(CheckedNodeKind::Expression(_))) {
        return Ok(false);
    }
    match measure {
        None => Ok(true),
        Some(measure) => match binding(measure) {
            Some(("measure", value)) => Ok(kind(value)?.is_some()),
            _ => Ok(false),
        },
    }
}

/// Checks owner presence and variant before package identity validation. The
/// lookup tables and every identity built for them are charged at the
/// `owner_schema` form.
pub(super) fn validate_owner_schema(
    wire: &CheckedPackageWireV2,
    bodies: &[CheckedJson],
    budget: &mut NativeJoinBudget,
) -> Result<(), ValidationFailure> {
    let rows = wire.semantic_graph.operand_rows.as_slice();
    let monitor_entries = entry_candidates(&wire.semantic_graph.nodes, bodies, rows, budget)?;
    let nodes = anonymous_function_nodes(
        &wire.semantic_graph.nodes,
        bodies,
        rows,
        &monitor_entries,
        budget,
    )?;
    for (position, node) in wire.semantic_graph.nodes.iter().enumerate() {
        let site = owner_site(position);
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
            super::validate_node_id_charged(
                owner_model,
                || node_pointer(position).key("owner_model"),
                budget,
                site,
            )?;
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
        let anonymous = node.owner.is_none()
            && node.declaration.is_none()
            && (monitor_entries.binary_search(&node.node_id).is_ok()
                || (kind == Some(CheckedNodeKind::Function(super::FunctionForm::PureFunction))
                    && anonymous_body_shape(
                        TermNode::of(&bodies[position]),
                        &rows[position],
                        &nodes,
                        budget,
                        site,
                    )?));
        check_one(
            requirement(
                kind,
                &node.node_tag,
                &node.semantic_form,
                node.declaration.as_ref(),
                TermNode::of(&bodies[position]).kind() == Some(super::BodyTerm::Application),
                node.semantic_type == node.node_id,
                node.owner.as_ref(),
                anonymous,
            ),
            node.owner.as_ref(),
            node_pointer(position),
        )?;
    }
    read_charge::end_stage(budget);
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
    operations: &[&str],
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
                operations.binary_search(&declared).is_ok()
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
    // Every table below is an exact-capacity sorted vector charged before it is
    // requested; none grows afterwards.
    let mut selected_sources = meter.vec::<(&str, &str)>(wire.lock.sources.len())?;
    selected_sources.extend(
        wire.lock
            .sources
            .iter()
            .map(|source| (source.authority.as_ref(), source.identity.as_ref())),
    );
    selected_sources.sort_unstable();
    // Of models with one identity the last wins, as the map it replaces did.
    let mut selected_models = meter.vec::<(&str, usize)>(models.len())?;
    selected_models.extend(
        models
            .iter()
            .enumerate()
            .map(|(model, entry)| (entry.identity.as_ref(), model)),
    );
    selected_models.sort_unstable();
    let mut distinct = 0;
    for at in 0..selected_models.len() {
        if selected_models
            .get(at + 1)
            .is_none_or(|next| next.0 != selected_models[at].0)
        {
            selected_models[distinct] = selected_models[at];
            distinct += 1;
        }
    }
    selected_models.truncate(distinct);
    let mut model_operations = meter.vec::<Vec<&str>>(models.len())?;
    for model in models {
        let count = model
            .object_types
            .values()
            .map(|object| object.operations.len())
            .sum();
        let mut operations = meter.vec::<&str>(count)?;
        operations.extend(
            model
                .object_types
                .values()
                .flat_map(|object| object.operations.iter())
                .map(|operation| operation.identity.as_ref()),
        );
        operations.sort_unstable();
        model_operations.push(operations);
    }
    let mut declared_entries = meter
        .vec::<(&crate::checked_package::shared::CheckedNodeId, usize)>(
            wire.source_map
                .iter()
                .filter(|entry| entry.role == CheckedOccurrenceRole::Declaration)
                .count(),
        )?;
    declared_entries.extend(
        wire.source_map
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.role == CheckedOccurrenceRole::Declaration)
            .map(|(entry_index, entry)| (&entry.node_id, entry_index)),
    );
    declared_entries.sort_unstable();
    let mut positions = meter.vec::<usize>(wire.semantic_graph.nodes.len())?;
    positions.extend(0..wire.semantic_graph.nodes.len());
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
                if selected_sources
                    .binary_search(&(authority.as_ref(), identity.as_ref()))
                    .is_err()
                {
                    return Err(missing_owner(node, at));
                }
                let first = declared_entries.partition_point(|(id, _)| *id < &node.node_id);
                let entries = declared_entries[first..]
                    .iter()
                    .take_while(|(id, _)| **id == node.node_id);
                {
                    for &(_, entry_index) in entries {
                        let entry = &wire.source_map[entry_index];
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
                let Ok(selected) =
                    selected_models.binary_search_by(|(name, _)| (*name).cmp(identity.as_ref()))
                else {
                    return Err(missing_owner(node, at));
                };
                let model_index = selected_models[selected].1;
                let model = &models[model_index];
                meter.charge(1, || at.clone())?;
                let operations = Some(model_operations[model_index].as_slice());
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
    use serde_json::{json, Value};

    fn node(index: usize, tag: &str, form: &str, body: Value) -> Value {
        let id = json!({"package":"test/owners", "path":[index.to_string()], "ordinal":0});
        json!({
            "schema_version":"quire.checked-semantic-graph/v2", "node_id":id,
            "node_tag":tag, "semantic_form":form, "semantic_type":id,
            "body":body, "dependencies":[], "occurrences":[]
        })
    }

    /// The owner-schema check over a test wire, its bodies read as handles.
    fn validate(wire: &CheckedPackageWireV2) -> Result<(), ValidationFailure> {
        let bodies = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| CheckedJson::from_value(&node.body).expect("test body"))
            .collect::<Vec<_>>();
        validate_owner_schema(wire, &bodies, &mut NativeJoinBudget::unlimited())
    }

    fn wire(nodes: Vec<Value>) -> CheckedPackageWireV2 {
        serde_json::from_value(json!({
            "contract_version":"quire.checked-package/v2", "name":"test/owners",
            "lock":{"sources":[], "edition":{"role":"edition", "definition":{"authority":"test", "identity":"edition"}},
                "profile_selections":[], "definition_selections":[], "required_features":[], "dependency_selections":[]},
            "semantic_graph":{"graph_version":"quire.checked-semantic-graph/v2", "nodes": nodes, "operand_rows": vec![Vec::<u64>::new(); nodes.len()]},
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
            "reference",
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
        assert!(validate(&wire(vec![
            function.clone(),
            wrong.clone(),
            expression.clone()
        ]))
        .is_ok());
        let failure = validate(&wire(vec![function, expression, wrong]))
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
            assert!(validate(&input).is_ok());
            eprintln!(
                "owner schema {count} irrelevant nodes: {:?}",
                started.elapsed()
            );
        }
    }
}
