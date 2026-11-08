//! Closed node-owner wire and selected-evidence joins for FR-038-AC-153..155.

use super::native_join_budget::{NativeJoinBudget, PackageReadStage};
use super::read_charge::{self, Site};
use super::stage_storage::{self, Path, Refusal, Site as StageSite};
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

fn node_path(position: usize) -> Path<'static> {
    Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position)
}
fn owner_failure(
    meter: &mut WorkMeter,
    position: usize,
    path: Path<'_>,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    node: Option<&super::CheckedNodeId>,
) -> ValidationFailure {
    let site = StageSite::new(PackageReadStage::Graph, position, "owner", path);
    let mut refusal = Refusal::new(code, path);
    if let Some(cause) = cause {
        refusal = refusal.because(cause);
    }
    if let Some(node) = node {
        refusal = refusal.at_node(node);
    }
    stage_storage::refuse(meter, &site, refusal)
}

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
    position: usize,
    budget: &mut NativeJoinBudget,
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
        let at = node_path(position);
        let at = if owner.is_some() { at.key("owner") } else { at };
        let mut meter = WorkMeter::over(budget);
        Err(owner_failure(
            &mut meter,
            position,
            at,
            CheckedPackageRefusalCode::MalformedWire,
            None,
            None,
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
            position,
            budget,
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

fn missing_owner(
    node: &CheckedSemanticNodeV2,
    position: usize,
    at: Path<'_>,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    owner_failure(
        meter,
        position,
        at,
        CheckedPackageRefusalCode::MissingDeclaration,
        Some(CheckedPackageRefusalCause::MissingSelection),
        Some(&node.node_id),
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
    meter.at_node(0, "owner");
    let mut selected_sources = meter.vec(wire.lock.sources.len())?;
    selected_sources.extend(
        wire.lock
            .sources
            .iter()
            .map(|source| (source.authority.as_ref(), source.identity.as_ref())),
    );
    selected_sources.sort_unstable();
    let mut selected_models = meter.vec(models.len())?;
    let mut model_operations = meter.vec(models.len())?;
    for model in models {
        selected_models.push((model.identity.as_ref(), model));
        let count = model
            .object_types
            .values()
            .map(|object| object.operations.len())
            .sum();
        let mut operations = meter.vec(count)?;
        operations.extend(
            model
                .object_types
                .values()
                .flat_map(|object| object.operations.iter())
                .map(|operation| operation.identity.as_ref()),
        );
        operations.sort_unstable();
        operations.dedup();
        model_operations.push((model.identity.as_ref(), operations));
    }
    selected_models.sort_unstable_by_key(|row| row.0);
    model_operations.sort_unstable_by_key(|row| row.0);
    let mut source_entries = meter.vec(
        wire.source_map
            .iter()
            .filter(|entry| entry.role == CheckedOccurrenceRole::Declaration)
            .count(),
    )?;
    source_entries.extend(
        wire.source_map
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.role == CheckedOccurrenceRole::Declaration)
            .map(|(i, entry)| (&entry.node_id, i, entry)),
    );
    source_entries.sort_unstable_by(|a, b| a.0.cmp(b.0).then(a.1.cmp(&b.1)));
    let mut positions = meter.vec(wire.semantic_graph.nodes.len())?;
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
            node_path(position).key("owner")
        } else {
            node_path(position).key("node_id")
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
                    return Err(missing_owner(node, position, at, meter));
                }
                let start = source_entries.partition_point(|entry| entry.0 < &node.node_id);
                let end = source_entries.partition_point(|entry| entry.0 <= &node.node_id);
                {
                    for &(_, entry_index, entry) in &source_entries[start..end] {
                        if node.occurrences.iter().any(|occurrence| {
                            occurrence.role == entry.role && occurrence.ordinal == entry.ordinal
                        }) {
                            for region in &entry.regions {
                                if region.source.authority != *authority
                                    || region.source.identity != *identity
                                {
                                    return Err(owner_failure(
                                        meter,
                                        position,
                                        Path::root().key("source_map").index(entry_index),
                                        CheckedPackageRefusalCode::InvalidPackage,
                                        Some(CheckedPackageRefusalCause::InvalidValue),
                                        Some(&node.node_id),
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
                let Some((_, model)) = selected_models
                    .binary_search_by_key(&identity.as_ref(), |row| row.0)
                    .ok()
                    .map(|i| &selected_models[i])
                else {
                    return Err(missing_owner(node, position, at, meter));
                };
                stage_storage::work(
                    meter,
                    &StageSite::new(PackageReadStage::Graph, position, "owner", at),
                    1,
                )?;
                let operations = model_operations
                    .binary_search_by_key(&identity.as_ref(), |row| row.0)
                    .ok()
                    .map(|i| model_operations[i].1.as_slice());
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
                    return Err(missing_owner(node, position, at, meter));
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
                    return Err(owner_failure(meter,position,node_path(position).key("node_id"),CheckedPackageRefusalCode::InvalidPackage,Some(CheckedPackageRefusalCause::InvalidDerivedShape),Some(&node.node_id)));
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
            .map(|node| node.body.clone())
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
    #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn owner_schema_lookup_and_join_storage_is_reserved_before_allocation() {
        let expression = node(
            1,
            "expression",
            "reference",
            json!({"term":"aggregate","members":[]}),
        );
        let function = node(
            0,
            "function",
            "pure_function",
            json!({"term":"aggregate","members":[{"term":"binding","name":"parameters","value":{"term":"aggregate","members":[]}},{"term":"binding","name":"body","value":{"term":"reference","target":expression["node_id"]}}]}),
        );
        let input = wire(vec![function, expression]);
        let bodies = input
            .semantic_graph
            .nodes
            .iter()
            .map(|n| n.body.clone())
            .collect::<Vec<_>>();
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let allocated = allocation_counter::measure(|| {
                result = Some(validate_owner_schema(&input, &bodies, &mut budget))
            })
            .bytes_total;
            (result.unwrap(), allocated, budget)
        };
        let (result, allocated, budget) = run(u64::MAX);
        result.unwrap();
        let required = budget.read_storage_consumed();
        assert!(required > 0);
        assert_eq!(allocated, required);
        let (result, allocated, budget) = run(required);
        result.unwrap();
        assert_eq!(allocated, budget.read_storage_consumed());
        let (result, allocated, budget) = run(required - 1);
        assert!(matches!(result, Err(ValidationFailure::Budget(_))));
        assert_eq!(allocated, budget.read_storage_consumed());
        assert!(budget.first_unpaid_read_storage().is_some());
    }
}
